//! Transaction Service IPC Bridge
//!
//! This module provides the interface between the Flutter UI and the Rust transaction service,
//! enabling transaction creation, signing, validation, and submission.

use sebure_core::{
    blockchain::{Transaction, TransactionData, Mempool},
    services::transaction_service::{TransactionService, TransactionServiceConfig, FeeEstimationModel},
    crypto::signature::KeyPair,
    types::{Result, Error, ShardId, TransactionType, DataType, Priority},
    storage::state_db::StateDB,
};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_ulonglong};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};

// Global instance of the transaction service
lazy_static! {
    static ref TRANSACTION_SERVICE: Mutex<Option<Arc<RwLock<TransactionService>>>> = Mutex::new(None);
}

/// Initialize the transaction service
///
/// # Safety
///
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_transaction_service_init() -> c_int {
    // Get shared resources
    let blockchain = match super::get_blockchain() {
        Some(blockchain) => blockchain,
        None => {
            error!("Failed to get blockchain instance");
            return -1;
        }
    };
    
    // Get mempool from blockchain
    let mempool = {
        let blockchain_guard = match blockchain.read() {
            Ok(guard) => guard,
            Err(_) => {
                error!("Failed to acquire blockchain read lock");
                return -1;
            }
        };
        blockchain_guard.mempool()
    };
    
    // Create state DB
    // In a real implementation, we would get this from a proper storage service
    // For now, we'll create a new in-memory state DB
    let storage_config = sebure_core::storage::StorageConfig {
        data_dir: "".to_string(),
        create_if_missing: true,
        ..Default::default()
    };
    
    let state_db = match StateDB::new("", &storage_config) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to create state DB: {}", e);
            return -1;
        }
    };
    
    // Create transaction service configuration
    let config = TransactionServiceConfig::default();
    
    // Create transaction service
    let transaction_service = TransactionService::new(mempool, state_db, config);
    
    // Store the service instance
    let service_instance = Arc::new(RwLock::new(transaction_service));
    *TRANSACTION_SERVICE.lock().unwrap() = Some(service_instance);
    
    info!("Initialized transaction service");
    0
}

/// Create a transaction
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_create_transaction(
    sender_public_key: *const c_char,
    sender_shard: c_uint,
    recipient_address: *const c_char,
    recipient_shard: c_uint,
    amount: c_ulonglong,
    fee: c_uint,
    transaction_type: c_int,
    tx_id_out: *mut *mut c_char,
) -> c_int {
    // Check pointers
    if sender_public_key.is_null() || recipient_address.is_null() || tx_id_out.is_null() {
        error!("Invalid pointer in sebure_create_transaction");
        return -1;
    }
    
    // Get the transaction service
    let service_lock = TRANSACTION_SERVICE.lock().unwrap();
    let service = match &*service_lock {
        Some(service) => service.clone(),
        None => {
            error!("Transaction service not initialized");
            return -1;
        }
    };
    
    // Convert C strings to Rust strings
    let sender_pk_cstr = CStr::from_ptr(sender_public_key);
    let sender_pk_str = match sender_pk_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in sender public key");
            return -1;
        }
    };
    
    let recipient_addr_cstr = CStr::from_ptr(recipient_address);
    let recipient_addr_str = match recipient_addr_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in recipient address");
            return -1;
        }
    };
    
    // Convert hex strings to bytes
    let sender_pk_bytes = match hex::decode(sender_pk_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in sender public key: {}", e);
            return -1;
        }
    };
    
    let recipient_addr_bytes = match hex::decode(recipient_addr_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in recipient address: {}", e);
            return -1;
        }
    };
    
    // Convert transaction type
    let tx_type = match transaction_type {
        0 => TransactionType::Transfer,
        1 => TransactionType::ContractDeploy,
        2 => TransactionType::ContractCall,
        3 => TransactionType::ValidatorRegister,
        4 => TransactionType::ValidatorUnregister,
        5 => TransactionType::Stake,
        6 => TransactionType::Unstake,
        7 => TransactionType::System,
        _ => {
            error!("Invalid transaction type: {}", transaction_type);
            return -1;
        }
    };
    
    // Create the transaction
    let service_guard = service.read().unwrap();
    let result = service_guard.create_transaction(
        &sender_pk_bytes,
        sender_shard as ShardId,
        &recipient_addr_bytes,
        recipient_shard as ShardId,
        amount,
        Some(fee),
        None, // Use default gas limit
        None, // Use account nonce
        tx_type,
        None, // No data
        None, // No dependencies
        None, // Use default priority
    );
    
    match result {
        Ok(tx) => {
            // Convert transaction ID to hex string
            let tx_id_hex = hex::encode(&tx.id);
            
            // Convert to C string
            let tx_id_cstr = match CString::new(tx_id_hex) {
                Ok(s) => s,
                Err(_) => {
                    error!("Failed to create C string for transaction ID");
                    return -1;
                }
            };
            
            // Transfer ownership to caller
            *tx_id_out = tx_id_cstr.into_raw();
            
            0
        },
        Err(e) => {
            error!("Failed to create transaction: {}", e);
            -1
        }
    }
}

