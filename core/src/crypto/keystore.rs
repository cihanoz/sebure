//! # Key Management and Storage
//! 
//! This module implements secure key management functionality for the SEBURE blockchain.
//! It provides utilities for generating, storing, and recovering cryptographic keys.

use crate::crypto::{KeyPair, Signature, Address, derive_address, seed_from_passphrase, HDWallet, Mnemonic, MnemonicSize};
use crate::types::{Result, Error};
use rand::RngCore;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt;
use serde::{Serialize, Deserialize};
use log::{debug, warn};

/// Encryption algorithm used for key storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionType {
    /// AES-256-GCM encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
}

/// Key derivation function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Number of iterations
    pub iterations: u32,
    /// Memory size cost parameter
    pub memory: u32,
    /// Salt for key derivation
    pub salt: Vec<u8>,
}

impl KdfParams {
    /// Create new KDF parameters with random salt
    pub fn new(iterations: u32, memory: u32) -> Self {
        let mut salt = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);
        
        Self {
            iterations,
            memory,
            salt,
        }
    }
    
    /// Create default KDF parameters
    pub fn default() -> Self {
        Self::new(10000, 65536)
    }
}

/// Encrypted key file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKeyFile {
    /// Version of the key file format
    pub version: u8,
    /// Public address corresponding to the key
    pub address: String,
    /// Encryption algorithm used
    pub crypto_type: EncryptionType,
    /// Key derivation function parameters
    pub kdf_params: KdfParams,
    /// Initialization vector
    pub iv: Vec<u8>,
    /// Encrypted private key data
    pub ciphertext: Vec<u8>,
    /// Authentication tag
    pub mac: Vec<u8>,
    /// Creation timestamp
    pub created_at: u64,
    /// Optional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Key information for display and usage
#[derive(Debug, Clone)]
pub struct KeyInfo {
    /// The key pair
    pub keypair: KeyPair,
    /// Derived blockchain address
    pub address: Address,
    /// Optional human-readable name
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
}

impl KeyInfo {
    /// Create a new KeyInfo from a keypair
    pub fn new(keypair: KeyPair, name: Option<String>) -> Result<Self> {
        let address = derive_address(&keypair.public_key())?;
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Ok(Self {
            keypair,
            address,
            name,
            created_at,
        })
    }
    
    /// Generate a new random key pair with info
    pub fn generate(name: Option<String>) -> Result<Self> {
        let keypair = KeyPair::generate();
        Self::new(keypair, name)
    }
    
    /// Create a key from a seed phrase
    pub fn from_seed_phrase(phrase: &str, name: Option<String>) -> Result<Self> {
        let seed = seed_from_passphrase(phrase);
        let keypair = KeyPair::from_seed(&seed)?;
        Self::new(keypair, name)
    }
    
    /// Sign a message with this key
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.keypair.sign(message)
    }
}

/// A keystore that manages multiple keys
pub struct KeyStore {
    /// Path to the keystore directory
    pub path: PathBuf,
    /// Currently loaded keys
    keys: Vec<KeyInfo>,
}

