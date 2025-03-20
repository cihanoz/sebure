use sebure_core::crypto::{
    KeyPair, Signature, Address, derive_address, sign, verify,
    sha256, blake3, HashAlgorithm, hash_data,
    seed_from_passphrase, KeyStore, KeyInfo
};
use sebure_core::types::Result;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_keypair_operations() {
    // Generate a random key pair
    let keypair = KeyPair::generate();
    
    // Test public key
    let public_key = keypair.public_key();
    assert_eq!(public_key.len(), 32);
    
    // Test private key
    let private_key = keypair.private_key();
    assert_eq!(private_key.len(), 32);
    
    // Test creating keypair from seed
    let keypair2 = KeyPair::from_seed(&private_key).unwrap();
    assert_eq!(keypair2.public_key(), public_key);
}

#[test]
fn test_signing_and_verification() {
    let keypair = KeyPair::generate();
    let message = b"Test message for signing";
    
    // Sign the message
    let signature = sign(&keypair, message);
    
    // Verify the signature
    let result = verify(&keypair.public_key(), message, &signature);
    assert!(result.is_ok());
    
    // Test failed verification with wrong message
    let wrong_message = b"Wrong message";
    let result = verify(&keypair.public_key(), wrong_message, &signature);
    assert!(result.is_err());
    
    // Test failed verification with wrong key
    let wrong_keypair = KeyPair::generate();
    let result = verify(&wrong_keypair.public_key(), message, &signature);
    assert!(result.is_err());
}

#[test]
fn test_address_derivation() {
    let keypair = KeyPair::generate();
    let public_key = keypair.public_key();
    
    // Derive address from public key
    let address = derive_address(&public_key).unwrap();
    
    // Address should be convertible to base58 and back
    let base58 = address.to_base58();
    let address2 = Address::from_base58(&base58).unwrap();
    assert_eq!(address, address2);
    
    // The same public key should always produce the same address
    let address3 = derive_address(&public_key).unwrap();
    assert_eq!(address, address3);
    
    // Different public keys should produce different addresses
    let keypair2 = KeyPair::generate();
    let address4 = derive_address(&keypair2.public_key()).unwrap();
    assert_ne!(address, address4);
}

#[test]
fn test_hashing() {
    let data = b"test data for hashing";
    
    // SHA-256 test
    let hash1 = sha256(data);
    let hash2 = hash_data(data, HashAlgorithm::Sha256);
    assert_eq!(hash1, hash2);
    
    // BLAKE3 test
    let hash3 = blake3(data);
    let hash4 = hash_data(data, HashAlgorithm::Blake3);
    assert_eq!(hash3, hash4);
    
    // Different data should produce different hashes
    let different_data = b"different data";
    let hash5 = sha256(different_data);
    assert_ne!(hash1, hash5);
}

#[test]
fn test_keystore_operations() -> Result<()> {
    // Create temporary directory
    let temp_dir = tempdir()?;
    let keystore_path = temp_dir.path();
    
    // Create keystore
    let mut keystore = KeyStore::new(keystore_path)?;
    assert_eq!(keystore.key_count(), 0);
    
    // Create a new key
    let password = "test_password";
    let key_info = keystore.create_key(Some("Test Key".to_string()), password)?;
    assert_eq!(keystore.key_count(), 1);
    
    // Test key loading
    let mut keystore2 = KeyStore::new(keystore_path)?;
    keystore2.load_all(password)?;
    assert_eq!(keystore2.key_count(), 1);
    
    let loaded_key = &keystore2.get_keys()[0];
    assert_eq!(loaded_key.address, key_info.address);
    assert_eq!(loaded_key.name.as_ref().unwrap(), "Test Key");
    
    // Test seed phrase generation and import
    let seed_phrase = KeyStore::generate_seed_phrase(12)?;
    let words: Vec<&str> = seed_phrase.split_whitespace().collect();
    assert_eq!(words.len(), 12);
    
    // Import from seed phrase
    let key_info2 = keystore.import_from_seed_phrase(&seed_phrase, Some("Seed Key".to_string()), password)?;
    assert_eq!(keystore.key_count(), 2);
    
    // Create the same key directly to verify
    let seed = seed_from_passphrase(&seed_phrase);
    let keypair = KeyPair::from_seed(&seed)?;
    let address = derive_address(&keypair.public_key())?;
    assert_eq!(key_info2.address, address);
    
    Ok(())
}
