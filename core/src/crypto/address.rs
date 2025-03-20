//! # Blockchain Addresses
//! 
//! This module implements address derivation functions for the SEBURE blockchain.
//! Addresses are derived from public keys using cryptographic hashing.

use crate::crypto::{hash_data, HashAlgorithm};
use crate::types::{Result, Error};
use ripemd::Ripemd160;
use sha2::{Sha256, Digest};
use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use bs58;

/// Length of the address checksum in bytes
const CHECKSUM_LENGTH: usize = 4;

/// Length of the address payload (without checksum) in bytes
const ADDRESS_PAYLOAD_LENGTH: usize = 20;

/// Length of the full address binary data (payload + checksum)
const ADDRESS_BINARY_LENGTH: usize = ADDRESS_PAYLOAD_LENGTH + CHECKSUM_LENGTH;

/// Blockchain address derived from a public key
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    /// The binary representation of the address
    data: Vec<u8>,
}

impl Address {
    /// Create a new address from raw binary data
    pub fn new(data: Vec<u8>) -> Result<Self> {
        if data.len() != ADDRESS_BINARY_LENGTH {
            return Err(Error::Crypto(format!(
                "Invalid address length: expected {}, got {}",
                ADDRESS_BINARY_LENGTH, data.len()
            )));
        }
        
        // Verify checksum
        let payload = &data[0..ADDRESS_PAYLOAD_LENGTH];
        let checksum = &data[ADDRESS_PAYLOAD_LENGTH..];
        let calculated_checksum = calculate_checksum(payload);
        
        if checksum != calculated_checksum {
            return Err(Error::Crypto("Invalid address checksum".to_string()));
        }
        
        Ok(Address { data })
    }
    
    /// Get the address as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Get the payload part of the address (without checksum)
    pub fn payload(&self) -> &[u8] {
        &self.data[0..ADDRESS_PAYLOAD_LENGTH]
    }
    
    /// Get the checksum part of the address
    pub fn checksum(&self) -> &[u8] {
        &self.data[ADDRESS_PAYLOAD_LENGTH..]
    }
    
    /// Convert to Base58 encoded string
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.data).into_string()
    }
    
    /// Try to parse a Base58 encoded address
    pub fn from_base58(s: &str) -> Result<Self> {
        match bs58::decode(s).into_vec() {
            Ok(data) => Self::new(data),
            Err(e) => Err(Error::Crypto(format!("Failed to decode Base58 address: {}", e))),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address({})", self.to_base58())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl FromStr for Address {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self> {
        Self::from_base58(s)
    }
}

/// Calculate checksum for an address payload
fn calculate_checksum(payload: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let hash1 = hasher.finalize();
    
    let mut hasher = Sha256::new();
    hasher.update(hash1);
    let hash2 = hasher.finalize();
    
    hash2[0..CHECKSUM_LENGTH].to_vec()
}

/// Derive an address from a public key using double hashing (SHA-256 + RIPEMD-160)
pub fn derive_address(public_key: &[u8]) -> Result<Address> {
    // First apply SHA-256 to the public key
    let sha256_hash = hash_data(public_key, HashAlgorithm::Sha256);
    
    // Then apply RIPEMD-160 to the result
    let mut ripemd = Ripemd160::new();
    ripemd.update(sha256_hash.as_bytes());
    let payload = ripemd.finalize().to_vec();
    
    if payload.len() != ADDRESS_PAYLOAD_LENGTH {
        return Err(Error::Crypto(format!(
            "Unexpected RIPEMD-160 output length: expected {}, got {}",
            ADDRESS_PAYLOAD_LENGTH, payload.len()
        )));
    }
    
    // Calculate checksum (double SHA-256, first 4 bytes)
    let checksum = calculate_checksum(&payload);
    
    // Combine payload and checksum
    let mut data = payload;
    data.extend_from_slice(&checksum);
    
    Address::new(data)
}

/// Create a random address (mainly for testing)
pub fn random_address() -> Address {
    use rand::RngCore;
    
    let mut rng = rand::thread_rng();
    let mut payload = vec![0u8; ADDRESS_PAYLOAD_LENGTH];
    rng.fill_bytes(&mut payload);
    
    let checksum = calculate_checksum(&payload);
    
    let mut data = payload;
    data.extend_from_slice(&checksum);
    
    Address { data }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;
    
    #[test]
    fn test_address_from_public_key() {
        let keypair = KeyPair::generate();
        let public_key = keypair.public_key();
        
        let address = derive_address(&public_key).unwrap();
        
        // Address binary data should be 24 bytes (20 bytes payload + 4 bytes checksum)
        assert_eq!(address.as_bytes().len(), ADDRESS_BINARY_LENGTH);
        
        // Same public key should produce the same address
        let address2 = derive_address(&public_key).unwrap();
        assert_eq!(address, address2);
        
        // Different public key should produce different address
        let keypair2 = KeyPair::generate();
        let address3 = derive_address(&keypair2.public_key()).unwrap();
        assert_ne!(address, address3);
    }
    
    #[test]
    fn test_address_base58() {
        let keypair = KeyPair::generate();
        let address = derive_address(&keypair.public_key()).unwrap();
        
        // Convert to Base58 and back
        let base58 = address.to_base58();
        let address2 = Address::from_base58(&base58).unwrap();
        
        assert_eq!(address, address2);
    }
    
    #[test]
    fn test_address_from_str() {
        let keypair = KeyPair::generate();
        let address = derive_address(&keypair.public_key()).unwrap();
        let base58 = address.to_base58();
        
        // Parse from string
        let address2: Address = base58.parse().unwrap();
        assert_eq!(address, address2);
    }
    
    #[test]
    fn test_address_checksum() {
        let keypair = KeyPair::generate();
        let address = derive_address(&keypair.public_key()).unwrap();
        
        // Get the address as Base58
        let base58 = address.to_base58();
        
        // Modify a character in the middle
        let mut chars: Vec<char> = base58.chars().collect();
        if let Some(c) = chars.get_mut(base58.len() / 2) {
            *c = if *c == 'A' { 'B' } else { 'A' };
        }
        let invalid_base58: String = chars.into_iter().collect();
        
        // Parsing should fail due to invalid checksum
        assert!(Address::from_base58(&invalid_base58).is_err());
    }
    
    #[test]
    fn test_random_address() {
        let addr1 = random_address();
        let addr2 = random_address();
        
        // Two random addresses should be different
        assert_ne!(addr1, addr2);
        
        // Check Base58 conversion works for random addresses
        let base58 = addr1.to_base58();
        let addr3 = Address::from_base58(&base58).unwrap();
        assert_eq!(addr1, addr3);
    }
}
