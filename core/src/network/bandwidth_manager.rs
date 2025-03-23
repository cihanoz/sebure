//! # Adaptive Bandwidth Manager
//! 
//! This module implements adaptive bandwidth allocation for the P2P network,
//! optimizing network usage based on current conditions and message priorities.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::network::{Message, MessageType};
use crate::types::{Result, Error, Priority};

/// Bandwidth allocation configuration
#[derive(Debug, Clone)]
pub struct BandwidthConfig {
    /// Maximum outbound bandwidth in bytes per second
    pub max_outbound_bandwidth: u64,
    
    /// Maximum inbound bandwidth in bytes per second
    pub max_inbound_bandwidth: u64,
    
    /// Burst factor for temporary bandwidth increases
    pub burst_factor: f64,
    
    /// Measurement window in seconds
    pub measurement_window: u64,
    
    /// Allocation update interval in seconds
    pub update_interval: u64,
    
    /// Minimum bandwidth allocation per peer in bytes per second
    pub min_peer_bandwidth: u64,
    
    /// Priority weights for bandwidth allocation
    pub priority_weights: HashMap<Priority, f64>,
    
    /// Message type weights for bandwidth allocation
    pub message_type_weights: HashMap<MessageType, f64>,
}

impl Default for BandwidthConfig {
    fn default() -> Self {
        let mut priority_weights = HashMap::new();
        priority_weights.insert(Priority::High, 3.0);
        priority_weights.insert(Priority::Normal, 1.0);
        priority_weights.insert(Priority::Low, 0.5);
        
        let mut message_type_weights = HashMap::new();
        message_type_weights.insert(MessageType::BlockAnnouncement, 2.0);
        message_type_weights.insert(MessageType::BlockHeader, 1.5);
        message_type_weights.insert(MessageType::BlockBody, 1.0);
        message_type_weights.insert(MessageType::TransactionAnnouncement, 1.5);
        message_type_weights.insert(MessageType::TransactionBatch, 1.0);
        message_type_weights.insert(MessageType::ShardSyncRequest, 1.0);
        message_type_weights.insert(MessageType::ShardStateResponse, 0.8);
        message_type_weights.insert(MessageType::ValidatorHandshake, 1.2);
        message_type_weights.insert(MessageType::PeerDiscovery, 0.5);
        message_type_weights.insert(MessageType::PeerExchange, 0.5);
        message_type_weights.insert(MessageType::StateSnapshot, 0.7);
        message_type_weights.insert(MessageType::CheckpointVote, 1.5);
        message_type_weights.insert(MessageType::NetworkHealth, 0.3);
        
        BandwidthConfig {
            max_outbound_bandwidth: 1024 * 1024, // 1 MB/s
            max_inbound_bandwidth: 1024 * 1024,  // 1 MB/s
            burst_factor: 2.0,
            measurement_window: 10,
            update_interval: 5,
            min_peer_bandwidth: 1024, // 1 KB/s
            priority_weights,
            message_type_weights,
        }
    }
}

/// Bandwidth usage record
#[derive(Debug, Clone)]
struct BandwidthRecord {
    /// Timestamp
    timestamp: Instant,
    
    /// Bytes transferred
    bytes: u64,
    
    /// Direction (true for outbound, false for inbound)
    outbound: bool,
}

/// Peer bandwidth allocation
#[derive(Debug)]
struct PeerBandwidth {
    /// Peer address
    address: SocketAddr,
    
    /// Outbound bandwidth allocation in bytes per second
    outbound_allocation: u64,
    
    /// Inbound bandwidth allocation in bytes per second
    inbound_allocation: u64,
    
    /// Outbound usage in the current window in bytes
    outbound_usage: u64,
    
    /// Inbound usage in the current window in bytes
    inbound_usage: u64,
    
    /// Outbound usage history
    outbound_history: VecDeque<BandwidthRecord>,
    
    /// Inbound usage history
    inbound_history: VecDeque<BandwidthRecord>,
    
    /// Last allocation update time
    last_update: Instant,
    
