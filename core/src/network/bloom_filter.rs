//! # Bloom Filter
//! 
//! This module implements a Bloom filter for efficient transaction propagation
//! in the P2P network, reducing bandwidth usage by filtering known transactions.

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::marker::PhantomData;

/// BloomFilter provides a space-efficient probabilistic data structure
/// to test whether an element is a member of a set.
#[derive(Debug, Clone)]
pub struct BloomFilter<T> {
    /// Bit array
    bits: Vec<bool>,
    
    /// Number of hash functions
    k: usize,
    
    /// Number of elements added
    count: usize,
    
    /// Phantom data for type parameter
    _marker: PhantomData<T>,
}

impl<T: Hash> BloomFilter<T> {
    /// Create a new Bloom filter with the given size and number of hash functions
    pub fn new(size: usize, k: usize) -> Self {
        BloomFilter {
            bits: vec![false; size],
            k,
            count: 0,
            _marker: PhantomData,
        }
    }
    
    /// Create a new Bloom filter with optimal parameters for the expected number of elements
    /// and desired false positive probability
    pub fn with_params(expected_elements: usize, false_positive_probability: f64) -> Self {
        // Calculate optimal size
        let size = Self::optimal_size(expected_elements, false_positive_probability);
        
        // Calculate optimal number of hash functions
        let k = Self::optimal_k(size, expected_elements);
        
        BloomFilter::new(size, k)
    }
    
    /// Calculate the optimal size for the given parameters
    fn optimal_size(expected_elements: usize, false_positive_probability: f64) -> usize {
        let size = -((expected_elements as f64) * false_positive_probability.ln()) / (2.0_f64.ln().powi(2));
        size.ceil() as usize
    }
    
    /// Calculate the optimal number of hash functions for the given parameters
    fn optimal_k(size: usize, expected_elements: usize) -> usize {
        let k = (size as f64 / expected_elements as f64) * 2.0_f64.ln();
        k.ceil() as usize
    }
    
    /// Add an element to the filter
    pub fn insert(&mut self, item: &T) {
        for i in 0..self.k {
            let index = self.get_index(item, i);
            self.bits[index] = true;
        }
        self.count += 1;
    }
    
    /// Check if an element might be in the filter
    pub fn contains(&self, item: &T) -> bool {
        for i in 0..self.k {
            let index = self.get_index(item, i);
            if !self.bits[index] {
                return false;
            }
        }
        true
    }
    
    /// Get the index for the given item and hash function
    fn get_index(&self, item: &T, hash_index: usize) -> usize {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        
        // Add the hash index to get different hash functions
        (hasher.finish() as usize + hash_index) % self.bits.len()
    }
    
    /// Get the number of elements added to the filter
    pub fn count(&self) -> usize {
        self.count
    }
    
    /// Get the size of the filter in bits
    pub fn size(&self) -> usize {
        self.bits.len()
    }
    
    /// Get the number of hash functions
    pub fn hash_functions(&self) -> usize {
        self.k
    }
    
    /// Get the estimated false positive probability
    pub fn false_positive_probability(&self) -> f64 {
        let n = self.count as f64;
        let m = self.bits.len() as f64;
        let k = self.k as f64;
        
        (1.0 - (1.0 - 1.0/m).powf(k * n)).powf(k)
    }
    
    /// Clear the filter
    pub fn clear(&mut self) {
        for bit in &mut self.bits {
            *bit = false;
        }
        self.count = 0;
    }
    
    /// Merge another Bloom filter into this one
    pub fn merge(&mut self, other: &BloomFilter<T>) -> Result<(), &'static str> {
        if self.bits.len() != other.bits.len() || self.k != other.k {
            return Err("Bloom filters must have the same size and number of hash functions");
        }
        
        for i in 0..self.bits.len() {
            self.bits[i] |= other.bits[i];
        }
        
        self.count += other.count;
        
        Ok(())
    }
    
    /// Create a serialized representation of the filter
    pub fn serialize(&self) -> Vec<u8> {
        // Calculate the number of bytes needed
        let bytes_needed = (self.bits.len() + 7) / 8;
        let mut result = Vec::with_capacity(bytes_needed);
        
        // Pack bits into bytes
        for i in 0..(bytes_needed) {
            let mut byte = 0u8;
            for j in 0..8 {
                let bit_index = i * 8 + j;
                if bit_index < self.bits.len() && self.bits[bit_index] {
                    byte |= 1 << j;
                }
            }
            result.push(byte);
        }
        
        result
    }
    
    /// Create a Bloom filter from a serialized representation
    pub fn deserialize(data: &[u8], size: usize, k: usize) -> Self {
        let mut bits = vec![false; size];
        
        // Unpack bytes into bits
        for (i, &byte) in data.iter().enumerate() {
            for j in 0..8 {
                let bit_index = i * 8 + j;
                if bit_index < size {
                    bits[bit_index] = (byte & (1 << j)) != 0;
                }
            }
        }
        
        // Count the number of set bits to estimate the count
        let count = bits.iter().filter(|&&b| b).count();
        
        BloomFilter {
            bits,
            k,
            count,
            _marker: PhantomData,
        }
    }
}

/// A specialized Bloom filter for transaction IDs
pub struct TransactionBloomFilter {
    /// Inner Bloom filter
    filter: BloomFilter<Vec<u8>>,
    
    /// Maximum number of transactions to track
    max_transactions: usize,
    
    /// Current transaction count
    transaction_count: usize,
}

