//! # Mesh Network Topology
//! 
//! This module implements a mesh network topology for the P2P network,
//! providing efficient and resilient peer connections.

use std::collections::{HashMap, HashSet, BTreeMap};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::network::{Peer, PeerInfo, PeerScore};
use crate::types::{Result, Error};

/// Mesh topology configuration
#[derive(Debug, Clone)]
pub struct MeshTopologyConfig {
    /// Minimum number of outbound connections
    pub min_outbound_connections: usize,
    
    /// Maximum number of outbound connections
    pub max_outbound_connections: usize,
    
    /// Maximum number of inbound connections
    pub max_inbound_connections: usize,
    
    /// Number of connections to maintain per region
    pub connections_per_region: usize,
    
    /// Connection retry interval in seconds
    pub connection_retry_interval: u64,
    
    /// Topology optimization interval in seconds
    pub optimization_interval: u64,
    
    /// Peer scoring weight for latency
    pub latency_score_weight: f32,
    
    /// Peer scoring weight for uptime
    pub uptime_score_weight: f32,
    
    /// Peer scoring weight for bandwidth
    pub bandwidth_score_weight: f32,
}

impl Default for MeshTopologyConfig {
    fn default() -> Self {
        MeshTopologyConfig {
            min_outbound_connections: 8,
            max_outbound_connections: 16,
            max_inbound_connections: 128,
            connections_per_region: 2,
            connection_retry_interval: 60,
            optimization_interval: 300,
            latency_score_weight: 0.4,
            uptime_score_weight: 0.3,
            bandwidth_score_weight: 0.3,
        }
    }
}

/// Geographic region for network topology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Region {
    /// North America
    NorthAmerica,
    
    /// South America
    SouthAmerica,
    
    /// Europe
    Europe,
    
    /// Asia
    Asia,
    
    /// Africa
    Africa,
    
    /// Oceania
    Oceania,
    
    /// Unknown region
    Unknown,
}

impl Region {
    /// Estimate region from IP address
    pub fn from_ip(addr: &SocketAddr) -> Self {
        // In a real implementation, we would use a GeoIP database
        // For now, just use a simple heuristic based on the IP
        match addr.ip() {
            std::net::IpAddr::V4(ip) => {
                let octets = ip.octets();
                match octets[0] {
                    0..=127 => Region::NorthAmerica,
                    128..=191 => Region::Europe,
                    192..=223 => Region::Asia,
                    _ => Region::Unknown,
                }
            },
            std::net::IpAddr::V6(_) => Region::Unknown,
        }
    }
}

/// Connection direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionDirection {
    /// Inbound connection (peer connected to us)
    Inbound,
    
    /// Outbound connection (we connected to peer)
    Outbound,
}

/// Peer connection information for mesh topology
#[derive(Debug)]
pub struct MeshPeerInfo {
    /// Peer address
    pub address: SocketAddr,
    
    /// Connection direction
    pub direction: ConnectionDirection,
    
    /// Geographic region
    pub region: Region,
    
    /// Connection time
    pub connected_at: Instant,
    
    /// Last ping time in milliseconds
    pub last_ping_ms: u64,
    
    /// Bytes sent
    pub bytes_sent: u64,
    
    /// Bytes received
    pub bytes_received: u64,
    
    /// Connection score (higher is better)
    pub score: f32,
}

impl MeshPeerInfo {
    /// Create a new mesh peer info
    pub fn new(address: SocketAddr, direction: ConnectionDirection) -> Self {
        MeshPeerInfo {
            address,
            direction,
            region: Region::from_ip(&address),
            connected_at: Instant::now(),
            last_ping_ms: 0,
            bytes_sent: 0,
            bytes_received: 0,
            score: 0.0,
        }
    }
    
