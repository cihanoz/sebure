//! # Hierarchical Deterministic Wallet
//! 
//! This module implements BIP-32, BIP-39, and BIP-44 standards for hierarchical deterministic wallets.
//! It provides functionality for mnemonic generation, seed derivation, and hierarchical key derivation.

use crate::crypto::{KeyPair, Hash, sha256};
use crate::types::{Result, Error};
use hmac::{Hmac, Mac};
use sha2::{Sha512, Digest};
use rand::{RngCore, rngs::OsRng};
use std::fmt;
use serde::{Serialize, Deserialize};

// Type alias for HMAC-SHA512
type HmacSha512 = Hmac<Sha512>;

/// BIP-39 word list (English)
const BIP39_WORDLIST: &[&str] = &include!("bip39_wordlist.rs");

/// Mnemonic phrase entropy sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MnemonicSize {
    /// 12 words (128 bits of entropy)
    Words12,
    /// 15 words (160 bits of entropy)
    Words15,
    /// 18 words (192 bits of entropy)
    Words18,
    /// 21 words (224 bits of entropy)
    Words21,
    /// 24 words (256 bits of entropy)
    Words24,
}

impl MnemonicSize {
    /// Get the entropy size in bytes
    pub fn entropy_bytes(&self) -> usize {
        match self {
            MnemonicSize::Words12 => 16, // 128 bits
            MnemonicSize::Words15 => 20, // 160 bits
            MnemonicSize::Words18 => 24, // 192 bits
            MnemonicSize::Words21 => 28, // 224 bits
            MnemonicSize::Words24 => 32, // 256 bits
        }
    }
    
    /// Get the number of words
    pub fn word_count(&self) -> usize {
        match self {
            MnemonicSize::Words12 => 12,
            MnemonicSize::Words15 => 15,
            MnemonicSize::Words18 => 18,
            MnemonicSize::Words21 => 21,
            MnemonicSize::Words24 => 24,
        }
    }
}

impl Default for MnemonicSize {
    fn default() -> Self {
        MnemonicSize::Words24 // Default to highest security
    }
}

/// BIP-44 purpose constant
pub const BIP44_PURPOSE: u32 = 44;

/// SEBURE coin type (using a placeholder value, should be registered with SLIP-0044)
pub const SEBURE_COIN_TYPE: u32 = 9999;

/// Hardened derivation flag
pub const HARDENED: u32 = 0x80000000;

/// BIP-32 derivation path component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationComponent {
    /// Normal derivation
    Normal(u32),
    /// Hardened derivation
    Hardened(u32),
}

impl DerivationComponent {
    /// Convert to raw index
    pub fn to_raw_index(&self) -> u32 {
        match self {
            DerivationComponent::Normal(index) => *index,
            DerivationComponent::Hardened(index) => index | HARDENED,
        }
    }
    
    /// Create from raw index
    pub fn from_raw_index(index: u32) -> Self {
        if index & HARDENED != 0 {
            DerivationComponent::Hardened(index & !HARDENED)
        } else {
            DerivationComponent::Normal(index)
        }
    }
}

impl fmt::Display for DerivationComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DerivationComponent::Normal(index) => write!(f, "{}", index),
            DerivationComponent::Hardened(index) => write!(f, "{}'", index),
        }
    }
}

/// BIP-32 derivation path
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationPath {
    /// Path components
    pub components: Vec<DerivationComponent>,
}

impl DerivationPath {
    /// Create a new empty derivation path
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
    
    /// Create a derivation path from components
    pub fn from_components(components: Vec<DerivationComponent>) -> Self {
        Self { components }
    }
    
    /// Parse a derivation path string (e.g., "m/44'/9999'/0'/0/0")
    pub fn from_string(path: &str) -> Result<Self> {
        let mut components = Vec::new();
        
        // Skip the "m/" prefix if present
        let path = path.trim_start_matches("m/");
        
        for component in path.split('/') {
            if component.is_empty() {
                continue;
            }
            
            if component.ends_with('\'') || component.ends_with('h') {
                // Hardened component
                let index_str = component.trim_end_matches(|c| c == '\'' || c == 'h');
                let index = index_str.parse::<u32>()
                    .map_err(|_| Error::Crypto(format!("Invalid derivation path component: {}", component)))?;
                components.push(DerivationComponent::Hardened(index));
            } else {
                // Normal component
                let index = component.parse::<u32>()
                    .map_err(|_| Error::Crypto(format!("Invalid derivation path component: {}", component)))?;
                components.push(DerivationComponent::Normal(index));
            }
        }
        
        Ok(Self { components })
    }
    
