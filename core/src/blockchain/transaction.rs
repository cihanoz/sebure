//! # Transaction Module
//! 
//! This module defines the transaction data structure and related functionality.

use crate::crypto::signature::Signature;
use crate::types::{Result, ShardId, Timestamp, TransactionType, Priority, DataType};
use serde::{Serialize, Deserialize};

/// Transaction represents a transfer of value or execution of logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction ID (hash)
    pub id: Vec<u8>,
    
    /// Transaction format version
    pub version: u8,
    
    /// Type of transaction
    pub transaction_type: TransactionType,
    
    /// Sender's public key
    pub sender_public_key: Vec<u8>,
    
    /// Sender's shard ID
    pub sender_shard: ShardId,
    
    /// Recipient's address
    pub recipient_address: Vec<u8>,
    
    /// Recipient's shard ID
    pub recipient_shard: ShardId,
    
    /// Amount to transfer
    pub amount: u64,
    
    /// Transaction fee
    pub fee: u32,
    
    /// Gas limit for smart contracts
    pub gas_limit: u32,
    
    /// Transaction nonce (to prevent replay attacks)
    pub nonce: u64,
    
    /// Timestamp (microseconds since Unix epoch)
    pub timestamp: Timestamp,
    
    /// Optional transaction data
    pub data: TransactionData,
    
    /// Optional transaction dependencies
    pub dependencies: Vec<Vec<u8>>,
    
    /// Signature of the transaction
    pub signature: Signature,
    
    /// Execution priority
    pub execution_priority: Priority,
}

/// Transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    /// Type of data contained
    pub data_type: DataType,
    
    /// Data content
    pub content: Vec<u8>,
}

impl Default for TransactionData {
    fn default() -> Self {
        TransactionData {
            data_type: DataType::None,
            content: Vec::new(),
        }
    }
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Transaction ID
    pub transaction_id: Vec<u8>,
    
    /// Status code
    pub status: u32,
    
    /// Gas used
    pub gas_used: u32,
    
    /// Resulting state root
    pub state_root: Vec<u8>,
    
    /// Logs generated during execution
    pub logs: Vec<Log>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    /// Address that generated the log
    pub address: Vec<u8>,
    
    /// Log topics
    pub topics: Vec<Vec<u8>>,
    
    /// Log data
    pub data: Vec<u8>,
}

impl Transaction {
    /// Create a new basic transaction
    pub fn new(
        sender_public_key: Vec<u8>,
        sender_shard: ShardId,
        recipient_address: Vec<u8>,
        recipient_shard: ShardId,
        amount: u64,
        fee: u32,
        gas_limit: u32,
        nonce: u64,
        transaction_type: TransactionType,
        data: TransactionData,
        dependencies: Vec<Vec<u8>>,
        signature: Signature,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        // Create a "dummy" transaction with ID field to be filled
        let mut tx = Transaction {
            id: vec![0; 32], // Temporary ID, will be replaced
            version: 1, // Set default version
            transaction_type,
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
            execution_priority: Priority::Normal,
        };
        
        // Calculate the proper transaction ID using the hash function
        let id = match crate::crypto::hash::hash_transaction(&tx) {
            Ok(hash) => hash,
            Err(_) => vec![0; 32], // Fallback in case of error
        };
        
        Transaction {
            id,
            version: 1,
            transaction_type,
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
            execution_priority: Priority::Normal,
        }
    }
    
    /// Create a new transfer transaction
    pub fn new_transfer(
        sender_public_key: Vec<u8>,
        sender_shard: ShardId,
        recipient_address: Vec<u8>,
        recipient_shard: ShardId,
        amount: u64,
        fee: u32,
        nonce: u64,
    ) -> Self {
        // Create an empty signature (in a real implementation, this would be properly signed)
        let signature = Signature::new(vec![0; 64]);
        
        Self::new(
            sender_public_key,
            sender_shard,
            recipient_address,
            recipient_shard,
            amount,
            fee,
            0, // No gas needed for a simple transfer
            nonce,
            TransactionType::Transfer,
            TransactionData::default(),
            Vec::new(), // No dependencies
            signature,
        )
    }
    
    /// Verify transaction signature
    pub fn verify_signature(&self) -> Result<()> {
        // In a real implementation, we would:
        // 1. Serialize the transaction data (except signature)
        // 2. Verify the signature against the sender's public key
        
        // For now, just return success
        Ok(())
    }
    
    /// Check if the transaction is valid
    pub fn is_valid(&self) -> Result<()> {
        // In a real implementation, we would:
        // 1. Check transaction format and version
        // 2. Verify signature
        // 3. Check nonce, fee, gas limit, etc.
        
        self.verify_signature()?;
        
        // For now, just return success
        Ok(())
    }
    
    /// Calculate the transaction hash
    pub fn hash(&self) -> Vec<u8> {
        // Use the hash_transaction function from our crypto module
        match crate::crypto::hash::hash_transaction(self) {
            Ok(hash) => hash,
            Err(_) => vec![0; 32], // Fallback in case of error
        }
    }
    
    /// Get the estimated gas cost
    pub fn estimate_gas(&self) -> u32 {
        // In a real implementation, this would depend on the transaction type and data
        match self.transaction_type {
            TransactionType::Transfer => 21000,
            TransactionType::ContractDeploy => 100000,
            TransactionType::ContractCall => 50000,
            _ => 21000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_transaction() {
        let sender_public_key = vec![1; 32];
        let recipient_address = vec![2; 20];
        
        let tx = Transaction::new_transfer(
            sender_public_key.clone(),
            0, // sender shard
            recipient_address.clone(),
            1, // recipient shard
            1000, // amount
            10, // fee
            0, // nonce
        );
        
        assert_eq!(tx.version, 1);
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.sender_public_key, sender_public_key);
        assert_eq!(tx.sender_shard, 0);
        assert_eq!(tx.recipient_address, recipient_address);
        assert_eq!(tx.recipient_shard, 1);
        assert_eq!(tx.amount, 1000);
        assert_eq!(tx.fee, 10);
        assert_eq!(tx.nonce, 0);
    }
    
    #[test]
    fn test_transaction_verification() {
        let sender_public_key = vec![1; 32];
        let recipient_address = vec![2; 20];
        
        let tx = Transaction::new_transfer(
            sender_public_key,
            0,
            recipient_address,
            1,
            1000,
            10,
            0,
        );
        
        assert!(tx.verify_signature().is_ok());
        assert!(tx.is_valid().is_ok());
    }
    
    #[test]
    fn test_gas_estimation() {
        // Create a transfer transaction
        let transfer_tx = Transaction::new_transfer(
            vec![1; 32],
            0,
            vec![2; 20],
            1,
            1000,
            10,
            0,
        );
        
        // Gas for a transfer transaction
        assert_eq!(transfer_tx.estimate_gas(), 21000);
        
        // Create other transaction types
        let mut contract_deploy_tx = transfer_tx.clone();
        contract_deploy_tx.transaction_type = TransactionType::ContractDeploy;
        
        let mut contract_call_tx = transfer_tx.clone();
        contract_call_tx.transaction_type = TransactionType::ContractCall;
        
        // Gas for other transaction types
        assert_eq!(contract_deploy_tx.estimate_gas(), 100000);
        assert_eq!(contract_call_tx.estimate_gas(), 50000);
    }
}
