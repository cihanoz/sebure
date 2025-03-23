//! # SEBURE Blockchain FFI
//! 
//! Foreign Function Interface (FFI) bindings for the SEBURE blockchain,
//! enabling integration with other languages like Dart for Flutter UI.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uint, c_ulonglong};
use std::sync::{Arc, Mutex, RwLock};
use lazy_static::lazy_static;
use sebure_core::{
    self, 
    blockchain::{Blockchain, BlockchainConfig},
    Consensus, ConsensusConfig,
    Network, NetworkConfig,
    Storage, StorageConfig
};

// Include bridge modules
mod validation_bridge;
mod transaction_bridge;

// Global instances for the FFI layer
lazy_static! {
    static ref STORAGE: Mutex<Option<Storage>> = Mutex::new(None);
    static ref NETWORK: Mutex<Option<Network>> = Mutex::new(None);
    static ref BLOCKCHAIN: Mutex<Option<Arc<RwLock<Blockchain>>>> = Mutex::new(None);
    // To make the Consensus trait usable with Send and Sync, we need to be more careful with it
    // For now, just comment it out until we properly implement the Send trait for Consensus
    // static ref CONSENSUS: Mutex<Option<Box<dyn Consensus>>> = Mutex::new(None);
}

// Helper function to get blockchain instance for validation service
// This is used internally by the validation bridge
fn get_blockchain() -> Option<Arc<RwLock<Blockchain>>> {
    match BLOCKCHAIN.lock() {
        Ok(lock) => lock.clone(),
        Err(_) => None,
    }
}

/// Error codes for FFI functions
#[repr(C)]
pub enum ErrorCode {
    /// No error
    Success = 0,
    /// Invalid argument
    InvalidArgument = 1,
    /// IO error
    IoError = 2,
    /// Network error
    NetworkError = 3,
    /// Consensus error
    ConsensusError = 4,
    /// Storage error
    StorageError = 5,
    /// Already initialized
    AlreadyInitialized = 6,
    /// Not initialized
    NotInitialized = 7,
    /// Internal error
    InternalError = 8,
    /// Unknown error
    Unknown = 99,
}

/// Map Rust Result to FFI ErrorCode
fn map_result<T>(_result: sebure_core::Result<T>) -> ErrorCode {
    // In a real implementation, we would properly map errors
    // For now, just return success
    ErrorCode::Success
}

/// Initialize the SEBURE blockchain core
/// 
/// # Safety
/// 
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_init() -> ErrorCode {
    match sebure_core::init() {
        Ok(_) => ErrorCode::Success,
        Err(_) => ErrorCode::Unknown,
    }
}

/// Initialize the blockchain with default configuration
/// 
/// # Safety
/// 
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_blockchain_init() -> ErrorCode {
    let mut blockchain_lock = match BLOCKCHAIN.lock() {
        Ok(lock) => lock,
        Err(_) => return ErrorCode::Unknown,
    };

    if blockchain_lock.is_some() {
        return ErrorCode::AlreadyInitialized;
    }
    
    match Blockchain::new() {
        Ok(blockchain) => {
            *blockchain_lock = Some(Arc::new(RwLock::new(blockchain)));
            ErrorCode::Success
        },
        Err(_) => ErrorCode::InternalError,
    }
}

/// Initialize storage with the provided data directory
/// 
/// # Safety
/// 
/// This function is unsafe because it takes a raw pointer and modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_storage_init(data_dir: *const c_char) -> ErrorCode {
    // Check if already initialized
    let mut storage_lock = match STORAGE.lock() {
        Ok(lock) => lock,
        Err(_) => return ErrorCode::Unknown,
    };

    if storage_lock.is_some() {
        return ErrorCode::AlreadyInitialized;
    }
    
    // Convert C string to Rust string
    let data_dir_cstr = if data_dir.is_null() {
        return ErrorCode::InvalidArgument;
    } else {
        CStr::from_ptr(data_dir)
    };
    
    let data_dir_str = match data_dir_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ErrorCode::InvalidArgument,
    };
    
    // Create storage configuration
    let config = StorageConfig {
        data_dir: data_dir_str.to_string(),
        ..StorageConfig::default()
    };
    
    // Initialize storage
    match Storage::new(config) {
        Ok(storage) => {
            *storage_lock = Some(storage);
            ErrorCode::Success
        },
        Err(_) => ErrorCode::StorageError,
    }
}

