//! # Memory Storage
//!
//! This module defines the in-memory storage for the state database.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::types::ShardId;

/// In-memory storage for testing or fallback
pub struct MemoryStorage {
    /// Account balances
    pub account_balances: Arc<Mutex<HashMap<Vec<u8>, u64>>>,
    
    /// Account nonces
    pub account_nonces: Arc<Mutex<HashMap<Vec<u8>, u64>>>,
    
    /// Contract code
    pub contract_code: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    
    /// Contract storage
    pub contract_storage: Arc<Mutex<HashMap<Vec<u8>, HashMap<Vec<u8>, Vec<u8>>>>>,
    
    /// Shard state roots
    pub shard_state_roots: Arc<Mutex<HashMap<ShardId, Vec<u8>>>>,
}

impl MemoryStorage {
    /// Create a new memory storage
    pub fn new() -> Self {
        MemoryStorage {
            account_balances: Arc::new(Mutex::new(HashMap::new())),
            account_nonces: Arc::new(Mutex::new(HashMap::new())),
            contract_code: Arc::new(Mutex::new(HashMap::new())),
            contract_storage: Arc::new(Mutex::new(HashMap::new())),
            shard_state_roots: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