    /// Create a standard BIP-44 derivation path
    pub fn bip44_path(account: u32, change: u32, address_index: u32) -> Self {
        Self {
            components: vec![
                DerivationComponent::Hardened(BIP44_PURPOSE),
                DerivationComponent::Hardened(SEBURE_COIN_TYPE),
                DerivationComponent::Hardened(account),
                DerivationComponent::Normal(change),
                DerivationComponent::Normal(address_index),
            ],
        }
    }
    
    /// Add a component to the path
    pub fn push(&mut self, component: DerivationComponent) {
        self.components.push(component);
    }
    
    /// Get the parent path
    pub fn parent(&self) -> Option<Self> {
        if self.components.is_empty() {
            None
        } else {
            let mut parent = self.clone();
            parent.components.pop();
            Some(parent)
        }
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m")?;
        for component in &self.components {
            write!(f, "/{}", component)?;
        }
        Ok(())
    }
}

impl Default for DerivationPath {
    fn default() -> Self {
        // Default to first account, external chain, first address
        Self::bip44_path(0, 0, 0)
    }
}

/// Extended key for HD wallet
#[derive(Clone)]
pub struct ExtendedKey {
    /// Key pair (public and private keys)
    pub key_pair: KeyPair,
    /// Chain code for derivation
    pub chain_code: [u8; 32],
    /// Depth in the derivation path
    pub depth: u8,
    /// Parent fingerprint
    pub parent_fingerprint: [u8; 4],
    /// Child number
    pub child_number: u32,
}

impl ExtendedKey {
    /// Create a new master extended key from seed
    pub fn from_seed(seed: &[u8]) -> Result<Self> {
        // HMAC-SHA512 with key "Bitcoin seed" (BIP-32 spec)
        let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed")
            .map_err(|e| Error::Crypto(format!("HMAC error: {}", e)))?;
        mac.update(seed);
        let result = mac.finalize().into_bytes();
        
        // Split the result into key and chain code
        let mut key = [0u8; 32];
        let mut chain_code = [0u8; 32];
        key.copy_from_slice(&result[0..32]);
        chain_code.copy_from_slice(&result[32..64]);
        
        // Create key pair from the derived key
        let key_pair = KeyPair::from_seed(&key)?;
        
        Ok(Self {
            key_pair,
            chain_code,
            depth: 0,
            parent_fingerprint: [0u8; 4],
            child_number: 0,
        })
    }
    
    /// Derive a child key
    pub fn derive_child(&self, index: u32) -> Result<Self> {
        let hardened = index >= HARDENED;
        
        let mut data = Vec::with_capacity(37);
        
        if hardened {
            // Hardened derivation: 0x00 || private_key || index
            data.push(0);
            data.extend_from_slice(&self.key_pair.private_key());
        } else {
            // Normal derivation: public_key || index
            data.extend_from_slice(&self.key_pair.public_key());
        }
        
        // Append the index in big-endian
        data.extend_from_slice(&index.to_be_bytes());
        
        // HMAC-SHA512 with chain code as key
        let mut mac = HmacSha512::new_from_slice(&self.chain_code)
            .map_err(|e| Error::Crypto(format!("HMAC error: {}", e)))?;
        mac.update(&data);
        let result = mac.finalize().into_bytes();
        
        // Split the result into key and chain code
        let mut key_mod = [0u8; 32];
        let mut chain_code = [0u8; 32];
        key_mod.copy_from_slice(&result[0..32]);
        chain_code.copy_from_slice(&result[32..64]);
        
        // Derive the child private key: parent_key + key_mod (mod n)
        let parent_key = self.key_pair.private_key();
        
        // Simple scalar addition (in a real implementation, this would use proper EC math)
        // This is a simplified version for demonstration
        let mut child_key = [0u8; 32];
        for i in 0..32 {
            child_key[i] = parent_key[i].wrapping_add(key_mod[i]);
        }
        
        // Create key pair from the derived key
        let key_pair = KeyPair::from_seed(&child_key)?;
        
        // Calculate parent fingerprint
        let parent_pub_key = self.key_pair.public_key();
        let parent_hash = sha256(&parent_pub_key);
        let mut fingerprint = [0u8; 4];
        fingerprint.copy_from_slice(&parent_hash[0..4]);
        
        Ok(Self {
            key_pair,
            chain_code,
            depth: self.depth + 1,
            parent_fingerprint: fingerprint,
            child_number: index,
        })
    }
    
