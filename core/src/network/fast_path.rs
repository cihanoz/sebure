//! # Fast Path Network Routes
//! 
//! This module implements fast path network routes for high-priority transactions
//! and critical network messages, ensuring minimal latency for important data.

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::network::{Peer, PeerInfo, PeerScore, Message, MessageType};
use crate::types::{Result, Error, Priority};

/// Fast path configuration
#[derive(Debug, Clone)]
pub struct FastPathConfig {
    /// Minimum number of fast path peers
    pub min_fast_path_peers: usize,
    
    /// Maximum number of fast path peers
    pub max_fast_path_peers: usize,
    
    /// Minimum ping time for fast path eligibility (in ms)
    pub max_ping_threshold: u64,
    
    /// Minimum uptime for fast path eligibility (in seconds)
    pub min_uptime_threshold: u64,
    
    /// Refresh interval for fast path peers (in seconds)
    pub refresh_interval: u64,
    
    /// Message types that should use fast path
    pub fast_path_message_types: Vec<MessageType>,
}

impl Default for FastPathConfig {
    fn default() -> Self {
        FastPathConfig {
            min_fast_path_peers: 3,
            max_fast_path_peers: 5,
            max_ping_threshold: 100,
            min_uptime_threshold: 300,
            refresh_interval: 60,
            fast_path_message_types: vec![
                MessageType::BlockAnnouncement,
                MessageType::TransactionAnnouncement,
                MessageType::CheckpointVote,
            ],
        }
    }
}

/// Fast path peer information
#[derive(Debug)]
struct FastPathPeerInfo {
    /// Peer address
    address: SocketAddr,
    
    /// Last ping time in milliseconds
    ping_ms: u64,
    
    /// Connection time
    connected_at: Instant,
    
    /// Last message sent time
    last_message_sent: Instant,
    
    /// Number of messages sent
    messages_sent: usize,
    
    /// Number of successful deliveries
    successful_deliveries: usize,
    
    /// Success rate (0.0 - 1.0)
    success_rate: f64,
}

impl FastPathPeerInfo {
    /// Create a new fast path peer info
    fn new(address: SocketAddr, ping_ms: u64) -> Self {
        let now = Instant::now();
        FastPathPeerInfo {
            address,
            ping_ms,
            connected_at: now,
            last_message_sent: now,
            messages_sent: 0,
            successful_deliveries: 0,
            success_rate: 1.0, // Assume 100% success initially
        }
    }
    
    /// Update the success rate
    fn update_success_rate(&mut self, success: bool) {
        self.messages_sent += 1;
        if success {
            self.successful_deliveries += 1;
        }
        
        self.success_rate = self.successful_deliveries as f64 / self.messages_sent as f64;
        self.last_message_sent = Instant::now();
    }
    
    /// Get the uptime in seconds
    fn uptime(&self) -> u64 {
        self.connected_at.elapsed().as_secs()
    }
    
    /// Calculate the peer score for fast path selection
    fn calculate_score(&self) -> f64 {
        // Lower ping is better
        let ping_score = if self.ping_ms == 0 {
            0.5 // Default if no ping yet
        } else if self.ping_ms < 50 {
            1.0
        } else if self.ping_ms < 100 {
            0.8
        } else if self.ping_ms < 200 {
            0.5
        } else {
            0.2
        };
        
        // Higher uptime is better
        let uptime_secs = self.uptime();
        let uptime_score = if uptime_secs < 60 {
            0.2
        } else if uptime_secs < 300 {
            0.5
        } else if uptime_secs < 1800 {
            0.8
        } else {
            1.0
        };
        
        // Combine scores with weights
        // Success rate is most important, then ping, then uptime
        self.success_rate * 0.6 + ping_score * 0.3 + uptime_score * 0.1
    }
}

/// Fast path manager for optimized message routing
pub struct FastPath {
    /// Configuration
    config: FastPathConfig,
    
    /// Fast path peers
    peers: HashMap<SocketAddr, FastPathPeerInfo>,
    
    /// Selected fast path peers
    fast_path_peers: HashSet<SocketAddr>,
    
    /// Last refresh time
    last_refresh: Instant,
}

