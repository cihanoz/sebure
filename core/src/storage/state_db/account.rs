//! # Account Information
//!
//! This module defines the account information structure for the state database.

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
