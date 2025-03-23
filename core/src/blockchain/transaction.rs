//! # Transaction Module
//! 
//! This module defines the transaction data structure and related functionality.

use crate::crypto::signature::Signature;
use crate::types::{Result, ShardId, Timestamp, TransactionType, Priority, DataType};
use serde::{Serialize, Deserialize};
use crate::crypto::hash;

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
    
    /// Transaction dependencies with type information
    pub dependencies: Vec<Dependency>,
    
    /// Signature of the transaction
    pub signature: Signature,
    
    /// Execution priority with dynamic adjustment
    pub execution_priority: Priority,
    
    /// Optimistic execution status
    pub optimistic_status: OptimisticStatus,
    
    /// Parallel execution markers
    pub parallel_markers: ParallelMarkers,
    
    /// Batch information (if part of a batch)
    pub batch_info: Option<BatchInfo>,
}

/// Transaction dependency with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Transaction ID this depends on
    pub transaction_id: Vec<u8>,
    
    /// Type of dependency
    pub dependency_type: DependencyType,
    
    /// Required state
    pub required_state: Option<Vec<u8>>,
}

/// Dependency type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard dependency - must execute after
    Hard,
    /// Soft dependency - can execute in parallel
    Soft,
    /// State dependency - requires specific state
    State,
}

/// Optimistic execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimisticStatus {
    /// Not yet executed
    Pending,
    /// Executed optimistically
    Executed,
    /// Confirmed valid
    Confirmed,
    /// Rolled back due to conflict
    RolledBack,
}

/// Parallel execution markers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelMarkers {
    /// Can this transaction execute in parallel
    pub parallelizable: bool,
    
    /// Execution group ID
    pub group_id: Option<u64>,
    
    /// Execution phase
    pub phase: u8,
}

/// Batch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInfo {
    /// Batch ID
    pub batch_id: Vec<u8>,
    
    /// Position in batch
    pub position: u32,
    
    /// Batch size
    pub batch_size: u32,
}

/// Transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    /// Type of data contained
    pub data_type: DataType,
    
    /// Data content
    pub content: Vec<u8>,
}

impl TransactionData {
    /// Get the type of data
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    
    /// Get the content of the data
    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
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
        
        // Calculate transaction ID based on the transaction data
        let id = hash::sha256(&[0; 32]); // We'll use a placeholder hash for now
        
        // Convert simple dependencies to Dependency structs
        let dependencies = dependencies.into_iter().map(|dep_id| Dependency {
            transaction_id: dep_id,
            dependency_type: DependencyType::Hard,
            required_state: None,
        }).collect();
        
