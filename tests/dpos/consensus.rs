use crate::tests::dpos::types::{Result, BlockHeight, ShardId};
use crate::tests::dpos::block::Block;
use crate::tests::dpos::validator::{Validator, ValidatorPool};

// Mock Consensus trait
pub trait Consensus {
    fn init(&mut self) -> Result<()>;
    fn is_scheduled_producer(&self, height: BlockHeight, shard: ShardId) -> bool;
    fn produce_block(&self, height: BlockHeight, shard: ShardId) -> Result<Block>;
    fn validate_block(&self, block: &Block) -> Result<()>;
    fn is_final(&self, block: &Block) -> bool;
    fn get_next_validator(&self, height: BlockHeight, shard: ShardId) -> Result<Validator>;
    fn update_validators(&mut self) -> Result<()>;
    fn get_validator_pool(&self) -> &ValidatorPool;
    fn get_validator_by_pubkey(&self, pubkey: &[u8]) -> Option<Validator>;
}

// ConsensusConfig structure
pub struct ConsensusConfig {
    pub block_interval_ms: u64,
    pub finality_confirmations: BlockHeight,
    pub shard_count: ShardId,
    pub blocks_per_epoch: BlockHeight,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            block_interval_ms: 2000, // 2 seconds
            finality_confirmations: 3,
            shard_count: 4,
            blocks_per_epoch: 100,
        }
    }
}
