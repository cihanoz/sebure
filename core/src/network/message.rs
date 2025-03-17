//! # Network Message
//! 
//! This module defines network message structures and serialization/deserialization.

use crate::types::{ShardId, Priority};
use serde::{Serialize, Deserialize};

/// Network message types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Block announcement
    BlockAnnouncement,
    
    /// Block header
    BlockHeader,
    
    /// Block body
    BlockBody,
    
    /// Transaction announcement
    TransactionAnnouncement,
    
    /// Transaction batch
    TransactionBatch,
    
    /// Shard synchronization request
    ShardSyncRequest,
    
    /// Shard state response
    ShardStateResponse,
    
    /// Validator handshake
    ValidatorHandshake,
    
    /// Peer discovery
    PeerDiscovery,
    
    /// Peer exchange
    PeerExchange,
    
    /// State snapshot
    StateSnapshot,
    
    /// Checkpoint vote
    CheckpointVote,
    
    /// Network health
    NetworkHealth,
}

/// Message represents a network communication packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Protocol version
    pub version: u8,
    
    /// Whether message is compressed
    pub compression: bool,
    
    /// Whether message is encrypted
    pub encryption: bool,
    
    /// Message priority
    pub priority: Priority,
    
    /// Message type
    pub message_type: MessageType,
    
    /// Optional shard ID
    pub shard_id: Option<ShardId>,
    
    /// Message data
    pub data: Vec<u8>,
    
    /// Message checksum
    pub checksum: [u8; 4],
    
    /// Sender node ID
    pub sender: Vec<u8>,
    
    /// Message signature
    pub signature: Vec<u8>,
}

impl Message {
    /// Create a new message
    pub fn new(
        message_type: MessageType,
        data: Vec<u8>,
        shard_id: Option<ShardId>,
        priority: Priority,
        sender: Vec<u8>,
    ) -> Self {
        // In a real implementation, we would:
        // 1. Compress data if needed
        // 2. Encrypt data if needed
        // 3. Calculate checksum
        // 4. Sign the message
        
        Message {
            version: 1,
            compression: false,
            encryption: false,
            priority,
            message_type,
            shard_id,
            data,
            checksum: [0; 4], // Placeholder
            sender,
            signature: vec![0; 64], // Placeholder
        }
    }
    
    /// Verify the message signature
    pub fn verify_signature(&self) -> bool {
        // In a real implementation, we would:
        // 1. Serialize the message data (except signature)
        // 2. Verify the signature against the sender's public key
        
        // For now, just return success
        true
    }
    
    /// Verify the message checksum
    pub fn verify_checksum(&self) -> bool {
        // In a real implementation, we would:
        // 1. Calculate the checksum of the message data
        // 2. Compare it with the message checksum
        
        // For now, just return success
        true
    }
    
    /// Serialize the message to bytes
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
    
    /// Deserialize a message from bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            MessageType::BlockAnnouncement,
            vec![1, 2, 3, 4],
            Some(0),
            Priority::High,
            vec![10, 11, 12],
        );
        
        assert_eq!(msg.version, 1);
        assert!(!msg.compression);
        assert!(!msg.encryption);
        assert_eq!(msg.priority, Priority::High);
        assert_eq!(msg.message_type, MessageType::BlockAnnouncement);
        assert_eq!(msg.shard_id, Some(0));
        assert_eq!(msg.data, vec![1, 2, 3, 4]);
        assert_eq!(msg.sender, vec![10, 11, 12]);
    }
    
    #[test]
    fn test_message_serialization() {
        let msg = Message::new(
            MessageType::BlockAnnouncement,
            vec![1, 2, 3, 4],
            Some(0),
            Priority::High,
            vec![10, 11, 12],
        );
        
        // Serialize to bytes
        let bytes = msg.serialize().unwrap();
        
        // Deserialize back to message
        let msg2 = Message::deserialize(&bytes).unwrap();
        
        // Verify that the deserialized message matches the original
        assert_eq!(msg.version, msg2.version);
        assert_eq!(msg.compression, msg2.compression);
        assert_eq!(msg.encryption, msg2.encryption);
        assert_eq!(msg.priority, msg2.priority);
        assert_eq!(msg.message_type, msg2.message_type);
        assert_eq!(msg.shard_id, msg2.shard_id);
        assert_eq!(msg.data, msg2.data);
        assert_eq!(msg.sender, msg2.sender);
    }
    
    #[test]
    fn test_message_verification() {
        let msg = Message::new(
            MessageType::BlockAnnouncement,
            vec![1, 2, 3, 4],
            Some(0),
            Priority::High,
            vec![10, 11, 12],
        );
        
        assert!(msg.verify_signature());
        assert!(msg.verify_checksum());
    }
}
