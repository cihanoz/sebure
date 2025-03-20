//! # Core Types
//! 
//! This module defines core types used throughout the SEBURE blockchain.

use std::fmt;
use std::io;
use std::result;
use serde::{Serialize, Deserialize};

/// Result type used throughout the SEBURE blockchain
pub type Result<T> = result::Result<T, Error>;

/// BlockHeight type
pub type BlockHeight = u64;

/// ShardId type
pub type ShardId = u16;

/// Timestamp type (microseconds since Unix epoch)
pub type Timestamp = u64;

/// Priority levels for network messages and transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    /// Low priority
    Low,
    
    /// Normal priority
    Normal,
    
    /// High priority
    High,
    
    /// Critical priority
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Transfer of tokens
    Transfer,
    
    /// Smart contract deployment
    ContractDeploy,
    
    /// Smart contract call
    ContractCall,
    
    /// Validator registration
    ValidatorRegister,
    
    /// Validator un-registration
    ValidatorUnregister,
    
    /// Staking deposit
    Stake,
    
    /// Staking withdrawal
    Unstake,
    
    /// System transaction
    System,
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Transfer
    }
}

/// Transaction data types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// No data
    None,
    
    /// Plain text data
    Text,
    
    /// Binary data
    Binary,
    
    /// JSON data
    Json,
    
    /// Smart contract code
    ContractCode,
    
    /// Smart contract call data
    ContractCallData,
}

impl Default for DataType {
    fn default() -> Self {
        DataType::None
    }
}

/// Core error type
#[derive(Debug)]
pub enum Error {
    /// IO error
    Io(io::Error),
    
    /// Serialization error
    Serialization(String),
    
    /// Cryptographic error
    Crypto(String),
    
    /// Validation error
    Validation(String),
    
    /// Network error
    Network(String),
    
    /// Consensus error
    Consensus(String),
    
    /// Storage error
    Storage(String),
    
    /// State error
    State(String),
    
    /// Block validation error
    BlockValidation(String),
    
    /// Transaction validation error
    TransactionValidation(String),
    
    /// System initialization error
    Initialization(String),
    
    /// Generic error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Serialization(e) => write!(f, "Serialization error: {}", e),
            Error::Crypto(e) => write!(f, "Cryptographic error: {}", e),
            Error::Validation(e) => write!(f, "Validation error: {}", e),
            Error::Network(e) => write!(f, "Network error: {}", e),
            Error::Consensus(e) => write!(f, "Consensus error: {}", e),
            Error::Storage(e) => write!(f, "Storage error: {}", e),
            Error::State(e) => write!(f, "State error: {}", e),
            Error::BlockValidation(e) => write!(f, "Block validation error: {}", e),
            Error::TransactionValidation(e) => write!(f, "Transaction validation error: {}", e),
            Error::Initialization(e) => write!(f, "Initialization error: {}", e),
            Error::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serialization(error.to_string())
    }
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: std::net::SocketAddr,
    
    /// Bootstrap peers
    pub bootstrap_peers: Vec<std::net::SocketAddr>,
    
    /// Maximum number of peers
    pub max_peers: usize,
    
    /// Announce interval in seconds
    pub announce_interval: u64,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_addr: "127.0.0.1:8765".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            max_peers: 50,
            announce_interval: 300, // 5 minutes
            connection_timeout: 30, // 30 seconds
        }
    }
}

/// Network implementation
pub struct Network {
    /// Network configuration
    config: NetworkConfig,
}

impl Network {
    /// Create a new network
    pub fn new(config: NetworkConfig) -> Self {
        Network {
            config,
        }
    }
    
    /// Start the network
    pub fn start(&self) -> Result<()> {
        // In a real implementation, this would start the network service
        Ok(())
    }
    
    /// Stop the network
    pub fn stop(&self) -> Result<()> {
        // In a real implementation, this would stop the network service
        Ok(())
    }
    
    /// Get the number of connected peers
    pub fn peer_count(&self) -> usize {
        // In a real implementation, this would return the actual count
        0
    }
}
