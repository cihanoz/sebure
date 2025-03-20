use std::collections::HashMap;
use crate::tests::dpos::types::{ShardId, Result};

// Simplified Validator structure
pub struct Validator {
    pub id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub stake: u64,
    pub assigned_shards: Vec<ShardId>,
    pub blocks_produced: u64,
    pub transactions_processed: u64,
    pub rewards: u64,
}

impl Validator {
    pub fn new(id: Vec<u8>, public_key: Vec<u8>, stake_address: Vec<u8>, stake: u64) -> Self {
        Validator {
            id,
            public_key,
            stake,
            assigned_shards: Vec::new(),
            blocks_produced: 0,
            transactions_processed: 0,
            rewards: 0,
        }
    }
    
    pub fn is_assigned_to_shard(&self, shard: ShardId) -> bool {
        self.assigned_shards.contains(&shard)
    }
    
    pub fn assign_shards(&mut self, shards: Vec<ShardId>) {
        self.assigned_shards = shards;
    }
    
    pub fn record_block_produced(&mut self, tx_count: u64) {
        self.blocks_produced += 1;
        self.transactions_processed += tx_count;
    }
    
    pub fn add_reward(&mut self, amount: u64) {
        self.rewards += amount;
    }
}

// Simplified ValidatorPool
pub struct ValidatorPool {
    validators: HashMap<Vec<u8>, Validator>,
}

impl ValidatorPool {
    pub fn new() -> Self {
        ValidatorPool {
            validators: HashMap::new(),
        }
    }
    
    pub fn add_validator(&mut self, validator: Validator) -> Result<()> {
        self.validators.insert(validator.id.clone(), validator);
        Ok(())
    }
    
    pub fn get_validator(&self, id: &[u8]) -> Option<&Validator> {
        self.validators.get(id)
    }
    
    pub fn get_validator_mut(&mut self, id: &[u8]) -> Option<&mut Validator> {
        self.validators.get_mut(id)
    }
    
    pub fn get_validator_by_pubkey(&self, pubkey: &[u8]) -> Option<&Validator> {
        self.validators.values().find(|v| v.public_key == pubkey)
    }
    
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
    
    pub fn assign_validators_to_shards(&mut self, shard_count: u16) -> Result<()> {
        // Simple round-robin assignment
        for (i, validator) in self.validators.values_mut().enumerate() {
            validator.assign_shards(vec![i as u16 % shard_count]);
        }
        Ok(())
    }
    
    pub fn select_validator_for_block(&self, height: u64, shard: ShardId) -> Option<&Validator> {
        // Simple selection algorithm based on height and shard
        let validators: Vec<&Validator> = self.validators.values()
            .filter(|v| v.is_assigned_to_shard(shard))
            .collect();
            
        if validators.is_empty() {
            return None;
        }
        
        // Round-robin selection based on block height
        let index = (height as usize) % validators.len();
        Some(validators[index])
    }
    
    pub fn get_all_validators(&self) -> Vec<Validator> {
        self.validators.values().cloned().collect()
    }
}

// Implement Clone for Validator for convenience in tests
impl Clone for Validator {
    fn clone(&self) -> Self {
        Validator {
            id: self.id.clone(),
            public_key: self.public_key.clone(),
            stake: self.stake,
            assigned_shards: self.assigned_shards.clone(),
            blocks_produced: self.blocks_produced,
            transactions_processed: self.transactions_processed,
            rewards: self.rewards,
        }
    }
}