impl KeyStore {
    /// Create a new keystore at the specified path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        
        Ok(Self {
            path,
            keys: Vec::new(),
        })
    }
    
    /// Load all keys from the keystore directory
    pub fn load_all(&mut self, password: &str) -> Result<()> {
        self.keys.clear();
        
        for entry in fs::read_dir(&self.path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match self.load_key_from_file(&path, password) {
                    Ok(key_info) => {
                        self.keys.push(key_info);
                    },
                    Err(e) => {
                        warn!("Failed to load key from {:?}: {}", path, e);
                    },
                }
            }
        }
        
        debug!("Loaded {} keys from keystore", self.keys.len());
        Ok(())
    }
    
    /// Get the number of keys in the store
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
    
    /// Get all loaded keys
    pub fn get_keys(&self) -> &[KeyInfo] {
        &self.keys
    }
    
    /// Get a key by address
    pub fn get_key_by_address(&self, address: &Address) -> Option<&KeyInfo> {
        self.keys.iter().find(|info| &info.address == address)
    }
    
    /// Create a new random key and store it
    pub fn create_key(&mut self, name: Option<String>, password: &str) -> Result<KeyInfo> {
        let key_info = KeyInfo::generate(name)?;
        self.store_key(&key_info, password)?;
        self.keys.push(key_info.clone());
        Ok(key_info)
    }
    
    /// Import a key from seed phrase
    pub fn import_from_seed_phrase(&mut self, phrase: &str, name: Option<String>, password: &str) -> Result<KeyInfo> {
        let key_info = KeyInfo::from_seed_phrase(phrase, name)?;
        self.store_key(&key_info, password)?;
        self.keys.push(key_info.clone());
        Ok(key_info)
    }
    
    /// Store a key to file
    fn store_key(&self, key_info: &KeyInfo, password: &str) -> Result<PathBuf> {
        // Generate a filename based on address
        let address_str = key_info.address.to_base58();
        let filename = format!("UTC--{}--{}.json", 
            key_info.created_at,
            &address_str);
        let file_path = self.path.join(filename);
        
        // Create the encrypted key file
        let encrypted = self.encrypt_key(key_info, password)?;
        
        // Write to file
        let json = serde_json::to_string_pretty(&encrypted)?;
        let mut file = fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        
        debug!("Stored key for address {} to {:?}", address_str, file_path);
        Ok(file_path)
    }
    
    /// Load a key from file
    fn load_key_from_file<P: AsRef<Path>>(&self, path: P, password: &str) -> Result<KeyInfo> {
        // Read the file
        let mut file = fs::File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        
        // Parse the JSON
        let encrypted: EncryptedKeyFile = serde_json::from_str(&content)?;
        
        // Decrypt the key
        self.decrypt_key(&encrypted, password)
    }
    
    /// Encrypt a key for storage
    fn encrypt_key(&self, key_info: &KeyInfo, password: &str) -> Result<EncryptedKeyFile> {
        // For simplicity in this example, we'll use a mock encryption
        // In a real implementation, use a proper crypto library like 'ring' or 'crypto'
        
        // Generate random IV
        let mut iv = vec![0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);
        
        // Get the private key bytes
        let private_key = key_info.keypair.private_key();
        
        // In a real implementation, derive an encryption key from the password and KDF params
        // and encrypt the private key with AES-GCM or ChaCha20-Poly1305
        // For this example, we'll use a very simple XOR "encryption" (NOT SECURE!)
        let kdf_params = KdfParams::default();
        let mut ciphertext = Vec::with_capacity(private_key.len());
        let password_bytes = password.as_bytes();
        
        for (i, &byte) in private_key.iter().enumerate() {
            let password_byte = password_bytes[i % password_bytes.len()];
            ciphertext.push(byte ^ password_byte);
        }
        
        // In a real implementation, compute a proper MAC
        // Here we're just using a hash of the ciphertext + password
        let mut mac_input = ciphertext.clone();
        mac_input.extend_from_slice(password.as_bytes());
        let mac = crate::crypto::sha256(&mac_input).to_vec();
        
        Ok(EncryptedKeyFile {
            version: 1,
            address: key_info.address.to_base58(),
            crypto_type: EncryptionType::Aes256Gcm,
            kdf_params,
            iv,
            ciphertext,
            mac,
            created_at: key_info.created_at,
            metadata: key_info.name.as_ref().map_or_else(
                HashMap::new,
                |name| {
                    let mut map = HashMap::new();
                    map.insert("name".to_string(), name.clone());
                    map
                }
            ),
        })
    }
    
    /// Decrypt a key from storage
    fn decrypt_key(&self, encrypted: &EncryptedKeyFile, password: &str) -> Result<KeyInfo> {
        // In a real implementation, verify the MAC first
        // For our simple example, regenerate and check
        let mut mac_input = encrypted.ciphertext.clone();
        mac_input.extend_from_slice(password.as_bytes());
        let computed_mac = crate::crypto::sha256(&mac_input).to_vec();
        
        if computed_mac != encrypted.mac {
            return Err(Error::Crypto("Invalid password or corrupted key file".to_string()));
        }
        
        // Decrypt the private key (simple XOR "decryption" for the example)
        let password_bytes = password.as_bytes();
        let mut private_key = Vec::with_capacity(encrypted.ciphertext.len());
        
        for (i, &byte) in encrypted.ciphertext.iter().enumerate() {
            let password_byte = password_bytes[i % password_bytes.len()];
            private_key.push(byte ^ password_byte);
        }
        
        // Create the keypair from the private key
        let keypair = KeyPair::from_seed(&private_key)?;
        
        // Verify that the public key matches the expected address
        let address = derive_address(&keypair.public_key())?;
        let expected_address = Address::from_base58(&encrypted.address)?;
        
        if address != expected_address {
            return Err(Error::Crypto("Key validation failed: address mismatch".to_string()));
        }
        
        // Get name from metadata
        let name = encrypted.metadata.get("name").cloned();
        
        Ok(KeyInfo {
            keypair,
            address,
            name,
            created_at: encrypted.created_at,
        })
    }
    
    /// Create a BIP-39 mnemonic phrase for key backup
    pub fn generate_mnemonic(size: MnemonicSize) -> Result<Mnemonic> {
        Mnemonic::generate(size)
    }
    
    /// Create a new HD wallet and store the master key
    pub fn create_hd_wallet(&mut self, size: MnemonicSize, passphrase: Option<&str>, name: Option<String>, password: &str) -> Result<(HDWallet, KeyInfo)> {
        // Create a new HD wallet
        let wallet = HDWallet::new(size, passphrase)?;
        
        // Get the mnemonic for backup
        let mnemonic = wallet.mnemonic().unwrap();
        
        // Derive the first account key (m/44'/9999'/0'/0/0)
        let key_pair = wallet.derive_address_key(0, 0, 0)?;
        
        // Create key info
        let mut key_info = KeyInfo::new(key_pair, name.clone())?;
        
        // Add HD wallet metadata
        let mut metadata = HashMap::new();
        if let Some(name_str) = name {
            metadata.insert("name".to_string(), name_str);
        }
        metadata.insert("hd_wallet".to_string(), "true".to_string());
        metadata.insert("derivation_path".to_string(), "m/44'/9999'/0'/0/0".to_string());
        
        // Store the key
        self.store_key_with_metadata(&key_info, metadata, password)?;
        
        // Add to loaded keys
        self.keys.push(key_info.clone());
        
        Ok((wallet, key_info))
    }
    
    /// Import an HD wallet from a mnemonic phrase
    pub fn import_hd_wallet(&mut self, phrase: &str, passphrase: Option<&str>, name: Option<String>, password: &str) -> Result<(HDWallet, KeyInfo)> {
        // Create an HD wallet from the mnemonic
        let wallet = HDWallet::from_mnemonic(phrase, passphrase)?;
        
        // Derive the first account key (m/44'/9999'/0'/0/0)
        let key_pair = wallet.derive_address_key(0, 0, 0)?;
        
        // Create key info
        let mut key_info = KeyInfo::new(key_pair, name.clone())?;
        
        // Add HD wallet metadata
        let mut metadata = HashMap::new();
        if let Some(name_str) = name {
            metadata.insert("name".to_string(), name_str);
        }
        metadata.insert("hd_wallet".to_string(), "true".to_string());
        metadata.insert("derivation_path".to_string(), "m/44'/9999'/0'/0/0".to_string());
        
        // Store the key
        self.store_key_with_metadata(&key_info, metadata, password)?;
        
        // Add to loaded keys
        self.keys.push(key_info.clone());
        
        Ok((wallet, key_info))
    }
    
    /// Store a key with additional metadata
    fn store_key_with_metadata(&self, key_info: &KeyInfo, metadata: HashMap<String, String>, password: &str) -> Result<PathBuf> {
        // Generate a filename based on address
        let address_str = key_info.address.to_base58();
        let filename = format!("UTC--{}--{}.json", 
            key_info.created_at,
            &address_str);
        let file_path = self.path.join(filename);
        
        // Create the encrypted key file
        let mut encrypted = self.encrypt_key(key_info, password)?;
        
        // Add metadata
        for (key, value) in metadata {
            encrypted.metadata.insert(key, value);
        }
        
        // Write to file
        let json = serde_json::to_string_pretty(&encrypted)?;
        let mut file = fs::File::create(&file_path)?;
        file.write_all(json.as_bytes())?;
        
        debug!("Stored key for address {} to {:?}", address_str, file_path);
        Ok(file_path)
    }
}

