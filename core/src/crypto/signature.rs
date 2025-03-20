//! # Digital Signatures
//! 
//! This module implements digital signature functionality using Ed25519.

use ed25519_dalek::{
    Keypair as Ed25519Keypair, PublicKey, SecretKey,
    Signature as Ed25519Signature, Signer, Verifier,
};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Serialize, Deserialize};
use std::fmt;
use crate::types::{Result, Error};

/// A digital signature
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(pub Vec<u8>);

impl Signature {
    /// Create a new signature from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Signature(bytes)
    }
    
    /// Get the underlying bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    /// Convert to a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
    
    /// Try to create a Signature from a hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        match hex::decode(hex_str) {
            Ok(bytes) => Ok(Signature(bytes)),
            Err(e) => Err(Error::Crypto(format!("Failed to decode hex: {}", e))),
        }
    }
}

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature({})", self.to_hex())
    }
}

impl fmt::Display for Signature {
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

/// A key pair for digital signatures
pub struct KeyPair {
    /// The Ed25519 key pair implementation
    inner: Ed25519Keypair,
}

impl Clone for KeyPair {
    fn clone(&self) -> Self {
        // We can't clone Ed25519Keypair directly, so we'll recreate it from the seed
        let private_key = self.private_key();
        KeyPair::from_seed(&private_key).expect("Failed to clone keypair")
    }
}

impl fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyPair")
            .field("public_key", &hex::encode(self.public_key()))
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        // Here we adapt for the compatibility issue between rand_core versions
        // Using a temporary array filled with random bytes from OsRng
        let mut seed = [0u8; 32];
        let mut csprng = OsRng;
        csprng.fill_bytes(&mut seed);
        
        // Use the seed to generate the keypair - should be compatible with ed25519_dalek
        Self::from_seed(&seed).expect("Failed to create keypair from random seed")
    }
    
    /// Create a key pair from a seed
    pub fn from_seed(seed: &[u8]) -> Result<Self> {
        if seed.len() != 32 {
            return Err(Error::Crypto("Seed must be 32 bytes".to_string()));
        }
        
        let secret = match SecretKey::from_bytes(seed) {
            Ok(secret) => secret,
            Err(e) => return Err(Error::Crypto(format!("Failed to create secret key: {}", e))),
        };
        
        let public = PublicKey::from(&secret);
        let keypair = Ed25519Keypair { secret, public };
        
        Ok(KeyPair { inner: keypair })
    }
    
    /// Get the public key
    pub fn public_key(&self) -> Vec<u8> {
        self.inner.public.to_bytes().to_vec()
    }
    
    /// Get the private key (secret key)
    pub fn private_key(&self) -> Vec<u8> {
        self.inner.secret.to_bytes().to_vec()
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        let signature = self.inner.sign(message);
        Signature(signature.to_bytes().to_vec())
    }
}

/// Sign a message with a key pair
pub fn sign(keypair: &KeyPair, message: &[u8]) -> Signature {
    keypair.sign(message)
}

/// Verify a signature against a public key and message
pub fn verify(public_key: &[u8], message: &[u8], signature: &Signature) -> Result<()> {
    let public = match PublicKey::from_bytes(public_key) {
        Ok(pk) => pk,
        Err(e) => return Err(Error::Crypto(format!("Invalid public key: {}", e))),
    };
    
    let sig = match Ed25519Signature::from_bytes(&signature.0) {
        Ok(s) => s,
        Err(e) => return Err(Error::Crypto(format!("Invalid signature: {}", e))),
    };
    
    match public.verify(message, &sig) {
        Ok(()) => Ok(()),
        Err(e) => Err(Error::Crypto(format!("Signature verification failed: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate();
        
        // Public key should be 32 bytes for Ed25519
        assert_eq!(keypair.public_key().len(), 32);
        
        // Private key should be 32 bytes for Ed25519
        assert_eq!(keypair.private_key().len(), 32);
    }
    
    #[test]
    fn test_keypair_from_seed() {
        let seed = [0u8; 32]; // All zeros seed
        
        // Create key pair from seed
        let keypair = KeyPair::from_seed(&seed).unwrap();
        
        // Public key should be 32 bytes
        assert_eq!(keypair.public_key().len(), 32);
        
        // Create another key pair from the same seed, should get the same keys
        let keypair2 = KeyPair::from_seed(&seed).unwrap();
        assert_eq!(keypair.public_key(), keypair2.public_key());
        assert_eq!(keypair.private_key(), keypair2.private_key());
        
        // Invalid seed length should fail
        assert!(KeyPair::from_seed(&[0u8; 16]).is_err());
    }
    
    #[test]
    fn test_signature() {
        let message = b"test message";
        let keypair = KeyPair::generate();
        
        // Sign the message
        let signature = keypair.sign(message);
        
        // Signature should be 64 bytes for Ed25519
        assert_eq!(signature.0.len(), 64);
        
        // Verify the signature
        let result = verify(&keypair.public_key(), message, &signature);
        assert!(result.is_ok());
        
        // Verify with wrong message should fail
        let wrong_message = b"wrong message";
        let result = verify(&keypair.public_key(), wrong_message, &signature);
        assert!(result.is_err());
        
        // Verify with wrong public key should fail
        let wrong_keypair = KeyPair::generate();
        let result = verify(&wrong_keypair.public_key(), message, &signature);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_signature_hex() {
        let message = b"test message";
        let keypair = KeyPair::generate();
        let signature = keypair.sign(message);
        
        // Convert to hex and back
        let hex = signature.to_hex();
        let signature2 = Signature::from_hex(&hex).unwrap();
        
        assert_eq!(signature, signature2);
    }
}
