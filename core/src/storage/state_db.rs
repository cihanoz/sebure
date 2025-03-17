//! # State Database Implementation
//! 
//! This module provides storage for blockchain state, including account balances,
//! smart contract state, and other state data.

use crate::types::{Result, Error, ShardId};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Keys used in the state database
pub enum StateDBKey {
    /// Key for account balance
    AccountBalance(Vec<u8>),
    
    /// Key for account nonce
    AccountNonce(Vec<u8>),
    
    /// Key for contract code
    ContractCode(Vec<u8>),
    
    /// Key for contract storage
    ContractStorage(Vec<u8>, Vec<u8>), // (contract_addr, storage_key)
    
    /// Key for shard state root
    ShardStateRoot(ShardId),
    
    /// Key for validator data
    ValidatorData(Vec<u8>),
    
    /// Key for staking data
    StakingData(Vec<u8>),
}

impl StateDBKey {
    /// Convert the key to a byte representation for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            StateDBKey::AccountBalance(addr) => {
                let mut key = Vec::with_capacity(1 + addr.len());
                key.push(0x01); // prefix for account balance
                key.extend_from_slice(addr);
                key
            },
            StateDBKey::AccountNonce(addr) => {
                let mut key = Vec::with_capacity(1 + addr.len());
                key.push(0x02); // prefix for account nonce
                key.extend_from_slice(addr);
                key
            },
            StateDBKey::ContractCode(addr) => {
                let mut key = Vec::with_capacity(1 + addr.len());
                key.push(0x03); // prefix for contract code
                key.extend_from_slice(addr);
                key
            },
            StateDBKey::ContractStorage(addr, storage_key) => {
                let mut key = Vec::with_capacity(1 + addr.len() + storage_key.len());
                key.push(0x04); // prefix for contract storage
                key.extend_from_slice(addr);
                key.extend_from_slice(storage_key);
                key
            },
            StateDBKey::ShardStateRoot(shard_id) => {
                let mut key = Vec::with_capacity(3);
                key.push(0x05); // prefix for shard state root
                key.extend_from_slice(&shard_id.to_be_bytes());
                key
            },
            StateDBKey::ValidatorData(addr) => {
                let mut key = Vec::with_capacity(1 + addr.len());
                key.push(0x06); // prefix for validator data
                key.extend_from_slice(addr);
                key
            },
            StateDBKey::StakingData(addr) => {
                let mut key = Vec::with_capacity(1 + addr.len());
                key.push(0x07); // prefix for staking data
                key.extend_from_slice(addr);
                key
            },
        }
    }
}

/// Account information in the state DB
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountInfo {
    /// Account balance
    pub balance: u64,
    
    /// Account nonce
    pub nonce: u64,
    
    /// Is this a contract account
    pub is_contract: bool,
}

impl Default for AccountInfo {
    fn default() -> Self {
        AccountInfo {
            balance: 0,
            nonce: 0,
            is_contract: false,
        }
    }
}

/// State database for storing blockchain state
pub struct StateDB {
    /// Database path
    path: String,
    
    /// In-memory account balances (for testing/prototype)
    // In a real implementation, this would be a database connection
    account_balances: Arc<Mutex<HashMap<Vec<u8>, u64>>>,
    
    /// In-memory account nonces
    account_nonces: Arc<Mutex<HashMap<Vec<u8>, u64>>>,
    
    /// In-memory contract code
    contract_code: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    
    /// In-memory contract storage
    contract_storage: Arc<Mutex<HashMap<Vec<u8>, HashMap<Vec<u8>, Vec<u8>>>>>,
    
    /// In-memory shard state roots
    shard_state_roots: Arc<Mutex<HashMap<ShardId, Vec<u8>>>>,
}

impl StateDB {
    /// Create a new state database at the specified path
    pub fn new(path: &str, _config: &super::StorageConfig) -> Result<Self> {
        // Ensure the directory exists
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            std::fs::create_dir_all(path_obj)
                .map_err(|e| Error::Io(e))?;
        }
        
        // In a real implementation, we would open the database here
        // For now, we just use in-memory hashmaps
        