/// Sign a transaction
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_sign_transaction(
    tx_id: *const c_char,
    private_key: *const c_char,
) -> c_int {
    // Check pointers
    if tx_id.is_null() || private_key.is_null() {
        error!("Invalid pointer in sebure_sign_transaction");
        return -1;
    }
    
    // Get the transaction service
    let service_lock = TRANSACTION_SERVICE.lock().unwrap();
    let service = match &*service_lock {
        Some(service) => service.clone(),
        None => {
            error!("Transaction service not initialized");
            return -1;
        }
    };
    
    // Convert C strings to Rust strings
    let tx_id_cstr = CStr::from_ptr(tx_id);
    let tx_id_str = match tx_id_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in transaction ID");
            return -1;
        }
    };
    
    let private_key_cstr = CStr::from_ptr(private_key);
    let private_key_str = match private_key_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in private key");
            return -1;
        }
    };
    
    // Convert hex strings to bytes
    let tx_id_bytes = match hex::decode(tx_id_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in transaction ID: {}", e);
            return -1;
        }
    };
    
    let private_key_bytes = match hex::decode(private_key_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in private key: {}", e);
            return -1;
        }
    };
    
    // Get the transaction from mempool
    let mut service_guard = service.write().unwrap();
    let mempool = service_guard.mempool.lock().unwrap();
    let tx_option = mempool.get_transaction(&tx_id_bytes);
    
    if let Some(mut tx) = tx_option {
        // Sign the transaction
        match service_guard.sign_transaction(&mut tx, &private_key_bytes) {
            Ok(_) => {
                // Transaction is now signed
                0
            },
            Err(e) => {
                error!("Failed to sign transaction: {}", e);
                -1
            }
        }
    } else {
        error!("Transaction not found in mempool: {}", tx_id_str);
        -1
    }
}

