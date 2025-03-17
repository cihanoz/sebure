//! # Validator Implementation
//! 
//! This module implements validator functionality for the consensus mechanism.

use serde::{Serialize, Deserialize};
use crate::types::{Result, Error, ShardId};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Performance metrics for a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    /// Number of blocks produced
    pub blocks_produced: u32,
    
    /// Number of transactions processed
    pub transactions_processed: u64,
    
    /// Number of missed slots
    pub missed_slots: u32,
    
    /// Amount of rewards earned
    pub rewards_earned: u64,
    
    /// Number of slashing events
    pub slashing_events: u16,
    
    /// Average response time in microseconds
    pub average_response_time: u64,
}

impl Default for ValidatorMetrics {
    fn default() -> Self {
        ValidatorMetrics {
            blocks_produced: 0,
            transactions_processed: 0,
            missed_slots: 0,
            rewards_earned: 0,
            slashing_events: 0,
            average_response_time: 0,
        }
    }
}

/// Validator representation as defined in the PRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator ID
    pub id: Vec<u8>,
    
    /// Validator public key
    pub public_key: Vec<u8>,
    
    /// Staking address
    pub staking_address: Vec<u8>,
    
    /// Amount staked by the validator
    pub staking_amount: u64,
    
    /// Amount delegated to the validator
    pub delegated_stake: u64,
    
    /// Commission rate (0.0 - 1.0)
    pub commission_rate: f32,
    
    /// Uptime (0.0 - 1.0)
    pub uptime: f32,
    
    /// Last active timestamp in microseconds
    pub last_active_timestamp: u64,
    
    /// Performance metrics
    pub performance_metrics: ValidatorMetrics,
    
    /// Voting power (0.0 - 1.0)
    pub voting_power: f32,
    
    /// Shard assignments
    pub shard_assignments: Vec<ShardId>,
    
    /// Hardware capability score
    pub hardware_capability: u32,
}

impl Validator {
    /// Create a new validator
    pub fn new(
        id: Vec<u8>,
        public_key: Vec<u8>,
        staking_address: Vec<u8>,
        staking_amount: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        Validator {
            id,
            public_key,
            staking_address,
            staking_amount,
            delegated_stake: 0,
            commission_rate: 0.05,  // 5% default commission
            uptime: 1.0,            // Start with perfect uptime
            last_active_timestamp: now,
            performance_metrics: ValidatorMetrics::default(),
            voting_power: 0.0,      // Will be calculated later
            shard_assignments: Vec::new(),
            hardware_capability: 100, // Default capability score
        }
    }
    
    /// Get the total stake (own stake + delegated)
    pub fn total_stake(&self) -> u64 {
        self.staking_amount + self.delegated_stake
    }
    
    /// Update the last active timestamp
    pub fn update_active_timestamp(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
            
        self.last_active_timestamp = now;
    }
    
    /// Record a block production
    pub fn record_block_produced(&mut self, transactions: u64) {
        self.performance_metrics.blocks_produced += 1;
        self.performance_metrics.transactions_processed += transactions;
        self.update_active_timestamp();
    }
    
    /// Record a missed slot
    pub fn record_missed_slot(&mut self) {
        self.performance_metrics.missed_slots += 1;
        
        // Adjust uptime based on missed slots
        // This is a simplified calculation for the prototype
        let total_slots = self.performance_metrics.blocks_produced + 
                          self.performance_metrics.missed_slots;
                          
        if total_slots > 0 {
            self.uptime = self.performance_metrics.blocks_produced as f32 / total_slots as f32;
        }
    }
    
    /// Add reward to the validator
    pub fn add_reward(&mut self, amount: u64) {
        self.performance_metrics.rewards_earned += amount;
    }
    
    /// Apply a slashing penalty
    pub fn apply_slashing(&mut self, penalty_percentage: f32) -> u64 {
        self.performance_metrics.slashing_events += 1;
        
        // Calculate penalty amount
        let penalty_amount = (self.staking_amount as f32 * penalty_percentage) as u64;
        
        // Ensure we don't underflow
        if penalty_amount > self.staking_amount {
            let actual_penalty = self.staking_amount;
            self.staking_amount = 0;
            return actual_penalty;
        }
        
        self.staking_amount -= penalty_amount;
        penalty_amount
    }
    
