//! # State Database Implementation
//! 
//! This module provides storage for blockchain state, including account balances,
//! smart contract state, and other state data. It implements both LevelDB for
//! key-value storage and LMDB for memory-mapped state access.

use crate::types::{Result, Error, ShardId};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use log::{info, warn, debug, error};

// LevelDB dependencies
use leveldb::database::Database as LevelDatabase;
use leveldb::options::{Options as LevelOptions, ReadOptions, WriteOptions};
use leveldb::iterator::Iterator as LevelIter;
use leveldb::kv::KV;  // Add missing KV trait import

// LMDB dependencies
use lmdb::{Environment, Database as LmdbDatabase, DatabaseFlags, EnvironmentFlags, Transaction, WriteFlags};

use super::database_types::{DatabaseBackend, DatabaseColumn};
use super::memory_storage::MemoryStorage;
use super::account::AccountInfo;
use super::iterator::LevelDBIterator;
use crate::storage::state_db::key_impl::DBKey;

/// State database for storing blockchain state
pub struct StateDB {
    /// Database path
    pub(crate) path: String,
    
    /// Database backend type
    pub(crate) backend: DatabaseBackend,
    
    /// LevelDB database instances (one per column family)
    pub(crate) level_dbs: Option<HashMap<DatabaseColumn, Arc<RwLock<LevelDatabase<DBKey>>>>>,
    
    /// LMDB environment
    pub(crate) lmdb_env: Option<Arc<RwLock<Environment>>>,
    
    /// LMDB databases (one per column family)
    pub(crate) lmdb_dbs: Option<HashMap<DatabaseColumn, Arc<RwLock<LmdbDatabase>>>>,
    
    /// In-memory fallback for testing or when databases aren't available
    pub(crate) memory_storage: Option<MemoryStorage>,
    
    /// Database version for migrations
    pub(crate) version: u32,
}

impl StateDB {
    /// Create a new state database at the specified path
    pub fn new(path: &str, config: &super::super::StorageConfig) -> Result<Self> {
        // Ensure the directory exists
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            std::fs::create_dir_all(path_obj)
                .map_err(|e| Error::Io(e))?;
        }
        
        // Determine which backend to use
        let backend = if cfg!(test) {
            // Use memory backend for tests
            DatabaseBackend::Memory
        } else {
            // Use LevelDB by default
            DatabaseBackend::LevelDB
        };
        
