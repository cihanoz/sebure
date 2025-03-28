//! # Cryptographic Utilities
//! 
//! This module implements cryptographic utilities for the SEBURE blockchain,
//! including hashing, signatures, key generation, address derivation, and secure key storage.

pub mod hash;
pub mod signature;
pub mod address;
pub mod keystore;
pub mod hdwallet;
pub mod bip39_wordlist;

// Re-export main types
pub use hash::Hash;
pub use hash::sha256;
pub use hash::blake3_hash as blake3;
pub use hash::hash_block_header as hash_block;
pub use hash::hash_transaction;
pub use signature::KeyPair;
pub use signature::Signature;
pub use signature::sign;
pub use signature::verify;
pub use address::Address;
pub use address::derive_address;
pub use keystore::KeyStore;
pub use keystore::KeyInfo;
pub use hdwallet::HDWallet;
pub use hdwallet::Mnemonic;
pub use hdwallet::MnemonicSize;
pub use hdwallet::DerivationPath;
pub use hdwallet::MultiSigWallet;
pub use hdwallet::MultiSigScheme;

/// Generates a secure random seed for cryptographic operations
pub fn generate_seed() -> Vec<u8> {
    use rand::RngCore;
    let mut seed = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);
    seed
}

/// Generates a deterministic seed from a passphrase for key derivation
pub fn seed_from_passphrase(passphrase: &str) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_seed() {
        let seed = generate_seed();
        assert_eq!(seed.len(), 32);
        
        // Generate another seed and ensure it's different
        let another_seed = generate_seed();
        assert_ne!(seed, another_seed);
    }
    
    #[test]
    fn test_seed_from_passphrase() {
        let seed1 = seed_from_passphrase("test passphrase");
        let seed2 = seed_from_passphrase("test passphrase");
        let seed3 = seed_from_passphrase("different passphrase");
        
        // Same passphrase should produce the same seed
        assert_eq!(seed1, seed2);
        
        // Different passphrases should produce different seeds
        assert_ne!(seed1, seed3);
    }
}
