//! # Chain Storage Implementation
//! 
//! This module provides storage for blockchain data, including blocks and transactions.

use crate::blockchain::{Block, Transaction};
use crate::types::{Result, Error, BlockHeight};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Keys used in the chain store database
pub enum ChainStoreKey {
    /// Key for storing a block by height
    BlockHeight(BlockHeight),
    
    /// Key for storing a block by hash
    BlockHash(Vec<u8>),
    
    /// Key for storing a transaction by ID
    Transaction(Vec<u8>),
    
    /// Key for the latest block height
    LatestBlockHeight,
    
    /// Key for the latest block hash
    LatestBlockHash,
    
    /// Key for the genesis block hash
    GenesisBlockHash,
}

impl ChainStoreKey {
    /// Convert the key to a byte representation for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ChainStoreKey::BlockHeight(height) => {
                let mut key = Vec::with_capacity(9);
                key.push(0x01); // prefix for block height
                key.extend_from_slice(&height.to_be_bytes());
                key
            },
            ChainStoreKey::BlockHash(hash) => {
                let mut key = Vec::with_capacity(1 + hash.len());
                key.push(0x02); // prefix for block hash
                key.extend_from_slice(hash);
                key
            },
            ChainStoreKey::Transaction(tx_id) => {
                let mut key = Vec::with_capacity(1 + tx_id.len());
                key.push(0x03); // prefix for transaction
                key.extend_from_slice(tx_id);
                key
            },
            ChainStoreKey::LatestBlockHeight => vec![0x04],
            ChainStoreKey::LatestBlockHash => vec![0x05],
            ChainStoreKey::GenesisBlockHash => vec![0x06],
        }
    }
}

/// Chain store for storing blockchain data
pub struct ChainStore {
    /// Database path
    path: String,
    
    /// In-memory cache of blocks by height (for testing/prototype)
    // In a real implementation, this would be a database connection
    blocks_by_height: Arc<Mutex<HashMap<BlockHeight, Block>>>,
    
    /// In-memory cache of blocks by hash (for testing/prototype)
    blocks_by_hash: Arc<Mutex<HashMap<Vec<u8>, Block>>>,
    
    /// In-memory cache of transactions by ID (for testing/prototype)
    transactions: Arc<Mutex<HashMap<Vec<u8>, Transaction>>>,
    
    /// Latest block height
    latest_height: Arc<Mutex<Option<BlockHeight>>>,
    
    /// Latest block hash
    latest_hash: Arc<Mutex<Option<Vec<u8>>>>,
    
    /// Genesis block hash
    genesis_hash: Arc<Mutex<Option<Vec<u8>>>>,
}

impl ChainStore {
    /// Create a new chain store at the specified path
    pub fn new(path: &str, _config: &super::StorageConfig) -> Result<Self> {
        // Ensure the directory exists
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            std::fs::create_dir_all(path_obj)
                .map_err(|e| Error::Io(e))?;
        }
        
        // In a real implementation, we would open the database here
        // For now, we just use in-memory hashmaps
        