    /// Derive a path from this key
    pub fn derive_path(&self, path: &DerivationPath) -> Result<Self> {
        let mut key = self.clone();
        
        for component in &path.components {
            key = key.derive_child(component.to_raw_index())?;
        }
        
        Ok(key)
    }
}

impl fmt::Debug for ExtendedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtendedKey")
            .field("depth", &self.depth)
            .field("parent_fingerprint", &hex::encode(self.parent_fingerprint))
            .field("child_number", &self.child_number)
            .field("chain_code", &hex::encode(self.chain_code))
            .field("public_key", &hex::encode(self.key_pair.public_key()))
            .field("private_key", &"[REDACTED]")
            .finish()
    }
}

/// Mnemonic phrase for wallet backup and recovery
#[derive(Clone, PartialEq, Eq)]
pub struct Mnemonic {
    /// The mnemonic phrase words
    words: Vec<String>,
}

impl Mnemonic {
    /// Generate a new random mnemonic
    pub fn generate(size: MnemonicSize) -> Result<Self> {
        let entropy_bytes = size.entropy_bytes();
        let mut entropy = vec![0u8; entropy_bytes];
        OsRng.fill_bytes(&mut entropy);
        
        Self::from_entropy(&entropy)
    }
    
    /// Create a mnemonic from entropy
    pub fn from_entropy(entropy: &[u8]) -> Result<Self> {
        // Validate entropy length
        let entropy_bits = entropy.len() * 8;
        if entropy_bits < 128 || entropy_bits > 256 || entropy_bits % 32 != 0 {
            return Err(Error::Crypto("Invalid entropy length".to_string()));
        }
        
        // Calculate checksum bits: ENT / 32
        let checksum_bits = entropy_bits / 32;
        
        // Calculate SHA-256 hash of entropy
        let hash = sha256(entropy);
        
        // Take the first checksum_bits of the hash
        let checksum = hash[0] >> (8 - checksum_bits);
        
        // Combine entropy and checksum
        let mut combined = Vec::with_capacity(entropy.len() + 1);
        combined.extend_from_slice(entropy);
        combined.push(checksum);
        
        // Convert to indices (11 bits per word)
        let total_bits = entropy_bits + checksum_bits;
        let word_count = total_bits / 11;
        
        let mut words = Vec::with_capacity(word_count);
        
        for i in 0..word_count {
            let start_bit = i * 11;
            let start_byte = start_bit / 8;
            let end_byte = (start_bit + 11 - 1) / 8;
            let start_bit_in_byte = start_bit % 8;
            
            let mut index = 0;
            
            if end_byte >= combined.len() {
                // Handle the case where we need bits from a byte that doesn't exist
                // This can happen for the last word when checksum bits are involved
                let bits_from_last_byte = 8 - start_bit_in_byte;
                index = (combined[start_byte] & ((1 << bits_from_last_byte) - 1)) as u16;
                index <<= (11 - bits_from_last_byte);
            } else if start_byte == end_byte {
                // All 11 bits are in the same byte
                index = ((combined[start_byte] >> (8 - start_bit_in_byte - 11)) & 0x7FF) as u16;
            } else {
                // Bits are split across two bytes
                let bits_from_first_byte = 8 - start_bit_in_byte;
                let bits_from_second_byte = 11 - bits_from_first_byte;
                
                index = (combined[start_byte] & ((1 << bits_from_first_byte) - 1)) as u16;
                index <<= bits_from_second_byte;
                index |= (combined[end_byte] >> (8 - bits_from_second_byte)) as u16;
            }
            
            if index as usize >= BIP39_WORDLIST.len() {
                return Err(Error::Crypto(format!("Invalid word index: {}", index)));
            }
            
            words.push(BIP39_WORDLIST[index as usize].to_string());
        }
        
        Ok(Self { words })
    }
    
    /// Create a mnemonic from a phrase
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let words: Vec<String> = phrase.split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        
        // Validate word count
        let word_count = words.len();
        if word_count != 12 && word_count != 15 && word_count != 18 && word_count != 21 && word_count != 24 {
            return Err(Error::Crypto(format!("Invalid word count: {}", word_count)));
        }
        
        // Validate each word is in the wordlist
        for word in &words {
            if !BIP39_WORDLIST.contains(&word.as_str()) {
                return Err(Error::Crypto(format!("Invalid word: {}", word)));
            }
        }
        
