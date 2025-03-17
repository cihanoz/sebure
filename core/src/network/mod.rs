//! # Network Layer
//! 
//! This module implements the P2P networking layer for the SEBURE blockchain.
//! It provides peer discovery, message propagation, and network topology management.

mod message;
mod peer;
mod protocol;

// Re-export main types
pub use message::Message;
pub use message::MessageType;
pub use peer::Peer;
pub use peer::PeerInfo;
pub use protocol::Protocol;

use crate::types::{Result, Error};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Network configuration options
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Local listen address
    pub listen_addr: SocketAddr,
    
    /// Known peer addresses to connect to
    pub bootstrap_peers: Vec<SocketAddr>,
    
    /// Maximum number of peers to connect to
    pub max_peers: usize,
    
    /// Peer announcement interval in seconds
    pub announce_interval: u64,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_addr: "127.0.0.1:8765".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            max_peers: 25,
            announce_interval: 300, // 5 minutes
            connection_timeout: 10,
        }
    }
}

/// Network state management
pub struct Network {
    /// Network configuration
    config: NetworkConfig,
    
    /// Connected peers
    peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    
    /// Running flag
    running: Arc<Mutex<bool>>,
}

impl Network {
    /// Create a new network instance with the provided configuration
    pub fn new(config: NetworkConfig) -> Self {
        Network {
            config,
            peers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Start the network service
    pub fn start(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::Network("Network already running".to_string()));
        }
        
        *running = true;
        
        // In a real implementation, we would:
        // 1. Start the listener
        // 2. Connect to bootstrap peers
        // 3. Start peer discovery
        // 4. Start message handling
        
        log::info!("Network started, listening on {}", self.config.listen_addr);
        
        Ok(())
    }
    
    /// Stop the network service
    pub fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err(Error::Network("Network not running".to_string()));
        }
        
        *running = false;
        
        // In a real implementation, we would:
        // 1. Disconnect from all peers
        // 2. Stop the listener
        // 3. Clean up resources
        
        log::info!("Network stopped");
        
        Ok(())
    }
    
    /// Get the number of connected peers
    pub fn peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }
    
    /// Broadcast a message to all connected peers
    pub fn broadcast(&self, message: Message) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        if peers.is_empty() {
            return Err(Error::Network("No peers connected".to_string()));
        }
        
        // In a real implementation, we would send the message to all peers
        log::debug!("Broadcasting message of type {:?} to {} peers", 
                   message.message_type, peers.len());
        
        Ok(())
    }
    
    /// Send a message to a specific peer
    pub fn send_to_peer(&self, peer_addr: &SocketAddr, message: Message) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        if !peers.contains_key(peer_addr) {
            return Err(Error::Network(format!("Peer {} not connected", peer_addr)));
        }
        
        // In a real implementation, we would send the message to the specific peer
        log::debug!("Sending message of type {:?} to peer {}", 
                   message.message_type, peer_addr);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    
    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.listen_addr.to_string(), "127.0.0.1:8765");
        assert_eq!(config.max_peers, 25);
    }
    
    #[test]
    fn test_network_start_stop() {
        let network = Network::new(NetworkConfig::default());
        
        // Start the network
        assert!(network.start().is_ok());
        
        // Starting again should fail
        assert!(network.start().is_err());
        
        // Stop the network
        assert!(network.stop().is_ok());
        
        // Stopping again should fail
        assert!(network.stop().is_err());
    }
    
    #[test]
    fn test_network_peer_count() {
        let network = Network::new(NetworkConfig::default());
        
        // Initially no peers
        assert_eq!(network.peer_count(), 0);
    }
}
