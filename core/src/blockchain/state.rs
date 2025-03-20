//! # Blockchain State Model
//! 
//! This module defines the state model for the SEBURE blockchain,
//! including account state and global blockchain state.

use serde::{Serialize, Deserialize};
use crate::types::{ShardId, Result};
use std::collections::HashMap;

/// Account type - determines the capabilities and behavior of the account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Standard user account
    User,
    
    /// Smart contract account
    Contract,
    
    /// Validator account
    Validator,
    
    /// System account
    System,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::User
    }
}

/// Account represents a user or smart contract on the blockchain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// Account address
    pub address: Vec<u8>,
    
    /// Account balance
    pub balance: u64,
    
    /// Account nonce (used to prevent replay attacks)
    pub nonce: u64,
    
    /// Account type
    pub account_type: AccountType,
    
    /// Smart contract code (if this is a contract account)
    pub code: Option<Vec<u8>>,
    
    /// Shard ID where this account is stored
    pub shard_id: ShardId,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last updated timestamp
    pub updated_at: u64,
}

impl Account {
    /// Create a new user account
    pub fn new_user(address: Vec<u8>, shard_id: ShardId, timestamp: u64) -> Self {
        Account {
            address,
            balance: 0,
            nonce: 0,
            account_type: AccountType::User,
            code: None,
            shard_id,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
    
    /// Create a new contract account
    pub fn new_contract(address: Vec<u8>, code: Vec<u8>, shard_id: ShardId, timestamp: u64) -> Self {
        Account {
            address,
            balance: 0,
            nonce: 0,
            account_type: AccountType::Contract,
            code: Some(code),
            shard_id,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
    
    /// Check if this is a contract account
    pub fn is_contract(&self) -> bool {
        self.account_type == AccountType::Contract && self.code.is_some()
    }
    
    /// Check if this is a validator account
    pub fn is_validator(&self) -> bool {
        self.account_type == AccountType::Validator
    }
    
    /// Convert the account to a serialized format
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| crate::types::Error::Serialization(e.to_string()))
    }
    
    /// Deserialize an account from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| crate::types::Error::Serialization(e.to_string()))
    }
}

/// ShardState represents the state of a single shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardState {
    /// Shard identifier
    pub shard_id: ShardId,
    
    /// State root hash
    pub state_root: Vec<u8>,
    
    /// Last updated block height
    pub last_updated_height: u64,
    
    /// Number of active accounts in this shard
    pub active_accounts: u32,
    
    /// Recent cross-shard transaction references
    pub recent_cross_shard_txs: Vec<Vec<u8>>,
    
    /// Neighboring shards that frequently interact with this shard
    pub neighbor_shards: Vec<ShardId>,
    
    /// Resource utilization metrics
    pub resource_utilization: f32,
}

/// GlobalState represents the combined state of all shards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalState {
    /// Current block height
    pub block_height: u64,
    
    /// Current state root (merkle root of all shard states)
    pub state_root: Vec<u8>,
    
    /// States of individual shards
    pub shard_states: HashMap<ShardId, ShardState>,
    
    /// Total transaction count
    pub total_transactions: u64,
    
    /// Total accounts count
    pub total_accounts: u32,
    
    /// Current validator set state root
    pub validator_state_root: Vec<u8>,
    
    /// Last updated timestamp
    pub last_updated: u64,
}

impl GlobalState {
    /// Create a new global state
    pub fn new() -> Self {
        GlobalState {
            block_height: 0,
            state_root: vec![0; 32],
            shard_states: HashMap::new(),
            total_transactions: 0,
            total_accounts: 0,
            validator_state_root: vec![0; 32],
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Add a shard state to the global state
    pub fn add_shard_state(&mut self, shard_state: ShardState) {
        self.shard_states.insert(shard_state.shard_id, shard_state);
        // In a real implementation, we would recalculate the state root here
    }
    
    /// Get a shard state by ID
    pub fn get_shard_state(&self, shard_id: ShardId) -> Option<&ShardState> {
        self.shard_states.get(&shard_id)
    }
    
    /// Update global metrics based on shard states
    pub fn update_metrics(&mut self) {
        self.total_accounts = self.shard_states.values()
            .map(|s| s.active_accounts)
            .sum();
            
        // Update other metrics as needed
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_account_creation() {
        let address = vec![1, 2, 3, 4];
        let shard_id = 0;
        let timestamp = 12345;
        
        // Test user account
        let user = Account::new_user(address.clone(), shard_id, timestamp);
        assert_eq!(user.address, address);
        assert_eq!(user.balance, 0);
        assert_eq!(user.nonce, 0);
        assert_eq!(user.account_type, AccountType::User);
        assert_eq!(user.shard_id, shard_id);
        assert_eq!(user.created_at, timestamp);
        assert!(!user.is_contract());
        
        // Test contract account
        let code = vec![5, 6, 7, 8];
        let contract = Account::new_contract(address.clone(), code.clone(), shard_id, timestamp);
        assert_eq!(contract.address, address);
        assert_eq!(contract.account_type, AccountType::Contract);
        assert_eq!(contract.code, Some(code));
        assert!(contract.is_contract());
    }
    
    #[test]
    fn test_account_serialization() {
        let address = vec![1, 2, 3, 4];
        let shard_id = 0;
        let timestamp = 12345;
        let user = Account::new_user(address, shard_id, timestamp);
        
        // Test serialization
        let serialized = user.serialize().unwrap();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized = Account::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, user);
    }
    
    #[test]
    fn test_global_state() {
        let mut global_state = GlobalState::new();
        assert_eq!(global_state.block_height, 0);
        assert!(global_state.shard_states.is_empty());
        
        // Add shard states
        let shard1 = ShardState {
            shard_id: 0,
            state_root: vec![1; 32],
            last_updated_height: 10,
            active_accounts: 100,
            recent_cross_shard_txs: Vec::new(),
            neighbor_shards: vec![1, 2],
            resource_utilization: 0.5,
        };
        
        let shard2 = ShardState {
            shard_id: 1,
            state_root: vec![2; 32],
            last_updated_height: 10,
            active_accounts: 200,
            recent_cross_shard_txs: Vec::new(),
            neighbor_shards: vec![0, 2],
            resource_utilization: 0.7,
        };
        
        global_state.add_shard_state(shard1);
        global_state.add_shard_state(shard2);
        
        assert_eq!(global_state.shard_states.len(), 2);
        assert_eq!(global_state.get_shard_state(0).unwrap().active_accounts, 100);
        assert_eq!(global_state.get_shard_state(1).unwrap().active_accounts, 200);
        
        // Update metrics
        global_state.update_metrics();
        assert_eq!(global_state.total_accounts, 300);
    }
}
