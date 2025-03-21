//! # Database Types
//!
//! This module defines the database types and column families for the state database.

/// Database backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseBackend {
    /// In-memory database (for testing)
    Memory,
    
    /// LevelDB database
    LevelDB,
    
    /// LMDB database
    LMDB,
}

/// Database column families/namespaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseColumn {
    /// Account balances
    AccountBalance,
    
    /// Account nonces
    AccountNonce,
    
    /// Contract code
    ContractCode,
    
    /// Contract storage
    ContractStorage,
    
    /// Shard state roots
    ShardStateRoot,
    
    /// Validator data
    ValidatorData,
    
    /// Staking data
    StakingData,
    
    /// Metadata
    Metadata,
}

impl DatabaseColumn {
    /// Get the column name
    pub fn name(&self) -> &'static str {
        match self {
            DatabaseColumn::AccountBalance => "account_balance",
            DatabaseColumn::AccountNonce => "account_nonce",
            DatabaseColumn::ContractCode => "contract_code",
            DatabaseColumn::ContractStorage => "contract_storage",
            DatabaseColumn::ShardStateRoot => "shard_state_root",
            DatabaseColumn::ValidatorData => "validator_data",
            DatabaseColumn::StakingData => "staking_data",
            DatabaseColumn::Metadata => "metadata",
        }
    }
    
    /// Get all column names
    pub fn all_names() -> Vec<&'static str> {
        vec![
            DatabaseColumn::AccountBalance.name(),
            DatabaseColumn::AccountNonce.name(),
            DatabaseColumn::ContractCode.name(),
            DatabaseColumn::ContractStorage.name(),
            DatabaseColumn::ShardStateRoot.name(),
            DatabaseColumn::ValidatorData.name(),
            DatabaseColumn::StakingData.name(),
            DatabaseColumn::Metadata.name(),
        ]
    }
}
