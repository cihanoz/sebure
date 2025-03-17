//! # Block Structure Implementation
//! 
//! This module implements the block structure as defined in the PRD.

use serde::{Serialize, Deserialize};
use crate::types::{ShardId, Timestamp, Result};

/// Block header containing metadata and cryptographic links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block index/height in the chain
    pub index: u64,
    
    /// Block creation timestamp in microseconds
    pub timestamp: Timestamp,
    
    /// Hash of the previous block in the chain
    pub previous_hash: Vec<u8>,
    
    /// Merkle root of the state tree
    pub state_root: Vec<u8>,
    
    /// Merkle root of transactions
    pub transaction_root: Vec<u8>,
    
    /// Merkle root of transaction receipts
    pub receipt_root: Vec<u8>,
    
    /// Merkle root of validator set
    pub validator_merkle: Vec<u8>,
    
    /// List of shard identifiers included in this block
    pub shard_identifiers: Vec<ShardId>,
    
    /// Aggregated BLS signature from validators
    pub aggregated_signature: Vec<u8>,
}

/// ShardData represents transactions and validation proof for a specific shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardData {
    /// Shard identifier
    pub shard_id: ShardId,
    
    /// References to transactions included in this shard
    pub transactions: Vec<Vec<u8>>, // TransactionRef is represented as Vec<u8> (transaction hash)
    
    /// Proof of execution for the shard's transactions
    pub execution_proof: Vec<u8>,
    
    /// Signatures from validators for this shard
    pub validator_signatures: Vec<Vec<u8>>,
}

/// Receipt for cross-shard transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardReceipt {
    /// Transaction identifier
    pub transaction_id: Vec<u8>,
    
    /// Source shard
    pub source_shard: ShardId,
    
    /// Destination shard
    pub destination_shard: ShardId,
    
    /// Execution status
    pub status: bool,
    
    /// Additional data or error message
    pub data: Vec<u8>,
}

/// ValidatorRef represents a reference to a validator
pub type ValidatorRef = Vec<u8>; // Validator public key or identifier

/// Block structure as defined in the PRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header containing metadata
    pub header: BlockHeader,
    
    /// Data from each shard included in this block
    pub shard_data: Vec<ShardData>,
    
    /// Receipts for cross-shard transactions
    pub cross_shard_receipts: Vec<CrossShardReceipt>,
    
    /// Set of validators that participated in this block
    pub validator_set: Vec<ValidatorRef>,
}

impl Block {
    /// Create a new block with the given parameters
    pub fn new(
        index: u64,
        timestamp: Timestamp,
        previous_hash: Vec<u8>,
        shard_ids: Vec<ShardId>,
    ) -> Self {
        // Create empty roots for now
        let empty_root = vec![0; 32];
        
        Block {
            header: BlockHeader {
                index,
                timestamp,
                previous_hash,
                state_root: empty_root.clone(),
                transaction_root: empty_root.clone(),
                receipt_root: empty_root.clone(),
                validator_merkle: empty_root.clone(),
                shard_identifiers: shard_ids,
                aggregated_signature: Vec::new(),
            },
            shard_data: Vec::new(),
            cross_shard_receipts: Vec::new(),
            validator_set: Vec::new(),
        }
    }
    
    /// Validate basic block properties
    pub fn validate_basic(&self) -> Result<()> {
        // In a real implementation, we would validate:
        // - All required fields are present
        // - Timestamps are reasonable
        // - Hashes have the correct length
        // - Signature is valid
        // etc.
        
        Ok(())
    }
    
    /// Add shard data to the block
    pub fn add_shard_data(&mut self, shard_data: ShardData) -> Result<()> {
        // Check if shard ID is in the block's shard identifiers
        if !self.header.shard_identifiers.contains(&shard_data.shard_id) {
            return Err(crate::types::Error::BlockValidation(
                format!("Shard ID {} not declared in block header", shard_data.shard_id)
            ));
        }
        
        self.shard_data.push(shard_data);
        Ok(())
    }
    
    /// Add a validator to the block's validator set
    pub fn add_validator(&mut self, validator: ValidatorRef) {
        self.validator_set.push(validator);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    fn current_time_micros() -> Timestamp {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        
        // Convert to microseconds
        since_epoch.as_secs() * 1_000_000 + since_epoch.subsec_micros() as u64
    }
    
    #[test]
    fn test_new_block() {
        let index = 1;
        let timestamp = current_time_micros();
        let previous_hash = vec![0; 32];
        let shard_ids = vec![0, 1];
        
        let block = Block::new(index, timestamp, previous_hash, shard_ids);
        
        assert_eq!(block.header.index, index);
        assert_eq!(block.header.timestamp, timestamp);
        assert_eq!(block.header.shard_identifiers, vec![0, 1]);
        assert!(block.shard_data.is_empty());
        assert!(block.validator_set.is_empty());
    }
    
    #[test]
    fn test_add_shard_data() {
        let mut block = Block::new(1, current_time_micros(), vec![0; 32], vec![0, 1]);
        
        let shard_data = ShardData {
            shard_id: 0,
            transactions: Vec::new(),
            execution_proof: Vec::new(),
            validator_signatures: Vec::new(),
        };
        
        assert!(block.add_shard_data(shard_data).is_ok());
        assert_eq!(block.shard_data.len(), 1);
        
        // Try to add shard data for a shard not in the block's shard identifiers
        let invalid_shard_data = ShardData {
            shard_id: 2, // Not in the block's shard identifiers
            transactions: Vec::new(),
            execution_proof: Vec::new(),
            validator_signatures: Vec::new(),
        };
        
        assert!(block.add_shard_data(invalid_shard_data).is_err());
    }
}
