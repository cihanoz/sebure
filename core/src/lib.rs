//! # SEBURE Core Blockchain Library
//! 
//! This library implements the core components of the SEBURE Blockchain platform.
//! It provides the fundamental blockchain functionality, consensus mechanisms,
//! cryptographic utilities, networking, and storage interfaces.

// Module exports
pub mod blockchain;
pub mod consensus;
pub mod crypto;
pub mod network;
pub mod storage;
pub mod types;
pub mod utils;

// Re-exports for commonly used types
pub use blockchain::Block;
pub use blockchain::Transaction;
pub use blockchain::{Account, AccountType, GlobalState};
pub use crypto::Hash;
pub use types::Result;
pub use consensus::{Consensus, ConsensusConfig};
pub use storage::{Storage, StorageConfig};
pub use network::{Network, NetworkConfig};
pub use utils::{serialize, deserialize, to_json, from_json, SerializationFormat};

/// Library version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initializes the SEBURE Core library
/// 
/// This function sets up logging and other global state needed by the library.
pub fn init() -> types::Result<()> {
    // Initialize logger with env_logger
    env_logger::try_init().ok();
    
    log::info!("SEBURE Core v{} initialized", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