    /// Update the peer score based on metrics
    pub fn update_score(&mut self, config: &MeshTopologyConfig) {
        // Calculate latency score (lower ping is better)
        let latency_score = if self.last_ping_ms == 0 {
            0.5 // Default if no ping yet
        } else if self.last_ping_ms < 50 {
            1.0
        } else if self.last_ping_ms < 100 {
            0.8
        } else if self.last_ping_ms < 200 {
            0.6
        } else if self.last_ping_ms < 500 {
            0.3
        } else {
            0.1
        };
        
        // Calculate uptime score (longer is better)
        let uptime_secs = self.connected_at.elapsed().as_secs();
        let uptime_score = if uptime_secs < 60 {
            0.2
        } else if uptime_secs < 300 {
            0.4
        } else if uptime_secs < 1800 {
            0.6
        } else if uptime_secs < 7200 {
            0.8
        } else {
            1.0
        };
        
        // Calculate bandwidth score (more is better)
        let total_bytes = self.bytes_sent + self.bytes_received;
        let bandwidth_score = if total_bytes < 1024 {
            0.1
        } else if total_bytes < 1024 * 10 {
            0.3
        } else if total_bytes < 1024 * 100 {
            0.5
        } else if total_bytes < 1024 * 1000 {
            0.8
        } else {
            1.0
        };
        
        // Combine scores with weights
        self.score = 
            latency_score * config.latency_score_weight +
            uptime_score * config.uptime_score_weight +
            bandwidth_score * config.bandwidth_score_weight;
    }
}

/// Mesh network topology manager
pub struct MeshTopology {
    /// Topology configuration
    config: MeshTopologyConfig,
    
    /// Connected peers by address
    peers: HashMap<SocketAddr, MeshPeerInfo>,
    
    /// Peers by region
    peers_by_region: HashMap<Region, HashSet<SocketAddr>>,
    
    /// Inbound connection count
    inbound_count: usize,
    
    /// Outbound connection count
    outbound_count: usize,
    
    /// Last optimization time
    last_optimization: Instant,
    
    /// Peer connection attempts
    connection_attempts: HashMap<SocketAddr, Instant>,
}

impl MeshTopology {
    /// Create a new mesh topology manager
    pub fn new(config: MeshTopologyConfig) -> Self {
        MeshTopology {
            config,
            peers: HashMap::new(),
            peers_by_region: HashMap::new(),
            inbound_count: 0,
            outbound_count: 0,
            last_optimization: Instant::now(),
            connection_attempts: HashMap::new(),
        }
    }
    
    /// Add a peer to the topology
    pub fn add_peer(&mut self, address: SocketAddr, direction: ConnectionDirection) -> Result<()> {
        // Check if already connected
        if self.peers.contains_key(&address) {
            return Ok(());
        }
        
        // Check connection limits
        match direction {
            ConnectionDirection::Inbound => {
                if self.inbound_count >= self.config.max_inbound_connections {
                    return Err(Error::Network("Max inbound connections reached".to_string()));
                }
                self.inbound_count += 1;
            },
            ConnectionDirection::Outbound => {
                if self.outbound_count >= self.config.max_outbound_connections {
                    return Err(Error::Network("Max outbound connections reached".to_string()));
                }
                self.outbound_count += 1;
            },
        }
        
        // Create peer info
        let peer_info = MeshPeerInfo::new(address, direction);
        let region = peer_info.region;
        
        // Add to peers map
        self.peers.insert(address, peer_info);
        
        // Add to region map
        self.peers_by_region
            .entry(region)
            .or_insert_with(HashSet::new)
            .insert(address);
        
        log::debug!("Added peer {} to mesh topology (direction: {:?}, region: {:?})",
                  address, direction, region);
        
        Ok(())
    }
    
    /// Remove a peer from the topology
    pub fn remove_peer(&mut self, address: &SocketAddr) -> Result<()> {
        if let Some(peer_info) = self.peers.remove(address) {
            // Update connection counts
            match peer_info.direction {
                ConnectionDirection::Inbound => self.inbound_count -= 1,
                ConnectionDirection::Outbound => self.outbound_count -= 1,
            }
            
            // Remove from region map
            if let Some(region_peers) = self.peers_by_region.get_mut(&peer_info.region) {
                region_peers.remove(address);
                
                // Clean up empty region sets
                if region_peers.is_empty() {
                    self.peers_by_region.remove(&peer_info.region);
                }
            }
            
            log::debug!("Removed peer {} from mesh topology", address);
            
            Ok(())
        } else {
            Err(Error::Network(format!("Peer {} not found in topology", address)))
        }
    }
    
