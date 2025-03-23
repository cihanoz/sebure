//! # Transaction Mempool Implementation
//! 
//! This module implements the transaction mempool, which stores pending
//! transactions before they are included in blocks.

use std::collections::{HashMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::blockchain::Transaction;
use crate::types::{Result, Error, ShardId, Priority};

/// Configuration for the transaction mempool
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum number of transactions in the mempool
    pub max_size: usize,
    
    /// Maximum transaction age before expiration (in seconds)
    pub max_age: u64,
    
    /// Minimum fee per byte to be accepted
    pub min_fee_per_byte: u64,
    
    /// Maximum size (in bytes) of a single transaction
    pub max_tx_size: usize,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        MempoolConfig {
            max_size: 10000,
            max_age: 3600, // 1 hour
            min_fee_per_byte: 1,
            max_tx_size: 1024 * 1024, // 1 MB
        }
    }
}

/// Transaction with additional mempool metadata
#[derive(Debug, Clone)]
struct MempoolTx {
    /// Reference to the actual transaction
    pub transaction: Arc<Transaction>,
    
    /// When the transaction was added to the mempool
    pub received_at: Instant,
    
    /// Estimated size in bytes
    pub size: usize,
    
    /// Fee per byte ratio (used for prioritization)
    pub fee_per_byte: u64,
    
    /// Whether this transaction is ready to be included in a block
    pub ready: bool,
}

impl MempoolTx {
    /// Create a new mempool transaction
    fn new(tx: Arc<Transaction>) -> Self {
        // Estimate size (in a real implementation, this would be more accurate)
        let size = tx.id.len() + tx.sender_public_key.len() + tx.recipient_address.len() + 
                  tx.data.content.len() + 100; // Add 100 bytes for fixed fields
        
        // Calculate fee per byte
        let fee_per_byte = if size > 0 {
            tx.fee as u64 / size as u64
        } else {
            0
        };
        
        MempoolTx {
            transaction: tx.clone(),
            received_at: Instant::now(),
            size,
            fee_per_byte,
            ready: true, // By default, transactions are ready (unless they have dependencies)
        }
    }
    
    /// Check if the transaction has expired
    fn is_expired(&self, max_age: Duration) -> bool {
        self.received_at.elapsed() > max_age
    }
}

/// A comparator for transaction priority that sorts by:
/// 1. Priority level (High > Normal > Low)
/// 2. Fee per byte (higher is better)
/// 3. Received time (earlier is better)
#[derive(PartialEq, Eq)]
struct TxPriorityOrder {
    /// Transaction ID
    tx_id: Vec<u8>,
    
    /// Transaction priority
    priority: Priority,
    
    /// Fee per byte
    fee_per_byte: u64,
    
    /// Received timestamp (as duration since UNIX_EPOCH)
    received_at: Duration,
}

impl TxPriorityOrder {
    fn new(tx: &MempoolTx) -> Self {
        TxPriorityOrder {
            tx_id: tx.transaction.id.clone(),
            priority: tx.transaction.execution_priority,
            fee_per_byte: tx.fee_per_byte,
            received_at: tx.received_at.elapsed(),
        }
    }
}

impl PartialOrd for TxPriorityOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxPriorityOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by priority
        let priority_cmp = self.priority.cmp(&other.priority);
        if priority_cmp != std::cmp::Ordering::Equal {
            return priority_cmp;
        }
        
        // Then by fee per byte
        let fee_cmp = self.fee_per_byte.cmp(&other.fee_per_byte);
        if fee_cmp != std::cmp::Ordering::Equal {
            return fee_cmp;
        }
        
        // Then by received time (earlier is better, so reverse ordering)
        other.received_at.cmp(&self.received_at)
    }
}

/// Transaction mempool for storing pending transactions
pub struct Mempool {
    /// Configuration for the mempool
    config: MempoolConfig,
    
    /// Transactions indexed by ID
    transactions: Arc<Mutex<HashMap<Vec<u8>, MempoolTx>>>,
    
    /// Transaction priority queue
    priority_index: Arc<Mutex<BTreeSet<TxPriorityOrder>>>,
    
