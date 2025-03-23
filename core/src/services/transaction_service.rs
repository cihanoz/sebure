//! # Transaction Service
//! 
//! This module implements the transaction service, which provides functionality
//! for creating, signing, validating, and submitting transactions.

use crate::blockchain::{Transaction, TransactionData, Mempool};
use crate::crypto::signature::{self, KeyPair, Signature};
use crate::crypto::hash;
use crate::types::{Result, Error, ShardId, TransactionType, DataType, Priority};
use crate::storage::state_db::StateDB;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use log::{info, debug, error, warn};

/// Fee estimation model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeEstimationModel {
    /// Fixed fee regardless of transaction size or type
    Fixed,
    
    /// Fee based on transaction size
    SizeBased,
    
    /// Fee based on transaction type
    TypeBased,
    
    /// Dynamic fee based on network congestion
    Dynamic,
}

/// Configuration for the transaction service
#[derive(Debug, Clone)]
pub struct TransactionServiceConfig {
    /// Default fee for transactions
    pub default_fee: u32,
    
    /// Fee estimation model
    pub fee_model: FeeEstimationModel,
    
    /// Network congestion multiplier (for dynamic fee model)
    pub congestion_multiplier: f32,
    
    /// Maximum transaction history items to keep per address
    pub max_history_items: usize,
    
    /// Whether to cache transaction history
    pub cache_history: bool,
}

impl Default for TransactionServiceConfig {
    fn default() -> Self {
        TransactionServiceConfig {
            default_fee: 10,
            fee_model: FeeEstimationModel::SizeBased,
            congestion_multiplier: 1.0,
            max_history_items: 100,
            cache_history: true,
        }
    }
}

/// Service for managing transactions, including creation, signing, validation,
/// fee estimation, and submission to the mempool.
pub struct TransactionService {
    /// Mempool for transaction submission
    mempool: Arc<Mutex<Mempool>>,
    
    /// State database for account information
    state_db: Arc<StateDB>,
    
    /// Cache for transaction history by address
    tx_history: Mutex<HashMap<Vec<u8>, Vec<Transaction>>>,
    
    /// Service configuration
    config: TransactionServiceConfig,
}

impl TransactionService {
    /// Creates a new TransactionService with the given mempool and state database.
    pub fn new(mempool: Arc<Mutex<Mempool>>, state_db: Arc<StateDB>, config: TransactionServiceConfig) -> Self {
        TransactionService {
            mempool,
            state_db,
            tx_history: Mutex::new(HashMap::new()),
            config,
        }
    }
    
    /// Creates a new transaction with the given parameters.
    pub fn create_transaction(
        &self,
        sender_public_key: &[u8],
        sender_shard: ShardId,
        recipient_address: &[u8],
        recipient_shard: ShardId,
        amount: u64,
        fee: Option<u32>,
        gas_limit: Option<u32>,
        nonce: Option<u64>,
        transaction_type: TransactionType,
        data: Option<TransactionData>,
        dependencies: Option<Vec<Vec<u8>>>,
        priority: Option<Priority>,
    ) -> Result<Transaction> {
        // Get the account nonce if not provided
        let nonce = match nonce {
            Some(n) => n,
            None => {
                // Calculate address from public key
                let address = hash::sha256(sender_public_key);
                self.state_db.get_account_nonce(&address).unwrap_or(0)
            }
        };
        
        // Use provided fee or estimate
        let fee = fee.unwrap_or_else(|| self.estimate_fee(
            transaction_type,
            data.as_ref().map(|d| d.content.len()).unwrap_or(0),
        ));
        
        // Use provided gas limit or estimate
        let gas_limit = gas_limit.unwrap_or_else(|| match transaction_type {
            TransactionType::Transfer => 21000,
            TransactionType::ContractDeploy => 100000,
            TransactionType::ContractCall => 50000,
            _ => 21000,
        });
        
        // Use provided data or create empty
        let data = data.unwrap_or_else(TransactionData::default);
        
        // Use provided dependencies or empty
        let dependencies = dependencies.unwrap_or_else(Vec::new);
        
        // Create an empty signature (will be signed later)
        let signature = Signature::new(vec![0; 64]);
        
        // Create the transaction
        let mut tx = Transaction::new(
            sender_public_key.to_vec(),
            sender_shard,
            recipient_address.to_vec(),
            recipient_shard,
            amount,
            fee,
            gas_limit,
            nonce,
            transaction_type,
            data,
            dependencies,
            signature,
        );
        
        // Set priority if provided
        if let Some(p) = priority {
            tx.execution_priority = p;
        }
        
        // Calculate transaction ID
        let tx_bytes = bincode::serialize(&tx)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        let tx_hash = hash::sha256(&tx_bytes);
        tx.id = tx_hash.to_vec();
        
        Ok(tx)
    }
    
