//! # Blockchain Core Module
//! 
//! This module implements the core blockchain data structures and logic,
//! including blocks, transactions, and blockchain state management.

mod block;
mod transaction;

// Re-export main types
pub use block::Block;
pub use block::BlockHeader;
pub use block::ShardData;
pub use transaction::Transaction;
pub use transaction::Receipt;

use crate::types::{Result, Error};

/// Blockchain represents the main chain structure
pub struct Blockchain {
    /// Current height of the blockchain
    pub height: u64,
    /// Genesis block hash
    pub genesis_hash: Vec<u8>,
    /// Latest block hash
    pub latest_hash: Vec<u8>,
}

impl Blockchain {
    /// Create a new blockchain with a genesis block
    pub fn new() -> Result<Self> {
        // In a real implementation, we would generate a proper genesis block
        let genesis_hash = vec![0; 32]; // Placeholder
        
        Ok(Blockchain {
            height: 0,
            genesis_hash: genesis_hash.clone(),
            latest_hash: genesis_hash,
        })
    }
    
    /// Add a block to the chain
    pub fn add_block(&mut self, block: &Block) -> Result<()> {
        // In a real implementation, we would validate the block
        // and update the chain state accordingly
        
        // For now, just increment height and update latest hash
        self.height += 1;
        self.latest_hash = block.header.previous_hash.clone();
        
        Ok(())
    }
    
    /// Get the current height of the blockchain
    pub fn get_height(&self) -> u64 {
        self.height
    }
    
    /// Get the latest block hash
    pub fn get_latest_hash(&self) -> &Vec<u8> {
        &self.latest_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_blockchain() {
        let blockchain = Blockchain::new().unwrap();
        assert_eq!(blockchain.height, 0);
        assert_eq!(blockchain.genesis_hash.len(), 32);
        assert_eq!(blockchain.latest_hash.len(), 32);
    }
}
