//! # Cryptographic Hashing
//! 
//! This module implements cryptographic hash functions for the SEBURE blockchain.

use sha2::{Sha256, Digest};
use blake3::Hasher as Blake3Hasher;
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::types::{Result, Error};
use crate::blockchain::{Block, Transaction};

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

/// Hash a blockchain block
///
/// The block hash includes:
/// - Block header
/// - Transaction refs in shard data
/// - Cross-shard receipts
///
/// Note: Validator set is not included in the hash to allow for validator set updates
pub fn hash_block(block: &Block) -> Result<Vec<u8>> {
    // In a full implementation, we would compute a Merkle tree of all components
    // For now, just serialize and hash the block header and key data
    
    // Create a custom structure with only the parts we want to hash
    #[derive(Serialize)]
    struct BlockHashData<'a> {
        header: &'a crate::blockchain::BlockHeader,
        shard_data_tx_refs: Vec<Vec<Vec<u8>>>,  // Just the transaction refs from shard data
        cross_shard_receipts: &'a Vec<crate::blockchain::CrossShardReceipt>,
    }
    
    // Extract transaction refs from shard data
    let shard_data_tx_refs: Vec<Vec<Vec<u8>>> = block.shard_data
        .iter()
        .map(|sd| sd.transactions.clone())
        .collect();
    
    let hash_data = BlockHashData {
        header: &block.header,
        shard_data_tx_refs,
        cross_shard_receipts: &block.cross_shard_receipts,
    };
    
    // Use BLAKE3 for fast hashing of blocks
    match hash_serialize(&hash_data, HashAlgorithm::Blake3) {
        Ok(hash) => Ok(hash.0),
        Err(e) => Err(e),
    }
}

/// Hash a transaction
///
/// Creates a hash of the transaction that can be used as its unique identifier.
/// The signature is excluded from the hash calculation to avoid circular dependency,
/// as the signature is generated using the transaction hash.
pub fn hash_transaction(tx: &Transaction) -> Result<Vec<u8>> {
    // Create a custom structure with the transaction data minus the signature
    #[derive(Serialize)]
    struct TransactionHashData<'a> {
        version: u8,
        transaction_type: crate::types::TransactionType,
        sender_public_key: &'a Vec<u8>,
        sender_shard: crate::types::ShardId,
        recipient_address: &'a Vec<u8>,
        recipient_shard: crate::types::ShardId,
        amount: u64,
        fee: u32,
        gas_limit: u32,
        nonce: u64,
        timestamp: crate::types::Timestamp,
        data: &'a crate::blockchain::TransactionData,
        dependencies: &'a Vec<Vec<u8>>,
        execution_priority: crate::types::Priority,
    }
    
    let hash_data = TransactionHashData {
        version: tx.version,
        transaction_type: tx.transaction_type,
        sender_public_key: &tx.sender_public_key,
        sender_shard: tx.sender_shard,
        recipient_address: &tx.recipient_address,
        recipient_shard: tx.recipient_shard,
        amount: tx.amount,
        fee: tx.fee,
        gas_limit: tx.gas_limit,
        nonce: tx.nonce,
        timestamp: tx.timestamp,
        data: &tx.data,
        dependencies: &tx.dependencies,
        execution_priority: tx.execution_priority,
    };
    
    // Use SHA-256 for transaction hashing
    match hash_serialize(&hash_data, HashAlgorithm::Sha256) {
        Ok(hash) => Ok(hash.0),
        Err(e) => Err(e),
    }
}

/// Calculate the Merkle root of a list of hashes
///
/// This implements a simple Merkle tree to create a single root hash from
/// a list of transaction or other hashes.
pub fn calculate_merkle_root(hashes: &[Vec<u8>]) -> Result<Vec<u8>> {
    if hashes.is_empty() {
        // Empty Merkle tree has a zero hash
        return Ok(vec![0; 32]);
    }
    
    if hashes.len() == 1 {
        // Just one hash, it is the root
        return Ok(hashes[0].clone());
    }
    
    // Recursive implementation of Merkle tree construction
    let mut next_level = Vec::new();
    
    for chunk in hashes.chunks(2) {
        if chunk.len() == 2 {
            // Hash the pair
            let mut combined = Vec::with_capacity(chunk[0].len() + chunk[1].len());
            combined.extend_from_slice(&chunk[0]);
            combined.extend_from_slice(&chunk[1]);
            
            let hash = blake3(&combined).0;
            next_level.push(hash);
        } else {
            // Odd number of hashes, duplicate the last one
            let mut combined = Vec::with_capacity(chunk[0].len() * 2);
            combined.extend_from_slice(&chunk[0]);
            combined.extend_from_slice(&chunk[0]);
            
            let hash = blake3(&combined).0;
            next_level.push(hash);
        }
    }
    
    // Recursively calculate the next level
    calculate_merkle_root(&next_level)
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
    
    #[test]
    fn test_merkle_root() {
        // Test with empty list
        let root = calculate_merkle_root(&[]).unwrap();
        assert_eq!(root.len(), 32);
        assert!(root.iter().all(|&b| b == 0));
        
        // Test with one hash
        let hash1 = sha256(b"data1").0;
        let root = calculate_merkle_root(&[hash1.clone()]).unwrap();
        assert_eq!(root, hash1);
        
        // Test with two hashes
        let hash1 = sha256(b"data1").0;
        let hash2 = sha256(b"data2").0;
        let root = calculate_merkle_root(&[hash1.clone(), hash2.clone()]).unwrap();
        assert_ne!(root, hash1);
        assert_ne!(root, hash2);
        
        // Test with multiple hashes
        let hashes = vec![
            sha256(b"data1").0,
            sha256(b"data2").0,
            sha256(b"data3").0,
            sha256(b"data4").0,
        ];
        let root = calculate_merkle_root(&hashes).unwrap();
        assert_eq!(root.len(), 32);
    }
}