        match backend {
            DatabaseBackend::Memory => {
                info!("Using in-memory database backend for state DB");
                Ok(StateDB {
                    path: path.to_string(),
                    backend: DatabaseBackend::Memory,
                    level_dbs: None,
                    lmdb_env: None,
                    lmdb_dbs: None,
                    memory_storage: Some(MemoryStorage::new()),
                    version: 1,
                })
            },
            DatabaseBackend::LevelDB => {
                info!("Initializing LevelDB backend for state DB at {}", path);
                Self::init_leveldb(path, config)
            },
            DatabaseBackend::LMDB => {
                info!("Initializing LMDB backend for state DB at {}", path);
                Self::init_lmdb(path, config)
            },
        }
    }
    
    /// Initialize LevelDB backend
    pub(crate) fn init_leveldb(path: &str, config: &super::super::StorageConfig) -> Result<Self> {
        let mut level_dbs = HashMap::new();
        
        // Create options
        let mut options = LevelOptions::new();
        options.create_if_missing = config.create_if_missing;
        
        // Create a database for each column
        for column in [
            DatabaseColumn::AccountBalance,
            DatabaseColumn::AccountNonce,
            DatabaseColumn::ContractCode,
            DatabaseColumn::ContractStorage,
            DatabaseColumn::ShardStateRoot,
            DatabaseColumn::ValidatorData,
            DatabaseColumn::StakingData,
            DatabaseColumn::Metadata,
        ].iter() {
            let column_path = format!("{}/{}", path, column.name());
            let column_path_obj = Path::new(&column_path);
            
            // Create directory if it doesn't exist
            if !column_path_obj.exists() {
                std::fs::create_dir_all(column_path_obj)
                    .map_err(|e| Error::Io(e))?;
            }
            
            // Open database
            let db = LevelDatabase::<DBKey>::open(column_path_obj, options)
                .map_err(|e| Error::Storage(format!("Failed to open LevelDB: {}", e)))?;
            
            level_dbs.insert(*column, Arc::new(RwLock::new(db)));
        }
        
        // Create state DB
        let mut state_db = StateDB {
            path: path.to_string(),
            backend: DatabaseBackend::LevelDB,
            level_dbs: Some(level_dbs),
            lmdb_env: None,
            lmdb_dbs: None,
            memory_storage: None,
            version: 1,
        };
        
        // Check if we need to perform migrations
        state_db.check_and_migrate()?;
        
        Ok(state_db)
    }
    
    /// Initialize LMDB backend
    pub(crate) fn init_lmdb(path: &str, config: &super::super::StorageConfig) -> Result<Self> {
        // Create LMDB environment
        let env_path = Path::new(path);
        
        // Create environment with appropriate flags
        let env = Environment::new()
            .set_flags(EnvironmentFlags::NO_SUB_DIR | EnvironmentFlags::NO_TLS)
            .set_max_dbs(10) // One for each column plus some extra
            .set_map_size(1024 * 1024 * 1024) // 1GB map size
            .open(env_path)
            .map_err(|e| Error::Storage(format!("Failed to open LMDB environment: {}", e)))?;
        
        let env = Arc::new(RwLock::new(env));
        let mut lmdb_dbs = HashMap::new();
        
        // Create a database for each column
        for column in [
            DatabaseColumn::AccountBalance,
            DatabaseColumn::AccountNonce,
            DatabaseColumn::ContractCode,
            DatabaseColumn::ContractStorage,
            DatabaseColumn::ShardStateRoot,
            DatabaseColumn::ValidatorData,
            DatabaseColumn::StakingData,
            DatabaseColumn::Metadata,
        ].iter() {
            // Open database
            let db = {
                let env_guard = env.read().unwrap();
                env_guard.create_db(Some(column.name()), DatabaseFlags::empty())
                    .map_err(|e| Error::Storage(format!("Failed to create LMDB database: {}", e)))?
            };
            
            lmdb_dbs.insert(*column, Arc::new(RwLock::new(db)));
        }
        
        // Create state DB
        let mut state_db = StateDB {
            path: path.to_string(),
            backend: DatabaseBackend::LMDB,
            level_dbs: None,
            lmdb_env: Some(env),
            lmdb_dbs: Some(lmdb_dbs),
            memory_storage: None,
            version: 1,
        };
        
        // Check if we need to perform migrations
        state_db.check_and_migrate()?;
        
        Ok(state_db)
    }
    
    /// Check if database needs migration and perform if necessary
    fn check_and_migrate(&mut self) -> Result<()> {
        // Get current version from metadata
        let current_version = self.get_db_version()?;
        
        if current_version < self.version {
            info!("Migrating database from version {} to {}", current_version, self.version);
            self.migrate_database(current_version)?;
        }
        
        Ok(())
    }
    
    /// Get database version from metadata
    fn get_db_version(&self) -> Result<u32> {
        // Try to get version from metadata
        match self.backend {
            DatabaseBackend::Memory => {
                // Memory backend always returns version 1
                Ok(1)
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::Metadata) {
                        let db_guard = db.read().unwrap();
                        let key = DBKey::new(b"version".to_vec());
                        match db_guard.get(&key, &ReadOptions::new()) {
                            Ok(Some(version_bytes)) => {
                                // Convert bytes to u32
                                if version_bytes.len() == 4 {
                                    let version = u32::from_be_bytes([
                                        version_bytes[0],
                                        version_bytes[1],
                                        version_bytes[2],
                                        version_bytes[3],
                                    ]);
                                    return Ok(version);
                                }
                                // Invalid version format, return 1
                                Ok(1)
                            },
                            _ => {
                                // No version found, return 1
                                Ok(1)
                            }
                        }
                    } else {
                        // No metadata database, return 1
                        Ok(1)
                    }
                } else {
                    // No level_dbs, return 1
                    Ok(1)
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::Metadata) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            let txn = env_guard.begin_ro_txn()
                                .map_err(|e| Error::Storage(format!("Failed to begin LMDB transaction: {}", e)))?;
                            
                            match txn.get(*db_guard, &b"version"[..]) {
                                Ok(version_bytes) => {
                                    // Convert bytes to u32
                                    if version_bytes.len() == 4 {
                                        let version = u32::from_be_bytes([
                                            version_bytes[0],
                                            version_bytes[1],
                                            version_bytes[2],
                                            version_bytes[3],
                                        ]);
                                        return Ok(version);
                                    }
                                    // Invalid version format, return 1
                                    Ok(1)
                                },
                                _ => {
                                    // No version found, return 1
                                    Ok(1)
                                }
                            }
                        } else {
                            // No metadata database, return 1
                            Ok(1)
                        }
                    } else {
                        // No lmdb_dbs, return 1
                        Ok(1)
                    }
                } else {
                    // No env, return 1
                    Ok(1)
                }
            },
        }
    }
    
    /// Migrate database from one version to another
    fn migrate_database(&mut self, from_version: u32) -> Result<()> {
        // Perform migrations based on version
        match from_version {
            1 => {
                // Migrate from version 1 to 2
                info!("Migrating database from version 1 to 2");
                // No migrations yet, just update version
                self.set_db_version(2)?;
            },
            _ => {
                // Unknown version, just update to current version
                warn!("Unknown database version {}, updating to {}", from_version, self.version);
                self.set_db_version(self.version)?;
            }
        }
        
        Ok(())
    }
    
    /// Set database version in metadata
    fn set_db_version(&self, version: u32) -> Result<()> {
        let version_bytes = version.to_be_bytes();
        
        match self.backend {
            DatabaseBackend::Memory => {
                // Memory backend doesn't persist version
                Ok(())
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::Metadata) {
                        let db_guard = db.write().unwrap();
                        let key = DBKey::new(b"version".to_vec());
                        db_guard.put(&key, &version_bytes, &WriteOptions::new())
                            .map_err(|e| Error::Storage(format!("Failed to write version to LevelDB: {}", e)))?;
                        Ok(())
                    } else {
                        Err(Error::Storage("Metadata database not found".to_string()))
                    }
                } else {
                    Err(Error::Storage("LevelDB databases not initialized".to_string()))
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::Metadata) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            let mut txn = env_guard.begin_rw_txn()
                                .map_err(|e| Error::Storage(format!("Failed to begin LMDB transaction: {}", e)))?;
                            
                            txn.put(*db_guard, &b"version"[..], &version_bytes, WriteFlags::empty())
                                .map_err(|e| Error::Storage(format!("Failed to write version to LMDB: {}", e)))?;
                            
                            txn.commit()
                                .map_err(|e| Error::Storage(format!("Failed to commit LMDB transaction: {}", e)))?;
                            
                            Ok(())
                        } else {
                            Err(Error::Storage("Metadata database not found".to_string()))
                        }
                    } else {
                        Err(Error::Storage("LMDB databases not initialized".to_string()))
                    }
                } else {
                    Err(Error::Storage("LMDB environment not initialized".to_string()))
                }
            },
        }
    }
    
    /// Close the state database
    pub fn close(&self) -> Result<()> {
        match self.backend {
            DatabaseBackend::Memory => {
                // Nothing to close for memory backend
                Ok(())
            },
            DatabaseBackend::LevelDB => {
                // LevelDB will be closed when dropped
                Ok(())
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    // Sync environment to ensure all data is written
                    let env_guard = env.read().unwrap();
                    env_guard.sync(true)
                        .map_err(|e| Error::Storage(format!("Failed to sync LMDB environment: {}", e)))?;
                }
                Ok(())
            },
        }
    }
    
    /// Get account information
    pub fn get_account_info(&self, address: &[u8]) -> Result<AccountInfo> {
        let balance = self.get_account_balance(address).unwrap_or(0);
        let nonce = self.get_account_nonce(address).unwrap_or(0);
        let is_contract = self.has_contract_code(address);
        
        Ok(AccountInfo {
            balance,
            nonce,
            is_contract,
        })
    }
    
    /// Get account balance
    pub fn get_account_balance(&self, address: &[u8]) -> Result<u64> {
        match self.backend {
            DatabaseBackend::Memory => {
                if let Some(memory_storage) = &self.memory_storage {
                    let balances = memory_storage.account_balances.lock().unwrap();
                    Ok(balances.get(address).cloned().unwrap_or(0))
                } else {
                    Err(Error::Storage("Memory storage not initialized".to_string()))
                }
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::AccountBalance) {
                        let db_guard = db.read().unwrap();
                        let key = DBKey::new(address.to_vec());
                        match db_guard.get(&key, &ReadOptions::new()) {
                            Ok(Some(balance_bytes)) => {
                                // Convert bytes to u64
                                if balance_bytes.len() == 8 {
                                    let balance = u64::from_be_bytes([
                                        balance_bytes[0],
                                        balance_bytes[1],
                                        balance_bytes[2],
                                        balance_bytes[3],
                                        balance_bytes[4],
                                        balance_bytes[5],
                                        balance_bytes[6],
                                        balance_bytes[7],
                                    ]);
                                    Ok(balance)
                                } else {
                                    Err(Error::Storage(format!("Invalid balance format for address {:?}", address)))
                                }
                            },
                            Ok(None) => Ok(0), // No balance found, return 0
                            Err(e) => Err(Error::Storage(format!("Failed to get balance from LevelDB: {}", e))),
                        }
                    } else {
                        Err(Error::Storage("Account balance database not found".to_string()))
                    }
                } else {
                    Err(Error::Storage("LevelDB databases not initialized".to_string()))
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::AccountBalance) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            let txn = env_guard.begin_ro_txn()
                                .map_err(|e| Error::Storage(format!("Failed to begin LMDB transaction: {}", e)))?;
                            
                            match txn.get(*db_guard, &address[..]) {
                                Ok(balance_bytes) => {
                                    // Convert bytes to u64
                                    if balance_bytes.len() == 8 {
                                        let balance = u64::from_be_bytes([
                                            balance_bytes[0],
                                            balance_bytes[1],
                                            balance_bytes[2],
                                            balance_bytes[3],
                                            balance_bytes[4],
                                            balance_bytes[5],
                                            balance_bytes[6],
                                            balance_bytes[7],
                                        ]);
                                        Ok(balance)
                                    } else {
                                        Err(Error::Storage(format!("Invalid balance format for address {:?}", address)))
                                    }
                                },
                                Err(_) => Ok(0), // No balance found, return 0
                            }
                        } else {
                            Err(Error::Storage("Account balance database not found".to_string()))
                        }
                    } else {
                        Err(Error::Storage("LMDB databases not initialized".to_string()))
                    }
                } else {
                    Err(Error::Storage("LMDB environment not initialized".to_string()))
                }
            },
        }
    }
    
    /// Set account balance
    pub fn set_account_balance(&self, address: &[u8], balance: u64) -> Result<()> {
        let balance_bytes = balance.to_be_bytes();
        
        match self.backend {
            DatabaseBackend::Memory => {
                if let Some(memory_storage) = &self.memory_storage {
                    let mut balances = memory_storage.account_balances.lock().unwrap();
                    balances.insert(address.to_vec(), balance);
                    Ok(())
                } else {
                    Err(Error::Storage("Memory storage not initialized".to_string()))
                }
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::AccountBalance) {
                        let db_guard = db.write().unwrap();
                        let key = DBKey::new(address.to_vec());
                        db_guard.put(&key, &balance_bytes, &WriteOptions::new())
                            .map_err(|e| Error::Storage(format!("Failed to write balance to LevelDB: {}", e)))
                    } else {
                        Err(Error::Storage("Account balance database not found".to_string()))
                    }
                } else {
                    Err(Error::Storage("LevelDB databases not initialized".to_string()))
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::AccountBalance) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            let mut txn = env_guard.begin_rw_txn()
                                .map_err(|e| Error::Storage(format!("Failed to begin LMDB transaction: {}", e)))?;
                            
                            txn.put(*db_guard, &address[..], &balance_bytes, WriteFlags::empty())
                                .map_err(|e| Error::Storage(format!("Failed to write balance to LMDB: {}", e)))?;
                            
                            txn.commit()
                                .map_err(|e| Error::Storage(format!("Failed to commit LMDB transaction: {}", e)))
                        } else {
                            Err(Error::Storage("Account balance database not found".to_string()))
                        }
                    } else {
                        Err(Error::Storage("LMDB databases not initialized".to_string()))
                    }
                } else {
                    Err(Error::Storage("LMDB environment not initialized".to_string()))
                }
            },
        }
    }
    
    /// Adjust account balance by the given amount (can be positive or negative)
    pub fn adjust_account_balance(&self, address: &[u8], amount: i64) -> Result<u64> {
        let current_balance = self.get_account_balance(address).unwrap_or(0);
        
        let new_balance = if amount >= 0 {
            current_balance + amount as u64
        } else {
            // Ensure we don't underflow
            if current_balance < ((-amount) as u64) {
                return Err(Error::State(format!(
                    "Insufficient balance: current={}, adjustment={}",
                    current_balance, amount
                )));
            }
            current_balance - ((-amount) as u64)
        };
        
        self.set_account_balance(address, new_balance)?;
        Ok(new_balance)
    }
    
    /// Get account nonce
    pub fn get_account_nonce(&self, address: &[u8]) -> Result<u64> {
        match self.backend {
            DatabaseBackend::Memory => {
                if let Some(memory_storage) = &self.memory_storage {
                    let nonces = memory_storage.account_nonces.lock().unwrap();
                    Ok(nonces.get(address).cloned().unwrap_or(0))
                } else {
                    Err(Error::Storage("Memory storage not initialized".to_string()))
                }
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::AccountNonce) {
                        let db_guard = db.read().unwrap();
                        let key = DBKey::new(address.to_vec());
                        match db_guard.get(&key, &ReadOptions::new()) {
                            Ok(Some(nonce_bytes)) => {
                                // Convert bytes to u64
                                if nonce_bytes.len() == 8 {
                                    let nonce = u64::from_be_bytes([
                                        nonce_bytes[0],
                                        nonce_bytes[1],
                                        nonce_bytes[2],
                                        nonce_bytes[3],
                                        nonce_bytes[4],
                                        nonce_bytes[5],
                                        nonce_bytes[6],
                                        nonce_bytes[7],
                                    ]);
                                    return Ok(nonce);
                                }
                                Err(Error::Storage(format!("Invalid nonce format for address {:?}", address)))
                            },
                            Ok(None) => Ok(0), // No nonce found, return 0
                            Err(e) => Err(Error::Storage(format!("Failed to get nonce from LevelDB: {}", e))),
                        }
                    } else {
                        Err(Error::Storage("Account nonce database not found".to_string()))
                    }
                } else {
                    Err(Error::Storage("LevelDB databases not initialized".to_string()))
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::AccountNonce) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            let txn = env_guard.begin_ro_txn()
                                .map_err(|e| Error::Storage(format!("Failed to begin LMDB transaction: {}", e)))?;
                            
                            match txn.get(*db_guard, &address[..]) {
                                Ok(nonce_bytes) => {
                                    // Convert bytes to u64
                                    if nonce_bytes.len() == 8 {
                                        let nonce = u64::from_be_bytes([
                                            nonce_bytes[0],
                                            nonce_bytes[1],
                                            nonce_bytes[2],
                                            nonce_bytes[3],
                                            nonce_bytes[4],
                                            nonce_bytes[5],
                                            nonce_bytes[6],
                                            nonce_bytes[7],
                                        ]);
                                        Ok(nonce)
                                    } else {
                                        Err(Error::Storage(format!("Invalid nonce format for address {:?}", address)))
                                    }
                                },
                                Err(_) => Ok(0), // No nonce found, return 0
                            }
                        } else {
                            Err(Error::Storage("Account nonce database not found".to_string()))
                        }
                    } else {
                        Err(Error::Storage("LMDB databases not initialized".to_string()))
                    }
                } else {
                    Err(Error::Storage("LMDB environment not initialized".to_string()))
                }
            },
        }
    }
    
    /// Set account nonce
    pub fn set_account_nonce(&self, address: &[u8], nonce: u64) -> Result<()> {
        let nonce_bytes = nonce.to_be_bytes();
        
        match self.backend {
            DatabaseBackend::Memory => {
                if let Some(memory_storage) = &self.memory_storage {
                    let mut nonces = memory_storage.account_nonces.lock().unwrap();
                    nonces.insert(address.to_vec(), nonce);
                    Ok(())
                } else {
                    Err(Error::Storage("Memory storage not initialized".to_string()))
                }
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::AccountNonce) {
                        let db_guard = db.write().unwrap();
                        let key = DBKey::new(address.to_vec());
                        db_guard.put(&key, &nonce_bytes, &WriteOptions::new())
                            .map_err(|e| Error::Storage(format!("Failed to write nonce to LevelDB: {}", e)))
                    } else {
                        Err(Error::Storage("Account nonce database not found".to_string()))
                    }
                } else {
                    Err(Error::Storage("LevelDB databases not initialized".to_string()))
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::AccountNonce) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            let mut txn = env_guard.begin_rw_txn()
                                .map_err(|e| Error::Storage(format!("Failed to begin LMDB transaction: {}", e)))?;
                            
                            txn.put(*db_guard, &address[..], &nonce_bytes, WriteFlags::empty())
                                .map_err(|e| Error::Storage(format!("Failed to write nonce to LMDB: {}", e)))?;
                            
                            txn.commit()
                                .map_err(|e| Error::Storage(format!("Failed to commit LMDB transaction: {}", e)))
                        } else {
                            Err(Error::Storage("Account nonce database not found".to_string()))
                        }
                    } else {
                        Err(Error::Storage("LMDB databases not initialized".to_string()))
                    }
                } else {
                    Err(Error::Storage("LMDB environment not initialized".to_string()))
                }
            },
        }
    }
    
    /// Increment account nonce
    pub fn increment_account_nonce(&self, address: &[u8]) -> Result<u64> {
        let current_nonce = self.get_account_nonce(address).unwrap_or(0);
        let new_nonce = current_nonce + 1;
        
        self.set_account_nonce(address, new_nonce)?;
        Ok(new_nonce)
    }
    
    /// Check if an address has contract code
    pub fn has_contract_code(&self, address: &[u8]) -> bool {
        match self.backend {
            DatabaseBackend::Memory => {
                if let Some(memory_storage) = &self.memory_storage {
                    let codes = memory_storage.contract_code.lock().unwrap();
                    codes.contains_key(address)
                } else {
                    false
                }
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::ContractCode) {
                        let db_guard = db.read().unwrap();
                        let key = DBKey::new(address.to_vec());
                        match db_guard.get(&key, &ReadOptions::new()) {
                            Ok(Some(_)) => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::ContractCode) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            match env_guard.begin_ro_txn() {
                                Ok(txn) => {
                                    match txn.get(*db_guard, &address[..]) {
                                        Ok(_) => true,
                                        _ => false,
                                    }
                                },
                                _ => false,
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
        }
    }
    
    /// Get contract code
    pub fn get_contract_code(&self, address: &[u8]) -> Result<Vec<u8>> {
        match self.backend {
            DatabaseBackend::Memory => {
                if let Some(memory_storage) = &self.memory_storage {
                    let codes = memory_storage.contract_code.lock().unwrap();
                    match codes.get(address) {
                        Some(code) => Ok(code.clone()),
                        None => Err(Error::Storage(format!("Contract code not found for address {:?}", address)))
                    }
                } else {
                    Err(Error::Storage("Memory storage not initialized".to_string()))
                }
            },
            DatabaseBackend::LevelDB => {
                if let Some(level_dbs) = &self.level_dbs {
                    if let Some(db) = level_dbs.get(&DatabaseColumn::ContractCode) {
                        let db_guard = db.read().unwrap();
                        let key = DBKey::new(address.to_vec());
                        match db_guard.get(&key, &ReadOptions::new()) {
                            Ok(Some(code)) => Ok(code),
                            Ok(None) => Err(Error::Storage(format!("Contract code not found for address {:?}", address))),
                            Err(e) => Err(Error::Storage(format!("Failed to get contract code from LevelDB: {}", e))),
                        }
                    } else {
                        Err(Error::Storage("Contract code database not found".to_string()))
                    }
                } else {
                    Err(Error::Storage("LevelDB databases not initialized".to_string()))
                }
            },
            DatabaseBackend::LMDB => {
                if let Some(env) = &self.lmdb_env {
                    if let Some(lmdb_dbs) = &self.lmdb_dbs {
                        if let Some(db) = lmdb_dbs.get(&DatabaseColumn::ContractCode) {
                            let env_guard = env.read().unwrap();
                            let db_guard = db.read().unwrap();
                            
                            let txn = env_guard.begin_ro_txn()
                                .map_err(|e| Error::Storage(format!("Failed to begin LMDB transaction: {}", e)))?;
                            
                            match txn.get(*db_guard, &address[..]) {
                                Ok(code) => Ok(code.to_vec()),
                                Err(_) => Err(Error::Storage(format!("Contract code not found for address {:?}", address))),
                            }
                        } else {
                            Err(Error::Storage("Contract code database not found".to_string()))
                        }
                    } else {
                        Err(Error::Storage("LMDB databases not initialized".to_string()))
                    }
                } else {
                    Err(Error::Storage("LMDB environment not initialized".to_string()))
                }
            },
        }
    }
}