        // Convert words to indices
        let mut indices = Vec::with_capacity(word_count);
        for word in &words {
            let index = BIP39_WORDLIST.iter()
                .position(|&w| w == word)
                .ok_or_else(|| Error::Crypto(format!("Word not found in wordlist: {}", word)))?;
            indices.push(index as u16);
        }
        
        // Calculate entropy bits: (word_count * 11) - (word_count * 11 / 33)
        let entropy_bits = word_count * 11 - word_count * 11 / 33;
        let entropy_bytes = entropy_bits / 8;
        
        // Convert indices to entropy
        let mut entropy = vec![0u8; entropy_bytes];
        
        for (i, &index) in indices.iter().enumerate() {
            let start_bit = i * 11;
            let start_byte = start_bit / 8;
            let start_bit_in_byte = start_bit % 8;
            
            if start_byte < entropy.len() {
                // Add the first part of the index to the current byte
                let bits_in_first_byte = std::cmp::min(8 - start_bit_in_byte, 11);
                let mask = (1 << bits_in_first_byte) - 1;
                let value = (index >> (11 - bits_in_first_byte)) & mask;
                entropy[start_byte] |= (value as u8) << (8 - start_bit_in_byte - bits_in_first_byte);
                
                // If there are remaining bits, add them to the next byte
                if bits_in_first_byte < 11 && start_byte + 1 < entropy.len() {
                    let bits_in_second_byte = 11 - bits_in_first_byte;
                    let mask = (1 << bits_in_second_byte) - 1;
                    let value = index & mask;
                    entropy[start_byte + 1] |= (value as u8) << (8 - bits_in_second_byte);
                }
            }
        }
        
        // Verify checksum
        let checksum_bits = entropy_bits / 32;
        let hash = sha256(&entropy);
        let calculated_checksum = hash[0] >> (8 - checksum_bits);
        
        // Extract checksum from the last word
        let last_word_index = indices[word_count - 1];
        let checksum_mask = (1 << checksum_bits) - 1;
        let extracted_checksum = (last_word_index & checksum_mask) as u8;
        
        if calculated_checksum != extracted_checksum {
            return Err(Error::Crypto("Invalid mnemonic checksum".to_string()));
        }
        
        Ok(Self { words })
    }
    
    /// Convert to a phrase string
    pub fn to_phrase(&self) -> String {
        self.words.join(" ")
    }
    
    /// Derive a seed from the mnemonic with an optional passphrase
    pub fn to_seed(&self, passphrase: Option<&str>) -> Vec<u8> {
        let mnemonic = self.to_phrase();
        let salt = format!("mnemonic{}", passphrase.unwrap_or(""));
        
        // PBKDF2 with HMAC-SHA512, 2048 iterations
        let mut seed = [0u8; 64];
        pbkdf2::pbkdf2::<Hmac<Sha512>>(
            mnemonic.as_bytes(),
            salt.as_bytes(),
            2048,
            &mut seed,
        );
        
        seed.to_vec()
    }
    
    /// Get the number of words
    pub fn word_count(&self) -> usize {
        self.words.len()
    }
    
    /// Get the mnemonic size
    pub fn size(&self) -> Result<MnemonicSize> {
        match self.word_count() {
            12 => Ok(MnemonicSize::Words12),
            15 => Ok(MnemonicSize::Words15),
            18 => Ok(MnemonicSize::Words18),
            21 => Ok(MnemonicSize::Words21),
            24 => Ok(MnemonicSize::Words24),
            _ => Err(Error::Crypto(format!("Invalid word count: {}", self.word_count()))),
        }
    }
}

impl fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mnemonic({} words)", self.words.len())
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show only the first and last words for security
        if self.words.len() >= 2 {
            write!(f, "{}...{} ({} words)", 
                self.words.first().unwrap(),
                self.words.last().unwrap(),
                self.words.len()
            )
        } else {
            write!(f, "[REDACTED] ({} words)", self.words.len())
        }
    }
}

/// HD wallet implementation
pub struct HDWallet {
    /// Master extended key
    master_key: ExtendedKey,
    /// Mnemonic for backup
    mnemonic: Option<Mnemonic>,
}