impl TransactionBloomFilter {
    /// Create a new transaction Bloom filter
    pub fn new(max_transactions: usize, false_positive_probability: f64) -> Self {
        TransactionBloomFilter {
            filter: BloomFilter::with_params(max_transactions, false_positive_probability),
            max_transactions,
            transaction_count: 0,
        }
    }
    
    /// Add a transaction ID to the filter
    pub fn add_transaction(&mut self, tx_id: &[u8]) {
        self.filter.insert(&tx_id.to_vec());
        self.transaction_count += 1;
        
        // Reset if we've reached the maximum
        if self.transaction_count >= self.max_transactions {
            self.reset();
        }
    }
    
    /// Check if a transaction ID might be in the filter
    pub fn contains_transaction(&self, tx_id: &[u8]) -> bool {
        self.filter.contains(&tx_id.to_vec())
    }
    
    /// Reset the filter
    pub fn reset(&mut self) {
        self.filter.clear();
        self.transaction_count = 0;
    }
    
    /// Serialize the filter
    pub fn serialize(&self) -> Vec<u8> {
        self.filter.serialize()
    }
    
    /// Get the size of the filter in bits
    pub fn size(&self) -> usize {
        self.filter.size()
    }
    
    /// Get the number of transactions in the filter
    pub fn transaction_count(&self) -> usize {
        self.transaction_count
    }
    
    /// Get the estimated false positive probability
    pub fn false_positive_probability(&self) -> f64 {
        self.filter.false_positive_probability()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bloom_filter_creation() {
        let filter = BloomFilter::<String>::new(1000, 3);
        
        assert_eq!(filter.size(), 1000);
        assert_eq!(filter.hash_functions(), 3);
        assert_eq!(filter.count(), 0);
    }
    
    #[test]
    fn test_bloom_filter_with_params() {
        let filter = BloomFilter::<String>::with_params(1000, 0.01);
        
        // Check that the size and k are reasonable
        assert!(filter.size() > 1000);
        assert!(filter.hash_functions() > 0);
    }
    
    #[test]
    fn test_bloom_filter_insert_contains() {
        let mut filter = BloomFilter::<String>::new(1000, 3);
        
        // Insert some items
        filter.insert(&"hello".to_string());
        filter.insert(&"world".to_string());
        
        // Check that they're in the filter
        assert!(filter.contains(&"hello".to_string()));
        assert!(filter.contains(&"world".to_string()));
        
        // Check that a non-inserted item is not in the filter
        assert!(!filter.contains(&"test".to_string()));
        
        // Check the count
        assert_eq!(filter.count(), 2);
    }
    
    #[test]
    fn test_bloom_filter_clear() {
        let mut filter = BloomFilter::<String>::new(1000, 3);
        
        // Insert some items
        filter.insert(&"hello".to_string());
        filter.insert(&"world".to_string());
        
        // Clear the filter
        filter.clear();
        
        // Check that the items are no longer in the filter
        assert!(!filter.contains(&"hello".to_string()));
        assert!(!filter.contains(&"world".to_string()));
        
        // Check the count
        assert_eq!(filter.count(), 0);
    }
    
    #[test]
    fn test_bloom_filter_merge() {
        let mut filter1 = BloomFilter::<String>::new(1000, 3);
        let mut filter2 = BloomFilter::<String>::new(1000, 3);
        
        // Insert different items in each filter
        filter1.insert(&"hello".to_string());
        filter2.insert(&"world".to_string());
        
        // Merge filter2 into filter1
        assert!(filter1.merge(&filter2).is_ok());
        
        // Check that both items are in filter1
        assert!(filter1.contains(&"hello".to_string()));
        assert!(filter1.contains(&"world".to_string()));
        
        // Check the count
        assert_eq!(filter1.count(), 2);
    }
    
    #[test]
    fn test_bloom_filter_serialize_deserialize() {
        let mut filter = BloomFilter::<String>::new(1000, 3);
        
        // Insert some items
        filter.insert(&"hello".to_string());
        filter.insert(&"world".to_string());
        
        // Serialize the filter
        let serialized = filter.serialize();
        
        // Deserialize the filter
        let deserialized = BloomFilter::<String>::deserialize(&serialized, 1000, 3);
        
        // Check that the deserialized filter contains the same items
        assert!(deserialized.contains(&"hello".to_string()));
        assert!(deserialized.contains(&"world".to_string()));
        assert!(!deserialized.contains(&"test".to_string()));
    }
    
    #[test]
    fn test_transaction_bloom_filter() {
        let mut filter = TransactionBloomFilter::new(1000, 0.01);
        
        // Add some transactions
        let tx1 = vec![1, 2, 3, 4];
        let tx2 = vec![5, 6, 7, 8];
        
        filter.add_transaction(&tx1);
        filter.add_transaction(&tx2);
        
        // Check that they're in the filter
        assert!(filter.contains_transaction(&tx1));
        assert!(filter.contains_transaction(&tx2));
        
        // Check that a non-added transaction is not in the filter
        assert!(!filter.contains_transaction(&vec![9, 10, 11, 12]));
        
        // Check the count
        assert_eq!(filter.transaction_count(), 2);
        
        // Reset the filter
        filter.reset();
        
        // Check that the transactions are no longer in the filter
        assert!(!filter.contains_transaction(&tx1));
        assert!(!filter.contains_transaction(&tx2));
        
        // Check the count
        assert_eq!(filter.transaction_count(), 0);
    }
}