        Ok(ChainStore {
            path: path.to_string(),
            blocks_by_height: Arc::new(Mutex::new(HashMap::new())),
            blocks_by_hash: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
            latest_height: Arc::new(Mutex::new(None)),
            latest_hash: Arc::new(Mutex::new(None)),
            genesis_hash: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Close the chain store
    pub fn close(&self) -> Result<()> {
        // In a real implementation, we would close the database connection
        Ok(())
    }
    
    /// Store a block in the chain store
    pub fn put_block(&self, block: Block) -> Result<()> {
        let height = block.header.index;
        let hash = block.header.previous_hash.clone();
        
        // Store block by height
        let mut blocks_by_height = self.blocks_by_height.lock().unwrap();
        blocks_by_height.insert(height, block.clone());
        
        // Store block by hash
        let mut blocks_by_hash = self.blocks_by_hash.lock().unwrap();
        blocks_by_hash.insert(hash.clone(), block);
        
        // Update latest height and hash if this is a new latest block
        let mut latest_height = self.latest_height.lock().unwrap();
        if latest_height.is_none() || latest_height.unwrap() < height {
            *latest_height = Some(height);
            
            let mut latest_hash = self.latest_hash.lock().unwrap();
            *latest_hash = Some(hash.clone());
        }
        
        // Update genesis hash if this is the genesis block
        if height == 0 {
            let mut genesis_hash = self.genesis_hash.lock().unwrap();
            *genesis_hash = Some(hash);
        }
        
        Ok(())
    }
    
    /// Store a transaction in the chain store
    pub fn put_transaction(&self, transaction: Transaction) -> Result<()> {
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(transaction.id.clone(), transaction);
        Ok(())
    }
    
    /// Get a block by its height
    pub fn get_block_by_height(&self, height: BlockHeight) -> Result<Block> {
        let blocks_by_height = self.blocks_by_height.lock().unwrap();
        blocks_by_height.get(&height)
            .cloned()
            .ok_or_else(|| Error::State(format!("Block not found at height {}", height)))
    }
    
    /// Get a block by its hash
    pub fn get_block_by_hash(&self, hash: &[u8]) -> Result<Block> {
        let blocks_by_hash = self.blocks_by_hash.lock().unwrap();
        blocks_by_hash.get(hash)
            .cloned()
            .ok_or_else(|| Error::State(format!("Block not found with hash {:?}", hash)))
    }
    
    /// Get a transaction by its ID
    pub fn get_transaction(&self, tx_id: &[u8]) -> Result<Transaction> {
        let transactions = self.transactions.lock().unwrap();
        transactions.get(tx_id)
            .cloned()
            .ok_or_else(|| Error::State(format!("Transaction not found with ID {:?}", tx_id)))
    }
    
    /// Get the latest block height
    pub fn get_latest_height(&self) -> Option<BlockHeight> {
        *self.latest_height.lock().unwrap()
    }
    
    /// Get the latest block hash
    pub fn get_latest_hash(&self) -> Option<Vec<u8>> {
        self.latest_hash.lock().unwrap().clone()
    }
    
    /// Get the genesis block hash
    pub fn get_genesis_hash(&self) -> Option<Vec<u8>> {
        self.genesis_hash.lock().unwrap().clone()
    }
    
    /// Check if a block exists by its height
    pub fn has_block_height(&self, height: BlockHeight) -> bool {
        let blocks_by_height = self.blocks_by_height.lock().unwrap();
        blocks_by_height.contains_key(&height)
    }
    
    /// Check if a block exists by its hash
    pub fn has_block_hash(&self, hash: &[u8]) -> bool {
        let blocks_by_hash = self.blocks_by_hash.lock().unwrap();
        blocks_by_hash.contains_key(hash)
    }
    
    /// Check if a transaction exists by its ID
    pub fn has_transaction(&self, tx_id: &[u8]) -> bool {
        let transactions = self.transactions.lock().unwrap();
        transactions.contains_key(tx_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Block;
    use crate::types::ShardId;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    fn current_time_micros() -> u64 {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        
        // Convert to microseconds
        since_epoch.as_secs() * 1_000_000 + since_epoch.subsec_micros() as u64
    }
    
    fn create_test_block(height: u64) -> Block {
        Block::new(
            height,
            current_time_micros(),
            vec![0; 32], // previous hash
            vec![0 as ShardId, 1], // shard IDs
        )
    }
    
    fn temp_dir() -> String {
        let mut dir = env::temp_dir();
        dir.push(format!("sebure-test-chain-{}", rand::random::<u64>()));
        dir.to_str().unwrap().to_string()
    }
    
    #[test]
    fn test_chain_store_creation() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let chain_store = ChainStore::new(&path, &config);
        assert!(chain_store.is_ok());
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_store_and_retrieve_block() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let chain_store = ChainStore::new(&path, &config).unwrap();
        
        // Create and store a block
        let block = create_test_block(1);
        let hash = block.header.previous_hash.clone();
        chain_store.put_block(block.clone()).unwrap();
        
        // Retrieve by height
        let retrieved = chain_store.get_block_by_height(1).unwrap();
        assert_eq!(retrieved.header.index, 1);
        
        // Retrieve by hash
        let retrieved = chain_store.get_block_by_hash(&hash).unwrap();
        assert_eq!(retrieved.header.index, 1);
        
        // Check existence
        assert!(chain_store.has_block_height(1));
        assert!(chain_store.has_block_hash(&hash));
        
        // Check latest height and hash
        assert_eq!(chain_store.get_latest_height(), Some(1));
        assert_eq!(chain_store.get_latest_hash(), Some(hash));
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_genesis_block() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let chain_store = ChainStore::new(&path, &config).unwrap();
        
        // Create and store genesis block
        let genesis = create_test_block(0);
        let hash = genesis.header.previous_hash.clone();
        chain_store.put_block(genesis).unwrap();
        
        // Check genesis hash
        assert_eq!(chain_store.get_genesis_hash(), Some(hash));
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_missing_block() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let chain_store = ChainStore::new(&path, &config).unwrap();
        
        // Try to retrieve a non-existent block
        assert!(chain_store.get_block_by_height(999).is_err());
        assert!(chain_store.get_block_by_hash(&vec![9, 9, 9]).is_err());
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
}