    /// Assign shards to the validator
    pub fn assign_shards(&mut self, shards: Vec<ShardId>) {
        self.shard_assignments = shards;
    }
    
    /// Check if the validator is assigned to a specific shard
    pub fn is_assigned_to_shard(&self, shard: ShardId) -> bool {
        self.shard_assignments.contains(&shard)
    }
    
    /// Calculate and update the voting power
    pub fn calculate_voting_power(&mut self, total_stake: u64) {
        if total_stake == 0 {
            self.voting_power = 0.0;
            return;
        }
        
        // Calculate voting power based on stake ratio and uptime
        let stake_ratio = self.total_stake() as f32 / total_stake as f32;
        self.voting_power = stake_ratio * self.uptime;
    }
}

/// A pool of validators
#[derive(Debug, Clone)]
pub struct ValidatorPool {
    /// Validators by ID
    validators: HashMap<Vec<u8>, Validator>,
    
    /// Validators by public key
    validators_by_pubkey: HashMap<Vec<u8>, Vec<u8>>, // Maps public key to validator ID
    
    /// Validators by shard
    validators_by_shard: HashMap<ShardId, HashSet<Vec<u8>>>, // Maps shard to validator IDs
    
    /// Total stake in the pool
    total_stake: u64,
}

impl ValidatorPool {
    /// Create a new validator pool
    pub fn new() -> Self {
        ValidatorPool {
            validators: HashMap::new(),
            validators_by_pubkey: HashMap::new(),
            validators_by_shard: HashMap::new(),
            total_stake: 0,
        }
    }
    
    /// Add a validator to the pool
    pub fn add_validator(&mut self, validator: Validator) -> Result<()> {
        let id = validator.id.clone();
        let pubkey = validator.public_key.clone();
        
        // Update total stake
        self.total_stake += validator.total_stake();
        
        // Add to validators by public key
        self.validators_by_pubkey.insert(pubkey, id.clone());
        
        // Add to validators by shard
        for shard in &validator.shard_assignments {
            let shard_validators = self.validators_by_shard
                .entry(*shard)
                .or_insert_with(HashSet::new);
                
            shard_validators.insert(id.clone());
        }
        
        // Add to validators
        self.validators.insert(id, validator);
        
        // Recalculate voting power for all validators
        self.recalculate_voting_power();
        
        Ok(())
    }
    
    /// Remove a validator from the pool
    pub fn remove_validator(&mut self, id: &[u8]) -> Result<Validator> {
        let validator = self.validators.remove(id)
            .ok_or_else(|| Error::Consensus(format!("Validator not found: {:?}", id)))?;
            
        // Update total stake
        self.total_stake -= validator.total_stake();
        
        // Remove from validators by public key
        self.validators_by_pubkey.remove(&validator.public_key);
        
        // Remove from validators by shard
        for shard in &validator.shard_assignments {
            if let Some(shard_validators) = self.validators_by_shard.get_mut(shard) {
                shard_validators.remove(id);
                
                // Clean up empty sets
                if shard_validators.is_empty() {
                    self.validators_by_shard.remove(shard);
                }
            }
        }
        
        // Recalculate voting power for all validators
        self.recalculate_voting_power();
        
        Ok(validator)
    }
    
    /// Get a validator by ID
    pub fn get_validator(&self, id: &[u8]) -> Option<&Validator> {
        self.validators.get(id)
    }
    
    /// Get a mutable reference to a validator by ID
    pub fn get_validator_mut(&mut self, id: &[u8]) -> Option<&mut Validator> {
        self.validators.get_mut(id)
    }
    
    /// Get a validator by public key
    pub fn get_validator_by_pubkey(&self, pubkey: &[u8]) -> Option<&Validator> {
        self.validators_by_pubkey.get(pubkey)
            .and_then(|id| self.validators.get(id))
    }
    
    /// Get validators for a specific shard
    pub fn get_validators_for_shard(&self, shard: ShardId) -> Vec<&Validator> {
        match self.validators_by_shard.get(&shard) {
            Some(validator_ids) => {
                validator_ids.iter()
                    .filter_map(|id| self.validators.get(id))
                    .collect()
            },
            None => Vec::new(),
        }
    }
    
