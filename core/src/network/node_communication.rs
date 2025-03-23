//! # Node Communication
//! 
//! This module implements communication mechanisms between nodes including
//! block propagation and transaction broadcasting.

use std::net::SocketAddr;
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::network::{Message, MessageType, Transport, TransactionBloomFilter, FastPath, FastPathConfig, BandwidthManager, BandwidthConfig};
use crate::blockchain::{Block, Transaction};
use crate::types::{Result, Error, Priority};

/// Block propagation configuration
#[derive(Debug, Clone)]
pub struct BlockPropagationConfig {
    /// Maximum blocks to announce at once
    pub max_blocks_to_announce: usize,
    
    /// Minimum time between block announcements (in seconds)
    pub min_announce_interval: u64,
    
    /// Number of peers to send full blocks to initially
    pub initial_block_relay_count: usize,
    
    /// Time to wait for block requests after announcement (in seconds)
    pub block_request_timeout: u64,
}

impl Default for BlockPropagationConfig {
    fn default() -> Self {
        BlockPropagationConfig {
            max_blocks_to_announce: 16,
            min_announce_interval: 1,
            initial_block_relay_count: 3,
            block_request_timeout: 10,
        }
    }
}

/// Transaction broadcasting configuration
#[derive(Debug, Clone)]
pub struct TransactionBroadcastConfig {
    /// Maximum transactions to announce at once
    pub max_transactions_to_announce: usize,
    
    /// Maximum transactions to include in a batch
    pub max_tx_batch_size: usize,
    
    /// Use Bloom filter for transaction announcements
    pub use_bloom_filter: bool,
    
    /// Minimum time between transaction broadcasts (in seconds)
    pub min_broadcast_interval: u64,
    
    /// Maximum transactions to track in Bloom filter
    pub max_bloom_filter_transactions: usize,
    
    /// False positive probability for Bloom filter
    pub bloom_filter_false_positive_probability: f64,
}

impl Default for TransactionBroadcastConfig {
    fn default() -> Self {
        TransactionBroadcastConfig {
            max_transactions_to_announce: 1000,
            max_tx_batch_size: 100,
            use_bloom_filter: true,
            min_broadcast_interval: 1,
            max_bloom_filter_transactions: 100000,
            bloom_filter_false_positive_probability: 0.01,
        }
    }
}

/// Communication manager for node interactions
pub struct NodeCommunication {
    /// Block propagation configuration
    block_config: BlockPropagationConfig,
    
    /// Transaction broadcasting configuration
    tx_config: TransactionBroadcastConfig,
    
    /// Transport layer
    transport: Arc<Transport>,
    
    /// Known block hashes by peer
    known_blocks: Arc<Mutex<HashMap<SocketAddr, HashSet<Vec<u8>>>>>,
    
    /// Known transaction hashes by peer
    known_txs: Arc<Mutex<HashMap<SocketAddr, HashSet<Vec<u8>>>>>,
    
    /// Last broadcast times
    last_broadcast: Arc<Mutex<HashMap<MessageType, Instant>>>,
    
    /// Running state
    running: Arc<Mutex<bool>>,
    
    /// Transaction Bloom filter
    tx_bloom_filter: Arc<Mutex<TransactionBloomFilter>>,
    
    /// Fast path routing
    fast_path: Arc<Mutex<FastPath>>,
    
    /// Bandwidth manager
    bandwidth_manager: Arc<Mutex<BandwidthManager>>,
}

impl NodeCommunication {
    /// Create a new node communication manager
    pub fn new(
        block_config: BlockPropagationConfig,
        tx_config: TransactionBroadcastConfig,
        transport: Arc<Transport>,
    ) -> Self {
        let mut last_broadcast = HashMap::new();
        last_broadcast.insert(MessageType::BlockAnnouncement, Instant::now());
        last_broadcast.insert(MessageType::TransactionAnnouncement, Instant::now());
        
        // Create transaction Bloom filter
        let tx_bloom_filter = TransactionBloomFilter::new(
            tx_config.max_bloom_filter_transactions,
            tx_config.bloom_filter_false_positive_probability,
        );
        
        // Create fast path routing
        let fast_path = FastPath::new(FastPathConfig::default());
        
        // Create bandwidth manager
        let bandwidth_manager = BandwidthManager::new(BandwidthConfig::default());
        
        NodeCommunication {
            block_config,
            tx_config,
            transport,
            known_blocks: Arc::new(Mutex::new(HashMap::new())),
            known_txs: Arc::new(Mutex::new(HashMap::new())),
            last_broadcast: Arc::new(Mutex::new(last_broadcast)),
            running: Arc::new(Mutex::new(false)),
            tx_bloom_filter: Arc::new(Mutex::new(tx_bloom_filter)),
            fast_path: Arc::new(Mutex::new(fast_path)),
            bandwidth_manager: Arc::new(Mutex::new(bandwidth_manager)),
        }
    }
    
