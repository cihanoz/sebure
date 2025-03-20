use crate::tests::dpos::types::{BlockHeight, Timestamp};
use crate::tests::dpos::validator::ValidatorPool;

// ConsensusState structure
pub struct ConsensusState {
    pub height: BlockHeight,
    pub epoch: u64,
    pub last_block_time: Timestamp,
    pub is_active: bool,
    pub validators: ValidatorPool,
}

impl ConsensusState {
    pub fn new() -> Self {
        ConsensusState {
            height: 0,
            epoch: 0,
            last_block_time: 0,
            is_active: false,
            validators: ValidatorPool::new(),
        }
    }
    
    pub fn get_epoch_for_height(&self, height: BlockHeight, blocks_per_epoch: BlockHeight) -> u64 {
        if blocks_per_epoch == 0 {
            return 0;
        }
        (height / blocks_per_epoch) as u64
    }
    
    pub fn is_epoch_start(&self, height: BlockHeight, blocks_per_epoch: BlockHeight) -> bool {
        if blocks_per_epoch == 0 {
            return false;
        }
        height % blocks_per_epoch == 0
    }
}