impl FastPath {
    /// Create a new fast path manager
    pub fn new(config: FastPathConfig) -> Self {
        FastPath {
            config,
            peers: HashMap::new(),
            fast_path_peers: HashSet::new(),
            last_refresh: Instant::now(),
        }
    }
    
    /// Add or update a peer
    pub fn update_peer(&mut self, address: SocketAddr, ping_ms: u64) {
        if let Some(peer_info) = self.peers.get_mut(&address) {
            peer_info.ping_ms = ping_ms;
        } else {
            self.peers.insert(address, FastPathPeerInfo::new(address, ping_ms));
        }
        
        // Refresh fast path peers if needed
        self.refresh_if_needed();
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, address: &SocketAddr) {
        self.peers.remove(address);
        self.fast_path_peers.remove(address);
        
        // Refresh fast path peers if we're below the minimum
        if self.fast_path_peers.len() < self.config.min_fast_path_peers {
            self.refresh_fast_path_peers();
        }
    }
    
    /// Update delivery status for a peer
    pub fn update_delivery_status(&mut self, address: &SocketAddr, success: bool) {
        if let Some(peer_info) = self.peers.get_mut(address) {
            peer_info.update_success_rate(success);
        }
    }
    
    /// Check if a message should use the fast path
    pub fn should_use_fast_path(&self, message: &Message) -> bool {
        // High priority messages always use fast path
        if message.priority == Priority::High {
            return true;
        }
        
        // Check if the message type is in the fast path list
        self.config.fast_path_message_types.contains(&message.message_type)
    }
    
    /// Get the fast path peers for a message
    pub fn get_fast_path_peers(&mut self, message: &Message) -> Vec<SocketAddr> {
        // Refresh if needed
        self.refresh_if_needed();
        
        // If we don't have enough fast path peers, return all peers
        if self.fast_path_peers.len() < self.config.min_fast_path_peers {
            return self.fast_path_peers.iter().cloned().collect();
        }
        
        // For high priority messages, use all fast path peers
        if message.priority == Priority::High {
            return self.fast_path_peers.iter().cloned().collect();
        }
        
        // For other messages, use a subset based on message type
        let count = match message.message_type {
            MessageType::BlockAnnouncement => self.fast_path_peers.len(),
            MessageType::TransactionAnnouncement => self.fast_path_peers.len() / 2 + 1,
            _ => self.config.min_fast_path_peers,
        };
        
        // Get the top 'count' peers
        let mut peers: Vec<_> = self.fast_path_peers.iter().cloned().collect();
        peers.truncate(count);
        peers
    }
    
    /// Refresh fast path peers if needed
    fn refresh_if_needed(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= Duration::from_secs(self.config.refresh_interval) {
            self.refresh_fast_path_peers();
        }
    }
    
