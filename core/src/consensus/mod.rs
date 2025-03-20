//! # Consensus Module
//! 
//! This module implements the consensus mechanism for the SEBURE blockchain,
//! using Delegated Proof of Stake (DPoS) with validator pools.

mod validator;
mod dpos;

// Re-export main types
pub use validator::Validator;
pub use validator::ValidatorPool;
pub use dpos::DPoSConsensus;

use crate::blockchain::Block;
use crate::types::{Result, BlockHeight, ShardId};
use std::sync::{Arc, Mutex};

/// Consensus configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Number of validators per pool
    pub validators_per_pool: usize,
    
    /// Number of blocks per epoch
    pub blocks_per_epoch: u64,
    
    /// Block production interval in milliseconds
    pub block_interval_ms: u64,
    
    /// Minimum stake required to be a validator
    pub min_stake: u64,
    
    /// Number of shards
    pub shard_count: u16,
    
    /// Whether to enable optimistic validation
    pub optimistic_validation: bool,
    
    /// Number of confirmations for finality
    pub finality_confirmations: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            validators_per_pool: 21,
            blocks_per_epoch: 100,
            block_interval_ms: 2000,  // 2 seconds
            min_stake: 1000,
            shard_count: 4,
            optimistic_validation: true,
            finality_confirmations: 3,
        }
    }
}

/// Consensus is the trait that all consensus implementations must implement
pub trait Consensus {
    /// Initialize the consensus mechanism
    fn init(&mut self) -> Result<()>;
    
    /// Check if the local node is scheduled to produce a block
    fn is_scheduled_producer(&self, height: BlockHeight, shard: ShardId) -> bool;
    
    /// Produce a new block
    fn produce_block(&self, height: BlockHeight, shard: ShardId) -> Result<Block>;
    
    /// Validate a block
    fn validate_block(&self, block: &Block) -> Result<()>;
    
    /// Check if a block is final
    fn is_final(&self, block: &Block) -> bool;
    
    /// Get the next validator for a specific height and shard
    fn get_next_validator(&self, height: BlockHeight, shard: ShardId) -> Result<Validator>;
    
    /// Update validator set with new information (e.g., stake changes)
    fn update_validators(&mut self) -> Result<()>;
    
    /// Get the current validator pool
    fn get_validator_pool(&self) -> &ValidatorPool;
    
    /// Get the validator for the given public key
    fn get_validator_by_pubkey(&self, pubkey: &[u8]) -> Option<Validator>;
    
    /// Get the list of all validators
    fn get_validators(&self) -> Result<Vec<Validator>>;
    
    /// Get the list of all shards
    fn get_shards(&self) -> Result<Vec<Shard>>;
}

/// ConsensusState represents the current state of the consensus mechanism
#[derive(Debug)]
pub struct ConsensusState {
    /// Current block height
    pub height: BlockHeight,
    
    /// Current epoch
    pub epoch: u64,
    
    /// Last block timestamp
    pub last_block_time: u64,
    
    /// Whether consensus is active
    pub is_active: bool,
    
    /// Current validator set
    pub validators: ValidatorPool,
}

impl ConsensusState {
    /// Create a new consensus state
    pub fn new() -> Self {
        ConsensusState {
            height: 0,
            epoch: 0,
            last_block_time: 0,
            is_active: false,
            validators: ValidatorPool::new(),
        }
    }
    
    /// Get the current epoch for a given height
    pub fn get_epoch_for_height(&self, height: BlockHeight, blocks_per_epoch: u64) -> u64 {
        height / blocks_per_epoch
    }
    
    /// Check if the height is the first block of an epoch
    pub fn is_epoch_start(&self, height: BlockHeight, blocks_per_epoch: u64) -> bool {
        height % blocks_per_epoch == 0
    }
    
    /// Get the shard for a given height
    pub fn get_shard_for_height(&self, height: BlockHeight, shard_count: u16) -> ShardId {
        (height % shard_count as u64) as ShardId
    }
}

/// Shard information
#[derive(Debug, Clone)]
pub struct Shard {
    /// Shard ID
    id: ShardId,
    
    /// Validators assigned to this shard
    validator_pool: Vec<ValidatorId>,
    
