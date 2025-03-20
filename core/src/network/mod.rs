//! # Network Layer
//! 
//! This module implements the P2P networking layer for the SEBURE blockchain.
//! It provides peer discovery, message propagation, and network topology management.

mod message;
mod peer;
mod protocol;
mod discovery;
mod transport;
mod node_communication;

// Re-export main types
pub use message::Message;
pub use message::MessageType;
pub use peer::Peer;
pub use peer::PeerInfo;
pub use peer::ConnectionState;
pub use peer::PeerScore;
pub use protocol::Protocol;
pub use protocol::ProtocolConfig;
pub use discovery::PeerDiscovery;
pub use discovery::DiscoveryConfig;
pub use discovery::DiscoveryMethod;
pub use transport::Transport;
pub use transport::TransportConfig;
pub use node_communication::NodeCommunication;
pub use node_communication::BlockPropagationConfig;
pub use node_communication::TransactionBroadcastConfig;

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
    
    /// Peer discovery
    discovery: Option<PeerDiscovery>,
    
    /// Transport layer
    transport: Option<Arc<Transport>>,
    
    /// Node communication
    communication: Option<Arc<NodeCommunication>>,
}

impl Network {
    /// Create a new network instance with the provided configuration
    pub fn new(config: NetworkConfig) -> Self {
        Network {
            config,
            peers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            discovery: None,
            transport: None,
            communication: None,
        }
    }
    
