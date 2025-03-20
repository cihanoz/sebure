use crate::tests::dpos::types::{BlockHeight, ShardId, Timestamp, Result};

// Simplified Block structure for testing
pub struct Block {
    pub header: BlockHeader,
    pub shard_data: Vec<ShardData>,
    pub validator_set: Vec<Vec<u8>>,
}

pub struct BlockHeader {
    pub index: BlockHeight,
    pub timestamp: Timestamp,
    pub previous_hash: Vec<u8>,
    pub shard_identifiers: Vec<ShardId>,
}

pub struct ShardData {
    pub shard_id: ShardId,
    pub transactions: Vec<Vec<u8>>,
    pub execution_proof: Vec<u8>,
    pub validator_signatures: Vec<u8>,
}

impl Block {
    pub fn new(height: BlockHeight, timestamp: Timestamp, previous_hash: Vec<u8>, shard_ids: Vec<ShardId>) -> Self {
        Block {
            header: BlockHeader {
                index: height,
                timestamp,
                previous_hash,
                shard_identifiers: shard_ids,
            },
            shard_data: Vec::new(),
            validator_set: Vec::new(),
        }
    }
    
    pub fn add_shard_data(&mut self, shard_data: ShardData) -> Result<()> {
        self.shard_data.push(shard_data);
        Ok(())
    }
}