    /// Get all validators
    pub fn get_all_validators(&self) -> Vec<&Validator> {
        self.validators.values().collect()
    }
    
    /// Get the number of validators
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
    
    /// Get the total stake
    pub fn get_total_stake(&self) -> u64 {
        self.total_stake
    }
    
    /// Recalculate voting power for all validators
    fn recalculate_voting_power(&mut self) {
        for validator in self.validators.values_mut() {
            validator.calculate_voting_power(self.total_stake);
        }
    }
    
    /// Select validator for block production based on height and shard
    pub fn select_validator_for_block(&self, height: u64, shard: ShardId) -> Option<&Validator> {
        let shard_validators = match self.validators_by_shard.get(&shard) {
            Some(validators) => validators,
            None => return None,
        };
        
        let validator_count = shard_validators.len();
        if validator_count == 0 {
            return None;
        }
        
        // Simple round-robin selection based on height
        let validator_index = (height % validator_count as u64) as usize;
        
        // Get validator by index
        shard_validators.iter()
            .skip(validator_index)
            .next()
            .and_then(|id| self.validators.get(id))
    }
    
    /// Assign validators to shards based on a deterministic algorithm
    pub fn assign_validators_to_shards(&mut self, shard_count: u16) -> Result<()> {
        // Clear existing shard assignments
        self.validators_by_shard.clear();
        
        // Get validators sorted by stake
        let mut validators_by_stake: Vec<(&Vec<u8>, &mut Validator)> = 
            self.validators.iter_mut().collect();
            
        validators_by_stake.sort_by(|a, b| 
            b.1.total_stake().cmp(&a.1.total_stake()));
        
        // Assign validators to shards, distributing them evenly
        for (idx, (id, validator)) in validators_by_stake.iter_mut().enumerate() {
            // Assign to shards
            let shards: Vec<ShardId> = (0..shard_count)
                .filter(|s| idx % shard_count as usize == *s as usize)
                .collect();
                
            validator.assign_shards(shards.clone());
            
            // Update validators_by_shard
            for shard in shards {
                let shard_validators = self.validators_by_shard
                    .entry(shard)
                    .or_insert_with(HashSet::new);
                    
                shard_validators.insert((*id).clone());
            }
        }
        
        Ok(())
    }
    