    /// Signs a transaction with the given private key.
    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8]) -> Result<()> {
        // Create a keypair from the private key
        let keypair = KeyPair::from_seed(private_key)?;
        
        // Serialize the transaction (excluding the signature)
        let mut tx_copy = tx.clone();
        tx_copy.signature = Signature::new(vec![0; 64]); // Clear signature for signing
        
        let tx_bytes = bincode::serialize(&tx_copy)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        // Sign the transaction
        let signature = keypair.sign(&tx_bytes);
        tx.signature = signature;
        
        Ok(())
    }
    
    /// Validates a transaction.
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Check basic transaction format
        if tx.id.is_empty() {
            return Err(Error::TransactionValidation("Transaction ID is empty".to_string()));
        }
        
        if tx.sender_public_key.is_empty() {
            return Err(Error::TransactionValidation("Sender public key is empty".to_string()));
        }
        
        if tx.recipient_address.is_empty() {
            return Err(Error::TransactionValidation("Recipient address is empty".to_string()));
        }
        
        // Verify signature
        let mut tx_copy = tx.clone();
        tx_copy.signature = Signature::new(vec![0; 64]); // Clear signature for verification
        
        let tx_bytes = bincode::serialize(&tx_copy)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        signature::verify(&tx.sender_public_key, &tx_bytes, &tx.signature)?;
        
        // Calculate address from public key
        let sender_address = hash::sha256(&tx.sender_public_key);
        
        // Check nonce
        let current_nonce = self.state_db.get_account_nonce(&sender_address).unwrap_or(0);
        if tx.nonce < current_nonce {
            return Err(Error::TransactionValidation(
                format!("Nonce too low: {} < {}", tx.nonce, current_nonce)
            ));
        }
        
        // Check balance for transfers
        if tx.transaction_type == TransactionType::Transfer {
            let balance = self.state_db.get_account_balance(&sender_address).unwrap_or(0);
            let total_cost = tx.amount + tx.fee as u64;
            
            if balance < total_cost {
                return Err(Error::TransactionValidation(
                    format!("Insufficient balance: {} < {}", balance, total_cost)
                ));
            }
        }
        
        // Verify transaction ID
        let tx_bytes = bincode::serialize(&tx_copy)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        let tx_hash = hash::sha256(&tx_bytes);
        
        if tx.id != tx_hash.to_vec() {
            return Err(Error::TransactionValidation(
                "Transaction ID does not match hash of transaction data".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Submits a transaction to the mempool.
    pub fn submit_transaction(&self, tx: Transaction) -> Result<()> {
        // Validate the transaction
        self.validate_transaction(&tx)?;
        
        // Add to mempool
        self.mempool.lock().unwrap().add_transaction(tx.clone())?;
        
        // Add to transaction history if caching is enabled
        if self.config.cache_history {
            let sender_address = hash::sha256(&tx.sender_public_key);
            self.add_to_history(sender_address.to_vec(), tx.clone());
            
            // Also add to recipient's history
            self.add_to_history(tx.recipient_address.clone(), tx);
        }
        
        Ok(())
    }
    
    /// Estimates the fee for a transaction.
    pub fn estimate_fee(&self, transaction_type: TransactionType, data_size: usize) -> u32 {
        match self.config.fee_model {
            FeeEstimationModel::Fixed => {
                self.config.default_fee
            },
            FeeEstimationModel::SizeBased => {
                // Base fee + additional fee per byte of data
                let base_fee = self.config.default_fee;
                let size_fee = (data_size / 100) as u32; // 1 fee unit per 100 bytes
                base_fee + size_fee
            },
            FeeEstimationModel::TypeBased => {
                match transaction_type {
                    TransactionType::Transfer => self.config.default_fee,
                    TransactionType::ContractDeploy => self.config.default_fee * 10,
                    TransactionType::ContractCall => self.config.default_fee * 5,
                    _ => self.config.default_fee * 2,
                }
            },
            FeeEstimationModel::Dynamic => {
                // Base fee adjusted by congestion multiplier
                let base_fee = match transaction_type {
                    TransactionType::Transfer => self.config.default_fee,
                    TransactionType::ContractDeploy => self.config.default_fee * 10,
                    TransactionType::ContractCall => self.config.default_fee * 5,
                    _ => self.config.default_fee * 2,
                };
                
                let size_fee = (data_size / 100) as u32; // 1 fee unit per 100 bytes
                
                // Apply congestion multiplier
                ((base_fee + size_fee) as f32 * self.config.congestion_multiplier) as u32
            },
        }
    }
    
    /// Gets the transaction history for an address.
    pub fn get_transaction_history(&self, address: &[u8]) -> Vec<Transaction> {
        if self.config.cache_history {
            // Try to get from cache first
            let history = self.tx_history.lock().unwrap();
            if let Some(txs) = history.get(address) {
                return txs.clone();
            }
        }
        
        // If not in cache or caching disabled, return empty
        // In a real implementation, we would query the blockchain
        Vec::new()
    }
    
    /// Adds a transaction to the history cache.
    fn add_to_history(&self, address: Vec<u8>, tx: Transaction) {
        if !self.config.cache_history {
            return;
        }
        
        let mut history = self.tx_history.lock().unwrap();
        let txs = history.entry(address).or_insert_with(Vec::new);
        
        // Add the transaction
        txs.push(tx);
        
        // Limit the history size
        if txs.len() > self.config.max_history_items {
            // Sort by timestamp (newest first) and truncate
            txs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            txs.truncate(self.config.max_history_items);
        }
    }
    
    /// Gets the balance for an address.
    pub fn get_balance(&self, address: &[u8]) -> Result<u64> {
        self.state_db.get_account_balance(address)
    }
    
    /// Creates a transfer transaction.
    pub fn create_transfer(
        &self,
        sender_private_key: &[u8],
        sender_public_key: &[u8],
        sender_shard: ShardId,
        recipient_address: &[u8],
        recipient_shard: ShardId,
        amount: u64,
        fee: Option<u32>,
    ) -> Result<Transaction> {
        // Create the transaction
        let mut tx = self.create_transaction(
            sender_public_key,
            sender_shard,
            recipient_address,
            recipient_shard,
            amount,
            fee,
            None, // Use default gas limit
            None, // Use account nonce
            TransactionType::Transfer,
            None, // No data
            None, // No dependencies
            None, // Use default priority
        )?;
        
        // Sign the transaction
        self.sign_transaction(&mut tx, sender_private_key)?;
        
        Ok(tx)
    }
    
    /// Updates the configuration.
    pub fn update_config(&mut self, config: TransactionServiceConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::MempoolConfig;
    use crate::storage::StorageConfig;
    use std::sync::Arc;
    
    // Helper function to create a test transaction service
    fn create_test_service() -> TransactionService {
        let mempool = Arc::new(Mutex::new(Mempool::new(MempoolConfig::default())));
        
        // Create an in-memory state DB for testing
        let storage_config = StorageConfig {
            data_dir: "".to_string(),
            create_if_missing: true,
            ..Default::default()
        };
        
        let state_db = Arc::new(StateDB::new("", &storage_config).unwrap());
        
        TransactionService::new(
            mempool,
            state_db,
            TransactionServiceConfig::default(),
        )
    }
    
    #[test]
    fn test_create_transaction() {
        let service = create_test_service();
        
        let sender_public_key = vec![1; 32];
        let recipient_address = vec![2; 20];
        
        let tx = service.create_transaction(
            &sender_public_key,
            0, // sender shard
            &recipient_address,
            1, // recipient shard
            1000, // amount
            Some(10), // fee
            None, // gas limit
            Some(0), // nonce
            TransactionType::Transfer,
            None, // data
            None, // dependencies
            None, // priority
        ).unwrap();
        
        assert_eq!(tx.sender_public_key, sender_public_key);
        assert_eq!(tx.recipient_address, recipient_address);
        assert_eq!(tx.amount, 1000);
        assert_eq!(tx.fee, 10);
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert!(!tx.id.is_empty());
    }
    
    #[test]
    fn test_sign_transaction() {
        let service = create_test_service();
        
        // Generate a keypair for testing
        let keypair = KeyPair::generate();
        let private_key = keypair.private_key();
        let public_key = keypair.public_key();
        
        let recipient_address = vec![2; 20];
        
        let mut tx = service.create_transaction(
            &public_key,
            0, // sender shard
            &recipient_address,
            1, // recipient shard
            1000, // amount
            Some(10), // fee
            None, // gas limit
            Some(0), // nonce
            TransactionType::Transfer,
            None, // data
            None, // dependencies
            None, // priority
        ).unwrap();
        
        // Sign the transaction
        service.sign_transaction(&mut tx, &private_key).unwrap();
        
        // Verify the signature
        let mut tx_copy = tx.clone();
        tx_copy.signature = Signature::new(vec![0; 64]); // Clear signature for verification
        
        let tx_bytes = bincode::serialize(&tx_copy).unwrap();
        let result = signature::verify(&public_key, &tx_bytes, &tx.signature);
        
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_fee_estimation() {
        let service = create_test_service();
        
        // Test fixed fee model
        let mut config = TransactionServiceConfig::default();
        config.fee_model = FeeEstimationModel::Fixed;
        config.default_fee = 10;
        
        let mut service = TransactionService::new(
            service.mempool.clone(),
            service.state_db.clone(),
            config,
        );
        
        let fee = service.estimate_fee(TransactionType::Transfer, 1000);
        assert_eq!(fee, 10);
        
        // Test size-based fee model
        let mut config = TransactionServiceConfig::default();
        config.fee_model = FeeEstimationModel::SizeBased;
        config.default_fee = 10;
        
        service.update_config(config);
        
        let fee = service.estimate_fee(TransactionType::Transfer, 1000);
        assert_eq!(fee, 10 + 10); // base fee + size fee (1000 bytes = 10 fee units)
        
        // Test type-based fee model
        let mut config = TransactionServiceConfig::default();
        config.fee_model = FeeEstimationModel::TypeBased;
        config.default_fee = 10;
        
        service.update_config(config);
        
        let transfer_fee = service.estimate_fee(TransactionType::Transfer, 0);
        let deploy_fee = service.estimate_fee(TransactionType::ContractDeploy, 0);
        let call_fee = service.estimate_fee(TransactionType::ContractCall, 0);
        
        assert_eq!(transfer_fee, 10);
        assert_eq!(deploy_fee, 100);
        assert_eq!(call_fee, 50);
    }
    
    #[test]
    fn test_transaction_history() {
        let service = create_test_service();
        
        let sender_public_key = vec![1; 32];
        let sender_address = hash::sha256(&sender_public_key);
        let recipient_address = vec![2; 20];
        
        // Create a transaction
        let tx = service.create_transaction(
            &sender_public_key,
            0, // sender shard
            &recipient_address,
            1, // recipient shard
            1000, // amount
            Some(10), // fee
            None, // gas limit
            Some(0), // nonce
            TransactionType::Transfer,
            None, // data
            None, // dependencies
            None, // priority
        ).unwrap();
        
        // Add to history
        service.add_to_history(sender_address.to_vec(), tx.clone());
        
        // Get history
        let history = service.get_transaction_history(&sender_address);
        
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, tx.id);
    }
}
