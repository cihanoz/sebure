//! # Cryptographic Hashing
//! 
//! This module implements cryptographic hash functions for the SEBURE blockchain.

use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::types::{Result, Error};

/// A cryptographic hash value
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Hash(pub Vec<u8>);

impl Hash {
    /// Create a new hash from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Hash(bytes)
    }
    
    /// Create a zero-filled hash of the specified length
    pub fn zero(len: usize) -> Self {
        Hash(vec![0; len])
    }
    
    /// Check if this is a zero hash
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
    
    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    /// Convert to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
    
    /// Try to create a Hash from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        match hex::decode(hex_str) {
            Ok(bytes) => Ok(Hash(bytes)),
            Err(e) => Err(Error::Crypto(format!("Failed to decode hex: {}", e))),
        }
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({})", self.to_hex())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = self.to_hex();
        let len = hex.len();
        
        if len <= 12 {
            write!(f, "{}", hex)
        } else {
            write!(f, "{}...{}", &hex[0..6], &hex[len-6..len])
        }
    }
}

/// Hash algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256 algorithm
    Sha256,
    /// BLAKE3 algorithm
    Blake3,
}

/// Hash data using the specified algorithm
pub fn hash_data(data: &[u8], algorithm: HashAlgorithm) -> Hash {
    match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            Hash(hasher.finalize().to_vec())
        },
        HashAlgorithm::Blake3 => {
            let mut hasher = Blake3Hasher::new();
            hasher.update(data);
            Hash(hasher.finalize().as_bytes().to_vec())
        },
    }
}

/// Hash data using SHA-256
pub fn sha256(data: &[u8]) -> Hash {
    hash_data(data, HashAlgorithm::Sha256)
}

/// Hash data using BLAKE3
pub fn blake3(data: &[u8]) -> Hash {
    hash_data(data, HashAlgorithm::Blake3)
}

/// Hash data that can be serialized using bincode
pub fn hash_serialize<T: serde::Serialize>(value: &T, algorithm: HashAlgorithm) -> Result<Hash> {
    match bincode::serialize(value) {
        Ok(data) => Ok(hash_data(&data, algorithm)),
        Err(e) => Err(Error::Serialization(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sha256_hash() {
        let data = b"test data";
        let hash = sha256(data);
        
        // SHA-256 of "test data" should be consistent
        assert_eq!(hash.0.len(), 32); // SHA-256 is 32 bytes
        
        // Hash the same data again, should get the same result
        let hash2 = sha256(data);
        assert_eq!(hash, hash2);
        
        // Hash different data, should get a different result
        let hash3 = sha256(b"different data");
        assert_ne!(hash, hash3);
    }
    
    #[test]
    fn test_blake3_hash() {
        let data = b"test data";
        let hash = blake3(data);
        
        // BLAKE3 hash should be 32 bytes
        assert_eq!(hash.0.len(), 32);
        
        // Hash the same data again, should get the same result
        let hash2 = blake3(data);
        assert_eq!(hash, hash2);
        
        // Hash different data, should get a different result
        let hash3 = blake3(b"different data");
        assert_ne!(hash, hash3);
    }
    
    #[test]
    fn test_hash_hex() {
        let data = b"test data";
        let hash = sha256(data);
        
        // Convert to hex and back
        let hex = hash.to_hex();
        let hash2 = Hash::from_hex(&hex).unwrap();
        
        assert_eq!(hash, hash2);
    }
    
    #[test]
    fn test_hash_display() {
        let hash = Hash(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let display = format!("{}", hash);
        
        // Short hashes should be displayed in full
        assert_eq!(display, "0102030405060708090a0b0c");
        
        // Longer hashes should be truncated
        let long_hash = Hash(vec![0; 32]);
        let long_display = format!("{}", long_hash);
        
        assert!(long_display.contains("..."));
    }
    
    #[test]
    fn test_hash_serialize() {
        #[derive(Serialize)]
        struct TestStruct {
            field1: u32,
            field2: String,
        }
        
        let test_struct = TestStruct {
            field1: 42,
            field2: "test".to_string(),
        };
        
        let hash = hash_serialize(&test_struct, HashAlgorithm::Sha256).unwrap();
        assert_eq!(hash.0.len(), 32);
    }
}