    /// Current state root
    state_root: Vec<u8>,
    
    /// Last known block height
    last_block_height: u64,
    
    /// Transaction count in this shard
    transaction_count: u64,
    
    /// Number of active accounts
    active_accounts: u32,
    
    /// Recent cross-shard transactions
    recent_cross_shard_transactions: Vec<Vec<u8>>,
    
    /// Connected shards
    neighbor_shards: Vec<ShardId>,
    
    /// Current resource utilization (0.0 - 1.0)
    resource_utilization: f32,
}

impl Shard {
    /// Create a new shard
    pub fn new(id: ShardId) -> Self {
        Shard {
            id,
            validator_pool: Vec::new(),
            state_root: vec![0; 32],
            last_block_height: 0,
            transaction_count: 0,
            active_accounts: 0,
            recent_cross_shard_transactions: Vec::new(),
            neighbor_shards: Vec::new(),
            resource_utilization: 0.0,
        }
    }
    
    /// Get the shard ID
    pub fn id(&self) -> ShardId {
        self.id
    }
    
    /// Get the validator pool
    pub fn validator_pool(&self) -> &Vec<ValidatorId> {
        &self.validator_pool
    }
    
    /// Get the transaction count
    pub fn transaction_count(&self) -> u64 {
        self.transaction_count
    }
    
    /// Get the number of active accounts
    pub fn active_accounts(&self) -> u32 {
        self.active_accounts
    }
    
    /// Get the resource utilization
    pub fn resource_utilization(&self) -> f32 {
        self.resource_utilization
    }
    
    /// Get the last block height
    pub fn last_block_height(&self) -> u64 {
        self.last_block_height
    }
    
    /// Get the neighbor shards
    pub fn neighbor_shards(&self) -> &Vec<ShardId> {
        &self.neighbor_shards
    }
}

/// Validator ID type
pub type ValidatorId = Vec<u8>;

/// Factory for creating consensus instances
pub struct ConsensusFactory;

impl ConsensusFactory {
    /// Create a new consensus instance
    pub fn create(config: ConsensusConfig) -> Box<dyn Consensus> {
        Box::new(DPoSConsensus::new(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_config_default() {
        let config = ConsensusConfig::default();
        
        assert_eq!(config.validators_per_pool, 21);
        assert_eq!(config.blocks_per_epoch, 100);
        assert_eq!(config.block_interval_ms, 2000);
        assert_eq!(config.min_stake, 1000);
        assert_eq!(config.shard_count, 4);
        assert!(config.optimistic_validation);
        assert_eq!(config.finality_confirmations, 3);
    }
    
    #[test]
    fn test_consensus_state() {
        let state = ConsensusState::new();
        
        assert_eq!(state.height, 0);
        assert_eq!(state.epoch, 0);
        assert_eq!(state.last_block_time, 0);
        assert!(!state.is_active);
        
        // Test epoch calculation
        assert_eq!(state.get_epoch_for_height(0, 100), 0);
        assert_eq!(state.get_epoch_for_height(99, 100), 0);
        assert_eq!(state.get_epoch_for_height(100, 100), 1);
        assert_eq!(state.get_epoch_for_height(250, 100), 2);
        
        // Test epoch start detection
        assert!(state.is_epoch_start(0, 100));
        assert!(!state.is_epoch_start(1, 100));
        assert!(state.is_epoch_start(100, 100));
        assert!(!state.is_epoch_start(101, 100));
        
        // Test shard assignment
        assert_eq!(state.get_shard_for_height(0, 4), 0);
        assert_eq!(state.get_shard_for_height(1, 4), 1);
        assert_eq!(state.get_shard_for_height(2, 4), 2);
        assert_eq!(state.get_shard_for_height(3, 4), 3);
        assert_eq!(state.get_shard_for_height(4, 4), 0);
    }
    
    #[test]
    fn test_consensus_factory() {
        let config = ConsensusConfig::default();
        let consensus = ConsensusFactory::create(config);
        
        // Basic validation that we got a consensus instance
        assert!(!consensus.is_scheduled_producer(0, 0)); // Should be false for a new instance
    }
}