/// Submit a transaction
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_submit_transaction(
    sender_public_key: *const c_char,
    sender_private_key: *const c_char,
    sender_shard: c_uint,
    recipient_address: *const c_char,
    recipient_shard: c_uint,
    amount: c_ulonglong,
    fee: c_uint,
    tx_id_out: *mut *mut c_char,
) -> c_int {
    // Check pointers
    if sender_public_key.is_null() || sender_private_key.is_null() || 
       recipient_address.is_null() || tx_id_out.is_null() {
        error!("Invalid pointer in sebure_submit_transaction");
        return -1;
    }
    
    // Get the transaction service
    let service_lock = TRANSACTION_SERVICE.lock().unwrap();
    let service = match &*service_lock {
        Some(service) => service.clone(),
        None => {
            error!("Transaction service not initialized");
            return -1;
        }
    };
    
    // Convert C strings to Rust strings
    let sender_pk_cstr = CStr::from_ptr(sender_public_key);
    let sender_pk_str = match sender_pk_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in sender public key");
            return -1;
        }
    };
    
    let sender_sk_cstr = CStr::from_ptr(sender_private_key);
    let sender_sk_str = match sender_sk_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in sender private key");
            return -1;
        }
    };
    
    let recipient_addr_cstr = CStr::from_ptr(recipient_address);
    let recipient_addr_str = match recipient_addr_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in recipient address");
            return -1;
        }
    };
    
    // Convert hex strings to bytes
    let sender_pk_bytes = match hex::decode(sender_pk_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in sender public key: {}", e);
            return -1;
        }
    };
    
    let sender_sk_bytes = match hex::decode(sender_sk_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in sender private key: {}", e);
            return -1;
        }
    };
    
    let recipient_addr_bytes = match hex::decode(recipient_addr_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in recipient address: {}", e);
            return -1;
        }
    };
    
    // Create and sign the transaction
    let service_guard = service.read().unwrap();
    let result = service_guard.create_transfer(
        &sender_sk_bytes,
        &sender_pk_bytes,
        sender_shard as ShardId,
        &recipient_addr_bytes,
        recipient_shard as ShardId,
        amount,
        Some(fee),
    );
    
    match result {
        Ok(tx) => {
            // Submit the transaction
            match service_guard.submit_transaction(tx.clone()) {
                Ok(_) => {
                    // Convert transaction ID to hex string
                    let tx_id_hex = hex::encode(&tx.id);
                    
                    // Convert to C string
                    let tx_id_cstr = match CString::new(tx_id_hex) {
                        Ok(s) => s,
                        Err(_) => {
                            error!("Failed to create C string for transaction ID");
                            return -1;
                        }
                    };
                    
                    // Transfer ownership to caller
                    *tx_id_out = tx_id_cstr.into_raw();
                    
                    0
                },
                Err(e) => {
                    error!("Failed to submit transaction: {}", e);
                    -1
                }
            }
        },
        Err(e) => {
            error!("Failed to create transaction: {}", e);
            -1
        }
    }
}

/// Estimate transaction fee
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_estimate_fee(
    transaction_type: c_int,
    data_size: c_uint,
    fee_out: *mut c_uint,
) -> c_int {
    // Check pointers
    if fee_out.is_null() {
        error!("Invalid pointer in sebure_estimate_fee");
        return -1;
    }
    
    // Get the transaction service
    let service_lock = TRANSACTION_SERVICE.lock().unwrap();
    let service = match &*service_lock {
        Some(service) => service.clone(),
        None => {
            error!("Transaction service not initialized");
            return -1;
        }
    };
    
    // Convert transaction type
    let tx_type = match transaction_type {
        0 => TransactionType::Transfer,
        1 => TransactionType::ContractDeploy,
        2 => TransactionType::ContractCall,
        3 => TransactionType::ValidatorRegister,
        4 => TransactionType::ValidatorUnregister,
        5 => TransactionType::Stake,
        6 => TransactionType::Unstake,
        7 => TransactionType::System,
        _ => {
            error!("Invalid transaction type: {}", transaction_type);
            return -1;
        }
    };
    
    // Estimate fee
    let service_guard = service.read().unwrap();
    let fee = service_guard.estimate_fee(tx_type, data_size as usize);
    
    // Return the fee
    *fee_out = fee;
    0
}