    /// Update peer metrics
    pub fn update_peer_metrics(
        &mut self,
        address: &SocketAddr,
        ping_ms: Option<u64>,
        bytes_sent: Option<u64>,
        bytes_received: Option<u64>,
    ) -> Result<()> {
        if let Some(peer_info) = self.peers.get_mut(address) {
            if let Some(ping) = ping_ms {
                peer_info.last_ping_ms = ping;
            }
            
            if let Some(sent) = bytes_sent {
                peer_info.bytes_sent = sent;
            }
            
            if let Some(received) = bytes_received {
                peer_info.bytes_received = received;
            }
            
            // Update score
            peer_info.update_score(&self.config);
            
            Ok(())
        } else {
            Err(Error::Network(format!("Peer {} not found in topology", address)))
        }
    }
    
    /// Get peers to connect to for optimal topology
    pub fn get_connection_candidates(&mut self, available_peers: &[SocketAddr]) -> Vec<SocketAddr> {
        // If we have enough outbound connections, no need for more
        if self.outbound_count >= self.config.min_outbound_connections {
            return Vec::new();
        }
        
        let mut candidates = Vec::new();
        let now = Instant::now();
        
        // Group available peers by region
        let mut peers_by_region: HashMap<Region, Vec<SocketAddr>> = HashMap::new();
        
        for &addr in available_peers {
            // Skip if already connected
            if self.peers.contains_key(&addr) {
                continue;
            }
            
            // Skip if recently attempted
            if let Some(attempt_time) = self.connection_attempts.get(&addr) {
                if now.duration_since(*attempt_time) < Duration::from_secs(self.config.connection_retry_interval) {
                    continue;
                }
            }
            
            // Add to region map
            let region = Region::from_ip(&addr);
            peers_by_region.entry(region).or_insert_with(Vec::new).push(addr);
        }
        
        // Calculate how many connections we need per region
        let needed_connections = self.config.min_outbound_connections - self.outbound_count;
        let mut connections_per_region = HashMap::new();
        
        // First, count existing connections per region
        for (region, peers) in &self.peers_by_region {
            let outbound_count = peers.iter()
                .filter(|&addr| {
                    self.peers.get(addr)
                        .map(|p| p.direction == ConnectionDirection::Outbound)
                        .unwrap_or(false)
                })
                .count();
            
            connections_per_region.insert(*region, outbound_count);
        }
        
        // Prioritize regions with fewer connections
        let mut regions: Vec<_> = Region::all().collect();
        regions.sort_by_key(|r| connections_per_region.get(r).cloned().unwrap_or(0));
        
        // Select candidates from each region
        for region in regions {
            if candidates.len() >= needed_connections {
                break;
            }
            
            if let Some(region_peers) = peers_by_region.get(&region) {
                let current_count = connections_per_region.get(&region).cloned().unwrap_or(0);
                let target_count = self.config.connections_per_region;
                
                if current_count < target_count {
                    let needed = std::cmp::min(
                        target_count - current_count,
                        needed_connections - candidates.len()
                    );
                    
                    // Take up to 'needed' peers from this region
                    for &addr in region_peers.iter().take(needed) {
                        candidates.push(addr);
                        self.connection_attempts.insert(addr, now);
                    }
                }
            }
        }
        
        // If we still need more connections, take any available peers
        if candidates.len() < needed_connections {
            for region_peers in peers_by_region.values() {
                for &addr in region_peers {
                    if !candidates.contains(&addr) {
                        candidates.push(addr);
                        self.connection_attempts.insert(addr, now);
                        
                        if candidates.len() >= needed_connections {
                            break;
                        }
                    }
                }
                
                if candidates.len() >= needed_connections {
                    break;
                }
            }
        }
        
        candidates
    }
    