    /// Start the network service
    pub fn start(&mut self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::Network("Network already running".to_string()));
        }
        
        *running = true;
        
        // Create protocol instance
        let protocol = Protocol::new(
            ProtocolConfig::default(),
            "sebure-mainnet".to_string(), // Network ID
            vec![1, 2, 3, 4], // Genesis hash (placeholder)
            vec![5, 6, 7, 8], // Node ID (placeholder)
            format!("sebure/{}", env!("CARGO_PKG_VERSION")), // User agent
        );
        
        // Start transport layer
        let mut transport = Transport::new(TransportConfig::default(), protocol);
        transport.start(self.config.listen_addr)?;
        let transport_arc = Arc::new(transport);
        self.transport = Some(transport_arc.clone());
        
        // Start peer discovery
        let discovery_config = DiscoveryConfig {
            dns_seeds: vec!["seed1.sebure.network".to_string(), "seed2.sebure.network".to_string()],
            ..DiscoveryConfig::default()
        };
        
        let mut discovery = PeerDiscovery::new(discovery_config, self.config.clone());
        discovery.start()?;
        self.discovery = Some(discovery);
        
        // Start node communication
        let communication = NodeCommunication::new(
            BlockPropagationConfig::default(),
            TransactionBroadcastConfig::default(),
            transport_arc,
        );
        communication.start()?;
        self.communication = Some(Arc::new(communication));
        
        log::info!("Network started, listening on {}", self.config.listen_addr);
        
        Ok(())
    }
    
    /// Stop the network service
    pub fn stop(&mut self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err(Error::Network("Network not running".to_string()));
        }
        
        *running = false;
        
        // Stop node communication
        if let Some(comm) = &self.communication {
            if let Err(e) = comm.stop() {
                log::error!("Error stopping node communication: {}", e);
            }
        }
        self.communication = None;
        
        // Stop peer discovery
        if let Some(discovery) = &mut self.discovery {
            if let Err(e) = discovery.stop() {
                log::error!("Error stopping peer discovery: {}", e);
            }
        }
        self.discovery = None;
        
        // Stop transport
        if let Some(transport) = &mut self.transport.take() {
            if let Ok(transport) = Arc::try_unwrap(transport.clone()) {
                let mut transport = transport;
                if let Err(e) = transport.stop() {
                    log::error!("Error stopping transport: {}", e);
                }
            }
        }
        
        // Clear peer list
        self.peers.lock().unwrap().clear();
        
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
        
        if let Some(transport) = &self.transport {
            // Get the addresses of all connected peers
            let peer_addrs: Vec<SocketAddr> = peers.keys().cloned().collect();
            
            for addr in peer_addrs {
                if let Err(e) = transport.send(&addr, &message) {
                    log::warn!("Failed to send message to {}: {:?}", addr, e);
                }
            }
            
            log::debug!("Broadcast message of type {:?} to {} peers", 
                      message.message_type, peers.len());
            
            Ok(())
        } else {
            Err(Error::Network("Transport not initialized".to_string()))
        }
    }
    
    /// Send a message to a specific peer
    pub fn send_to_peer(&self, peer_addr: &SocketAddr, message: Message) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        if !peers.contains_key(peer_addr) {
            return Err(Error::Network(format!("Peer {} not connected", peer_addr)));
        }
        
        if let Some(transport) = &self.transport {
            if let Err(e) = transport.send(peer_addr, &message) {
                log::warn!("Failed to send message to {}: {:?}", peer_addr, e);
                return Err(Error::Network(format!("Failed to send message: {:?}", e)));
            }
            
            log::debug!("Sent message of type {:?} to peer {}", 
                      message.message_type, peer_addr);
            
            Ok(())
        } else {
            Err(Error::Network("Transport not initialized".to_string()))
        }
    }
    
    /// Process network events and maintain connections
    pub fn process(&mut self) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Ok(());
        }
        
        // Process peer discovery
        if let Some(discovery) = &mut self.discovery {
            let new_peers = discovery.process()?;
            
            // Connect to new peers
            for peer_addr in new_peers {
                self.connect_to_peer(peer_addr)?;
            }
        }
        
        // In a real implementation, we would:
        // 1. Process incoming messages
        // 2. Handle timeouts and reconnects
        // 3. Perform peer maintenance
        
        Ok(())
    }
    
    /// Connect to a specific peer
    pub fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        let mut peers = self.peers.lock().unwrap();
        
        // Check if already connected
        if peers.contains_key(&addr) {
            return Ok(());
        }
        
        // Check if max peers reached
        if peers.len() >= self.config.max_peers {
            return Err(Error::Network("Max peer limit reached".to_string()));
        }
        
        // Establish connection
        if let Some(transport) = &self.transport {
            if let Err(e) = transport.connect(addr) {
                return Err(Error::Network(format!("Failed to connect to {}: {:?}", addr, e)));
            }
            
            // Create peer info
            let info = PeerInfo {
                address: addr,
                node_id: Vec::new(), // Will be filled after handshake
                version: 1,
                user_agent: String::new(), // Will be filled after handshake
                is_validator: false,
                last_known_height: 0,
                shard_subscriptions: Vec::new(),
            };
            
            // Create and add peer
            let mut peer = Peer::new(info);
            peer.update_state(ConnectionState::Connected);
            
            peers.insert(addr, peer);
            
            log::info!("Connected to peer {}", addr);
            
            Ok(())
        } else {
            Err(Error::Network("Transport not initialized".to_string()))
        }
    }
    
    /// Disconnect from a specific peer
    pub fn disconnect_from_peer(&self, addr: &SocketAddr) -> Result<()> {
        let mut peers = self.peers.lock().unwrap();
        
        if !peers.contains_key(addr) {
            return Err(Error::Network(format!("Peer {} not connected", addr)));
        }
        
        // Remove from transport
        if let Some(transport) = &self.transport {
            if let Err(e) = transport.disconnect(addr) {
                log::warn!("Error disconnecting from {}: {:?}", addr, e);
            }
        }
        
        // Clear from communication service
        if let Some(comm) = &self.communication {
            comm.clear_peer(addr);
        }
        
        // Remove from peers list
        peers.remove(addr);
        
        log::info!("Disconnected from peer {}", addr);
        
        Ok(())
    }
    
    /// Announce a new block to all connected peers
    pub fn announce_block(&self, block: &crate::blockchain::Block) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Network not running".to_string()));
        }
        
        if let Some(comm) = &self.communication {
            let peers = self.peers.lock().unwrap();
            let peer_addrs: Vec<SocketAddr> = peers.keys().cloned().collect();
            
            if !peer_addrs.is_empty() {
                comm.announce_block(block, &peer_addrs)?;
            }
            
            Ok(())
        } else {
            Err(Error::Network("Communication service not initialized".to_string()))
        }
    }
    
    /// Broadcast transactions to all connected peers
    pub fn broadcast_transactions(&self, transactions: &[crate::blockchain::Transaction]) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Network not running".to_string()));
        }
        
        if let Some(comm) = &self.communication {
            let peers = self.peers.lock().unwrap();
            let peer_addrs: Vec<SocketAddr> = peers.keys().cloned().collect();
            
            if !peer_addrs.is_empty() {
                comm.broadcast_transactions(transactions, &peer_addrs)?;
            }
            
            Ok(())
        } else {
            Err(Error::Network("Communication service not initialized".to_string()))
        }
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
