//! # State Database Keys
//!
//! This module defines the keys used in the state database.

use crate::types::ShardId;

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