        Ok(StateDB {
            path: path.to_string(),
            account_balances: Arc::new(Mutex::new(HashMap::new())),
            account_nonces: Arc::new(Mutex::new(HashMap::new())),
            contract_code: Arc::new(Mutex::new(HashMap::new())),
            contract_storage: Arc::new(Mutex::new(HashMap::new())),
            shard_state_roots: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Close the state database
    pub fn close(&self) -> Result<()> {
        // In a real implementation, we would close the database connection
        Ok(())
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
        let balances = self.account_balances.lock().unwrap();
        Ok(balances.get(address).cloned().unwrap_or(0))
    }
    
    /// Set account balance
    pub fn set_account_balance(&self, address: &[u8], balance: u64) -> Result<()> {
        let mut balances = self.account_balances.lock().unwrap();
        balances.insert(address.to_vec(), balance);
        Ok(())
    }
    
    /// Adjust account balance by the given amount (can be positive or negative)
    pub fn adjust_account_balance(&self, address: &[u8], amount: i64) -> Result<u64> {
        let mut balances = self.account_balances.lock().unwrap();
        
        let current_balance = balances.get(address).cloned().unwrap_or(0);
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
        
        balances.insert(address.to_vec(), new_balance);
        Ok(new_balance)
    }
    
    /// Get account nonce
    pub fn get_account_nonce(&self, address: &[u8]) -> Result<u64> {
        let nonces = self.account_nonces.lock().unwrap();
        Ok(nonces.get(address).cloned().unwrap_or(0))
    }
    
    /// Set account nonce
    pub fn set_account_nonce(&self, address: &[u8], nonce: u64) -> Result<()> {
        let mut nonces = self.account_nonces.lock().unwrap();
        nonces.insert(address.to_vec(), nonce);
        Ok(())
    }
    
    /// Increment account nonce
    pub fn increment_account_nonce(&self, address: &[u8]) -> Result<u64> {
        let mut nonces = self.account_nonces.lock().unwrap();
        let current_nonce = nonces.get(address).cloned().unwrap_or(0);
        let new_nonce = current_nonce + 1;
        nonces.insert(address.to_vec(), new_nonce);
        Ok(new_nonce)
    }
    
    /// Get contract code
    pub fn get_contract_code(&self, address: &[u8]) -> Result<Vec<u8>> {
        let codes = self.contract_code.lock().unwrap();
        codes.get(address)
            .cloned()
            .ok_or_else(|| Error::State(format!("Contract code not found for address {:?}", address)))
    }
    
    /// Check if contract code exists
    pub fn has_contract_code(&self, address: &[u8]) -> bool {
        let codes = self.contract_code.lock().unwrap();
        codes.contains_key(address)
    }
    
    /// Set contract code
    pub fn set_contract_code(&self, address: &[u8], code: Vec<u8>) -> Result<()> {
        let mut codes = self.contract_code.lock().unwrap();
        codes.insert(address.to_vec(), code);
        Ok(())
    }
    
    /// Get contract storage value
    pub fn get_contract_storage(&self, address: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        let storage = self.contract_storage.lock().unwrap();
        
        match storage.get(address) {
            Some(contract_storage) => {
                contract_storage.get(key)
                    .cloned()
                    .ok_or_else(|| Error::State(format!(
                        "Contract storage key {:?} not found for address {:?}",
                        key, address
                    )))
            },
            None => Err(Error::State(format!(
                "Contract storage not found for address {:?}",
                address
            ))),
        }
    }
    
    /// Set contract storage value
    pub fn set_contract_storage(&self, address: &[u8], key: &[u8], value: Vec<u8>) -> Result<()> {
        let mut storage = self.contract_storage.lock().unwrap();
        
        let contract_storage = storage
            .entry(address.to_vec())
            .or_insert_with(HashMap::new);
            
        contract_storage.insert(key.to_vec(), value);
        Ok(())
    }
    
    /// Get shard state root
    pub fn get_shard_state_root(&self, shard_id: ShardId) -> Result<Vec<u8>> {
        let roots = self.shard_state_roots.lock().unwrap();
        
        roots.get(&shard_id)
            .cloned()
            .ok_or_else(|| Error::State(format!(
                "State root not found for shard {}",
                shard_id
            )))
    }
    
    /// Set shard state root
    pub fn set_shard_state_root(&self, shard_id: ShardId, root: Vec<u8>) -> Result<()> {
        let mut roots = self.shard_state_roots.lock().unwrap();
        roots.insert(shard_id, root);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    fn temp_dir() -> String {
        let mut dir = env::temp_dir();
        dir.push(format!("sebure-test-state-{}", rand::random::<u64>()));
        dir.to_str().unwrap().to_string()
    }
    
    #[test]
    fn test_state_db_creation() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let state_db = StateDB::new(&path, &config);
        assert!(state_db.is_ok());
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_account_balance() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let state_db = StateDB::new(&path, &config).unwrap();
        
        let address = vec![1, 2, 3, 4];
        
        // Initial balance should be 0
        assert_eq!(state_db.get_account_balance(&address).unwrap(), 0);
        
        // Set balance
        state_db.set_account_balance(&address, 100).unwrap();
        assert_eq!(state_db.get_account_balance(&address).unwrap(), 100);
        
        // Adjust balance
        state_db.adjust_account_balance(&address, 50).unwrap();
        assert_eq!(state_db.get_account_balance(&address).unwrap(), 150);
        
        state_db.adjust_account_balance(&address, -30).unwrap();
        assert_eq!(state_db.get_account_balance(&address).unwrap(), 120);
        
        // Attempt to withdraw too much
        assert!(state_db.adjust_account_balance(&address, -200).is_err());
        
        // Balance should remain unchanged
        assert_eq!(state_db.get_account_balance(&address).unwrap(), 120);
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_account_nonce() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let state_db = StateDB::new(&path, &config).unwrap();
        
        let address = vec![1, 2, 3, 4];
        
        // Initial nonce should be 0
        assert_eq!(state_db.get_account_nonce(&address).unwrap(), 0);
        
        // Set nonce
        state_db.set_account_nonce(&address, 5).unwrap();
        assert_eq!(state_db.get_account_nonce(&address).unwrap(), 5);
        
        // Increment nonce
        state_db.increment_account_nonce(&address).unwrap();
        assert_eq!(state_db.get_account_nonce(&address).unwrap(), 6);
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_contract_code() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let state_db = StateDB::new(&path, &config).unwrap();
        
        let address = vec![1, 2, 3, 4];
        let code = vec![5, 6, 7, 8];
        
        // Initially, no contract code
        assert!(!state_db.has_contract_code(&address));
        assert!(state_db.get_contract_code(&address).is_err());
        
        // Set contract code
        state_db.set_contract_code(&address, code.clone()).unwrap();
        
        // Now should have contract code
        assert!(state_db.has_contract_code(&address));
        assert_eq!(state_db.get_contract_code(&address).unwrap(), code);
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_contract_storage() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let state_db = StateDB::new(&path, &config).unwrap();
        
        let address = vec![1, 2, 3, 4];
        let key = vec![5, 6, 7, 8];
        let value = vec![9, 10, 11, 12];
        
        // Initially, storage should be empty
        assert!(state_db.get_contract_storage(&address, &key).is_err());
        
        // Set storage value
        state_db.set_contract_storage(&address, &key, value.clone()).unwrap();
        
        // Now should be able to get storage value
        assert_eq!(state_db.get_contract_storage(&address, &key).unwrap(), value);
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_shard_state_root() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let state_db = StateDB::new(&path, &config).unwrap();
        
        let shard_id = 1;
        let root = vec![1, 2, 3, 4];
        
        // Initially, no state root
        assert!(state_db.get_shard_state_root(shard_id).is_err());
        
        // Set state root
        state_db.set_shard_state_root(shard_id, root.clone()).unwrap();
        
        // Now should have state root
        assert_eq!(state_db.get_shard_state_root(shard_id).unwrap(), root);
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
    
    #[test]
    fn test_account_info() {
        let path = temp_dir();
        let config = super::super::StorageConfig::default();
        
        let state_db = StateDB::new(&path, &config).unwrap();
        
        let address = vec![1, 2, 3, 4];
        
        // Initially, account should have default values
        let info = state_db.get_account_info(&address).unwrap();
        assert_eq!(info.balance, 0);
        assert_eq!(info.nonce, 0);
        assert!(!info.is_contract);
        
        // Set account values
        state_db.set_account_balance(&address, 100).unwrap();
        state_db.set_account_nonce(&address, 5).unwrap();
        
        // Now account info should reflect these values
        let info = state_db.get_account_info(&address).unwrap();
        assert_eq!(info.balance, 100);
        assert_eq!(info.nonce, 5);
        assert!(!info.is_contract);
        
        // Set contract code to make it a contract account
        state_db.set_contract_code(&address, vec![5, 6, 7, 8]).unwrap();
        
        // Now should be a contract account
        let info = state_db.get_account_info(&address).unwrap();
        assert_eq!(info.balance, 100);
        assert_eq!(info.nonce, 5);
        assert!(info.is_contract);
        
        // Clean up
        std::fs::remove_dir_all(path).ok();
    }
}
