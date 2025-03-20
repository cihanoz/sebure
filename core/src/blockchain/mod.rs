//! # Blockchain Core Module
//! 
//! This module implements the core blockchain data structures and logic,
//! including blocks, transactions, and blockchain state management.

mod block;
mod transaction;
mod state;
mod mempool;

// Re-export main types
pub use block::Block;
pub use block::BlockHeader;
pub use block::ShardData;
pub use transaction::Transaction;
pub use transaction::Receipt;
pub use state::{Account, AccountType, ShardState, GlobalState};
pub use mempool::{Mempool, MempoolConfig};

use crate::types::{Result, Error, ShardId};
use crate::crypto::hash;
use crate::storage::ChainStore;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for the blockchain
#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    /// Maximum number of transactions per block
    pub max_transactions_per_block: usize,
    
    /// Target block time in seconds
    pub target_block_time: u64,
    
    /// Maximum block size in bytes
    pub max_block_size: usize,
    
    /// Minimum number of confirmations for finality
    pub finality_confirmations: u64,
    
    /// Mempool configuration
    pub mempool_config: mempool::MempoolConfig,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        BlockchainConfig {
            max_transactions_per_block: 1000,
            target_block_time: 2, // 2 seconds
            max_block_size: 1024 * 1024, // 1 MB
            finality_confirmations: 3, // 3 blocks
            mempool_config: mempool::MempoolConfig::default(),
        }
    }
}

/// Blockchain represents the main chain structure
pub struct Blockchain {
    /// Configuration for this blockchain
    config: BlockchainConfig,
    
    /// Mempool for pending transactions
    mempool: Arc<Mempool>,
    
    /// Chain store for persistence
    chain_store: Option<Arc<ChainStore>>,
    
    /// Current height of the blockchain
    height: Arc<Mutex<u64>>,
    
    /// Genesis block hash
    genesis_hash: Arc<Mutex<Vec<u8>>>,
    
    /// Latest block hash
    latest_hash: Arc<Mutex<Vec<u8>>>,
    
    /// Cached blocks by height
    blocks: Arc<Mutex<HashMap<u64, Block>>>,
}