    /// Optimize the network topology
    pub fn optimize(&mut self) -> (Vec<SocketAddr>, Vec<SocketAddr>) {
        let now = Instant::now();
        
        // Only optimize periodically
        if now.duration_since(self.last_optimization) < Duration::from_secs(self.config.optimization_interval) {
            return (Vec::new(), Vec::new());
        }
        
        self.last_optimization = now;
        
        // Update all peer scores
        for peer_info in self.peers.values_mut() {
            peer_info.update_score(&self.config);
        }
        
        // Identify peers to disconnect (low-scoring outbound connections)
        let mut to_disconnect = Vec::new();
        
        // Group outbound peers by region with scores
        let mut outbound_by_region: HashMap<Region, BTreeMap<SocketAddr, f32>> = HashMap::new();
        
        for (addr, peer_info) in &self.peers {
            if peer_info.direction == ConnectionDirection::Outbound {
                outbound_by_region
                    .entry(peer_info.region)
                    .or_insert_with(BTreeMap::new)
                    .insert(*addr, peer_info.score);
            }
        }
        
        // For each region, keep only the best connections_per_region peers
        for (region, peers) in &outbound_by_region {
            // Sort peers by score (descending)
            let mut peers_vec: Vec<_> = peers.iter().collect();
            peers_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Keep only the best connections_per_region peers
            if peers_vec.len() > self.config.connections_per_region {
                for (addr, _) in peers_vec.iter().skip(self.config.connections_per_region) {
                    to_disconnect.push(**addr);
                }
            }
        }
        
        // Identify peers to connect to
        let mut available_peers = Vec::new();
        
        // In a real implementation, we would get this from the peer discovery service
        // For now, just return an empty list
        
        let to_connect = self.get_connection_candidates(&available_peers);
        
        (to_connect, to_disconnect)
    }
    
    /// Get all connected peers
    pub fn get_all_peers(&self) -> impl Iterator<Item = &SocketAddr> {
        self.peers.keys()
    }
    
    /// Get peers in a specific region
    pub fn get_peers_in_region(&self, region: Region) -> impl Iterator<Item = &SocketAddr> {
        self.peers_by_region
            .get(&region)
            .map(|peers| peers.iter())
            .unwrap_or_else(|| [].iter())
    }
    
    /// Get the best peers for message propagation
    pub fn get_best_propagation_peers(&self, count: usize) -> Vec<SocketAddr> {
        // Create a list of peers sorted by score
        let mut peers: Vec<_> = self.peers.iter()
            .map(|(addr, info)| (*addr, info.score))
            .collect();
        
        // Sort by score (descending)
        peers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take the top 'count' peers
        peers.iter()
            .take(count)
            .map(|(addr, _)| *addr)
            .collect()
    }
    
    /// Get the best peers for each region
    pub fn get_best_region_peers(&self, per_region: usize) -> Vec<SocketAddr> {
        let mut result = Vec::new();
        
        // For each region, get the best peers
        for region in Region::all() {
            let mut region_peers: Vec<_> = self.peers.iter()
                .filter(|(_, info)| info.region == region)
                .map(|(addr, info)| (*addr, info.score))
                .collect();
            
            // Sort by score (descending)
            region_peers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Take the top 'per_region' peers
            for (addr, _) in region_peers.iter().take(per_region) {
                result.push(*addr);
            }
        }
        
        result
    }
}

impl Region {
    /// Get all regions
    pub fn all() -> Vec<Region> {
        vec![
            Region::NorthAmerica,
            Region::SouthAmerica,
            Region::Europe,
            Region::Asia,
            Region::Africa,
            Region::Oceania,
            Region::Unknown,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    
    fn create_test_addr(a: u8, b: u8, c: u8, d: u8, port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(a, b, c, d)), port)
    }
    
    #[test]
    fn test_mesh_topology_config_default() {
        let config = MeshTopologyConfig::default();
        
        assert_eq!(config.min_outbound_connections, 8);
        assert_eq!(config.max_outbound_connections, 16);
        assert_eq!(config.max_inbound_connections, 128);
        assert_eq!(config.connections_per_region, 2);
    }
    