/// Get transaction history
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_get_transaction_history(
    address: *const c_char,
    count_out: *mut c_uint,
    tx_ids_out: *mut *mut c_char,
    amounts_out: *mut c_ulonglong,
    timestamps_out: *mut c_ulonglong,
    is_outgoing_out: *mut c_int,
) -> c_int {
    // Check pointers
    if address.is_null() || count_out.is_null() || tx_ids_out.is_null() || 
       amounts_out.is_null() || timestamps_out.is_null() || is_outgoing_out.is_null() {
        error!("Invalid pointer in sebure_get_transaction_history");
        return -1;
    }
    
    // Get the transaction service
    let service_lock = TRANSACTION_SERVICE.lock().unwrap();
    let service = match &*service_lock {
        Some(service) => service.clone(),
        None => {
            error!("Transaction service not initialized");
            return -1;
        }
    };
    
    // Convert C string to Rust string
    let address_cstr = CStr::from_ptr(address);
    let address_str = match address_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in address");
            return -1;
        }
    };
    
    // Convert hex string to bytes
    let address_bytes = match hex::decode(address_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in address: {}", e);
            return -1;
        }
    };
    
    // Get transaction history
    let service_guard = service.read().unwrap();
    let history = service_guard.get_transaction_history(&address_bytes);
    
    // Return empty if no history
    if history.is_empty() {
        *count_out = 0;
        return 0;
    }
    
    // Allocate memory for the results
    let count = history.len().min(100); // Limit to 100 transactions
    *count_out = count as c_uint;
    
    // Allocate arrays
    let tx_ids = calloc(count, std::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    let amounts = calloc(count, std::mem::size_of::<c_ulonglong>()) as *mut c_ulonglong;
    let timestamps = calloc(count, std::mem::size_of::<c_ulonglong>()) as *mut c_ulonglong;
    let is_outgoing = calloc(count, std::mem::size_of::<c_int>()) as *mut c_int;
    
    // Fill the arrays
    for (i, tx) in history.iter().take(count).enumerate() {
        // Transaction ID
        let tx_id_hex = hex::encode(&tx.id);
        let tx_id_cstr = match CString::new(tx_id_hex) {
            Ok(s) => s,
            Err(_) => {
                error!("Failed to create C string for transaction ID");
                return -1;
            }
        };
        *tx_ids.add(i) = tx_id_cstr.into_raw();
        
        // Amount
        *amounts.add(i) = tx.amount;
        
        // Timestamp
        *timestamps.add(i) = tx.timestamp;
        
        // Is outgoing (1 if sender address matches, 0 otherwise)
        let sender_address = sebure_core::crypto::hash::sha256(&tx.sender_public_key);
        *is_outgoing.add(i) = if sender_address == address_bytes { 1 } else { 0 };
    }
    
    // Set output pointers
    *tx_ids_out = tx_ids;
    *amounts_out = amounts;
    *timestamps_out = timestamps;
    *is_outgoing_out = is_outgoing;
    
    0
}

/// Get account balance
///
/// # Safety
///
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_get_balance(
    address: *const c_char,
    balance_out: *mut c_ulonglong,
) -> c_int {
    // Check pointers
    if address.is_null() || balance_out.is_null() {
        error!("Invalid pointer in sebure_get_balance");
        return -1;
    }
    
    // Get the transaction service
    let service_lock = TRANSACTION_SERVICE.lock().unwrap();
    let service = match &*service_lock {
        Some(service) => service.clone(),
        None => {
            error!("Transaction service not initialized");
            return -1;
        }
    };
    
    // Convert C string to Rust string
    let address_cstr = CStr::from_ptr(address);
    let address_str = match address_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            error!("Invalid UTF-8 in address");
            return -1;
        }
    };
    
    // Convert hex string to bytes
    let address_bytes = match hex::decode(address_str) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Invalid hex in address: {}", e);
            return -1;
        }
    };
    
    // Get balance
    let service_guard = service.read().unwrap();
    match service_guard.get_balance(&address_bytes) {
        Ok(balance) => {
            *balance_out = balance;
            0
        },
        Err(e) => {
            error!("Failed to get balance: {}", e);
            -1
        }
    }
}

// Helper function for memory allocation
unsafe fn calloc(count: usize, size: usize) -> *mut std::ffi::c_void {
    let layout = std::alloc::Layout::from_size_align(count * size, std::mem::align_of::<usize>())
        .expect("Invalid layout");
    let ptr = std::alloc::alloc_zeroed(layout);
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    ptr as *mut std::ffi::c_void
}

#[cfg(test)]
mod tests {
    // Tests would go here
}