    /// Refresh the fast path peers
    fn refresh_fast_path_peers(&mut self) {
        self.last_refresh = Instant::now();
        
        // Clear current fast path peers
        self.fast_path_peers.clear();
        
        // Filter eligible peers
        let mut eligible_peers: Vec<_> = self.peers.values()
            .filter(|p| {
                p.ping_ms <= self.config.max_ping_threshold &&
                p.uptime() >= self.config.min_uptime_threshold
            })
            .collect();
        
        // Sort by score (descending)
        eligible_peers.sort_by(|a, b| {
            b.calculate_score().partial_cmp(&a.calculate_score()).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Take the top peers
        let count = std::cmp::min(
            self.config.max_fast_path_peers,
            std::cmp::max(self.config.min_fast_path_peers, eligible_peers.len())
        );
        
        for peer in eligible_peers.iter().take(count) {
            self.fast_path_peers.insert(peer.address);
        }
        
        log::debug!("Refreshed fast path peers: {} selected from {} eligible peers",
                  self.fast_path_peers.len(), eligible_peers.len());
    }
    
    /// Get the number of fast path peers
    pub fn fast_path_peer_count(&self) -> usize {
        self.fast_path_peers.len()
    }
    
    /// Get all fast path peers
    pub fn get_all_fast_path_peers(&self) -> impl Iterator<Item = &SocketAddr> {
        self.fast_path_peers.iter()
    }
    
    /// Check if a peer is on the fast path
    pub fn is_fast_path_peer(&self, address: &SocketAddr) -> bool {
        self.fast_path_peers.contains(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    
    fn create_test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }
    
    fn create_test_message(message_type: MessageType, priority: Priority) -> Message {
        Message {
            version: 1,
            compression: false,
            encryption: false,
            priority,
            message_type,
            shard_id: None,
            data: Vec::new(),
            checksum: [0; 4],
            sender: Vec::new(),
            signature: Vec::new(),
        }
    }
    
    #[test]
    fn test_fast_path_config_default() {
        let config = FastPathConfig::default();
        
        assert_eq!(config.min_fast_path_peers, 3);
        assert_eq!(config.max_fast_path_peers, 5);
        assert_eq!(config.max_ping_threshold, 100);
        assert_eq!(config.min_uptime_threshold, 300);
        assert_eq!(config.refresh_interval, 60);
        assert!(config.fast_path_message_types.contains(&MessageType::BlockAnnouncement));
    }
    
    #[test]
    fn test_fast_path_peer_info() {
        let addr = create_test_addr(8000);
        let mut peer_info = FastPathPeerInfo::new(addr, 50);
        
        assert_eq!(peer_info.address, addr);
        assert_eq!(peer_info.ping_ms, 50);
        assert_eq!(peer_info.messages_sent, 0);
        assert_eq!(peer_info.successful_deliveries, 0);
        assert_eq!(peer_info.success_rate, 1.0);
        
        // Update success rate
        peer_info.update_success_rate(true);
        assert_eq!(peer_info.messages_sent, 1);
        assert_eq!(peer_info.successful_deliveries, 1);
        assert_eq!(peer_info.success_rate, 1.0);
        
        peer_info.update_success_rate(false);
        assert_eq!(peer_info.messages_sent, 2);
        assert_eq!(peer_info.successful_deliveries, 1);
        assert_eq!(peer_info.success_rate, 0.5);
    }
    
    #[test]
    fn test_fast_path_creation() {
        let config = FastPathConfig::default();
        let fast_path = FastPath::new(config);
        
        assert_eq!(fast_path.peers.len(), 0);
        assert_eq!(fast_path.fast_path_peers.len(), 0);
    }
    
    #[test]
    fn test_fast_path_update_peer() {
        let config = FastPathConfig::default();
        let mut fast_path = FastPath::new(config);
        
        let addr1 = create_test_addr(8000);
        let addr2 = create_test_addr(8001);
        
        // Add peers
        fast_path.update_peer(addr1, 50);
        fast_path.update_peer(addr2, 100);
        
        assert_eq!(fast_path.peers.len(), 2);
        
        // Update a peer
        fast_path.update_peer(addr1, 30);
        
        assert_eq!(fast_path.peers.len(), 2);
        assert_eq!(fast_path.peers.get(&addr1).unwrap().ping_ms, 30);
    }
    
    #[test]
    fn test_fast_path_remove_peer() {
        let config = FastPathConfig::default();
        let mut fast_path = FastPath::new(config);
        
        let addr1 = create_test_addr(8000);
        let addr2 = create_test_addr(8001);
        
        // Add peers
        fast_path.update_peer(addr1, 50);
        fast_path.update_peer(addr2, 100);
        
        assert_eq!(fast_path.peers.len(), 2);
        
        // Remove a peer
        fast_path.remove_peer(&addr1);
        
        assert_eq!(fast_path.peers.len(), 1);
        assert!(!fast_path.peers.contains_key(&addr1));
        assert!(fast_path.peers.contains_key(&addr2));
    }
    
    #[test]
    fn test_should_use_fast_path() {
        let config = FastPathConfig::default();
        let fast_path = FastPath::new(config);
        
        // High priority message should use fast path
        let high_priority = create_test_message(MessageType::PeerExchange, Priority::High);
        assert!(fast_path.should_use_fast_path(&high_priority));
        
        // Block announcement should use fast path
        let block_announcement = create_test_message(MessageType::BlockAnnouncement, Priority::Normal);
        assert!(fast_path.should_use_fast_path(&block_announcement));
        
        // Other message should not use fast path
        let other_message = create_test_message(MessageType::PeerExchange, Priority::Low);
        assert!(!fast_path.should_use_fast_path(&other_message));
    }
}