impl Blockchain {
    /// Create a new blockchain with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(BlockchainConfig::default())
    }
    
    /// Create a new blockchain with given configuration
    pub fn with_config(config: BlockchainConfig) -> Result<Self> {
        let mempool = Arc::new(Mempool::new(config.mempool_config.clone()));
        let empty_hash = vec![0; 32]; // Placeholder until genesis block is created
        
        Ok(Blockchain {
            config,
            mempool,
            chain_store: None,
            height: Arc::new(Mutex::new(0)),
            genesis_hash: Arc::new(Mutex::new(empty_hash.clone())),
            latest_hash: Arc::new(Mutex::new(empty_hash)),
            blocks: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Set the chain store for persistence
    pub fn set_chain_store(&mut self, chain_store: Arc<ChainStore>) {
        self.chain_store = Some(chain_store);
    }
    
    /// Generate a genesis block
    pub fn generate_genesis_block(&self, 
                                 timestamp: Option<u64>, 
                                 coinbase_recipient: Option<Vec<u8>>,
                                 initial_balances: Option<HashMap<Vec<u8>, u64>>) -> Result<Block> {
        // Get current timestamp if not provided
        let timestamp = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64
        });
        
        // Create an empty block with index 0 (genesis)
        let mut genesis_block = Block::new(
            0, // Genesis block has height 0
            timestamp,
            vec![0; 32], // No previous block, use zeros
            vec![0 as ShardId], // Start with just one shard
        );
        
        // Add initial balances if provided
        if let Some(_balances) = initial_balances {
            // In a real implementation, we would add transactions to set initial balances
            // For now, we just acknowledge that this would happen
        }
        
        Ok(genesis_block)
    }
    
    /// Initialize the blockchain with a genesis block
    pub fn initialize_with_genesis(&mut self, genesis_block: Block) -> Result<()> {
        // Ensure we don't already have a genesis block
        {
            let height = self.height.lock().unwrap();
            if *height > 0 {
                return Err(Error::BlockValidation("Blockchain already initialized".to_string()));
            }
        }
        
        // Compute the hash of the genesis block
        let block_hash = hash::sha256(&[0; 32]); // Use a placeholder hash for now
        
        // Store the genesis block
        {
            let mut blocks = self.blocks.lock().unwrap();
            blocks.insert(0, genesis_block.clone());
        }
        
        // Update genesis and latest hash
        {
            let mut genesis_hash = self.genesis_hash.lock().unwrap();
            *genesis_hash = block_hash.to_vec();
            
            let mut latest_hash = self.latest_hash.lock().unwrap();
            *latest_hash = block_hash.to_vec();
        }
        
        // If we have a chain store, persist the genesis block
        if let Some(ref chain_store) = self.chain_store {
            chain_store.put_block(genesis_block)?;
        }
        
        Ok(())
    }
    
    /// Create a new block building upon the latest block
    pub fn create_block(&self, 
                       shard_ids: Vec<ShardId>,
                       timestamp: Option<u64>) -> Result<Block> {
        let height;
        let prev_hash;
        
        // Get current height and latest hash
        {
            let h = self.height.lock().unwrap();
            height = *h + 1; // New block height
            
            let hash = self.latest_hash.lock().unwrap();
            prev_hash = hash.clone();
        }
        
        // Get current timestamp if not provided
        let timestamp = timestamp.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64
        });
        
        // Create the new block
        let mut block = Block::new(
            height,
            timestamp,
            prev_hash,
            shard_ids.clone(),
        );
        
        // Add transactions from mempool for each shard
        for &shard_id in &shard_ids {
            let transactions = self.mempool.get_transactions_for_block(
                shard_id,
                self.config.max_transactions_per_block,
            );
            
            if !transactions.is_empty() {
                let tx_hashes = transactions.iter()
                    .map(|tx| tx.id().clone())
                    .collect::<Vec<_>>();
                
                let shard_data = ShardData {
                    shard_id,
                    transactions: tx_hashes,
                    execution_proof: Vec::new(),
                    validator_signatures: Vec::new(),
                };
                
                // Add the shard data to the block
                let mut shard_data_vec = block.shard_data.clone();
                shard_data_vec.push(shard_data);
                block.shard_data = shard_data_vec;
            }
        }
        
        Ok(block)
    }
    
    /// Add a block to the chain after validation
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        // Validate the block
        self.validate_block(&block)?;
        
        // Compute the block hash
        let block_hash = hash::sha256(&[0; 32]); // Use a placeholder hash for now
        
        // Update chain state
        {
            let mut height = self.height.lock().unwrap();
            *height = block.header.index;
            
            let mut latest_hash = self.latest_hash.lock().unwrap();
            *latest_hash = block_hash.to_vec();
            
            let mut blocks = self.blocks.lock().unwrap();
            blocks.insert(block.header.index, block.clone());
        }
        
        // Remove included transactions from mempool
        for shard_data in &block.shard_data {
            for tx_hash in &shard_data.transactions {
                let _ = self.mempool.remove_transaction(tx_hash);
            }
        }
        
        // If we have a chain store, persist the block
        if let Some(ref chain_store) = self.chain_store {
            chain_store.put_block(block)?;
        }
        
        Ok(())
    }
    
    /// Validate a block
    pub fn validate_block(&self, block: &Block) -> Result<()> {
        // Check basic block properties
        block.validate_basic()?;
        
        // Get current height
        let current_height = *self.height.lock().unwrap();
        
        // Check block height
        if block.header.index != current_height + 1 {
            return Err(Error::BlockValidation(
                format!("Invalid block height: expected {}, got {}", 
                        current_height + 1, block.header.index)
            ));
        }
        
        // Check previous hash
        {
            let latest_hash = self.latest_hash.lock().unwrap();
            if block.header.previous_hash != *latest_hash {
                return Err(Error::BlockValidation(
                    format!("Invalid previous hash: expected {:?}, got {:?}", 
                            latest_hash, block.header.previous_hash)
                ));
            }
        }
        
        // Validate timestamps
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        if block.header.timestamp > now + 60_000_000 { // 1 minute in the future
            return Err(Error::BlockValidation(
                format!("Block timestamp too far in the future: {}", block.header.timestamp)
            ));
        }
        
        // Get the previous block
        let prev_block = {
            let blocks = self.blocks.lock().unwrap();
            blocks.get(&current_height).cloned()
        };
        
        if let Some(prev) = prev_block {
            if block.header.timestamp <= prev.header.timestamp {
                return Err(Error::BlockValidation(
                    format!("Block timestamp not greater than previous block: {} <= {}", 
                            block.header.timestamp, prev.header.timestamp)
                ));
            }
        }
        
        // In a real implementation, we would also validate:
        // - Transaction validity
        // - State transitions
        // - Merkle roots
        // - Signatures
        
        Ok(())
    }
    
    /// Get a block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Block> {
        // First try in-memory cache
        {
            let blocks = self.blocks.lock().unwrap();
            if let Some(block) = blocks.get(&height) {
                return Ok(block.clone());
            }
        }
        
        // If not in memory and we have a chain store, try there
        if let Some(ref chain_store) = self.chain_store {
            return chain_store.get_block_by_height(height);
        }
        
        Err(Error::State(format!("Block not found at height {}", height)))
    }
    
    /// Get a block by hash
    pub fn get_block_by_hash(&self, hash: &[u8]) -> Result<Block> {
        // If we have a chain store, use it
        if let Some(ref chain_store) = self.chain_store {
            return chain_store.get_block_by_hash(hash);
        }
        
        // Otherwise, we'd need to search through in-memory blocks
        // This is inefficient and would be improved in a real implementation
        Err(Error::State(format!("Block not found with hash {:?}", hash)))
    }
    
    /// Get the current height of the blockchain
    pub fn get_height(&self) -> u64 {
        *self.height.lock().unwrap()
    }
    
    /// Get the latest block hash
    pub fn get_latest_hash(&self) -> Vec<u8> {
        self.latest_hash.lock().unwrap().clone()
    }
    
    /// Get the genesis block hash
    pub fn get_genesis_hash(&self) -> Vec<u8> {
        self.genesis_hash.lock().unwrap().clone()
    }
    
    /// Get the mempool
    pub fn mempool(&self) -> Arc<Mempool> {
        self.mempool.clone()
    }
    
    /// Check if the chain has a block at the given height
    pub fn has_block_at_height(&self, height: u64) -> bool {
        // First check in-memory cache
        {
            let blocks = self.blocks.lock().unwrap();
            if blocks.contains_key(&height) {
                return true;
            }
        }
        
        // If not in memory and we have a chain store, check there
        if let Some(ref chain_store) = self.chain_store {
            return chain_store.has_block_height(height);
        }
        
        false
    }
    
    /// Check if the chain has a block with the given hash
    pub fn has_block_with_hash(&self, hash: &[u8]) -> bool {
        // If we have a chain store, check there
        if let Some(ref chain_store) = self.chain_store {
            return chain_store.has_block_hash(hash);
        }
        
        // Otherwise, we'd need to search through in-memory blocks
        // This is inefficient and would be improved in a real implementation
        false
    }
    
    /// Add a transaction to the mempool
    pub fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        self.mempool.add_transaction(transaction)
    }
    
    /// Get a transaction by ID
    pub fn get_transaction(&self, tx_id: &[u8]) -> Result<Transaction> {
        // First check mempool
        if let Some(tx) = self.mempool.get_transaction(tx_id) {
            return Ok(tx);
        }
        
        // If not in mempool and we have a chain store, check there
        if let Some(ref chain_store) = self.chain_store {
            return chain_store.get_transaction(tx_id);
        }
        
        Err(Error::State(format!("Transaction not found with ID {:?}", tx_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ShardId;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    fn current_time_micros() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }
    
    #[test]
    fn test_new_blockchain() {
        let blockchain = Blockchain::new().unwrap();
        assert_eq!(blockchain.get_height(), 0);
        assert_eq!(blockchain.get_genesis_hash().len(), 32);
        assert_eq!(blockchain.get_latest_hash().len(), 32);
    }
    
    #[test]
    fn test_genesis_block() {
        let blockchain = Blockchain::new().unwrap();
        
        // Generate a genesis block
        let genesis = blockchain.generate_genesis_block(None, None, None).unwrap();
        
        assert_eq!(genesis.header.index, 0);
        assert_eq!(genesis.header.previous_hash, vec![0; 32]);
        
        // Test timestamp is reasonable
        let now = current_time_micros();
        assert!(genesis.header.timestamp <= now);
        assert!(genesis.header.timestamp > now - 1000000); // Within 1 second
    }
    
    #[test]
    fn test_blockchain_initialization() {
        let mut blockchain = Blockchain::new().unwrap();
        
        // Generate and add genesis block
        let genesis = blockchain.generate_genesis_block(None, None, None).unwrap();
        blockchain.initialize_with_genesis(genesis.clone()).unwrap();
        
        // Check blockchain state
        assert_eq!(blockchain.get_height(), 0);
        
        // Retrieve genesis block
        let retrieved = blockchain.get_block_by_height(0).unwrap();
        assert_eq!(retrieved.header.index, 0);
    }
    
    #[test]
    fn test_create_and_add_block() {
        let mut blockchain = Blockchain::new().unwrap();
        
        // Initialize with genesis block
        let genesis = blockchain.generate_genesis_block(None, None, None).unwrap();
        blockchain.initialize_with_genesis(genesis).unwrap();
        
        // Create a new block
        let block = blockchain.create_block(vec![0 as ShardId], None).unwrap();
        
        assert_eq!(block.header.index, 1);
        assert_eq!(block.header.previous_hash, blockchain.get_latest_hash());
        
        // Add the block to the chain
        blockchain.add_block(block.clone()).unwrap();
        
        // Check chain state
        assert_eq!(blockchain.get_height(), 1);
        
        // Retrieve the block
        let retrieved = blockchain.get_block_by_height(1).unwrap();
        assert_eq!(retrieved.header.index, 1);
    }
    
    #[test]
    fn test_block_validation() {
        let mut blockchain = Blockchain::new().unwrap();
        
        // Initialize with genesis block
        let genesis = blockchain.generate_genesis_block(None, None, None).unwrap();
        blockchain.initialize_with_genesis(genesis).unwrap();
        
        // Create a valid block
        let mut valid_block = blockchain.create_block(vec![0 as ShardId], None).unwrap();
        
        // Block with wrong height should fail
        let mut invalid_block = valid_block.clone();
        invalid_block.header.index = 999;
        assert!(blockchain.validate_block(&invalid_block).is_err());
        
        // Block with wrong previous hash should fail
        let mut invalid_block = valid_block.clone();
        invalid_block.header.previous_hash = vec![9; 32];
        assert!(blockchain.validate_block(&invalid_block).is_err());
        
        // Block with future timestamp should fail
        let mut invalid_block = valid_block.clone();
        invalid_block.header.timestamp = current_time_micros() + 120_000_000; // 2 minutes in the future
        assert!(blockchain.validate_block(&invalid_block).is_err());
        
        // Valid block should pass
        assert!(blockchain.validate_block(&valid_block).is_ok());
    }
    
    #[test]
    fn test_transaction_lifecycle() {
        let mut blockchain = Blockchain::new().unwrap();
        
        // Initialize with genesis block
        let genesis = blockchain.generate_genesis_block(None, None, None).unwrap();
        blockchain.initialize_with_genesis(genesis).unwrap();
        
        // Create a transaction
        let sender_key = vec![1; 32];
        let recipient = vec![2; 20];
        let tx = Transaction::new_transfer(
            sender_key,
            0, // sender shard
            recipient,
            0, // recipient shard
            1000, // amount
            10, // fee
            0, // nonce
        );
        
        // Add to mempool
        blockchain.add_transaction(tx.clone()).unwrap();
        
        // Should be retrievable from mempool
        let retrieved = blockchain.get_transaction(&tx.id).unwrap();
        assert_eq!(retrieved.id, tx.id);
        
        // Create a block - should include the transaction
        let block = blockchain.create_block(vec![0 as ShardId], None).unwrap();
        
        // Check if the block contains the transaction
        // In a real implementation, this would be more sophisticated
        // with proper transaction processing and Merkle trees
        assert!(!block.shard_data.is_empty());
        
        // Add block to chain
        blockchain.add_block(block).unwrap();
        
        // Transaction should no longer be in mempool
        assert!(blockchain.mempool.get_transaction(&tx.id).is_none());
    }
}