impl HDWallet {
    /// Create a new random HD wallet
    pub fn new(size: MnemonicSize, passphrase: Option<&str>) -> Result<Self> {
        let mnemonic = Mnemonic::generate(size)?;
        let seed = mnemonic.to_seed(passphrase);
        let master_key = ExtendedKey::from_seed(&seed)?;
        
        Ok(Self {
            master_key,
            mnemonic: Some(mnemonic),
        })
    }
    
    /// Create an HD wallet from a mnemonic phrase
    pub fn from_mnemonic(phrase: &str, passphrase: Option<&str>) -> Result<Self> {
        let mnemonic = Mnemonic::from_phrase(phrase)?;
        let seed = mnemonic.to_seed(passphrase);
        let master_key = ExtendedKey::from_seed(&seed)?;
        
        Ok(Self {
            master_key,
            mnemonic: Some(mnemonic),
        })
    }
    
    /// Create an HD wallet from a seed
    pub fn from_seed(seed: &[u8]) -> Result<Self> {
        let master_key = ExtendedKey::from_seed(seed)?;
        
        Ok(Self {
            master_key,
            mnemonic: None,
        })
    }
    
    /// Get the mnemonic phrase
    pub fn mnemonic(&self) -> Option<&Mnemonic> {
        self.mnemonic.as_ref()
    }
    
    /// Get the master extended key
    pub fn master_key(&self) -> &ExtendedKey {
        &self.master_key
    }
    
    /// Derive a key at a specific path
    pub fn derive_key(&self, path: &DerivationPath) -> Result<ExtendedKey> {
        self.master_key.derive_path(path)
    }
    
    /// Derive a key pair at a specific path
    pub fn derive_key_pair(&self, path: &DerivationPath) -> Result<KeyPair> {
        let extended_key = self.derive_key(path)?;
        Ok(extended_key.key_pair.clone())
    }
    
    /// Derive an account key (BIP-44)
    pub fn derive_account(&self, account: u32) -> Result<ExtendedKey> {
        let path = DerivationPath::from_components(vec![
            DerivationComponent::Hardened(BIP44_PURPOSE),
            DerivationComponent::Hardened(SEBURE_COIN_TYPE),
            DerivationComponent::Hardened(account),
        ]);
        
        self.derive_key(&path)
    }
    
    /// Derive an address key (BIP-44)
    pub fn derive_address_key(&self, account: u32, change: u32, address_index: u32) -> Result<KeyPair> {
        let path = DerivationPath::bip44_path(account, change, address_index);
        self.derive_key_pair(&path)
    }
}

impl fmt::Debug for HDWallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HDWallet")
            .field("mnemonic", &self.mnemonic)
            .field("master_key", &self.master_key)
            .finish()
    }
}

/// Multi-signature scheme
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiSigScheme {
    /// M-of-N multi-signature
    MOfN {
        /// Required signatures
        required: u8,
        /// Total participants
        total: u8,
    },
}

impl MultiSigScheme {
    /// Create a new M-of-N multi-signature scheme
    pub fn m_of_n(required: u8, total: u8) -> Result<Self> {
        if required == 0 || required > total {
            return Err(Error::Crypto(format!(
                "Invalid multi-signature scheme: {}/{}", required, total
            )));
        }
        
        Ok(MultiSigScheme::MOfN { required, total })
    }
    
    /// Get the required signatures
    pub fn required_signatures(&self) -> u8 {
        match self {
            MultiSigScheme::MOfN { required, .. } => *required,
        }
    }
    
    /// Get the total participants
    pub fn total_participants(&self) -> u8 {
        match self {
            MultiSigScheme::MOfN { total, .. } => *total,
        }
    }
}

impl fmt::Display for MultiSigScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultiSigScheme::MOfN { required, total } => write!(f, "{}-of-{}", required, total),
        }
    }
}

/// Multi-signature wallet
#[derive(Debug, Clone)]
pub struct MultiSigWallet {
    /// Multi-signature scheme
    pub scheme: MultiSigScheme,
    /// Public keys of participants
    pub public_keys: Vec<Vec<u8>>,
}

impl MultiSigWallet {
    /// Create a new multi-signature wallet
    pub fn new(scheme: MultiSigScheme, public_keys: Vec<Vec<u8>>) -> Result<Self> {
        let total = scheme.total_participants() as usize;
        
        if public_keys.len() != total {
            return Err(Error::Crypto(format!(
                "Expected {} public keys, got {}", total, public_keys.len()
            )));
        }
        
        Ok(Self { scheme, public_keys })
    }
    
