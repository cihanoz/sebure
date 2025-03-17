//! # Network Protocol
//! 
//! This module defines the network protocol used for P2P communication.

use serde::{Serialize, Deserialize};
use std::fmt;
use crate::types::{Result, Error};

/// Protocol version for network communication
pub const PROTOCOL_VERSION: u8 = 1;

/// Protocol identifier
pub const PROTOCOL_ID: &str = "sebure/1.0.0";

/// Protocol capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolCapability {
    /// Basic blockchain protocol
    Core = 0x01,
    
    /// Validator consensus protocol
    Validator = 0x02,
    
    /// Light client protocol
    LightClient = 0x04,
    
    /// Transaction relay only
    TransactionRelay = 0x08,
    
    /// Shard sync protocol
    ShardSync = 0x10,
    
    /// Block archive protocol
    Archive = 0x20,
}

/// Protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Maximum number of transactions in a batch
    pub max_tx_batch: usize,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    
    /// Handshake timeout in seconds
    pub handshake_timeout: u64,
    
    /// Capabilities of this node
    pub capabilities: Vec<ProtocolCapability>,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        ProtocolConfig {
            max_message_size: 4 * 1024 * 1024, // 4 MB
            max_tx_batch: 1000,
            heartbeat_interval: 60,            // 1 minute
            connection_timeout: 10,            // 10 seconds
            handshake_timeout: 5,              // 5 seconds
            capabilities: vec![ProtocolCapability::Core, ProtocolCapability::TransactionRelay],
        }
    }
}

/// Protocol handshake message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handshake {
    /// Protocol version
    pub version: u8,
    
    /// Protocol capabilities
    pub capabilities: Vec<ProtocolCapability>,
    
    /// User agent string
    pub user_agent: String,
    
    /// Node ID
    pub node_id: Vec<u8>,
    
    /// Current block height
    pub block_height: u64,
    
    /// Genesis block hash
    pub genesis_hash: Vec<u8>,
    
    /// Network ID
    pub network_id: String,
    
    /// Timestamp of the handshake
    pub timestamp: u64,
}

impl Handshake {
    /// Create a new handshake message
    pub fn new(
        capabilities: Vec<ProtocolCapability>,
        user_agent: String,
        node_id: Vec<u8>,
        block_height: u64,
        genesis_hash: Vec<u8>,
        network_id: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Handshake {
            version: PROTOCOL_VERSION,
            capabilities,
            user_agent,
            node_id,
            block_height,
            genesis_hash,
            network_id,
            timestamp,
        }
    }
    
    /// Validate the handshake
    pub fn validate(&self, expected_network_id: &str) -> Result<()> {
        // Check protocol version
        if self.version != PROTOCOL_VERSION {
            return Err(Error::Network(format!(
                "Protocol version mismatch: expected {}, got {}",
                PROTOCOL_VERSION, self.version
            )));
        }
        
        // Check network ID
        if self.network_id != expected_network_id {
            return Err(Error::Network(format!(
                "Network ID mismatch: expected {}, got {}",
                expected_network_id, self.network_id
            )));
        }
        
        // Check timestamp (not too old or future)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let time_diff = if now > self.timestamp {
            now - self.timestamp
        } else {
            self.timestamp - now
        };
        
        // Allow 5 minutes time difference
        if time_diff > 300 {
            return Err(Error::Network(format!(
                "Handshake timestamp too far from current time ({} seconds)",
                time_diff
            )));
        }
        
        Ok(())
    }
}

/// Network protocol implementation
pub struct Protocol {
    /// Protocol configuration
    config: ProtocolConfig,
    
    /// Network ID
    network_id: String,
    
    /// Genesis block hash
    genesis_hash: Vec<u8>,
    
    /// Node ID
    node_id: Vec<u8>,
    
    /// User agent string
    user_agent: String,
}

impl Protocol {
    /// Create a new protocol instance
    pub fn new(
        config: ProtocolConfig,
        network_id: String,
        genesis_hash: Vec<u8>,
        node_id: Vec<u8>,
        user_agent: String,
    ) -> Self {
        Protocol {
            config,
            network_id,
            genesis_hash,
            node_id,
            user_agent,
        }
    }
    
    /// Create a handshake message
    pub fn create_handshake(&self, block_height: u64) -> Handshake {
        Handshake::new(
            self.config.capabilities.clone(),
            self.user_agent.clone(),
            self.node_id.clone(),
            block_height,
            self.genesis_hash.clone(),
            self.network_id.clone(),
        )
    }
    
    /// Validate a received handshake
    pub fn validate_handshake(&self, handshake: &Handshake) -> Result<()> {
        handshake.validate(&self.network_id)
    }
    
    /// Check if the protocol supports a capability
    pub fn supports_capability(&self, capability: ProtocolCapability) -> bool {
        self.config.capabilities.contains(&capability)
    }
    
    /// Get the protocol configuration
    pub fn config(&self) -> &ProtocolConfig {
        &self.config
    }
}

/// Protocol error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolErrorCode {
    /// No error
    None = 0,
    
    /// Invalid message format
    InvalidMessage = 1,
    
    /// Protocol version mismatch
    VersionMismatch = 2,
    
    /// Network ID mismatch
    NetworkMismatch = 3,
    
    /// Handshake failed
    HandshakeFailed = 4,
    
    /// Capability not supported
    CapabilityNotSupported = 5,
    
    /// Message too large
    MessageTooLarge = 6,
    
    /// Rate limited
    RateLimited = 7,
    
    /// Invalid operation
    InvalidOperation = 8,
    
    /// Internal error
    InternalError = 9,
}

