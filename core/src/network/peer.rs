//! # Peer Management
//! 
//! This module implements peer connection management for the P2P network.

use std::net::SocketAddr;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::types::Result;

/// Connection state of a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    
    /// Connection in progress
    Connecting,
    
    /// Handshake in progress
    Handshaking,
    
    /// Fully connected
    Connected,
}

/// Peer scoring levels for network quality control
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PeerScore {
    /// Banned peer
    Banned = -100,
    
    /// Peer with poor quality (slow, unreliable)
    Poor = 0,
    
    /// Average peer
    Average = 50,
    
    /// Good quality peer
    Good = 75,
    
    /// Excellent quality peer (fast, reliable)
    Excellent = 100,
}

impl Default for PeerScore {
    fn default() -> Self {
        PeerScore::Average
    }
}

/// Information about a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer network address
    pub address: SocketAddr,
    
    /// Peer node ID
    pub node_id: Vec<u8>,
    
    /// Protocol version
    pub version: u8,
    
    /// User agent string
    pub user_agent: String,
    
    /// Is this a validator node
    pub is_validator: bool,
    
    /// Last known height of peer's blockchain
    pub last_known_height: u64,
    
    /// List of shards this peer is interested in
    pub shard_subscriptions: Vec<u16>,
}

/// Peer connection tracking and statistics
#[derive(Debug)]
pub struct Peer {
    /// Peer information
    pub info: PeerInfo,
    
    /// Current connection state
    pub state: ConnectionState,
    
    /// Quality score for this peer
    pub score: PeerScore,
    
    /// Connection timestamp
    pub connected_at: Option<Instant>,
    
    /// Last ping time in milliseconds
    pub last_ping_ms: u64,
    
    /// Count of received messages
    pub messages_received: u64,
    
    /// Count of sent messages
    pub messages_sent: u64,
    
    /// Count of failed messages
    pub messages_failed: u64,
    
    /// Bytes received
    pub bytes_received: u64,
    
    /// Bytes sent
    pub bytes_sent: u64,
}

impl Peer {
    /// Create a new peer with the given info
    pub fn new(info: PeerInfo) -> Self {
        Peer {
            info,
            state: ConnectionState::Disconnected,
            score: PeerScore::default(),
            connected_at: None,
            last_ping_ms: 0,
            messages_received: 0,
            messages_sent: 0,
            messages_failed: 0,
            bytes_received: 0,
            bytes_sent: 0,
        }
    }
    
    /// Check if the peer is connected
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }
    
    /// Get the connection duration, if connected
    pub fn connection_duration(&self) -> Option<Duration> {
        self.connected_at.map(|t| t.elapsed())
    }
    
    /// Update the peer state
    pub fn update_state(&mut self, state: ConnectionState) {
        if state == ConnectionState::Connected && self.state != ConnectionState::Connected {
            self.connected_at = Some(Instant::now());
        }
        
        self.state = state;
    }
    
    /// Record a received message
    pub fn record_received(&mut self, bytes: usize) {
        self.messages_received += 1;
        self.bytes_received += bytes as u64;
    }
    
    /// Record a sent message
    pub fn record_sent(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.bytes_sent += bytes as u64;
    }
    
    /// Record a failed message
    pub fn record_failed(&mut self) {
        self.messages_failed += 1;
        
        // In a real implementation, we might adjust the score based on failed messages
        self.adjust_score(-1);
    }
    
    /// Update the ping time
    pub fn update_ping(&mut self, ping_ms: u64) {
        self.last_ping_ms = ping_ms;
        
        // Adjust score based on ping time
        if ping_ms < 50 {
            self.adjust_score(1);
        } else if ping_ms > 300 {
            self.adjust_score(-1);
        }
    }
    
    /// Adjust the peer score
    pub fn adjust_score(&mut self, delta: i32) {
        let current_score = self.score as i32;
        let new_score = (current_score + delta).clamp(-100, 100);
        
        self.score = match new_score {
            -100 => PeerScore::Banned,
            s if s < 25 => PeerScore::Poor,
            s if s < 65 => PeerScore::Average,
            s if s < 90 => PeerScore::Good,
            _ => PeerScore::Excellent,
        };
    }
    
    /// Ban this peer
    pub fn ban(&mut self) {
        self.score = PeerScore::Banned;
        self.update_state(ConnectionState::Disconnected);
    }
}

/// Peer database for tracking and scoring peers
pub struct PeerDatabase {
    /// Known peers by address
    peers: std::collections::HashMap<SocketAddr, Peer>,
    
    /// Maximum number of peers to track
    max_peers: usize,
}

impl PeerDatabase {
    /// Create a new peer database
    pub fn new(max_peers: usize) -> Self {
        PeerDatabase {
            peers: std::collections::HashMap::new(),
            max_peers,
        }
    }
    