    #[test]
    fn test_region_from_ip() {
        // Test North America
        let addr = create_test_addr(10, 0, 0, 1, 8000);
        assert_eq!(Region::from_ip(&addr), Region::NorthAmerica);
        
        // Test Europe
        let addr = create_test_addr(130, 0, 0, 1, 8000);
        assert_eq!(Region::from_ip(&addr), Region::Europe);
        
        // Test Asia
        let addr = create_test_addr(200, 0, 0, 1, 8000);
        assert_eq!(Region::from_ip(&addr), Region::Asia);
    }
    
    #[test]
    fn test_mesh_peer_info_creation() {
        let addr = create_test_addr(10, 0, 0, 1, 8000);
        let direction = ConnectionDirection::Outbound;
        
        let peer_info = MeshPeerInfo::new(addr, direction);
        
        assert_eq!(peer_info.address, addr);
        assert_eq!(peer_info.direction, direction);
        assert_eq!(peer_info.region, Region::NorthAmerica);
        assert_eq!(peer_info.last_ping_ms, 0);
        assert_eq!(peer_info.bytes_sent, 0);
        assert_eq!(peer_info.bytes_received, 0);
        assert_eq!(peer_info.score, 0.0);
    }
    
    #[test]
    fn test_mesh_peer_info_update_score() {
        let addr = create_test_addr(10, 0, 0, 1, 8000);
        let direction = ConnectionDirection::Outbound;
        let config = MeshTopologyConfig::default();
        
        let mut peer_info = MeshPeerInfo::new(addr, direction);
        
        // Initial score should be 0
        assert_eq!(peer_info.score, 0.0);
        
        // Update with good metrics
        peer_info.last_ping_ms = 30;
        peer_info.bytes_sent = 10000;
        peer_info.bytes_received = 20000;
        
        // Force connected_at to be older
        peer_info.connected_at = Instant::now() - Duration::from_secs(7200);
        
        peer_info.update_score(&config);
        
        // Score should be high with good metrics
        assert!(peer_info.score > 0.8);
    }
    
    #[test]
    fn test_mesh_topology_add_remove_peer() {
        let config = MeshTopologyConfig::default();
        let mut topology = MeshTopology::new(config);
        
        let addr1 = create_test_addr(10, 0, 0, 1, 8000);
        let addr2 = create_test_addr(130, 0, 0, 1, 8000);
        
        // Add peers
        assert!(topology.add_peer(addr1, ConnectionDirection::Outbound).is_ok());
        assert!(topology.add_peer(addr2, ConnectionDirection::Inbound).is_ok());
        
        // Check counts
        assert_eq!(topology.outbound_count, 1);
        assert_eq!(topology.inbound_count, 1);
        
        // Check region maps
        assert_eq!(topology.peers_by_region.len(), 2);
        assert!(topology.peers_by_region.contains_key(&Region::NorthAmerica));
        assert!(topology.peers_by_region.contains_key(&Region::Europe));
        
        // Remove peers
        assert!(topology.remove_peer(&addr1).is_ok());
        assert_eq!(topology.outbound_count, 0);
        assert_eq!(topology.inbound_count, 1);
        
        assert!(topology.remove_peer(&addr2).is_ok());
        assert_eq!(topology.outbound_count, 0);
        assert_eq!(topology.inbound_count, 0);
        
        // Region maps should be empty
        assert_eq!(topology.peers_by_region.len(), 0);
    }
    
    #[test]
    fn test_mesh_topology_update_metrics() {
        let config = MeshTopologyConfig::default();
        let mut topology = MeshTopology::new(config);
        
        let addr = create_test_addr(10, 0, 0, 1, 8000);
        
        // Add peer
        assert!(topology.add_peer(addr, ConnectionDirection::Outbound).is_ok());
        
        // Update metrics
        assert!(topology.update_peer_metrics(&addr, Some(50), Some(1000), Some(2000)).is_ok());
        
        // Check metrics were updated
        let peer_info = topology.peers.get(&addr).unwrap();
        assert_eq!(peer_info.last_ping_ms, 50);
        assert_eq!(peer_info.bytes_sent, 1000);
        assert_eq!(peer_info.bytes_received, 2000);
        assert!(peer_info.score > 0.0);
    }
}
