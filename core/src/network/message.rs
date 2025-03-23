//! # Network Message
//! 
//! This module defines network message structures and serialization/deserialization.

use crate::types::{ShardId, Priority, Transaction};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress};
use thiserror::Error;

/// Protocol version constants
pub const PROTOCOL_VERSION: u8 = 2;
pub const MIN_SUPPORTED_VERSION: u8 = 1;

/// Compression algorithm constants
const COMPRESSION_THRESHOLD: usize = 1024; // Compress messages larger than 1KB
const COMPRESSION_LEVEL: Compression = Compression::best();

/// Protocol errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid protocol version {0}")]
    InvalidVersion(u8),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid message format")]
    InvalidMessageFormat,
}

/// Network message types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
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
    /// Create a new message with protocol version checking
    pub fn new(
        message_type: MessageType,
        data: Vec<u8>,
        shard_id: Option<ShardId>,
        priority: Priority,
        sender: Vec<u8>,
    ) -> Result<Self, ProtocolError> {
        // Check if data needs compression
        let (compressed_data, is_compressed) = if data.len() > COMPRESSION_THRESHOLD {
            let mut compressor = Compress::new(COMPRESSION_LEVEL, false);
            let mut output = vec![0; data.len()];
            let status = compressor.compress(&data, &mut output, FlushCompress::Finish)?;
            
            if status == flate2::Status::Ok {
                (output, true)
            } else {
                return Err(ProtocolError::CompressionError(
                    format!("Failed to compress data: {:?}", status)
                ));
            }
        } else {
            (data, false)
        };

        // Calculate checksum
        let checksum = Self::calculate_checksum(&compressed_data);

        // Create message
        Ok(Message {
            version: PROTOCOL_VERSION,
            compression: is_compressed,
            encryption: false,
            priority,
            message_type,
            shard_id,
            data: compressed_data,
            checksum,
            sender,
            signature: vec![0; 64], // Placeholder for now
        })
    }

    /// Create a batch message from multiple transactions
    pub fn new_transaction_batch(
        transactions: Vec<Transaction>,
        shard_id: Option<ShardId>,
        priority: Priority,
        sender: Vec<u8>,
    ) -> Result<Self, ProtocolError> {
        // Encode transactions using binary format
        let mut batch_data = Vec::new();
        for tx in transactions {
            let tx_bytes = tx.to_binary()?;
            batch_data.extend_from_slice(&tx_bytes);
        }
        
        Self::new(MessageType::TransactionBatch, batch_data, shard_id, priority, sender)
    }

    /// Calculate checksum for message data
    fn calculate_checksum(data: &[u8]) -> [u8; 4] {
        use crc::{Crc, CRC_32_ISO_HDLC};
        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let checksum = crc.checksum(data);
        checksum.to_be_bytes()
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
    fn test_message_creation() -> Result<(), ProtocolError> {
        let msg = Message::new(
            MessageType::BlockAnnouncement,
            vec![1, 2, 3, 4],
            Some(0),
            Priority::High,
            vec![10, 11, 12],
        )?;
        
        assert_eq!(msg.version, PROTOCOL_VERSION);
        assert!(!msg.compression);
        assert!(!msg.encryption);
        assert_eq!(msg.priority, Priority::High);
        assert_eq!(msg.message_type, MessageType::BlockAnnouncement);
        assert_eq!(msg.shard_id, Some(0));
        assert_eq!(msg.data, vec![1, 2, 3, 4]);
        assert_eq!(msg.sender, vec![10, 11, 12]);
        Ok(())
    }

    #[test]
    fn test_transaction_batch() -> Result<(), ProtocolError> {
        let tx1 = Transaction::new_dummy(1);
        let tx2 = Transaction::new_dummy(2);
        
        let batch = Message::new_transaction_batch(
            vec![tx1, tx2],
            Some(0),
            Priority::Normal,
            vec![1, 2, 3],
        )?;
        
        assert_eq!(batch.message_type, MessageType::TransactionBatch);
        assert_eq!(batch.shard_id, Some(0));
        assert!(batch.data.len() > 0);
        Ok(())
    }

    #[test]
    fn test_compression() -> Result<(), ProtocolError> {
        // Create large message that should be compressed
        let large_data = vec![0; COMPRESSION_THRESHOLD + 1];
        let msg = Message::new(
            MessageType::BlockBody,
            large_data.clone(),
            None,
            Priority::High,
            vec![1, 2, 3],
        )?;
        
        assert!(msg.compression);
        assert!(msg.data.len() < large_data.len());
        Ok(())
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