    /// Add a peer to the database
    pub fn add_peer(&mut self, info: PeerInfo) -> Result<()> {
        let addr = info.address;
        
        if !self.peers.contains_key(&addr) {
            if self.peers.len() >= self.max_peers {
                // In a real implementation, we would evict the lowest-scored peer
                // For now, just return an error
                return Err(crate::types::Error::Network("Peer database full".to_string()));
            }
            
            self.peers.insert(addr, Peer::new(info));
        }
        
        Ok(())
    }
    
    /// Get a peer by address
    pub fn get_peer(&self, addr: &SocketAddr) -> Option<&Peer> {
        self.peers.get(addr)
    }
    
    /// Get a mutable reference to a peer by address
    pub fn get_peer_mut(&mut self, addr: &SocketAddr) -> Option<&mut Peer> {
        self.peers.get_mut(addr)
    }
    
    /// Remove a peer from the database
    pub fn remove_peer(&mut self, addr: &SocketAddr) -> Option<Peer> {
        self.peers.remove(addr)
    }
    
    /// Get all peers
    pub fn get_all_peers(&self) -> impl Iterator<Item = &Peer> {
        self.peers.values()
    }
    
    /// Get all connected peers
    pub fn get_connected_peers(&self) -> impl Iterator<Item = &Peer> {
        self.peers.values().filter(|p| p.is_connected())
    }
    
    /// Get peers with a minimum score
    pub fn get_peers_with_min_score(&self, min_score: PeerScore) -> impl Iterator<Item = &Peer> {
        self.peers.values().filter(move |p| p.score >= min_score)
    }
    
    /// Count all peers
    pub fn count_all(&self) -> usize {
        self.peers.len()
    }
    
    /// Count connected peers
    pub fn count_connected(&self) -> usize {
        self.peers.values().filter(|p| p.is_connected()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    
    fn create_test_peer_info(port: u16) -> PeerInfo {
        PeerInfo {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
            node_id: vec![port as u8, (port >> 8) as u8, 0, 0],
            version: 1,
            user_agent: format!("test-peer-{}", port),
            is_validator: false,
            last_known_height: 0,
            shard_subscriptions: Vec::new(),
        }
    }
    
    #[test]
    fn test_peer_creation() {
        let info = create_test_peer_info(8000);
        let peer = Peer::new(info.clone());
        
        assert_eq!(peer.info.address, info.address);
        assert_eq!(peer.state, ConnectionState::Disconnected);
        assert_eq!(peer.score, PeerScore::Average);
        assert_eq!(peer.messages_received, 0);
    }
    
    #[test]
    fn test_peer_update_state() {
        let info = create_test_peer_info(8001);
        let mut peer = Peer::new(info);
        
        // Initially disconnected
        assert_eq!(peer.state, ConnectionState::Disconnected);
        assert!(peer.connected_at.is_none());
        
        // Update to connecting
        peer.update_state(ConnectionState::Connecting);
        assert_eq!(peer.state, ConnectionState::Connecting);
        assert!(peer.connected_at.is_none());
        
        // Update to connected
        peer.update_state(ConnectionState::Connected);
        assert_eq!(peer.state, ConnectionState::Connected);
        assert!(peer.connected_at.is_some());
        
        // Should have a connection duration now
        assert!(peer.connection_duration().is_some());
    }
    
    #[test]
    fn test_peer_scoring() {
        let info = create_test_peer_info(8002);
        let mut peer = Peer::new(info);
        
        // Initial score is Average
        assert_eq!(peer.score, PeerScore::Average);
        
        // Improve score
        peer.adjust_score(30);
        assert_eq!(peer.score, PeerScore::Good);
        
        // Improve more
        peer.adjust_score(30);
        assert_eq!(peer.score, PeerScore::Excellent);
        
        // Lower score
        peer.adjust_score(-60);
        assert_eq!(peer.score, PeerScore::Average);
        
        // Lower more
        peer.adjust_score(-60);
        assert_eq!(peer.score, PeerScore::Poor);
        
        // Ban peer
        peer.ban();
        assert_eq!(peer.score, PeerScore::Banned);
        assert_eq!(peer.state, ConnectionState::Disconnected);
    }
    
    #[test]
    fn test_peer_database() {
        let mut db = PeerDatabase::new(10);
        
        // Add some peers
        let info1 = create_test_peer_info(8010);
        let info2 = create_test_peer_info(8011);
        let info3 = create_test_peer_info(8012);
        
        db.add_peer(info1.clone()).unwrap();
        db.add_peer(info2.clone()).unwrap();
        db.add_peer(info3.clone()).unwrap();
        
        assert_eq!(db.count_all(), 3);
        assert_eq!(db.count_connected(), 0);
        
        // Connect a peer
        let peer = db.get_peer_mut(&info1.address).unwrap();
        peer.update_state(ConnectionState::Connected);
        
        assert_eq!(db.count_connected(), 1);
        
        // Get connected peers
        let connected: Vec<_> = db.get_connected_peers().collect();
        assert_eq!(connected.len(), 1);
        assert_eq!(connected[0].info.address, info1.address);
    }
}
