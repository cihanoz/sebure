//! # Hash module
//!
//! Contains cryptographic hash functions used in the blockchain

use blake3;
use sha2::{Digest, Sha256};
use std::fmt;

/// Hash result type (32 bytes)
pub type Hash = [u8; 32];

/// SHA-256 hash function
pub fn sha256(data: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// BLAKE3 hash function (faster than SHA-256)
pub fn blake3_hash(data: &[u8]) -> Hash {
    let hash = blake3::hash(data);
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_bytes());
    result
}

/// Format a hash as a hexadecimal string
pub fn hash_to_hex(hash: &Hash) -> String {
    hex::encode(hash)
}

/// Parse a hexadecimal string into a hash
pub fn hex_to_hash(hex_str: &str) -> Result<Hash, hex::FromHexError> {
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 32 {
        return Err(hex::FromHexError::InvalidStringLength);
    }
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes);
    Ok(hash)
}

/// Merkle tree implementation
pub struct MerkleTree {
    /// Nodes in the tree
    nodes: Vec<Hash>,
    /// Number of leaf nodes
    leaf_count: usize,
}

impl MerkleTree {
    /// Create a new Merkle tree from a list of leaf values
    pub fn new(leaves: &[Hash]) -> Self {
        if leaves.is_empty() {
            return Self {
                nodes: vec![],
                leaf_count: 0,
            };
        }
        
        let mut leaf_count = leaves.len();
        // Round up to the next power of 2
        let mut size = leaf_count.next_power_of_two();
        let mut nodes = vec![[0u8; 32]; 2 * size - 1];
        
        // Copy leaf values
        for (i, leaf) in leaves.iter().enumerate() {
            nodes[size - 1 + i] = *leaf;
        }
        
        // Fill remaining leaves with the last leaf
        for i in leaf_count..size {
            nodes[size - 1 + i] = leaves[leaf_count - 1];
        }
        
        // Calculate internal nodes
        for i in (0..(size - 1)).rev() {
            let left = nodes[2 * i + 1];
            let right = nodes[2 * i + 2];
            
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(&left);
            combined.extend_from_slice(&right);
            
            nodes[i] = sha256(&combined);
        }
        
        Self { nodes, leaf_count }
    }
    
    /// Get the root hash of the tree
    pub fn root(&self) -> Option<Hash> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.nodes[0])
        }
    }
    
    /// Generate a proof for a leaf at the given index
    pub fn generate_proof(&self, index: usize) -> Vec<Hash> {
        if index >= self.leaf_count {
            return vec![];
        }
        
        let mut proof = Vec::new();
        let mut i = self.nodes.len() / 2 + index;
        
        while i > 0 {
            let sibling = if i % 2 == 0 { i - 1 } else { i + 1 };
            proof.push(self.nodes[sibling]);
            i = (i - 1) / 2;
        }
        
        proof
    }
    
    /// Verify a proof for a leaf value
    pub fn verify_proof(root: &Hash, leaf: &Hash, proof: &[Hash], index: usize) -> bool {
        let mut current = *leaf;
        let mut idx = index;
        
        for sibling in proof {
            let mut combined = Vec::with_capacity(64);
            
            if idx % 2 == 0 {
                // Current is right sibling
                combined.extend_from_slice(sibling);
                combined.extend_from_slice(&current);
            } else {
                // Current is left sibling
                combined.extend_from_slice(&current);
                combined.extend_from_slice(sibling);
            }
            
            current = sha256(&combined);
            idx /= 2;
        }
        
        &current == root
    }
}

/// Calculate the hash of a block header
pub fn hash_block_header(
    index: u64,
    timestamp: u64,
    previous_hash: &Hash,
    merkle_root: &Hash,
    validator_merkle: &Hash,
    nonce: u64,
    shard_identifiers: &[u16],
) -> Hash {
    let mut data = Vec::new();
    
    // Convert index to bytes and append
    data.extend_from_slice(&index.to_be_bytes());
    
    // Append timestamp
    data.extend_from_slice(&timestamp.to_be_bytes());
    
    // Append previous hash
    data.extend_from_slice(previous_hash);
    
    // Append merkle root
    data.extend_from_slice(merkle_root);
    
    // Append validator merkle root
    data.extend_from_slice(validator_merkle);
    
    // Append nonce
    data.extend_from_slice(&nonce.to_be_bytes());
    
    // Append shard identifiers
    for &shard_id in shard_identifiers {
        data.extend_from_slice(&shard_id.to_be_bytes());
    }
    
    sha256(&data)
}

/// Calculate the hash of a transaction
pub fn hash_transaction(
    version: u8,
    transaction_type: u8,
    sender_public_key: &[u8],
    sender_shard: u16,
    recipient_address: &[u8],
    recipient_shard: u16,
    amount: u64,
    fee: u32,
    gas_limit: u32,
    nonce: u64,
    timestamp: u64,
    data_type: u8,
    data_content: &[u8],
    dependencies: &[Vec<u8>],
) -> Hash {
    let mut data = Vec::new();
    
    // Append version
    data.push(version);
    
    // Append transaction type
    data.push(transaction_type);
    
    // Append sender public key
    data.extend_from_slice(sender_public_key);
    
    // Append sender shard
    data.extend_from_slice(&sender_shard.to_be_bytes());
    
    // Append recipient address
    data.extend_from_slice(recipient_address);
    
    // Append recipient shard
    data.extend_from_slice(&recipient_shard.to_be_bytes());
    
    // Append amount
    data.extend_from_slice(&amount.to_be_bytes());
    
    // Append fee
    data.extend_from_slice(&fee.to_be_bytes());
    
    // Append gas limit
    data.extend_from_slice(&gas_limit.to_be_bytes());
    
    // Append nonce
    data.extend_from_slice(&nonce.to_be_bytes());
    
    // Append timestamp
    data.extend_from_slice(&timestamp.to_be_bytes());
    
    // Append data type
    data.push(data_type);
    
    // Append data content
    data.extend_from_slice(data_content);
    
    // Append dependencies
    for dependency in dependencies {
        data.extend_from_slice(dependency);
    }
    
    sha256(&data)
}