    /// Update a validator's stake
    pub fn update_validator_stake(&mut self, id: &[u8], new_stake: u64) -> Result<()> {
        let validator = self.validators.get_mut(id)
            .ok_or_else(|| Error::Consensus(format!("Validator not found: {:?}", id)))?;
            
        // Update total stake
        self.total_stake -= validator.staking_amount;
        self.total_stake += new_stake;
        
        // Update validator stake
        validator.staking_amount = new_stake;
        
        // Recalculate voting power
        self.recalculate_voting_power();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_validator(id: u8, stake: u64) -> Validator {
        Validator::new(
            vec![id],
            vec![100 + id],
            vec![200 + id],
            stake,
        )
    }
    
    #[test]
    fn test_validator_creation() {
        let validator = create_test_validator(1, 1000);
        
        assert_eq!(validator.id, vec![1]);
        assert_eq!(validator.public_key, vec![101]);
        assert_eq!(validator.staking_address, vec![201]);
        assert_eq!(validator.staking_amount, 1000);
        assert_eq!(validator.delegated_stake, 0);
        assert_eq!(validator.total_stake(), 1000);
    }
    
    #[test]
    fn test_validator_record_block() {
        let mut validator = create_test_validator(1, 1000);
        let original_timestamp = validator.last_active_timestamp;
        
        // Wait a bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        validator.record_block_produced(10);
        
        assert_eq!(validator.performance_metrics.blocks_produced, 1);
        assert_eq!(validator.performance_metrics.transactions_processed, 10);
        assert!(validator.last_active_timestamp > original_timestamp);
    }
    
    #[test]
    fn test_validator_missed_slot() {
        let mut validator = create_test_validator(1, 1000);
        
        // Initially perfect uptime
        assert_eq!(validator.uptime, 1.0);
        
        // Record 3 blocks and 1 missed slot
        validator.record_block_produced(10);
        validator.record_block_produced(5);
        validator.record_block_produced(7);
        validator.record_missed_slot();
        
        assert_eq!(validator.performance_metrics.blocks_produced, 3);
        assert_eq!(validator.performance_metrics.missed_slots, 1);
        assert_eq!(validator.uptime, 0.75); // 3 out of 4 slots = 75% uptime
    }
    
    #[test]
    fn test_validator_slashing() {
        let mut validator = create_test_validator(1, 1000);
        
        // Apply 10% slashing penalty
        let penalty = validator.apply_slashing(0.1);
        
        assert_eq!(penalty, 100); // 10% of 1000
        assert_eq!(validator.staking_amount, 900);
        assert_eq!(validator.performance_metrics.slashing_events, 1);
        
        // Apply slashing penalty greater than stake
        let penalty = validator.apply_slashing(2.0);
        
        assert_eq!(penalty, 900); // All remaining stake
        assert_eq!(validator.staking_amount, 0);
        assert_eq!(validator.performance_metrics.slashing_events, 2);
    }
    
    #[test]
    fn test_validator_voting_power() {
        let mut validator = create_test_validator(1, 1000);
        
        // Calculate voting power with total stake of 10000
        validator.calculate_voting_power(10000);
        
        // Voting power should be stake ratio * uptime = 0.1 * 1.0 = 0.1
        assert_eq!(validator.voting_power, 0.1);
        
        // Reduce uptime to 0.8 and recalculate
        validator.uptime = 0.8;
        validator.calculate_voting_power(10000);
        
        // Now voting power should be 0.1 * 0.8 = 0.08
        assert!((validator.voting_power - 0.08).abs() < 0.001);
    }
    
    #[test]
    fn test_validator_pool() {
        let mut pool = ValidatorPool::new();
        
        // Add validators
        let v1 = create_test_validator(1, 1000);
        let v2 = create_test_validator(2, 2000);
        let v3 = create_test_validator(3, 3000);
        
        pool.add_validator(v1).unwrap();
        pool.add_validator(v2).unwrap();
        pool.add_validator(v3).unwrap();
        
        assert_eq!(pool.validator_count(), 3);
        assert_eq!(pool.get_total_stake(), 6000);
        
        // Check validator retrieval
        let validator = pool.get_validator(&vec![2]).unwrap();
        assert_eq!(validator.staking_amount, 2000);
        
        let validator = pool.get_validator_by_pubkey(&vec![103]).unwrap();
        assert_eq!(validator.id, vec![3]);
        
        // Check validator removal
        let removed = pool.remove_validator(&vec![2]).unwrap();
        assert_eq!(removed.id, vec![2]);
        assert_eq!(pool.validator_count(), 2);
        assert_eq!(pool.get_total_stake(), 4000);
        
        // Check updating stake
        pool.update_validator_stake(&vec![1], 1500).unwrap();
        assert_eq!(pool.get_total_stake(), 4500);
        let validator = pool.get_validator(&vec![1]).unwrap();
        assert_eq!(validator.staking_amount, 1500);
    }
    
    #[test]
    fn test_shard_assignment() {
        let mut pool = ValidatorPool::new();
        
        // Add validators
        for i in 1..=10 {
            let v = create_test_validator(i, i as u64 * 1000);
            pool.add_validator(v).unwrap();
        }
        
        // Assign to 4 shards
        pool.assign_validators_to_shards(4).unwrap();
        
        // Each shard should have some validators
        for shard in 0..4 {
            let validators = pool.get_validators_for_shard(shard);
            assert!(!validators.is_empty());
        }
        
        // Check that all validators are assigned to at least one shard
        for validator in pool.get_all_validators() {
            assert!(!validator.shard_assignments.is_empty());
        }
        
        // Check validator selection by height and shard
        let v1 = pool.select_validator_for_block(0, 0);
        let v2 = pool.select_validator_for_block(1, 0);
        
        // Different heights should potentially select different validators
        // (though with only a few validators, they might be the same)
        assert!(v1.is_some());
        assert!(v2.is_some());
    }
}