    /// Start the communication service
    pub fn start(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err(Error::Network("Communication already running".to_string()));
        }
        
        *running = true;
        
        log::info!("Node communication service started");
        
        Ok(())
    }
    
    /// Stop the communication service
    pub fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        *running = false;
        
        log::info!("Node communication service stopped");
        
        Ok(())
    }
    
    /// Announce a new block to connected peers
    pub fn announce_block(&self, block: &Block, peers: &[SocketAddr]) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        // Check if enough time has passed since the last announcement
        let now = Instant::now();
        let mut last_broadcast = self.last_broadcast.lock().unwrap();
        let last_time = last_broadcast.get(&MessageType::BlockAnnouncement).unwrap();
        
        if now.duration_since(*last_time) < Duration::from_secs(self.block_config.min_announce_interval) {
            log::debug!("Skipping block announcement due to rate limiting");
            return Ok(());
        }
        
        // Update the last broadcast time
        last_broadcast.insert(MessageType::BlockAnnouncement, now);
        
        // Create the block announcement message
        // Note: We'll rely on Block::hash being defined in the blockchain module
        // or call a helper function from there to get the hash
        let block_hash = Vec::new(); // To be implemented
        let height = block.header.index;
        
        // Create announcement data - using simple serialization approach for now
        let announcement_data = match bincode::serialize(&(block_hash.clone(), height)) {
            Ok(data) => data,
            Err(e) => {
                return Err(Error::Serialization(format!("Failed to serialize block announcement: {}", e)));
            }
        };
        
        let announcement = Message::new(
            MessageType::BlockAnnouncement,
            announcement_data,
            None, // Shard ID
            Priority::High,
            Vec::new(), // Sender ID will be filled by the network layer
        );
        
        // Track which peers we've sent announcements to
        let mut known_blocks = self.known_blocks.lock().unwrap();
        
        // Send the announcement to all peers
        for &peer_addr in peers {
            // Add the block hash to the peer's known blocks
            let peer_blocks = known_blocks.entry(peer_addr).or_insert_with(HashSet::new);
            peer_blocks.insert(block_hash.clone());
            
            // Send the announcement
            if let Err(e) = self.transport.send(&peer_addr, &announcement) {
                log::warn!("Failed to send block announcement to {}: {:?}", peer_addr, e);
            }
        }
        
        // Send the full block to a subset of peers
        self.relay_block_to_initial_peers(block, peers)
    }
    
    /// Send a block to a subset of peers
    fn relay_block_to_initial_peers(&self, block: &Block, peers: &[SocketAddr]) -> Result<()> {
        if peers.is_empty() {
            return Ok(());
        }
        
        // Select a subset of peers to receive the full block
        let count = std::cmp::min(self.block_config.initial_block_relay_count, peers.len());
        let mut selected_peers = Vec::with_capacity(count);
        
        // In a real implementation, we would use a better peer selection algorithm
        // For now, just take the first few peers
        for i in 0..count {
            selected_peers.push(peers[i]);
        }
        
        // Serialize the full block
        let block_data = match bincode::serialize(block) {
            Ok(data) => data,
            Err(e) => {
                return Err(Error::Serialization(format!("Failed to serialize block: {}", e)));
            }
        };
        
        // Create block body message
        let block_body_msg = Message::new(
            MessageType::BlockBody,
            block_data,
            None, // Shard ID
            Priority::High,
            Vec::new(), // Sender ID will be filled by the network layer
        );
        
        // Send the full block to selected peers
        for &peer_addr in &selected_peers {
            if let Err(e) = self.transport.send(&peer_addr, &block_body_msg) {
                log::warn!("Failed to send block body to {}: {:?}", peer_addr, e);
            } else {
                log::debug!("Sent full block to {}", peer_addr);
            }
        }
        
        Ok(())
    }
    
    /// Handle a block announcement from a peer
    pub fn handle_block_announcement(&self, peer_addr: &SocketAddr, data: &[u8]) -> Result<(Vec<u8>, u64)> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        // Deserialize the announcement data
        let (block_hash, height): (Vec<u8>, u64) = match bincode::deserialize(data) {
            Ok(data) => data,
            Err(e) => {
                return Err(Error::Deserialization(format!("Failed to deserialize block announcement: {}", e)));
            }
        };
        
        // Add the block hash to the peer's known blocks
        let mut known_blocks = self.known_blocks.lock().unwrap();
        let peer_blocks = known_blocks.entry(*peer_addr).or_insert_with(HashSet::new);
        peer_blocks.insert(block_hash.clone());
        
        Ok((block_hash, height))
    }
    
    /// Request a block from a peer
    pub fn request_block(&self, peer_addr: &SocketAddr, block_hash: &[u8]) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        // Create block request message
        let block_request_msg = Message::new(
            MessageType::BlockHeader,
            block_hash.to_vec(),
            None, // Shard ID
            Priority::High,
            Vec::new(), // Sender ID will be filled by the network layer
        );
        
        // Send the request
        if let Err(e) = self.transport.send(peer_addr, &block_request_msg) {
            log::warn!("Failed to send block request to {}: {:?}", peer_addr, e);
            return Err(Error::Network(format!("Failed to send block request: {:?}", e)));
        }
        
        Ok(())
    }
    
    /// Broadcast transactions to connected peers
    pub fn broadcast_transactions(&self, transactions: &[Transaction], peers: &[SocketAddr]) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        // Check if enough time has passed since the last broadcast
        let now = Instant::now();
        let mut last_broadcast = self.last_broadcast.lock().unwrap();
        let last_time = last_broadcast.get(&MessageType::TransactionAnnouncement).unwrap();
        
        if now.duration_since(*last_time) < Duration::from_secs(self.tx_config.min_broadcast_interval) {
            log::debug!("Skipping transaction broadcast due to rate limiting");
            return Ok(());
        }
        
        // Update the last broadcast time
        last_broadcast.insert(MessageType::TransactionAnnouncement, now);
        
        if transactions.is_empty() || peers.is_empty() {
            return Ok(());
        }
        
        // Limit the number of transactions to announce
        let tx_count = std::cmp::min(transactions.len(), self.tx_config.max_transactions_to_announce);
        let transactions = &transactions[..tx_count];
        
        // Prepare transaction hashes for announcement
        let mut tx_hashes = Vec::with_capacity(tx_count);
        for tx in transactions {
            tx_hashes.push(tx.id.clone());
        }
        
        // Create the announcement data
        let announcement_data = if self.tx_config.use_bloom_filter {
            self.create_bloom_filter_announcement(&tx_hashes)
        } else {
            match bincode::serialize(&tx_hashes) {
                Ok(data) => data,
                Err(e) => {
                    return Err(Error::Serialization(format!("Failed to serialize transaction hashes: {}", e)));
                }
            }
        };
        
        // Create announcement message
        let announcement = Message::new(
            MessageType::TransactionAnnouncement,
            announcement_data,
            None, // Shard ID
            Priority::High,
            Vec::new(), // Sender ID will be filled by the network layer
        );
        
        // Track which peers we've sent announcements to
        let mut known_txs = self.known_txs.lock().unwrap();
        
        // Send the announcement to all peers
        for &peer_addr in peers {
            // Add the transaction hashes to the peer's known transactions
            let peer_txs = known_txs.entry(peer_addr).or_insert_with(HashSet::new);
            for hash in &tx_hashes {
                peer_txs.insert(hash.clone());
            }
            
            // Send the announcement
            if let Err(e) = self.transport.send(&peer_addr, &announcement) {
                log::warn!("Failed to send transaction announcement to {}: {:?}", peer_addr, e);
            }
        }
        
        // Send transaction batches to peers
        self.send_transaction_batches(transactions, peers)
    }
    
    /// Create a Bloom filter for transaction announcements
    fn create_bloom_filter_announcement(&self, tx_hashes: &[Vec<u8>]) -> Vec<u8> {
        let mut bloom_filter = self.tx_bloom_filter.lock().unwrap();
        
        // Add transaction hashes to the Bloom filter
        for hash in tx_hashes {
            bloom_filter.add_transaction(hash);
        }
        
        // Serialize the Bloom filter
        bloom_filter.serialize()
    }
    
    /// Send transaction batches to peers
    fn send_transaction_batches(&self, transactions: &[Transaction], peers: &[SocketAddr]) -> Result<()> {
        if transactions.is_empty() || peers.is_empty() {
            return Ok(());
        }
        
        // Split transactions into batches
        let batch_size = self.tx_config.max_tx_batch_size;
        let batch_count = (transactions.len() + batch_size - 1) / batch_size;
        
        for batch_idx in 0..batch_count {
            let start = batch_idx * batch_size;
            let end = std::cmp::min(start + batch_size, transactions.len());
            let batch = &transactions[start..end];
            
            // Serialize the batch
            let batch_data = match bincode::serialize(batch) {
                Ok(data) => data,
                Err(e) => {
                    return Err(Error::Serialization(format!("Failed to serialize transaction batch: {}", e)));
                }
            };
            
            // Create batch message
            let batch_msg = Message::new(
                MessageType::TransactionBatch,
                batch_data,
                None, // Shard ID
                Priority::Low,
                Vec::new(), // Sender ID will be filled by the network layer
            );
            
            // Send the batch to all peers
            for &peer_addr in peers {
                if let Err(e) = self.transport.send(&peer_addr, &batch_msg) {
                    log::warn!("Failed to send transaction batch to {}: {:?}", peer_addr, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle a transaction announcement from a peer
    pub fn handle_transaction_announcement(&self, peer_addr: &SocketAddr, data: &[u8]) -> Result<Vec<Vec<u8>>> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        // Check if the data is a Bloom filter or direct transaction hashes
        let tx_hashes: Vec<Vec<u8>> = if self.tx_config.use_bloom_filter {
            // Try to deserialize as a list of hashes first (for backward compatibility)
            match bincode::deserialize(data) {
                Ok(hashes) => hashes,
                Err(_) => {
                    // If that fails, assume it's a Bloom filter
                    // In a real implementation, we would check our local transaction pool
                    // against the Bloom filter to find potential matches
                    // For now, just return an empty list
                    Vec::new()
                }
            }
        } else {
            // Direct transaction hashes
            match bincode::deserialize(data) {
                Ok(hashes) => hashes,
                Err(e) => {
                    return Err(Error::Deserialization(format!("Failed to deserialize transaction announcement: {}", e)));
                }
            }
        };
        
        // Add the transaction hashes to the peer's known transactions
        let mut known_txs = self.known_txs.lock().unwrap();
        let peer_txs = known_txs.entry(*peer_addr).or_insert_with(HashSet::new);
        
        // Collect unknown transaction hashes
        let mut unknown_hashes = Vec::new();
        for hash in &tx_hashes {
            if !peer_txs.contains(hash) {
                unknown_hashes.push(hash.clone());
                peer_txs.insert(hash.clone());
            }
        }
        
        Ok(unknown_hashes)
    }
    
    /// Request transactions from a peer
    pub fn request_transactions(&self, peer_addr: &SocketAddr, tx_hashes: &[Vec<u8>]) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        if tx_hashes.is_empty() {
            return Ok(());
        }
        
        // Serialize the transaction hashes
        let data = match bincode::serialize(tx_hashes) {
            Ok(data) => data,
            Err(e) => {
                return Err(Error::Serialization(format!("Failed to serialize transaction hashes: {}", e)));
            }
        };
        
        // Create transaction request message
        let tx_request_msg = Message::new(
            MessageType::TransactionAnnouncement, // Reuse announcement type for requests
            data,
            None, // Shard ID
            Priority::Low,
            Vec::new(), // Sender ID will be filled by the network layer
        );
        
        // Send the request
        if let Err(e) = self.transport.send(peer_addr, &tx_request_msg) {
            log::warn!("Failed to send transaction request to {}: {:?}", peer_addr, e);
            return Err(Error::Network(format!("Failed to send transaction request: {:?}", e)));
        }
        
        Ok(())
    }
    
    /// Handle a block or transaction message
    pub fn handle_message(&self, peer_addr: &SocketAddr, message: &Message) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Err(Error::Network("Communication not running".to_string()));
        }
        
        match message.message_type {
            MessageType::BlockAnnouncement => {
                let (block_hash, height) = self.handle_block_announcement(peer_addr, &message.data)?;
                log::debug!("Received block announcement from {}: {} at height {}", peer_addr, hex::encode(&block_hash), height);
            },
            
            MessageType::BlockHeader => {
                log::debug!("Received block header from {}", peer_addr);
                // In a real implementation, we would process the block header
            },
            
            MessageType::BlockBody => {
                log::debug!("Received block body from {}", peer_addr);
                // In a real implementation, we would process the block body
            },
            
            MessageType::TransactionAnnouncement => {
                let unknown_hashes = self.handle_transaction_announcement(peer_addr, &message.data)?;
                log::debug!("Received transaction announcement from {}: {} new transactions", 
                           peer_addr, unknown_hashes.len());
                
                // Request unknown transactions if any
                if !unknown_hashes.is_empty() {
                    self.request_transactions(peer_addr, &unknown_hashes)?;
                }
            },
            
            MessageType::TransactionBatch => {
                log::debug!("Received transaction batch from {}", peer_addr);
                // In a real implementation, we would process the transaction batch
            },
            
            _ => {
                log::debug!("Received message of type {:?} from {}", message.message_type, peer_addr);
            }
        }
        
        Ok(())
    }
    
    /// Check if a block hash is known to a peer
    pub fn is_block_known_to_peer(&self, peer_addr: &SocketAddr, block_hash: &[u8]) -> bool {
        let known_blocks = self.known_blocks.lock().unwrap();
        
        if let Some(peer_blocks) = known_blocks.get(peer_addr) {
            peer_blocks.contains(block_hash)
        } else {
            false
        }
    }
    
    /// Check if a transaction hash is known to a peer
    pub fn is_transaction_known_to_peer(&self, peer_addr: &SocketAddr, tx_hash: &[u8]) -> bool {
        let known_txs = self.known_txs.lock().unwrap();
        
        if let Some(peer_txs) = known_txs.get(peer_addr) {
            peer_txs.contains(tx_hash)
        } else {
            false
        }
    }
    
    /// Clear known items for a disconnected peer
    pub fn clear_peer(&self, peer_addr: &SocketAddr) {
        let mut known_blocks = self.known_blocks.lock().unwrap();
        let mut known_txs = self.known_txs.lock().unwrap();
        
        known_blocks.remove(peer_addr);
        known_txs.remove(peer_addr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    use crate::network::{Protocol, ProtocolConfig, TransportConfig};
    
    fn create_test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }
    
    fn create_test_protocol() -> Protocol {
        Protocol::new(
            ProtocolConfig::default(),
            "sebure-testnet".to_string(),
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            "sebure-test/1.0.0".to_string(),
        )
    }
    
    fn create_test_transport() -> Arc<Transport> {
        let config = TransportConfig::default();
        let protocol = create_test_protocol();
        
        Arc::new(Transport::new(config, protocol))
    }
    
    #[test]
    fn test_node_communication_creation() {
        let block_config = BlockPropagationConfig::default();
        let tx_config = TransactionBroadcastConfig::default();
        let transport = create_test_transport();
        
        let comm = NodeCommunication::new(block_config, tx_config, transport);
        
        // Initially not running
        assert!(!*comm.running.lock().unwrap());
    }
    
    #[test]
    fn test_node_communication_start_stop() {
        let block_config = BlockPropagationConfig::default();
        let tx_config = TransactionBroadcastConfig::default();
        let transport = create_test_transport();
        
        let comm = NodeCommunication::new(block_config, tx_config, transport);
        
        // Start the service
        assert!(comm.start().is_ok());
        assert!(*comm.running.lock().unwrap());
        
        // Starting again should fail
        assert!(comm.start().is_err());
        
        // Stop the service
        assert!(comm.stop().is_ok());
        assert!(!*comm.running.lock().unwrap());
        
        // Stopping again should fail
        assert!(comm.stop().is_err());
    }
    
    // Note: More comprehensive communication tests would require
    // actual networking or mocking, which is beyond the scope of unit tests.
}
