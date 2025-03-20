use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

use crate::tests::dpos::types::{BlockHeight, ShardId, Timestamp, Result, Error};
use crate::tests::dpos::block::{Block, ShardData};
use crate::tests::dpos::consensus::{Consensus, ConsensusConfig};
use crate::tests::dpos::consensus_state::ConsensusState;
use crate::tests::dpos::validator::{Validator, ValidatorPool};
use crate::tests::dpos::reward::RewardSchedule;

// DPoS consensus implementation
pub struct DPoSConsensus {
    pub config: ConsensusConfig,
    pub state: Arc<Mutex<ConsensusState>>,
    pub local_public_key: Option<Vec<u8>>,
    pub block_history: Arc<Mutex<HashMap<BlockHeight, Block>>>,
    pub reward_schedule: RewardSchedule,
    pub block_schedule: Arc<Mutex<HashMap<BlockHeight, HashMap<ShardId, Vec<u8>>>>>,
}

impl DPoSConsensus {
    pub fn new(config: ConsensusConfig) -> Self {
        DPoSConsensus {
            config,
            state: Arc::new(Mutex::new(ConsensusState::new())),
            local_public_key: None,
            block_history: Arc::new(Mutex::new(HashMap::new())),
            reward_schedule: RewardSchedule::default(),
            block_schedule: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn set_local_public_key(&mut self, public_key: Vec<u8>) {
        self.local_public_key = Some(public_key);
    }
    
    pub fn current_time_micros() -> Timestamp {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as Timestamp
    }
    
    pub fn is_block_production_time(&self, last_block_time: Timestamp) -> bool {
        let now = Self::current_time_micros();
        let elapsed = now - last_block_time;
        
        let interval_micros = self.config.block_interval_ms as u64 * 1000;
        
        elapsed >= interval_micros
    }
    
    pub fn add_block_to_history(&self, block: Block) {
        let mut history = self.block_history.lock().unwrap();
        let height = block.header.index;
        
        history.insert(height, block);
        
        if height > self.config.finality_confirmations {
            let oldest_to_keep = height - self.config.finality_confirmations;
            history.retain(|h, _| *h >= oldest_to_keep);
        }
    }
    
    pub fn calculate_block_reward(&self, block: &Block) -> u64 {
        // Determine which halving period we're in
        let halving_period = block.header.index / self.reward_schedule.halving_interval;
        
        // Calculate the divisor for the halving (1, 2, 4, 8, etc.)
        let halving_divisor = if halving_period == 0 {
            1
        } else {
            1u64 << halving_period // 2^halving_period: 2, 4, 8, etc.
        };
        
        // Base reward for producing a block (divided by halving divisor)
        let base_reward = self.reward_schedule.base_block_reward / halving_divisor;
        
        // Additional reward for transactions (based on transaction count)
        let tx_count = block.shard_data.iter()
            .map(|shard| shard.transactions.len())
            .sum::<usize>() as u64;
            
        let tx_reward = tx_count * self.reward_schedule.per_transaction_reward / halving_divisor;
        
        base_reward + tx_reward
    }
    
    pub fn get_scheduled_validator(&self, height: BlockHeight, shard: ShardId) -> Option<Vec<u8>> {
        // First check block schedule
        let block_schedule = self.block_schedule.lock().unwrap();
        if let Some(height_schedule) = block_schedule.get(&height) {
            if let Some(validator_key) = height_schedule.get(&shard) {
                return Some(validator_key.clone());
            }
        }
        
        // If not found in schedule, use the validator selection algorithm
        let state = self.state.lock().unwrap();
        state.validators.select_validator_for_block(height, shard)
            .map(|v| v.public_key.clone())
    }
}

impl Consensus for DPoSConsensus {
    fn init(&mut self) -> Result<()> {
        // Initialize the consensus state
        let mut state = self.state.lock().unwrap();
        
        // Start with height 0
        state.height = 0;
        state.epoch = 0;
        state.last_block_time = Self::current_time_micros();
        state.is_active = true;
        
        // Initialize validator pool if needed
        if state.validators.validator_count() == 0 {
            // In a real implementation, we would load validators from storage
            // For now, we'll leave it empty
        }
        
        // Assign validators to shards
        state.validators.assign_validators_to_shards(self.config.shard_count)?;
        
        Ok(())
    }
    
    fn is_scheduled_producer(&self, height: BlockHeight, shard: ShardId) -> bool {
        // Check if local node is a validator
        if let Some(public_key) = &self.local_public_key {
            // First check the block production schedule
            if let Some(scheduled_validator) = self.get_scheduled_validator(height, shard) {
                return scheduled_validator == *public_key;
            }
            
            // Fall back to selecting based on algorithm
            let state = self.state.lock().unwrap();
            if let Some(next_validator) = state.validators.select_validator_for_block(height, shard) {
                // Check if the local node is the next validator
                return next_validator.public_key == *public_key;
            }
        }
        
        false
    }
    
    fn produce_block(&self, height: BlockHeight, shard: ShardId) -> Result<Block> {
        let state = self.state.lock().unwrap();
        
        // Check if we're at the right height
        if state.height != height {
            return Err(Error::Consensus(format!(
                "Invalid height: expected {}, got {}",
                state.height, height
            )));
        }
        
        // Create a new block
        let timestamp = Self::current_time_micros();
        
        // Get previous block hash (would come from storage in a real implementation)
        let previous_hash = vec![0; 32]; // Placeholder
        
        // Get shard IDs for this block
        let shard_ids = vec![shard];
        
        // Create the block
        let mut block = Block::new(height, timestamp, previous_hash, shard_ids);
        
        // Add our validator to the validator set
        if let Some(public_key) = &self.local_public_key {
            block.validator_set.push(public_key.clone());
        }
        
        // Create empty shard data
        let shard_data = ShardData {
            shard_id: shard,
            transactions: Vec::new(),
            execution_proof: Vec::new(),
            validator_signatures: Vec::new(),
        };
        
        // Add shard data to the block
        block.add_shard_data(shard_data)?;
        
        Ok(block)
    }
    
    fn validate_block(&self, block: &Block) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        // Check block height
        if block.header.index != state.height + 1 {
            return Err(Error::BlockValidation(format!(
                "Invalid block height: expected {}, got {}",
                state.height + 1, block.header.index
            )));
        }
        
        // Verify block timestamp
        let now = Self::current_time_micros();
        if block.header.timestamp > now + 10_000_000 { // Allow 10 seconds in future
            return Err(Error::BlockValidation("Timestamp too far in the future".into()));
        }
        
        Ok(())
    }
    
    fn is_final(&self, block: &Block) -> bool {
        let height = block.header.index;
        
        // A block is final if it has enough confirmations
        if height + self.config.finality_confirmations <= self.state.lock().unwrap().height {
            return true;
        }
        
        false
    }
    
    fn get_next_validator(&self, height: BlockHeight, shard: ShardId) -> Result<Validator> {
        let state = self.state.lock().unwrap();
        
        if let Some(validator) = state.validators.select_validator_for_block(height, shard) {
            Ok(validator.clone())
        } else {
            Err(Error::Consensus(format!(
                "No validator scheduled for height {} and shard {}",
                height, shard
            )))
        }
    }
    
    fn update_validators(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        // Reassign validators to shards
        state.validators.assign_validators_to_shards(self.config.shard_count)?;
        
        Ok(())
    }
    
    fn get_validator_pool(&self) -> &ValidatorPool {
        // In a real implementation, we would clone the validator pool
        // This is just for the test
        panic!("Not implemented for tests");
    }
    
    fn get_validator_by_pubkey(&self, pubkey: &[u8]) -> Option<Validator> {
        let state = self.state.lock().unwrap();
        
        state.validators.get_validator_by_pubkey(pubkey)
            .cloned()
    }
}