    /// Create a multi-signature address
    pub fn create_address(&self) -> Result<crate::crypto::Address> {
        // Create a redeem script: <required> <pubkey1> <pubkey2> ... <pubkeyN> <total> CHECKMULTISIG
        let mut script = Vec::new();
        
        // Add required signatures
        script.push(self.scheme.required_signatures());
        
        // Add public keys
        for pubkey in &self.public_keys {
            script.extend_from_slice(pubkey);
        }
        
        // Add total participants
        script.push(self.scheme.total_participants());
        
        // Add CHECKMULTISIG opcode (placeholder)
        script.push(0xAE); // CHECKMULTISIG opcode in Bitcoin
        
        // Hash the script to create the address
        crate::crypto::derive_address(&script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mnemonic_generation() -> Result<()> {
        let mnemonic = Mnemonic::generate(MnemonicSize::Words12)?;
        assert_eq!(mnemonic.word_count(), 12);
        
        let phrase = mnemonic.to_phrase();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
        
        // Each word should be in the wordlist
        for word in words {
            assert!(BIP39_WORDLIST.contains(&word));
        }
        
        Ok(())
    }
    
    #[test]
    fn test_mnemonic_from_phrase() -> Result<()> {
        let phrase = "abandon ability able about above absent absorb abstract absurd abuse access accident";
        let mnemonic = Mnemonic::from_phrase(phrase)?;
        
        assert_eq!(mnemonic.word_count(), 12);
        assert_eq!(mnemonic.to_phrase(), phrase);
        
        Ok(())
    }
    
    #[test]
    fn test_mnemonic_to_seed() -> Result<()> {
        let phrase = "abandon ability able about above absent absorb abstract absurd abuse access accident";
        let mnemonic = Mnemonic::from_phrase(phrase)?;
        
        // Without passphrase
        let seed1 = mnemonic.to_seed(None);
        assert_eq!(seed1.len(), 64);
        
        // With passphrase
        let seed2 = mnemonic.to_seed(Some("test passphrase"));
        assert_eq!(seed2.len(), 64);
        
        // Different passphrases should produce different seeds
        assert_ne!(seed1, seed2);
        
        Ok(())
    }
    
    #[test]
    fn test_derivation_path() -> Result<()> {
        let path_str = "m/44'/9999'/0'/0/0";
        let path = DerivationPath::from_string(path_str)?;
        
        assert_eq!(path.components.len(), 5);
        assert_eq!(path.to_string(), path_str);
        
        // Test BIP-44 path creation
        let bip44_path = DerivationPath::bip44_path(0, 0, 0);
        assert_eq!(bip44_path.to_string(), path_str);
        
        Ok(())
    }
    
    #[test]
    fn test_hd_wallet() -> Result<()> {
        // Create a wallet with a random mnemonic
        let wallet = HDWallet::new(MnemonicSize::Words12, None)?;
        
        // Derive keys at different paths
        let account0 = wallet.derive_account(0)?;
        assert_eq!(account0.depth, 3);
        
        let key1 = wallet.derive_address_key(0, 0, 0)?;
        let key2 = wallet.derive_address_key(0, 0, 1)?;
        
        // Different paths should produce different keys
        assert_ne!(key1.public_key(), key2.public_key());
        
        // Create a wallet from the mnemonic
        let mnemonic = wallet.mnemonic().unwrap();
        let phrase = mnemonic.to_phrase();
        let wallet2 = HDWallet::from_mnemonic(&phrase, None)?;
        
        // Derive the same key from both wallets
        let key1_again = wallet2.derive_address_key(0, 0, 0)?;
        
        // Keys should be the same
        assert_eq!(key1.public_key(), key1_again.public_key());
        
        Ok(())
    }
    
    #[test]
    fn test_multi_sig() -> Result<()> {
        // Create a 2-of-3 multi-signature scheme
        let scheme = MultiSigScheme::m_of_n(2, 3)?;
        
        // Generate 3 key pairs
        let key1 = KeyPair::generate();
        let key2 = KeyPair::generate();
        let key3 = KeyPair::generate();
        
        let public_keys = vec![
            key1.public_key(),
            key2.public_key(),
            key3.public_key(),
        ];
        
        // Create a multi-signature wallet
        let wallet = MultiSigWallet::new(scheme, public_keys)?;
        
        // Create an address
        let address = wallet.create_address()?;
        
        // Address should be valid
        assert_eq!(address.as_bytes().len(), 24); // 20 bytes payload + 4 bytes checksum
        
        Ok(())
    }