impl fmt::Debug for KeyStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyStore")
            .field("path", &self.path)
            .field("key_count", &self.key_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_temp_keystore() -> Result<(KeyStore, tempfile::TempDir)> {
        let temp_dir = tempfile::tempdir()?;
        let keystore = KeyStore::new(temp_dir.path())?;
        Ok((keystore, temp_dir))
    }
    
    #[test]
    fn test_keystore_create_and_load() -> Result<()> {
        let (mut keystore, _temp_dir) = create_temp_keystore()?;
        
        // Create a new key
        let password = "test_password";
        let key_info = keystore.create_key(Some("Test Key".to_string()), password)?;
        
        // Verify it's in the store
        assert_eq!(keystore.key_count(), 1);
        
        // Create a new keystore and load keys
        let mut keystore2 = KeyStore::new(keystore.path.clone())?;
        keystore2.load_all(password)?;
        
        // Verify the key was loaded
        assert_eq!(keystore2.key_count(), 1);
        let loaded_key = &keystore2.get_keys()[0];
        assert_eq!(loaded_key.address, key_info.address);
        assert_eq!(loaded_key.name.as_ref().unwrap(), "Test Key");
        
        // Try with wrong password
        let mut keystore3 = KeyStore::new(keystore.path.clone())?;
        let result = keystore3.load_all("wrong_password");
        
        // The function should succeed but load 0 keys
        assert!(result.is_ok());
        assert_eq!(keystore3.key_count(), 0);
        
        Ok(())
    }
    
    #[test]
    fn test_mnemonic() -> Result<()> {
        let (mut keystore, _temp_dir) = create_temp_keystore()?;
        
        // Generate a mnemonic
        let mnemonic = KeyStore::generate_mnemonic(MnemonicSize::Words12)?;
        let phrase = mnemonic.to_phrase();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
        
        // Import HD wallet from the mnemonic
        let password = "test_password";
        let (wallet, key_info) = keystore.import_hd_wallet(&phrase, None, Some("HD Wallet".to_string()), password)?;
        
        // Verify the wallet has the mnemonic
        assert!(wallet.mnemonic().is_some());
        assert_eq!(wallet.mnemonic().unwrap().to_phrase(), phrase);
        
        // Verify the key was stored
        assert_eq!(keystore.key_count(), 1);
        
        // Create a new keystore and load keys
        let mut keystore2 = KeyStore::new(keystore.path.clone())?;
        keystore2.load_all(password)?;
        
        // Verify the key was loaded
        assert_eq!(keystore2.key_count(), 1);
        let loaded_key = &keystore2.get_keys()[0];
        assert_eq!(loaded_key.address, key_info.address);
        
        Ok(())
    }
    
    #[test]
    fn test_key_signing() -> Result<()> {
        let (mut keystore, _temp_dir) = create_temp_keystore()?;
        
        // Create a new key
        let password = "test_password";
        let key_info = keystore.create_key(None, password)?;
        
        // Sign a message
        let message = b"Test message to sign";
        let signature = key_info.sign(message);
        
        // Verify the signature
        let result = crate::crypto::verify(&key_info.keypair.public_key(), message, &signature);
        assert!(result.is_ok());
        
        Ok(())
    }
}