    /// Transactions organized by shard
    shard_index: Arc<Mutex<HashMap<ShardId, Vec<Vec<u8>>>>>,
    
    /// Dependency tracking (tx_id -> dependent tx_ids)
    dependencies: Arc<Mutex<HashMap<Vec<u8>, Vec<Vec<u8>>>>>,
    
    /// Reverse dependency tracking (tx_id -> required tx_ids)
    reverse_dependencies: Arc<Mutex<HashMap<Vec<u8>, Vec<Vec<u8>>>>>,
}

impl Mempool {
    /// Create a new transaction mempool
    pub fn new(config: MempoolConfig) -> Self {
        Mempool {
            config,
            transactions: Arc::new(Mutex::new(HashMap::new())),
            priority_index: Arc::new(Mutex::new(BTreeSet::new())),
            shard_index: Arc::new(Mutex::new(HashMap::new())),
            dependencies: Arc::new(Mutex::new(HashMap::new())),
            reverse_dependencies: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Add a transaction to the mempool
    pub fn add_transaction(&self, tx: &Transaction) -> Result<()> {
        // Basic validation
        if tx.id.is_empty() {
            return Err(Error::BlockValidation("Transaction ID is empty".to_string()));
        }
        
        // Verify the transaction
        tx.is_valid()?;
        
        let tx_id = tx.id.clone();
        let mempool_tx = MempoolTx::new(Arc::new(tx.clone()));
        
        // Check size
        if mempool_tx.size > self.config.max_tx_size {
            return Err(Error::BlockValidation(
                format!("Transaction size {} exceeds maximum {}", mempool_tx.size, self.config.max_tx_size)
            ));
        }
        
        // Check fee per byte
        if mempool_tx.fee_per_byte < self.config.min_fee_per_byte {
            return Err(Error::BlockValidation(
                format!("Fee per byte {} is below minimum {}", mempool_tx.fee_per_byte, self.config.min_fee_per_byte)
            ));
        }
        
        // Update mempool data structures
        {
            let mut transactions = self.transactions.lock().unwrap();
            
            // Check if we're at capacity
            if transactions.len() >= self.config.max_size && !transactions.contains_key(&tx_id) {
                return Err(Error::BlockValidation("Mempool is full".to_string()));
            }
            
            // Check if this transaction is already in the mempool
            if transactions.contains_key(&tx_id) {
                return Err(Error::BlockValidation("Transaction already in mempool".to_string()));
            }
            
            // Add to main index
            transactions.insert(tx_id.clone(), mempool_tx.clone());
        }
        
        // Add to priority index
        {
            let mut priority_index = self.priority_index.lock().unwrap();
            priority_index.insert(TxPriorityOrder::new(&mempool_tx));
        }
        
        // Add to shard index
        {
            let mut shard_index = self.shard_index.lock().unwrap();
            let shard_txs = shard_index.entry(tx.sender_shard).or_insert_with(Vec::new);
            shard_txs.push(tx_id.clone());
        }
        
        // Add dependency tracking if needed
        if !tx.dependencies.is_empty() {
            let mut mempool_tx = mempool_tx;
            mempool_tx.ready = false; // Not ready until all dependencies are met
            
            // Update in transactions map
            {
                let mut transactions = self.transactions.lock().unwrap();
                transactions.insert(tx_id.clone(), mempool_tx);
            }
            
            // Add reverse dependencies
            let mut reverse_deps = self.reverse_dependencies.lock().unwrap();
            for dep_id in &tx.dependencies {
                let deps = reverse_deps.entry(dep_id.clone()).or_insert_with(|| Vec::new());
                deps.push(tx_id.clone());
            }
            
            // Check which dependencies are not met
            let mut deps = self.dependencies.lock().unwrap();
            let mut unmet_deps = Vec::new();
            
            {
                let transactions = self.transactions.lock().unwrap();
                for dep_id in &tx.dependencies {
                    if !transactions.contains_key(dep_id) {
                        unmet_deps.push(dep_id.clone());
                    }
                }
            }
            
            if !unmet_deps.is_empty() {
                deps.insert(tx_id.clone(), unmet_deps);
            } else {
                // All dependencies are met, mark as ready
                let mut transactions = self.transactions.lock().unwrap();
                if let Some(tx) = transactions.get_mut(&tx_id) {
                    tx.ready = true;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a transaction by its ID
    pub fn get_transaction(&self, tx_id: &[u8]) -> Option<Arc<Transaction>> {
        let transactions = self.transactions.lock().unwrap();
        transactions.get(tx_id).map(|mempool_tx| mempool_tx.transaction.clone())
    }
    
    /// Remove a transaction from the mempool
    pub fn remove_transaction(&self, tx_id: &[u8]) -> Result<()> {
        // Get the transaction first to check if it exists
        let tx_option = self.transactions.lock().unwrap().get(tx_id).cloned();
        
        if let Some(mempool_tx) = tx_option {
            // Remove from main index
            {
                let mut transactions = self.transactions.lock().unwrap();
                transactions.remove(tx_id);
            }
            
            // Remove from priority index
            {
                let mut priority_index = self.priority_index.lock().unwrap();
                priority_index.remove(&TxPriorityOrder::new(&mempool_tx));
            }
            
            // Remove from shard index
            {
                let mut shard_index = self.shard_index.lock().unwrap();
                if let Some(shard_txs) = shard_index.get_mut(&mempool_tx.transaction.sender_shard) {
                    shard_txs.retain(|id| id != tx_id);
                }
            }
            
            // Update dependency tracking
            {
                let mut deps = self.dependencies.lock().unwrap();
                deps.remove(tx_id);
            }
            
            // Update reverse dependencies
            {
                let mut reverse_deps = self.reverse_dependencies.lock().unwrap();
                if let Some(dependent_txs) = reverse_deps.remove(tx_id) {
                    let mut deps = self.dependencies.lock().unwrap();
                    
                    // For each transaction that depends on this one
                    for dep_tx_id in dependent_txs {
                        if let Some(unmet_deps) = deps.get_mut(&dep_tx_id) {
                            // Remove this transaction from the unmet dependencies
                            unmet_deps.retain(|id| id != tx_id);
                            
                            // If all dependencies are now met, mark as ready
                            if unmet_deps.is_empty() {
                                deps.remove(&dep_tx_id);
                                
                                let mut transactions = self.transactions.lock().unwrap();
                                if let Some(tx) = transactions.get_mut(&dep_tx_id) {
                                    tx.ready = true;
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(())
        } else {
            Err(Error::State(format!("Transaction {:?} not found in mempool", tx_id)))
        }
    }
    
    /// Get the number of transactions in the mempool
    pub fn size(&self) -> usize {
        let transactions = self.transactions.lock().unwrap();
        transactions.len()
    }
    
    /// Get a batch of transactions for inclusion in a block, filtered by shard
    pub fn get_transactions_for_block(&self, shard_id: ShardId, max_count: usize) -> Vec<Arc<Transaction>> {
        let mut result = Vec::with_capacity(max_count);
        let mut included_tx_ids = Vec::new();
        
        // Get transactions from priority index, filtered by readiness and shard
        {
            let transactions = self.transactions.lock().unwrap();
            let priority_index = self.priority_index.lock().unwrap();
            
            for priority_tx in priority_index.iter().rev() {
                if result.len() >= max_count {
                    break;
                }
                
                if let Some(mempool_tx) = transactions.get(&priority_tx.tx_id) {
                    if mempool_tx.ready && mempool_tx.transaction.sender_shard == shard_id {
                        result.push(mempool_tx.transaction.clone());
                        included_tx_ids.push(mempool_tx.transaction.id.clone());
                    }
                }
            }
        }
        
        result
    }
    
    /// Remove expired transactions from the mempool
    pub fn remove_expired(&self) -> usize {
        let max_age = Duration::from_secs(self.config.max_age);
        let mut expired_tx_ids = Vec::new();
        
        // Find expired transactions
        {
            let transactions = self.transactions.lock().unwrap();
            for (tx_id, mempool_tx) in transactions.iter() {
                if mempool_tx.is_expired(max_age) {
                    expired_tx_ids.push(tx_id.clone());
                }
            }
        }
        
        // Remove them
        for tx_id in &expired_tx_ids {
            let _ = self.remove_transaction(tx_id);
        }
        
        expired_tx_ids.len()
    }
    
    /// Clear all transactions from the mempool
    pub fn clear(&self) {
        let mut transactions = self.transactions.lock().unwrap();
        let mut priority_index = self.priority_index.lock().unwrap();
        let mut shard_index = self.shard_index.lock().unwrap();
        let mut dependencies = self.dependencies.lock().unwrap();
        let mut reverse_dependencies = self.reverse_dependencies.lock().unwrap();
        
        transactions.clear();
        priority_index.clear();
        shard_index.clear();
        dependencies.clear();
        reverse_dependencies.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::signature::Signature;
    use crate::types::{DataType, TransactionType};
    
    fn create_test_transaction(
        id: Vec<u8>,
        sender_shard: ShardId,
        amount: u64,
        fee: u32,
        priority: Priority,
        dependencies: Vec<Vec<u8>>,
    ) -> Transaction {
        let sender_key = vec![1; 32];
        let recipient = vec![2; 20];
        
        let mut tx = Transaction::new_transfer(
            sender_key,
            sender_shard,
            recipient,
            sender_shard, // same shard for simplicity
            amount,
            fee,
            0, // nonce
        );
        
        // Override the auto-generated ID
        tx.id = id;
        tx.execution_priority = priority;
        tx.dependencies = dependencies;
        
        tx
    }
    
    #[test]
    fn test_mempool_basics() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);
        
        // Empty mempool
        assert_eq!(mempool.size(), 0);
        
        // Add a transaction
        let tx = Arc::new(create_test_transaction(
            vec![1, 2, 3, 4],
            0, // shard 0
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        ));
        
        assert!(mempool.add_transaction(&tx).is_ok());
        assert_eq!(mempool.size(), 1);
        
        // Get the transaction
        let retrieved = mempool.get_transaction(&tx.id).unwrap();
        assert_eq!(retrieved.id, tx.id);
        
        // Remove the transaction
        assert!(mempool.remove_transaction(&tx.id).is_ok());
        assert_eq!(mempool.size(), 0);
        assert!(mempool.get_transaction(&tx.id).is_none());
    }
    
    #[test]
    fn test_mempool_priority() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);
        
        // Add transactions with different priorities
        let tx1 = create_test_transaction(
            vec![1, 1, 1, 1],
            0,
            1000,
            10,
            Priority::Low,
            Vec::new(),
        );
        
        let tx2 = create_test_transaction(
            vec![2, 2, 2, 2],
            0,
            1000,
            20, // higher fee
            Priority::Normal,
            Vec::new(),
        );
        
        let tx3 = create_test_transaction(
            vec![3, 3, 3, 3],
            0,
            1000,
            5, // lower fee
            Priority::High,
            Vec::new(),
        );
        
        mempool.add_transaction(&tx1).unwrap();
        mempool.add_transaction(&tx2).unwrap();
        mempool.add_transaction(&tx3).unwrap();
        
        // Get transactions by priority
        let transactions = mempool.get_transactions_for_block(0, 3);
        
        // Should be ordered by priority: High, Normal, Low
        assert_eq!(transactions.len(), 3);
        assert_eq!(transactions[0].id, tx3.id); // High priority
        assert_eq!(transactions[1].id, tx2.id); // Normal priority
        assert_eq!(transactions[2].id, tx1.id); // Low priority
    }
    
    #[test]
    fn test_mempool_dependencies() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);
        
        // Create transaction with a dependency
        let tx1_id = vec![1, 1, 1, 1];
        let tx2_id = vec![2, 2, 2, 2];
        
        let tx2 = create_test_transaction(
            tx2_id.clone(),
            0,
            1000,
            10,
            Priority::Normal,
            vec![tx1_id.clone()], // depends on tx1
        );
        
        // Add tx2 first (which depends on tx1)
        mempool.add_transaction(&tx2).unwrap();
        
        // Check that it's not ready
        {
            let transactions = mempool.transactions.lock().unwrap();
            let tx = transactions.get(&tx2_id).unwrap();
            assert!(!tx.ready);
        }
        
        // Now add tx1
        let tx1 = create_test_transaction(
            tx1_id.clone(),
            0,
            500,
            5,
            Priority::Normal,
            Vec::new(),
        );
        
        mempool.add_transaction(&tx1).unwrap();
        
        // Check that tx2 is now ready
        {
            let transactions = mempool.transactions.lock().unwrap();
            let tx = transactions.get(&tx2_id).unwrap();
            assert!(tx.ready);
        }
        
        // Get transactions for block
        let block_txs = mempool.get_transactions_for_block(0, 2);
        
        // Both should be included
        assert_eq!(block_txs.len(), 2);
        
        // Remove tx1
        mempool.remove_transaction(&tx1_id).unwrap();
        
        // Check that tx2 is not ready again
        {
            let transactions = mempool.transactions.lock().unwrap();
            let tx = transactions.get(&tx2_id).unwrap();
            assert!(!tx.ready);
        }
        
        // Should not be included in block txs now
        let block_txs = mempool.get_transactions_for_block(0, 2);
        assert_eq!(block_txs.len(), 0);
    }
    
    #[test]
    fn test_mempool_shard_filtering() {
        let config = MempoolConfig::default();
        let mempool = Mempool::new(config);
        
        // Add transactions from different shards
        let tx1 = create_test_transaction(
            vec![1, 1, 1, 1],
            0, // shard 0
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        );
        
        let tx2 = create_test_transaction(
            vec![2, 2, 2, 2],
            1, // shard 1
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        );
        
        let tx3 = create_test_transaction(
            vec![3, 3, 3, 3],
            0, // shard 0
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        );
        
        mempool.add_transaction(&tx1).unwrap();
        mempool.add_transaction(&tx2).unwrap();
        mempool.add_transaction(&tx3).unwrap();
        
        // Get transactions for shard 0
        let shard0_txs = mempool.get_transactions_for_block(0, 10);
        assert_eq!(shard0_txs.len(), 2);
        
        // Get transactions for shard 1
        let shard1_txs = mempool.get_transactions_for_block(1, 10);
        assert_eq!(shard1_txs.len(), 1);
        assert_eq!(shard1_txs[0].id, tx2.id);
    }
    
    #[test]
    fn test_mempool_expiration() {
        // Create a config with very short expiration
        let config = MempoolConfig {
            max_age: 1, // 1 second expiration
            ..MempoolConfig::default()
        };
        
        let mempool = Mempool::new(config);
        
        // Add a transaction
        let tx = create_test_transaction(
            vec![1, 2, 3, 4],
            0,
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        );
        
        mempool.add_transaction(&tx).unwrap();
        assert_eq!(mempool.size(), 1);
        
        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));
        
        // Remove expired transactions
        let removed = mempool.remove_expired();
        assert_eq!(removed, 1);
        assert_eq!(mempool.size(), 0);
    }
    
    #[test]
    fn test_mempool_full() {
        // Create a config with very small capacity
        let config = MempoolConfig {
            max_size: 2,
            ..MempoolConfig::default()
        };
        
        let mempool = Mempool::new(config);
        
        // Add two transactions
        let tx1 = create_test_transaction(
            vec![1, 1, 1, 1],
            0,
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        );
        
        let tx2 = create_test_transaction(
            vec![2, 2, 2, 2],
            0,
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        );
        
        mempool.add_transaction(&tx1).unwrap();
        mempool.add_transaction(&tx2).unwrap();
        assert_eq!(mempool.size(), 2);
        
        // Try to add a third transaction
        let tx3 = create_test_transaction(
            vec![3, 3, 3, 3],
            0,
            1000,
            10,
            Priority::Normal,
            Vec::new(),
        );
        
        // Should fail because mempool is full
        let result = mempool.add_transaction(tx3);
        assert!(result.is_err());
    }
}