/// Initialize network with the provided listen address
/// 
/// # Safety
/// 
/// This function is unsafe because it takes a raw pointer and modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_network_init(listen_addr: *const c_char) -> ErrorCode {
    // Check if already initialized
    let mut network_lock = match NETWORK.lock() {
        Ok(lock) => lock,
        Err(_) => return ErrorCode::Unknown,
    };

    if network_lock.is_some() {
        return ErrorCode::AlreadyInitialized;
    }
    
    // Convert C string to Rust string
    let addr_cstr = if listen_addr.is_null() {
        return ErrorCode::InvalidArgument;
    } else {
        CStr::from_ptr(listen_addr)
    };
    
    let addr_str = match addr_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ErrorCode::InvalidArgument,
    };
    
    // Parse address
    let addr = match addr_str.parse() {
        Ok(a) => a,
        Err(_) => return ErrorCode::InvalidArgument,
    };
    
    // Create network configuration
    let config = NetworkConfig {
        listen_addr: addr,
        ..NetworkConfig::default()
    };
    
    // Initialize network
    let network = Network::new(config);
    *network_lock = Some(network);
    
    ErrorCode::Success
}

/// Start the network service
/// 
/// # Safety
/// 
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_network_start() -> ErrorCode {
    // Check if initialized
    let mut network_lock = match NETWORK.lock() {
        Ok(lock) => lock,
        Err(_) => return ErrorCode::Unknown,
    };

    let network = match network_lock.as_mut() {
        Some(n) => n,
        None => return ErrorCode::NotInitialized,
    };
    
    // Start network
    match network.start() {
        Ok(_) => ErrorCode::Success,
        Err(_) => ErrorCode::NetworkError,
    }
}

/// Create a new account
/// 
/// # Safety
/// 
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_create_account(
    public_key_out: *mut *mut c_char,
    private_key_out: *mut *mut c_char,
) -> ErrorCode {
    if public_key_out.is_null() || private_key_out.is_null() {
        return ErrorCode::InvalidArgument;
    }
    
    // Generate a new key pair
    let keypair = sebure_core::crypto::KeyPair::generate();
    
    // Get public and private keys
    let public_key = keypair.public_key();
    let private_key = keypair.private_key();
    
    // Convert to hex strings
    let public_key_hex = hex::encode(&public_key);
    let private_key_hex = hex::encode(&private_key);
    
    // Convert to C strings
    let public_key_cstr = match CString::new(public_key_hex) {
        Ok(s) => s,
        Err(_) => return ErrorCode::Unknown,
    };
    
    let private_key_cstr = match CString::new(private_key_hex) {
        Ok(s) => s,
        Err(_) => return ErrorCode::Unknown,
    };
    
    // Transfer ownership to caller
    *public_key_out = public_key_cstr.into_raw();
    *private_key_out = private_key_cstr.into_raw();
    
    ErrorCode::Success
}

/// Get account balance
/// 
/// # Safety
/// 
/// This function is unsafe because it takes raw pointers.
#[no_mangle]
pub unsafe extern "C" fn sebure_get_account_balance(
    address: *const c_char,
    balance_out: *mut c_ulonglong,
) -> ErrorCode {
    if address.is_null() || balance_out.is_null() {
        return ErrorCode::InvalidArgument;
    }
    
    // Check if storage is initialized
    let storage_lock = match STORAGE.lock() {
        Ok(lock) => lock,
        Err(_) => return ErrorCode::Unknown,
    };

    let storage = match storage_lock.as_ref() {
        Some(s) => s,
        None => return ErrorCode::NotInitialized,
    };
    
    // Convert C string to Rust string
    let addr_cstr = CStr::from_ptr(address);
    let addr_str = match addr_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ErrorCode::InvalidArgument,
    };
    
    // Parse hex address
    let addr_bytes = match hex::decode(addr_str) {
        Ok(b) => b,
        Err(_) => return ErrorCode::InvalidArgument,
    };
    
    // For now just return a dummy balance
    *balance_out = 0;
    ErrorCode::Success
}

/// Free a string allocated by the FFI layer
/// 
/// # Safety
/// 
/// This function is unsafe because it takes ownership of a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn sebure_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Shutdown the SEBURE blockchain core
/// 
/// # Safety
/// 
/// This function is unsafe because it modifies global state.
#[no_mangle]
pub unsafe extern "C" fn sebure_shutdown() -> ErrorCode {
    // Close all components in reverse order of initialization
    
    // Close network
    {
        let mut network_lock = match NETWORK.lock() {
            Ok(lock) => lock,
            Err(_) => return ErrorCode::Unknown,
        };

        if let Some(network) = network_lock.as_mut() {
            if let Err(_) = network.stop() {
                return ErrorCode::NetworkError;
            }
        }
        *network_lock = None;
    }
    
    // Close storage
    {
        let mut storage_lock = match STORAGE.lock() {
            Ok(lock) => lock,
            Err(_) => return ErrorCode::Unknown,
        };

        // No need to explicitly close storage in this implementation
        *storage_lock = None;
    }
    
    ErrorCode::Success
}