        Transaction {
            id: id.to_vec(),
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
            optimistic_status: OptimisticStatus::Pending,
            parallel_markers: ParallelMarkers {
                parallelizable: true,
                group_id: None,
                phase: 0,
            },
            batch_info: None,
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
    
    /// Get the transaction ID
    pub fn id(&self) -> &Vec<u8> {
        &self.id
    }
    
    /// Get the transaction type
    pub fn transaction_type(&self) -> TransactionType {
        self.transaction_type
    }
    
    /// Get the sender's public key
    pub fn sender_public_key(&self) -> &Vec<u8> {
        &self.sender_public_key
    }
    
    /// Get the sender's shard ID
    pub fn sender_shard(&self) -> ShardId {
        self.sender_shard
    }
    
    /// Get the recipient's address
    pub fn recipient_address(&self) -> &Vec<u8> {
        &self.recipient_address
    }
    
    /// Get the recipient's shard ID
    pub fn recipient_shard(&self) -> ShardId {
        self.recipient_shard
    }
    
    /// Get the transaction amount
    pub fn amount(&self) -> u64 {
        self.amount
    }
    
    /// Get the transaction fee
    pub fn fee(&self) -> u32 {
        self.fee
    }
    
    /// Get the transaction nonce
    pub fn nonce(&self) -> u64 {
        self.nonce
    }
    
    /// Get the transaction timestamp
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
    
    /// Get the transaction data
    pub fn data(&self) -> &TransactionData {
        &self.data
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

    // Optimistic execution methods
    
    /// Mark transaction as executed optimistically
    pub fn mark_executed(&mut self) {
        self.optimistic_status = OptimisticStatus::Executed;
    }
    
    /// Mark transaction as confirmed
    pub fn mark_confirmed(&mut self) {
        self.optimistic_status = OptimisticStatus::Confirmed;
    }
    
    /// Mark transaction as rolled back
    pub fn mark_rolled_back(&mut self) {
        self.optimistic_status = OptimisticStatus::RolledBack;
    }
    
    /// Check if transaction has conflicts
    pub fn has_conflicts(&self) -> bool {
        matches!(self.optimistic_status, OptimisticStatus::RolledBack)
    }

    // Dependency tracking methods
    
    /// Add a new dependency
    pub fn add_dependency(&mut self, transaction_id: Vec<u8>, dependency_type: DependencyType) {
        self.dependencies.push(Dependency {
            transaction_id,
            dependency_type,
            required_state: None,
        });
    }
    
    /// Remove a dependency by transaction ID
    pub fn remove_dependency(&mut self, transaction_id: &Vec<u8>) {
        self.dependencies.retain(|d| &d.transaction_id != transaction_id);
    }
    
    /// Verify all dependencies are satisfied
    pub fn verify_dependencies(&self, satisfied_ids: &[Vec<u8>]) -> bool {
        self.dependencies.iter().all(|d| {
            match d.dependency_type {
                DependencyType::Hard => satisfied_ids.contains(&d.transaction_id),
                _ => true // Soft and State dependencies don't block execution
            }
        })
    }

    // Parallel processing methods
    
    /// Set parallel execution group
    pub fn set_parallel_group(&mut self, group_id: u64, phase: u8) {
        self.parallel_markers.group_id = Some(group_id);
        self.parallel_markers.phase = phase;
    }
    
    /// Check if transaction can execute in parallel
    pub fn can_execute_in_parallel(&self) -> bool {
        self.parallel_markers.parallelizable
    }
    
    /// Get execution phase
    pub fn get_execution_phase(&self) -> u8 {
        self.parallel_markers.phase
    }

    // Batching methods
    
    /// Add transaction to a batch
    pub fn add_to_batch(&mut self, batch_id: Vec<u8>, position: u32, batch_size: u32) {
        self.batch_info = Some(BatchInfo {
            batch_id,
            position,
            batch_size,
        });
    }
    
    /// Remove from batch
    pub fn remove_from_batch(&mut self) {
        self.batch_info = None;
    }
    
    /// Get batch information
    pub fn get_batch_info(&self) -> Option<&BatchInfo> {
        self.batch_info.as_ref()
    }

    // Prioritization methods
    
    /// Adjust execution priority
    pub fn adjust_priority(&mut self, new_priority: Priority) {
        self.execution_priority = new_priority;
    }
    
    /// Get current priority
    pub fn get_priority(&self) -> Priority {
        self.execution_priority
    }
    
    /// Check if transaction should be expedited
    pub fn should_expedite(&self) -> bool {
        matches!(self.execution_priority, Priority::High)
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

    #[test]
    fn test_optimistic_execution() {
        let mut tx = Transaction::new_transfer(
            vec![1; 32],
            0,
            vec![2; 20],
            1,
            1000,
            10,
            0,
        );
        
        // Initial state
        assert!(matches!(tx.optimistic_status, OptimisticStatus::Pending));
        
        // Test state transitions
        tx.mark_executed();
        assert!(matches!(tx.optimistic_status, OptimisticStatus::Executed));
        
        tx.mark_confirmed();
        assert!(matches!(tx.optimistic_status, OptimisticStatus::Confirmed));
        
        tx.mark_rolled_back();
        assert!(matches!(tx.optimistic_status, OptimisticStatus::RolledBack));
        assert!(tx.has_conflicts());
    }

    #[test]
    fn test_dependency_management() {
        let mut tx = Transaction::new_transfer(
            vec![1; 32],
            0,
            vec![2; 20],
            1,
            1000,
            10,
            0,
        );
        
        // Add dependencies
        tx.add_dependency(vec![1; 32], DependencyType::Hard);
        tx.add_dependency(vec![2; 32], DependencyType::Soft);
        assert_eq!(tx.dependencies.len(), 2);
        
        // Verify dependencies
        assert!(tx.verify_dependencies(&[vec![1; 32], vec![2; 32]]));
        assert!(!tx.verify_dependencies(&[vec![2; 32]])); // Missing hard dependency
        
        // Remove dependency
        tx.remove_dependency(&vec![1; 32]);
        assert_eq!(tx.dependencies.len(), 1);
    }

    #[test]
    fn test_parallel_processing() {
        let mut tx = Transaction::new_transfer(
            vec![1; 32],
            0,
            vec![2; 20],
            1,
            1000,
            10,
            0,
        );
        
        // Default parallel settings
        assert!(tx.can_execute_in_parallel());
        assert_eq!(tx.get_execution_phase(), 0);
        
        // Configure parallel group
        tx.set_parallel_group(123, 2);
        assert_eq!(tx.parallel_markers.group_id, Some(123));
        assert_eq!(tx.get_execution_phase(), 2);
    }

    #[test]
    fn test_batching() {
        let mut tx = Transaction::new_transfer(
            vec![1; 32],
            0,
            vec![2; 20],
            1,
            1000,
            10,
            0,
        );
        
        // Add to batch
        tx.add_to_batch(vec![1; 32], 5, 10);
        let batch_info = tx.get_batch_info().unwrap();
        assert_eq!(batch_info.position, 5);
        assert_eq!(batch_info.batch_size, 10);
        
        // Remove from batch
        tx.remove_from_batch();
        assert!(tx.get_batch_info().is_none());
    }

    #[test]
    fn test_prioritization() {
        let mut tx = Transaction::new_transfer(
            vec![1; 32],
            0,
            vec![2; 20],
            1,
            1000,
            10,
            0,
        );
        
        // Default priority
        assert!(!tx.should_expedite());
        
        // Adjust priority
        tx.adjust_priority(Priority::High);
        assert!(tx.should_expedite());
        assert!(matches!(tx.get_priority(), Priority::High));
    }
}
