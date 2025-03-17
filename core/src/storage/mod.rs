//! # Storage Module
//! 
//! This module provides storage interfaces for the SEBURE blockchain,
//! including chain storage, state database, and configuration storage.

mod chain_store;
mod state_db;

// Re-export main types
pub use chain_store::ChainStore;
pub use state_db::StateDB;

use crate::types::{Result, Error};
use std::path::Path;

/// Storage configuration options
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Base data directory
    pub data_dir: String,
    
    /// Chain database path (relative to data_dir)
    pub chain_path: String,
    
    /// State database path (relative to data_dir)
    pub state_path: String,
    
    /// Maximum database open files
    pub max_open_files: i32,
    
    /// Database cache size in MB
    pub cache_size: usize,
    
    /// Whether to create database if it doesn't exist
    pub create_if_missing: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            data_dir: ".sebure".to_string(),
            chain_path: "chain".to_string(),
            state_path: "state".to_string(),
            max_open_files: 100,
            cache_size: 512,  // 512 MB
            create_if_missing: true,
        }
    }
}

/// Storage manages all storage components for the blockchain
pub struct Storage {
    /// Storage configuration
    config: StorageConfig,
    
    /// Chain storage for blocks and transactions
    chain_store: ChainStore,
    
    /// State database for account balances and state
    state_db: StateDB,
}

impl Storage {
    /// Create a new storage instance with the given configuration
    pub fn new(config: StorageConfig) -> Result<Self> {
        // Ensure data directory exists
        let data_dir = Path::new(&config.data_dir);
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir)
                .map_err(|e| Error::Io(e))?;
        }
        
        // Create chain store path
        let chain_path = data_dir.join(&config.chain_path);
        let chain_store = ChainStore::new(chain_path.to_str().unwrap(), &config)?;
        
        // Create state DB path
        let state_path = data_dir.join(&config.state_path);
        let state_db = StateDB::new(state_path.to_str().unwrap(), &config)?;
        
        Ok(Storage {
            config,
            chain_store,
            state_db,
        })
    }
    
    /// Get a reference to the chain store
    pub fn chain_store(&self) -> &ChainStore {
        &self.chain_store
    }
    
    /// Get a mutable reference to the chain store
    pub fn chain_store_mut(&mut self) -> &mut ChainStore {
        &mut self.chain_store
    }
    
    /// Get a reference to the state database
    pub fn state_db(&self) -> &StateDB {
        &self.state_db
    }
    
    /// Get a mutable reference to the state database
    pub fn state_db_mut(&mut self) -> &mut StateDB {
        &mut self.state_db
    }
    
    /// Close all storage components (should be called before shutdown)
    pub fn close(&mut self) -> Result<()> {
        // Close databases in order
        self.state_db.close()?;
        self.chain_store.close()?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::PathBuf;
    
    fn temp_dir() -> String {
        let mut dir = env::temp_dir();
        dir.push(format!("sebure-test-{}", rand::random::<u64>()));
        dir.to_str().unwrap().to_string()
    }
    
    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        
        assert_eq!(config.data_dir, ".sebure");
        assert_eq!(config.chain_path, "chain");
        assert_eq!(config.state_path, "state");
        assert_eq!(config.max_open_files, 100);
        assert_eq!(config.cache_size, 512);
        assert!(config.create_if_missing);
    }
    
    #[test]
    fn test_storage_creation() {
        let data_dir = temp_dir();
        
        let config = StorageConfig {
            data_dir: data_dir.clone(),
            chain_path: "chain-test".to_string(),
            state_path: "state-test".to_string(),
            ..StorageConfig::default()
        };
        
        // Create storage
        let storage = Storage::new(config.clone());
        assert!(storage.is_ok());
        
        // Check that directories were created
        let base_path = PathBuf::from(&data_dir);
        let chain_path = base_path.join("chain-test");
        let state_path = base_path.join("state-test");
        
        assert!(base_path.exists());
        assert!(chain_path.exists());
        assert!(state_path.exists());
        
        // Clean up
        std::fs::remove_dir_all(data_dir).ok();
    }
    
    #[test]
    fn test_storage_access() {
        let data_dir = temp_dir();
        
        let config = StorageConfig {
            data_dir: data_dir.clone(),
            ..StorageConfig::default()
        };
        
        // Create storage
        let mut storage = Storage::new(config).unwrap();
        
        // Access components
        let chain_store = storage.chain_store();
        let state_db = storage.state_db();
        
        // Basic functionality test
        assert!(chain_store.get_block_by_height(0).is_err()); // No genesis block yet
        assert_eq!(state_db.get_account_balance(&vec![0; 20]).unwrap_or(0), 0);
        
        // Close storage
        let result = storage.close();
        assert!(result.is_ok());
        
        // Clean up
        std::fs::remove_dir_all(data_dir).ok();
    }
}
