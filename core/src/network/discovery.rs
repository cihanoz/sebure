//! # Node Discovery
//! 
//! This module implements peer discovery mechanisms for the P2P network.

use std::collections::HashSet;
use std::net::{SocketAddr, IpAddr};
use std::time::{Duration, Instant};

use crate::network::{Peer, PeerInfo, NetworkConfig, Message, MessageType};
use crate::types::{Result, Error};

/// Discovery methods for finding peers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiscoveryMethod {
    /// Manual configuration (bootstrap peers)
    Manual,
    
    /// DNS seeds
    DnsSeed,
    
    /// Peer exchange
    PeerExchange,
    
    /// Local network discovery
    LocalDiscovery,
}

/// Configuration for peer discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Enabled discovery methods
    pub methods: Vec<DiscoveryMethod>,
    
    /// DNS seed hostnames
    pub dns_seeds: Vec<String>,
    
    /// Peer exchange interval in seconds
    pub peer_exchange_interval: u64,
    
    /// Maximum peers to exchange
    pub max_peers_to_exchange: usize,
    
    /// Local discovery port
    pub local_discovery_port: u16,
    
    /// Maximum peers to discover
    pub max_discovery_peers: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        DiscoveryConfig {
            methods: vec![DiscoveryMethod::Manual, DiscoveryMethod::PeerExchange],
            dns_seeds: Vec::new(),
            peer_exchange_interval: 300, // 5 minutes
            max_peers_to_exchange: 10,
            local_discovery_port: 8766,
            max_discovery_peers: 100,
        }
    }
}

/// PeerDiscovery manages the discovery of new peers
pub struct PeerDiscovery {
    /// Discovery configuration
    config: DiscoveryConfig,
    
    /// Network configuration
    network_config: NetworkConfig,
    
    /// Discovered peers
    discovered_peers: HashSet<SocketAddr>,
    
    /// Last discovery times by method
    last_discovery: std::collections::HashMap<DiscoveryMethod, Instant>,
    
    /// Running state
    running: bool,
}

impl PeerDiscovery {
    /// Create a new peer discovery instance
    pub fn new(config: DiscoveryConfig, network_config: NetworkConfig) -> Self {
        let mut last_discovery = std::collections::HashMap::new();
        
        // Initialize last discovery times to now minus the interval
        // so that discovery will trigger immediately on start
        let now = Instant::now();
        for method in &config.methods {
            last_discovery.insert(*method, now - Duration::from_secs(config.peer_exchange_interval));
        }
        
        PeerDiscovery {
            config,
            network_config,
            discovered_peers: HashSet::new(),
            last_discovery,
            running: false,
        }
    }
    
