//! Transaction types and serialization

use serde::{Serialize, Deserialize};
use thiserror::Error;
use crate::types::{ShardId, Priority};

/// Transaction type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    ContractCall,
    ValidatorOperation,
}

/// Transaction data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionData {
    None,
    ContractCall {
        contract_address: [u8; 20],
        method: String,
        params: Vec<u8>,
    },
    ValidatorOperation {
        operation: u8,
        data: Vec<u8>,
    },
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: [u8; 32],
    pub version: u8,
    pub type_: TransactionType,
    pub sender_public_key: [u8; 32],
    pub sender_shard: ShardId,
    pub recipient_address: [u8; 20],
    pub recipient_shard: ShardId,
    pub amount: u64,
    pub fee: u32,
    pub gas_limit: u32,
    pub nonce: u64,
    pub timestamp: u64,
    pub data: TransactionData,
    pub dependencies: Vec<[u8; 32]>,
    pub signature: [u8; 64],
    pub execution_priority: Priority,
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        id: [u8; 32],
        version: u8,
        type_: TransactionType,
        sender_public_key: [u8; 32],
        sender_shard: ShardId,
        recipient_address: [u8; 20],
        recipient_shard: ShardId,
        amount: u64,
        fee: u32,
        gas_limit: u32,
        nonce: u64,
        timestamp: u64,
        data: TransactionData,
        dependencies: Vec<[u8; 32]>,
        signature: [u8; 64],
        execution_priority: Priority,
    ) -> Self {
        Transaction {
            id,
            version,
            type_,
            sender_public_key,
            sender_shard,
            recipient_address,
            recipient_shard,
            amount,
            fee,
            gas_limit,
            nonce,
            timestamp,
            data,
            dependencies,
            signature,
            execution_priority,
        }
    }

    /// Create a dummy transaction for testing
    pub fn new_dummy(nonce: u64) -> Self {
        Self::new(
            [0; 32],
            1,
            TransactionType::Transfer,
            [0; 32],
            0,
            [0; 20],
            0,
            100,
            10,
            100000,
            nonce,
            1234567890,
            TransactionData::None,
            vec![],
            [0; 64],
            Priority::Normal,
        )
    }

    /// Serialize transaction to binary format
    pub fn to_binary(&self) -> Result<Vec<u8>, TransactionError> {
        bincode::serialize(self)
            .map_err(|e| TransactionError::SerializationError(e.to_string()))
    }

    /// Deserialize transaction from binary format
    pub fn from_binary(data: &[u8]) -> Result<Self, TransactionError> {
        bincode::deserialize(data)
            .map_err(|e| TransactionError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_serialization() -> Result<(), TransactionError> {
        let tx = Transaction::new_dummy(1);
        let bytes = tx.to_binary()?;
        let tx2 = Transaction::from_binary(&bytes)?;
        
        assert_eq!(tx.id, tx2.id);
        assert_eq!(tx.version, tx2.version);
        assert_eq!(tx.type_, tx2.type_);
        assert_eq!(tx.sender_public_key, tx2.sender_public_key);
        assert_eq!(tx.sender_shard, tx2.sender_shard);
        assert_eq!(tx.recipient_address, tx2.recipient_address);
        assert_eq!(tx.recipient_shard, tx2.recipient_shard);
        assert_eq!(tx.amount, tx2.amount);
        assert_eq!(tx.fee, tx2.fee);
        assert_eq!(tx.gas_limit, tx2.gas_limit);
        assert_eq!(tx.nonce, tx2.nonce);
        assert_eq!(tx.timestamp, tx2.timestamp);
        assert_eq!(tx.signature, tx2.signature);
        assert_eq!(tx.execution_priority, tx2.execution_priority);
        Ok(())
    }
}