impl fmt::Display for ProtocolErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolErrorCode::None => write!(f, "No error"),
            ProtocolErrorCode::InvalidMessage => write!(f, "Invalid message format"),
            ProtocolErrorCode::VersionMismatch => write!(f, "Protocol version mismatch"),
            ProtocolErrorCode::NetworkMismatch => write!(f, "Network ID mismatch"),
            ProtocolErrorCode::HandshakeFailed => write!(f, "Handshake failed"),
            ProtocolErrorCode::CapabilityNotSupported => write!(f, "Capability not supported"),
            ProtocolErrorCode::MessageTooLarge => write!(f, "Message too large"),
            ProtocolErrorCode::RateLimited => write!(f, "Rate limited"),
            ProtocolErrorCode::InvalidOperation => write!(f, "Invalid operation"),
            ProtocolErrorCode::InternalError => write!(f, "Internal error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_config_default() {
        let config = ProtocolConfig::default();
        
        assert_eq!(config.max_message_size, 4 * 1024 * 1024);
        assert_eq!(config.max_tx_batch, 1000);
        assert_eq!(config.heartbeat_interval, 60);
        assert_eq!(config.connection_timeout, 10);
        assert_eq!(config.handshake_timeout, 5);
        assert_eq!(config.capabilities.len(), 2);
        assert!(config.capabilities.contains(&ProtocolCapability::Core));
        assert!(config.capabilities.contains(&ProtocolCapability::TransactionRelay));
    }
    
    #[test]
    fn test_handshake_creation() {
        let capabilities = vec![ProtocolCapability::Core, ProtocolCapability::TransactionRelay];
        let user_agent = "sebure-test/1.0.0".to_string();
        let node_id = vec![1, 2, 3, 4];
        let block_height = 100;
        let genesis_hash = vec![5, 6, 7, 8];
        let network_id = "sebure-testnet".to_string();
        
        let handshake = Handshake::new(
            capabilities.clone(),
            user_agent.clone(),
            node_id.clone(),
            block_height,
            genesis_hash.clone(),
            network_id.clone(),
        );
        
        assert_eq!(handshake.version, PROTOCOL_VERSION);
        assert_eq!(handshake.capabilities, capabilities);
        assert_eq!(handshake.user_agent, user_agent);
        assert_eq!(handshake.node_id, node_id);
        assert_eq!(handshake.block_height, block_height);
        assert_eq!(handshake.genesis_hash, genesis_hash);
        assert_eq!(handshake.network_id, network_id);
    }
    
    #[test]
    fn test_handshake_validation() {
        let capabilities = vec![ProtocolCapability::Core];
        let user_agent = "sebure-test/1.0.0".to_string();
        let node_id = vec![1, 2, 3, 4];
        let block_height = 100;
        let genesis_hash = vec![5, 6, 7, 8];
        let network_id = "sebure-testnet".to_string();
        
        let valid_handshake = Handshake::new(
            capabilities.clone(),
            user_agent.clone(),
            node_id.clone(),
            block_height,
            genesis_hash.clone(),
            network_id.clone(),
        );
        
        // Valid handshake should validate
        assert!(valid_handshake.validate(&network_id).is_ok());
        
        // Wrong network ID should fail
        assert!(valid_handshake.validate("wrong-network").is_err());
        
        // Create a handshake with wrong version
        let mut invalid_version = valid_handshake.clone();
        invalid_version.version = PROTOCOL_VERSION + 1;
        assert!(invalid_version.validate(&network_id).is_err());
        
        // Create a handshake with wrong network ID
        let mut invalid_network = valid_handshake.clone();
        invalid_network.network_id = "wrong-network".to_string();
        assert!(invalid_network.validate(&network_id).is_err());
        
        // Create a handshake with timestamp too far in the past
        let mut invalid_timestamp = valid_handshake.clone();
        invalid_timestamp.timestamp = 0; // Jan 1, 1970
        assert!(invalid_timestamp.validate(&network_id).is_err());
    }
    
    #[test]
    fn test_protocol_creation() {
        let config = ProtocolConfig::default();
        let network_id = "sebure-testnet".to_string();
        let genesis_hash = vec![1, 2, 3, 4];
        let node_id = vec![5, 6, 7, 8];
        let user_agent = "sebure-test/1.0.0".to_string();
        
        let protocol = Protocol::new(
            config.clone(),
            network_id.clone(),
            genesis_hash.clone(),
            node_id.clone(),
            user_agent.clone(),
        );
        
        // Test handshake creation
        let block_height = 100;
        let handshake = protocol.create_handshake(block_height);
        
        assert_eq!(handshake.version, PROTOCOL_VERSION);
        assert_eq!(handshake.capabilities, config.capabilities);
        assert_eq!(handshake.user_agent, user_agent);
        assert_eq!(handshake.node_id, node_id);
        assert_eq!(handshake.block_height, block_height);
        assert_eq!(handshake.genesis_hash, genesis_hash);
        assert_eq!(handshake.network_id, network_id);
        
        // Test capability support
        assert!(protocol.supports_capability(ProtocolCapability::Core));
        assert!(protocol.supports_capability(ProtocolCapability::TransactionRelay));
        assert!(!protocol.supports_capability(ProtocolCapability::Validator));
    }
}