    /// Start the discovery process
    pub fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(Error::Network("Discovery already running".to_string()));
        }
        
        self.running = true;
        
        // Add bootstrap peers to discovered list
        for peer in &self.network_config.bootstrap_peers {
            self.discovered_peers.insert(*peer);
        }
        
        log::info!("Peer discovery started with {} methods", self.config.methods.len());
        
        Ok(())
    }
    
    /// Stop the discovery process
    pub fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Err(Error::Network("Discovery not running".to_string()));
        }
        
        self.running = false;
        
        log::info!("Peer discovery stopped");
        
        Ok(())
    }
    
    /// Process a discovery cycle
    pub fn process(&mut self) -> Result<Vec<SocketAddr>> {
        if !self.running {
            return Ok(Vec::new());
        }
        
        let mut new_peers = Vec::new();
        let now = Instant::now();
        
        // Process each discovery method if it's time
        for method in &self.config.methods {
            let last_time = self.last_discovery.get(method).unwrap_or(&now).clone();
            
            let interval = match method {
                DiscoveryMethod::Manual => Duration::from_secs(3600), // Once per hour
                DiscoveryMethod::DnsSeed => Duration::from_secs(1800), // 30 minutes
                DiscoveryMethod::PeerExchange => Duration::from_secs(self.config.peer_exchange_interval),
                DiscoveryMethod::LocalDiscovery => Duration::from_secs(60), // Every minute
            };
            
            if now.duration_since(last_time) >= interval {
                let mut discovered = self.discover_with_method(*method)?;
                new_peers.append(&mut discovered);
                
                self.last_discovery.insert(*method, now);
            }
        }
        
        // Filter out already known peers
        new_peers.retain(|addr| !self.discovered_peers.contains(addr));
        
        // Add new peers to the discovered set
        for peer in &new_peers {
            self.discovered_peers.insert(*peer);
        }
        
        // Limit discovery set size
        if self.discovered_peers.len() > self.config.max_discovery_peers {
            // This is not the most efficient way, but it works for now
            let mut peers: Vec<_> = self.discovered_peers.iter().cloned().collect();
            peers.sort(); // Sort for deterministic behavior
            peers.truncate(self.config.max_discovery_peers);
            
            self.discovered_peers = peers.into_iter().collect();
        }
        
        Ok(new_peers)
    }
    
    /// Discover peers using a specific method
    fn discover_with_method(&self, method: DiscoveryMethod) -> Result<Vec<SocketAddr>> {
        match method {
            DiscoveryMethod::Manual => {
                // Manual discovery just returns bootstrap peers
                Ok(self.network_config.bootstrap_peers.clone())
            },
            
            DiscoveryMethod::DnsSeed => {
                self.discover_from_dns_seeds()
            },
            
            DiscoveryMethod::PeerExchange => {
                // This would be implemented in a real system by
                // requesting peers from connected nodes
                Ok(Vec::new())
            },
            
            DiscoveryMethod::LocalDiscovery => {
                self.discover_local_network()
            },
        }
    }
    
    /// Discover peers from DNS seeds
    fn discover_from_dns_seeds(&self) -> Result<Vec<SocketAddr>> {
        let mut discovered = Vec::new();
        
        for seed in &self.config.dns_seeds {
            match std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:0", seed)) {
                Ok(addrs) => {
                    for addr in addrs {
                        if let IpAddr::V4(ipv4) = addr.ip() {
                            // Use the default port
                            let port = self.network_config.listen_addr.port();
                            discovered.push(SocketAddr::new(IpAddr::V4(ipv4), port));
                        }
                    }
                },
                Err(e) => {
                    log::warn!("Failed to resolve DNS seed {}: {}", seed, e);
                }
            }
        }
        
        Ok(discovered)
    }
    
    /// Discover peers on the local network
    fn discover_local_network(&self) -> Result<Vec<SocketAddr>> {
        // This would use broadcast or multicast to discover peers on the local network
        // For now, just return an empty list
        Ok(Vec::new())
    }
    
    /// Handle a peer exchange message
    pub fn handle_peer_exchange(&mut self, peers: Vec<SocketAddr>) -> Result<()> {
        if !self.running {
            return Ok(());
        }
        
        for peer in peers {
            self.discovered_peers.insert(peer);
        }
        
        Ok(())
    }
    
    /// Create a peer exchange message
    pub fn create_peer_exchange(&self, connected_peers: &[&Peer]) -> Result<Message> {
        // Select a subset of connected peers to exchange
        let mut selected = Vec::new();
        let mut rng = rand::thread_rng();
        
        let peers: Vec<_> = connected_peers.iter().collect();
        if !peers.is_empty() {
            use rand::seq::SliceRandom;
            let subset_count = std::cmp::min(self.config.max_peers_to_exchange, peers.len());
            
            for peer in peers.choose_multiple(&mut rng, subset_count) {
                selected.push(peer.info.address);
            }
        }
        
        // Serialize the peer list
        let data = bincode::serialize(&selected)?;
        
        // Create the message
        Ok(Message::new(
            MessageType::PeerExchange,
            data,
            None,
            crate::types::Priority::Low,
            Vec::new(), // The node ID will be filled in by the sender
        ))
    }
    
    /// Get the number of discovered peers
    pub fn discovered_count(&self) -> usize {
        self.discovered_peers.len()
    }
    
    /// Get all discovered peers
    pub fn get_all_discovered(&self) -> impl Iterator<Item = &SocketAddr> {
        self.discovered_peers.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    
    fn create_test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }
    
    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        
        assert_eq!(config.methods.len(), 2);
        assert!(config.methods.contains(&DiscoveryMethod::Manual));
        assert!(config.methods.contains(&DiscoveryMethod::PeerExchange));
        assert_eq!(config.peer_exchange_interval, 300);
    }
    
    #[test]
    fn test_discovery_creation() {
        let discovery_config = DiscoveryConfig::default();
        let mut network_config = NetworkConfig::default();
        
        // Add some bootstrap peers
        network_config.bootstrap_peers = vec![
            create_test_addr(9000),
            create_test_addr(9001),
        ];
        
        let discovery = PeerDiscovery::new(discovery_config, network_config);
        
        assert_eq!(discovery.discovered_peers.len(), 0);
        assert!(!discovery.running);
    }
    
    #[test]
    fn test_discovery_start_stop() {
        let discovery_config = DiscoveryConfig::default();
        let mut network_config = NetworkConfig::default();
        
        // Add some bootstrap peers
        network_config.bootstrap_peers = vec![
            create_test_addr(9000),
            create_test_addr(9001),
        ];
        
        let mut discovery = PeerDiscovery::new(discovery_config, network_config);
        
        // Start discovery
        assert!(discovery.start().is_ok());
        assert!(discovery.running);
        
        // Bootstrap peers should be in discovered list
        assert_eq!(discovery.discovered_peers.len(), 2);
        
        // Stop discovery
        assert!(discovery.stop().is_ok());
        assert!(!discovery.running);
    }
    
    #[test]
    fn test_discovery_process() {
        let discovery_config = DiscoveryConfig::default();
        let mut network_config = NetworkConfig::default();
        
        // Add some bootstrap peers
        network_config.bootstrap_peers = vec![
            create_test_addr(9000),
            create_test_addr(9001),
        ];
        
        let mut discovery = PeerDiscovery::new(discovery_config, network_config);
        
        // Start discovery
        assert!(discovery.start().is_ok());
        
        // Process discovery - should return bootstrap peers initially
        let new_peers = discovery.process().unwrap();
        assert_eq!(new_peers.len(), 2);
        
        // Process again - should not return any new peers
        let new_peers = discovery.process().unwrap();
        assert_eq!(new_peers.len(), 0);
    }
}