    /// Peer weight for allocation (higher gets more bandwidth)
    weight: f64,
}

impl PeerBandwidth {
    /// Create a new peer bandwidth allocation
    fn new(address: SocketAddr, outbound_allocation: u64, inbound_allocation: u64) -> Self {
        PeerBandwidth {
            address,
            outbound_allocation,
            inbound_allocation,
            outbound_usage: 0,
            inbound_usage: 0,
            outbound_history: VecDeque::new(),
            inbound_history: VecDeque::new(),
            last_update: Instant::now(),
            weight: 1.0,
        }
    }
    
    /// Record bandwidth usage
    fn record_usage(&mut self, bytes: u64, outbound: bool, now: Instant) {
        let record = BandwidthRecord {
            timestamp: now,
            bytes,
            outbound,
        };
        
        if outbound {
            self.outbound_usage += bytes;
            self.outbound_history.push_back(record);
        } else {
            self.inbound_usage += bytes;
            self.inbound_history.push_back(record);
        }
    }
    
    /// Prune old records outside the measurement window
    fn prune_old_records(&mut self, window: Duration) {
        let now = Instant::now();
        let cutoff = now - window;
        
        // Prune outbound records
        while let Some(record) = self.outbound_history.front() {
            if record.timestamp < cutoff {
                self.outbound_usage = self.outbound_usage.saturating_sub(record.bytes);
                self.outbound_history.pop_front();
            } else {
                break;
            }
        }
        
        // Prune inbound records
        while let Some(record) = self.inbound_history.front() {
            if record.timestamp < cutoff {
                self.inbound_usage = self.inbound_usage.saturating_sub(record.bytes);
                self.inbound_history.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// Get the current outbound bandwidth usage in bytes per second
    fn get_outbound_usage_rate(&self, window: Duration) -> u64 {
        if window.as_secs() == 0 {
            return 0;
        }
        
        self.outbound_usage / window.as_secs()
    }
    
    /// Get the current inbound bandwidth usage in bytes per second
    fn get_inbound_usage_rate(&self, window: Duration) -> u64 {
        if window.as_secs() == 0 {
            return 0;
        }
        
        self.inbound_usage / window.as_secs()
    }
    
    /// Update the peer weight based on message history
    fn update_weight(&mut self, priority_weights: &HashMap<Priority, f64>, message_type_weights: &HashMap<MessageType, f64>) {
        // In a real implementation, we would analyze the message history
        // and adjust the weight based on the importance of the peer's messages
        // For now, just use a default weight
        self.weight = 1.0;
    }
}

/// Message queue entry
#[derive(Debug)]
struct QueuedMessage {
    /// Message to send
    message: Message,
    
    /// Destination peer
    peer: SocketAddr,
    
    /// Enqueue time
    enqueue_time: Instant,
    
    /// Message size in bytes
    size: usize,
    
    /// Message score for prioritization
    score: f64,
}

/// Bandwidth manager for adaptive allocation
pub struct BandwidthManager {
    /// Configuration
    config: BandwidthConfig,
    
    /// Peer bandwidth allocations
    peer_bandwidths: HashMap<SocketAddr, PeerBandwidth>,
    
    /// Total outbound bandwidth allocation
    total_outbound_allocation: u64,
    
    /// Total inbound bandwidth allocation
    total_inbound_allocation: u64,
    
    /// Last allocation update time
    last_allocation_update: Instant,
    
    /// Outbound message queue
    outbound_queue: VecDeque<QueuedMessage>,
    
    /// Network congestion level (0.0 - 1.0)
    congestion_level: f64,
    
    /// Current burst mode (temporary bandwidth increase)
    burst_mode: bool,
    
    /// Burst mode end time
    burst_end_time: Option<Instant>,
}

impl BandwidthManager {
    /// Create a new bandwidth manager
    pub fn new(config: BandwidthConfig) -> Self {
        BandwidthManager {
            config,
            peer_bandwidths: HashMap::new(),
            total_outbound_allocation: 0,
            total_inbound_allocation: 0,
            last_allocation_update: Instant::now(),
            outbound_queue: VecDeque::new(),
            congestion_level: 0.0,
            burst_mode: false,
            burst_end_time: None,
        }
    }
    
    /// Add a peer to the bandwidth manager
    pub fn add_peer(&mut self, address: SocketAddr) {
        if self.peer_bandwidths.contains_key(&address) {
            return;
        }
        
        // Calculate initial allocations
        let peer_count = self.peer_bandwidths.len() as u64 + 1;
        let outbound_allocation = std::cmp::max(
            self.config.min_peer_bandwidth,
            self.config.max_outbound_bandwidth / peer_count
        );
        
        let inbound_allocation = std::cmp::max(
            self.config.min_peer_bandwidth,
            self.config.max_inbound_bandwidth / peer_count
        );
        
        // Create peer bandwidth allocation
        let peer_bandwidth = PeerBandwidth::new(address, outbound_allocation, inbound_allocation);
        
        // Update total allocations
        self.total_outbound_allocation += outbound_allocation;
        self.total_inbound_allocation += inbound_allocation;
        
        // Add to peer bandwidths
        self.peer_bandwidths.insert(address, peer_bandwidth);
        
        // Rebalance allocations
        self.update_allocations();
    }
    
    /// Remove a peer from the bandwidth manager
    pub fn remove_peer(&mut self, address: &SocketAddr) {
        if let Some(peer_bandwidth) = self.peer_bandwidths.remove(address) {
            // Update total allocations
            self.total_outbound_allocation -= peer_bandwidth.outbound_allocation;
            self.total_inbound_allocation -= peer_bandwidth.inbound_allocation;
            
            // Rebalance allocations
            self.update_allocations();
        }
    }
    
    /// Record bandwidth usage for a peer
    pub fn record_usage(&mut self, address: &SocketAddr, bytes: u64, outbound: bool) {
        let now = Instant::now();
        
        if let Some(peer_bandwidth) = self.peer_bandwidths.get_mut(address) {
            peer_bandwidth.record_usage(bytes, outbound, now);
            
            // Update congestion level
            self.update_congestion_level();
        }
    }
    
    /// Update bandwidth allocations based on current usage
    pub fn update_allocations(&mut self) {
        let now = Instant::now();
        let update_interval = Duration::from_secs(self.config.update_interval);
        
        // Only update periodically
        if now.duration_since(self.last_allocation_update) < update_interval {
            return;
        }
        
        self.last_allocation_update = now;
        
        // Prune old records
        let window = Duration::from_secs(self.config.measurement_window);
        for peer_bandwidth in self.peer_bandwidths.values_mut() {
            peer_bandwidth.prune_old_records(window);
            peer_bandwidth.update_weight(&self.config.priority_weights, &self.config.message_type_weights);
        }
        
        // Calculate total weight
        let total_weight: f64 = self.peer_bandwidths.values().map(|pb| pb.weight).sum();
        
        if total_weight <= 0.0 {
            return;
        }
        
        // Calculate new allocations
        let mut new_outbound_total = 0;
        let mut new_inbound_total = 0;
        
        let max_outbound = if self.burst_mode {
            (self.config.max_outbound_bandwidth as f64 * self.config.burst_factor) as u64
        } else {
            self.config.max_outbound_bandwidth
        };
        
        let max_inbound = if self.burst_mode {
            (self.config.max_inbound_bandwidth as f64 * self.config.burst_factor) as u64
        } else {
            self.config.max_inbound_bandwidth
        };
        
        for peer_bandwidth in self.peer_bandwidths.values_mut() {
            // Calculate new allocations based on weight
            let weight_fraction = peer_bandwidth.weight / total_weight;
            
            let new_outbound = std::cmp::max(
                self.config.min_peer_bandwidth,
                (max_outbound as f64 * weight_fraction) as u64
            );
            
            let new_inbound = std::cmp::max(
                self.config.min_peer_bandwidth,
                (max_inbound as f64 * weight_fraction) as u64
            );
            
            // Update peer allocations
            peer_bandwidth.outbound_allocation = new_outbound;
            peer_bandwidth.inbound_allocation = new_inbound;
            
            // Update totals
            new_outbound_total += new_outbound;
            new_inbound_total += new_inbound;
        }
        
        // Update total allocations
        self.total_outbound_allocation = new_outbound_total;
        self.total_inbound_allocation = new_inbound_total;
        
        // Check if we need to exit burst mode
        if self.burst_mode {
            if let Some(end_time) = self.burst_end_time {
                if now >= end_time {
                    self.burst_mode = false;
                    self.burst_end_time = None;
                    
                    // Rebalance allocations again
                    self.update_allocations();
                }
            }
        }
    }
    
    /// Update the network congestion level
    fn update_congestion_level(&mut self) {
        let window = Duration::from_secs(self.config.measurement_window);
        
        // Calculate total usage rates
        let mut total_outbound_rate = 0;
        let mut total_inbound_rate = 0;
        
        for peer_bandwidth in self.peer_bandwidths.values() {
            total_outbound_rate += peer_bandwidth.get_outbound_usage_rate(window);
            total_inbound_rate += peer_bandwidth.get_inbound_usage_rate(window);
        }
        
        // Calculate congestion level as the ratio of usage to allocation
        let outbound_congestion = if self.total_outbound_allocation > 0 {
            total_outbound_rate as f64 / self.total_outbound_allocation as f64
        } else {
            0.0
        };
        
        let inbound_congestion = if self.total_inbound_allocation > 0 {
            total_inbound_rate as f64 / self.total_inbound_allocation as f64
        } else {
            0.0
        };
        
        // Use the higher of the two congestion levels
        self.congestion_level = outbound_congestion.max(inbound_congestion).min(1.0);
    }
    
    /// Enqueue a message for sending
    pub fn enqueue_message(&mut self, message: Message, peer: SocketAddr, size: usize) -> Result<()> {
        // Check if the peer exists
        if !self.peer_bandwidths.contains_key(&peer) {
            return Err(Error::Network(format!("Peer {} not found in bandwidth manager", peer)));
        }
        
        // Calculate message score for prioritization
        let score = self.calculate_message_score(&message);
        
        // Create queued message
        let queued_message = QueuedMessage {
            message,
            peer,
            enqueue_time: Instant::now(),
            size,
            score,
        };
        
        // Add to queue
        self.outbound_queue.push_back(queued_message);
        
        // Sort queue by score (higher score first)
        self.sort_queue();
        
        Ok(())
    }
    
    /// Calculate a score for message prioritization
    fn calculate_message_score(&self, message: &Message) -> f64 {
        // Start with priority weight
        let priority_weight = self.config.priority_weights
            .get(&message.priority)
            .cloned()
            .unwrap_or(1.0);
        
        // Add message type weight
        let message_type_weight = self.config.message_type_weights
            .get(&message.message_type)
            .cloned()
            .unwrap_or(1.0);
        
        // Combine weights
        priority_weight * message_type_weight
    }
    
    /// Sort the outbound queue by score
    fn sort_queue(&mut self) {
        let mut vec: Vec<_> = self.outbound_queue.drain(..).collect();
        
        vec.sort_by(|a, b| {
            // Sort by score (descending)
            b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        self.outbound_queue = vec.into_iter().collect();
    }
    
    /// Get the next message to send based on bandwidth allocations
    pub fn get_next_message(&mut self) -> Option<(Message, SocketAddr)> {
        // Update allocations if needed
        self.update_allocations();
        
        // Check if queue is empty
        if self.outbound_queue.is_empty() {
            return None;
        }
        
        let now = Instant::now();
        let window = Duration::from_secs(self.config.measurement_window);
        
        // Find the first message that can be sent within bandwidth limits
        for i in 0..self.outbound_queue.len() {
            let queued_message = &self.outbound_queue[i];
            
            if let Some(peer_bandwidth) = self.peer_bandwidths.get(&queued_message.peer) {
                let current_rate = peer_bandwidth.get_outbound_usage_rate(window);
                
                // Check if sending this message would exceed the allocation
                if current_rate < peer_bandwidth.outbound_allocation {
                    // Remove the message from the queue
                    let queued_message = self.outbound_queue.remove(i).unwrap();
                    
                    // Return the message and peer
                    return Some((queued_message.message, queued_message.peer));
                }
            }
        }
        
        // If we're here, all messages would exceed bandwidth limits
        // Check if we should enter burst mode
        if !self.burst_mode && self.congestion_level > 0.9 {
            self.enter_burst_mode();
            
            // Try again with burst mode
            return self.get_next_message();
        }
        
        // No message can be sent within limits
        None
    }
    
    /// Enter burst mode for temporary bandwidth increase
    fn enter_burst_mode(&mut self) {
        self.burst_mode = true;
        self.burst_end_time = Some(Instant::now() + Duration::from_secs(30));
        
        // Update allocations with burst factor
        self.update_allocations();
        
        log::debug!("Entered bandwidth burst mode");
    }
    
    /// Get the current congestion level (0.0 - 1.0)
    pub fn get_congestion_level(&self) -> f64 {
        self.congestion_level
    }
    
    /// Get the number of queued messages
    pub fn get_queue_size(&self) -> usize {
        self.outbound_queue.len()
    }
    
    /// Get the bandwidth allocation for a peer
    pub fn get_peer_allocation(&self, address: &SocketAddr) -> Option<(u64, u64)> {
        self.peer_bandwidths.get(address).map(|pb| (pb.outbound_allocation, pb.inbound_allocation))
    }
    
    /// Get the bandwidth usage for a peer
    pub fn get_peer_usage(&self, address: &SocketAddr) -> Option<(u64, u64)> {
        let window = Duration::from_secs(self.config.measurement_window);
        
        self.peer_bandwidths.get(address).map(|pb| {
            (pb.get_outbound_usage_rate(window), pb.get_inbound_usage_rate(window))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    
    fn create_test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }
    
    fn create_test_message(message_type: MessageType, priority: Priority, size: usize) -> Message {
        Message {
            version: 1,
            compression: false,
            encryption: false,
            priority,
            message_type,
            shard_id: None,
            data: vec![0; size],
            checksum: [0; 4],
            sender: Vec::new(),
            signature: Vec::new(),
        }
    }
    
    #[test]
    fn test_bandwidth_config_default() {
        let config = BandwidthConfig::default();
        
        assert_eq!(config.max_outbound_bandwidth, 1024 * 1024);
        assert_eq!(config.max_inbound_bandwidth, 1024 * 1024);
        assert_eq!(config.burst_factor, 2.0);
        assert_eq!(config.measurement_window, 10);
        assert_eq!(config.update_interval, 5);
        assert_eq!(config.min_peer_bandwidth, 1024);
        
        // Check priority weights
        assert_eq!(*config.priority_weights.get(&Priority::High).unwrap(), 3.0);
        assert_eq!(*config.priority_weights.get(&Priority::Normal).unwrap(), 1.0);
        assert_eq!(*config.priority_weights.get(&Priority::Low).unwrap(), 0.5);
        
        // Check message type weights
        assert_eq!(*config.message_type_weights.get(&MessageType::BlockAnnouncement).unwrap(), 2.0);
    }
    
    #[test]
    fn test_peer_bandwidth_creation() {
        let addr = create_test_addr(8000);
        let peer_bandwidth = PeerBandwidth::new(addr, 1024, 1024);
        
        assert_eq!(peer_bandwidth.address, addr);
        assert_eq!(peer_bandwidth.outbound_allocation, 1024);
        assert_eq!(peer_bandwidth.inbound_allocation, 1024);
        assert_eq!(peer_bandwidth.outbound_usage, 0);
        assert_eq!(peer_bandwidth.inbound_usage, 0);
        assert_eq!(peer_bandwidth.weight, 1.0);
    }
    
    #[test]
    fn test_peer_bandwidth_record_usage() {
        let addr = create_test_addr(8000);
        let mut peer_bandwidth = PeerBandwidth::new(addr, 1024, 1024);
        
        // Record some usage
        let now = Instant::now();
        peer_bandwidth.record_usage(100, true, now);
        peer_bandwidth.record_usage(200, false, now);
        
        assert_eq!(peer_bandwidth.outbound_usage, 100);
        assert_eq!(peer_bandwidth.inbound_usage, 200);
        assert_eq!(peer_bandwidth.outbound_history.len(), 1);
        assert_eq!(peer_bandwidth.inbound_history.len(), 1);
    }
    
    #[test]
    fn test_bandwidth_manager_creation() {
        let config = BandwidthConfig::default();
        let manager = BandwidthManager::new(config);
        
        assert_eq!(manager.peer_bandwidths.len(), 0);
        assert_eq!(manager.total_outbound_allocation, 0);
        assert_eq!(manager.total_inbound_allocation, 0);
        assert_eq!(manager.outbound_queue.len(), 0);
        assert_eq!(manager.congestion_level, 0.0);
        assert!(!manager.burst_mode);
    }
    
    #[test]
    fn test_bandwidth_manager_add_remove_peer() {
        let config = BandwidthConfig::default();
        let mut manager = BandwidthManager::new(config);
        
        let addr1 = create_test_addr(8000);
        let addr2 = create_test_addr(8001);
        
        // Add peers
        manager.add_peer(addr1);
        manager.add_peer(addr2);
        
        assert_eq!(manager.peer_bandwidths.len(), 2);
        assert!(manager.peer_bandwidths.contains_key(&addr1));
        assert!(manager.peer_bandwidths.contains_key(&addr2));
        
        // Remove a peer
        manager.remove_peer(&addr1);
        
        assert_eq!(manager.peer_bandwidths.len(), 1);
        assert!(!manager.peer_bandwidths.contains_key(&addr1));
        assert!(manager.peer_bandwidths.contains_key(&addr2));
    }
    
    #[test]
    fn test_bandwidth_manager_record_usage() {
        let config = BandwidthConfig::default();
        let mut manager = BandwidthManager::new(config);
        
        let addr = create_test_addr(8000);
        
        // Add peer
        manager.add_peer(addr);
        
        // Record usage
        manager.record_usage(&addr, 100, true);
        manager.record_usage(&addr, 200, false);
        
        // Check usage
        let peer_bandwidth = manager.peer_bandwidths.get(&addr).unwrap();
        assert_eq!(peer_bandwidth.outbound_usage, 100);
        assert_eq!(peer_bandwidth.inbound_usage, 200);
    }
    
    #[test]
    fn test_bandwidth_manager_enqueue_message() {
        let config = BandwidthConfig::default();
        let mut manager = BandwidthManager::new(config);
        
        let addr = create_test_addr(8000);
        
        // Add peer
        manager.add_peer(addr);
        
        // Create message
        let message = create_test_message(MessageType::BlockAnnouncement, Priority::High, 100);
        
        // Enqueue message
        assert!(manager.enqueue_message(message, addr, 100).is_ok());
        
        assert_eq!(manager.outbound_queue.len(), 1);
    }
    
    #[test]
    fn test_bandwidth_manager_get_next_message() {
        let config = BandwidthConfig::default();
        let mut manager = BandwidthManager::new(config);
        
        let addr = create_test_addr(8000);
        
        // Add peer
        manager.add_peer(addr);
        
        // Create message
        let message = create_test_message(MessageType::BlockAnnouncement, Priority::High, 100);
        
        // Enqueue message
        assert!(manager.enqueue_message(message, addr, 100).is_ok());
        
        // Get next message
        let next = manager.get_next_message();
        assert!(next.is_some());
        
        let (next_message, next_addr) = next.unwrap();
        assert_eq!(next_message.message_type, MessageType::BlockAnnouncement);
        assert_eq!(next_message.priority, Priority::High);
        assert_eq!(next_addr, addr);
        
        // Queue should be empty now
        assert_eq!(manager.outbound_queue.len(), 0);
    }
}
