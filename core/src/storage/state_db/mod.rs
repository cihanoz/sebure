//! # State Database Module
//! 
//! This module provides storage for blockchain state, including account balances,
//! smart contract state, and other state data.

pub mod account;
pub mod database_types;
pub mod key_impl;
pub mod keys;
pub mod memory_storage;
pub mod state_db;
pub mod iterator;

// Re-export main types
pub use account::AccountInfo;
pub use database_types::{DatabaseBackend, DatabaseColumn};
pub use keys::StateDBKey;
pub use state_db::StateDB;
