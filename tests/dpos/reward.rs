// Reward schedule for validators
#[derive(Debug, Clone)]
pub struct RewardSchedule {
    pub base_block_reward: u64,
    pub per_transaction_reward: u64,
    pub validation_reward: u64,
    pub halving_interval: u64,
}

impl Default for RewardSchedule {
    fn default() -> Self {
        RewardSchedule {
            base_block_reward: 100,
            per_transaction_reward: 1,
            validation_reward: 10,
            halving_interval: 1_000_000, // 1 million blocks
        }
    }
}
