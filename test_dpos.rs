use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

type BlockHeight = u64;
type ShardId = u16;
type Timestamp = u64;

#[derive(Debug)]
pub enum Error {
    Consensus(String),
    BlockValidation(String),
}

type Result<T> = std::result::Result<T, Error>;

// Simplified Block structure for testing
struct Block {
    header: BlockHeader,
    shard_data: Vec<ShardData>,
    validator_set: Vec<Vec<u8>>,
}

struct BlockHeader {
    index: BlockHeight,
    timestamp: Timestamp,
    previous_hash: Vec<u8>,
    shard_identifiers: Vec<ShardId>,
}

struct ShardData {
    shard_id: ShardId,
    transactions: Vec<Vec<u8>>,
    execution_proof: Vec<u8>,
    validator_signatures: Vec<u8>,
}

impl Block {
    fn new(height: BlockHeight, timestamp: Timestamp, previous_hash: Vec<u8>, shard_ids: Vec<ShardId>) -> Self {
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
    
    fn add_shard_data(&mut self, shard_data: ShardData) -> Result<()> {
        self.shard_data.push(shard_data);
        Ok(())
    }
}

// Simplified Validator structure
struct Validator {
    id: Vec<u8>,
    public_key: Vec<u8>,
    stake: u64,
    assigned_shards: Vec<ShardId>,
    blocks_produced: u64,
    transactions_processed: u64,
    rewards: u64,
}

impl Validator {
    fn new(id: Vec<u8>, public_key: Vec<u8>, stake_address: Vec<u8>, stake: u64) -> Self {
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
    
    fn is_assigned_to_shard(&self, shard: ShardId) -> bool {
        self.assigned_shards.contains(&shard)
    }
    
    fn assign_shards(&mut self, shards: Vec<ShardId>) {
        self.assigned_shards = shards;
    }
    
    fn record_block_produced(&mut self, tx_count: u64) {
        self.blocks_produced += 1;
        self.transactions_processed += tx_count;
    }
    
    fn add_reward(&mut self, amount: u64) {
        self.rewards += amount;
    }
}

// Simplified ValidatorPool
struct ValidatorPool {
    validators: HashMap<Vec<u8>, Validator>,
}

impl ValidatorPool {
    fn new() -> Self {
        ValidatorPool {
            validators: HashMap::new(),
        }
    }
    
    fn add_validator(&mut self, validator: Validator) -> Result<()> {
        self.validators.insert(validator.id.clone(), validator);
        Ok(())
    }
    
    fn get_validator(&self, id: &[u8]) -> Option<&Validator> {
        self.validators.get(id)
    }
    
    fn get_validator_mut(&mut self, id: &[u8]) -> Option<&mut Validator> {
        self.validators.get_mut(id)
    }
    
    fn get_validator_by_pubkey(&self, pubkey: &[u8]) -> Option<&Validator> {
        self.validators.values().find(|v| v.public_key == pubkey)
    }
    
    fn validator_count(&self) -> usize {
        self.validators.len()
    }
    
    fn assign_validators_to_shards(&mut self, shard_count: u16) -> Result<()> {
        // Simple round-robin assignment
        for (i, validator) in self.validators.values_mut().enumerate() {
            validator.assign_shards(vec![i as u16 % shard_count]);
        }
        Ok(())
    }
    
    fn select_validator_for_block(&self, height: BlockHeight, shard: ShardId) -> Option<&Validator> {
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
}

// ConsensusState structure
struct ConsensusState {
    height: BlockHeight,
    epoch: u64,
    last_block_time: Timestamp,
    is_active: bool,
    validators: ValidatorPool,
}

impl ConsensusState {
    fn new() -> Self {
        ConsensusState {
            height: 0,
            epoch: 0,
            last_block_time: 0,
            is_active: false,
            validators: ValidatorPool::new(),
        }
    }
    
    fn get_epoch_for_height(&self, height: BlockHeight, blocks_per_epoch: BlockHeight) -> u64 {
        if blocks_per_epoch == 0 {
            return 0;
        }
        (height / blocks_per_epoch) as u64
    }
    
    fn is_epoch_start(&self, height: BlockHeight, blocks_per_epoch: BlockHeight) -> bool {
        if blocks_per_epoch == 0 {
            return false;
        }
        height % blocks_per_epoch == 0
    }
}

// ConsensusConfig structure
struct ConsensusConfig {
    block_interval_ms: u64,
    finality_confirmations: BlockHeight,
    shard_count: ShardId,
    blocks_per_epoch: BlockHeight,
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

// Mock Consensus trait
trait Consensus {
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

// Reward schedule for validators
#[derive(Debug, Clone)]
struct RewardSchedule {
    base_block_reward: u64,
    per_transaction_reward: u64,
    validation_reward: u64,
    halving_interval: u64,
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

// DPoS consensus implementation
struct DPoSConsensus {
    config: ConsensusConfig,
    state: Arc<Mutex<ConsensusState>>,
    local_public_key: Option<Vec<u8>>,
    block_history: Arc<Mutex<HashMap<BlockHeight, Block>>>,
    reward_schedule: RewardSchedule,
    block_schedule: Arc<Mutex<HashMap<BlockHeight, HashMap<ShardId, Vec<u8>>>>>,
}

impl DPoSConsensus {
    fn new(config: ConsensusConfig) -> Self {
        DPoSConsensus {
            config,
            state: Arc::new(Mutex::new(ConsensusState::new())),
            local_public_key: None,
            block_history: Arc::new(Mutex::new(HashMap::new())),
            reward_schedule: RewardSchedule::default(),
            block_schedule: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn set_local_public_key(&mut self, public_key: Vec<u8>) {
        self.local_public_key = Some(public_key);
    }
    
    fn current_time_micros() -> Timestamp {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as Timestamp
    }
    
    fn is_block_production_time(&self, last_block_time: Timestamp) -> bool {
        let now = Self::current_time_micros();
        let elapsed = now - last_block_time;
        
        let interval_micros = self.config.block_interval_ms as u64 * 1000;
        
        elapsed >= interval_micros
    }
    
    fn add_block_to_history(&self, block: Block) {
        let mut history = self.block_history.lock().unwrap();
        let height = block.header.index;
        
        history.insert(height, block);
        
        if height > self.config.finality_confirmations {
            let oldest_to_keep = height - self.config.finality_confirmations;
            history.retain(|h, _| *h >= oldest_to_keep);
        }
    }
    
    fn calculate_block_reward(&self, block: &Block) -> u64 {
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
}

// Test functions
fn create_test_validator(id: u8, stake: u64) -> Validator {
    Validator::new(
        vec![id],
        vec![100 + id],
        vec![200 + id],
        stake,
    )
}

fn setup_consensus_with_validators() -> DPoSConsensus {
    let config = ConsensusConfig::default();
    let mut consensus = DPoSConsensus::new(config);
    
    // Add some test validators
    {
        let mut state = consensus.state.lock().unwrap();
        for i in 1..=10 {
            let mut validator = create_test_validator(i, i as u64 * 1000);
            validator.assign_shards(vec![i as u16 % 4]);
            state.validators.add_validator(validator).unwrap();
        }
    }
    
    consensus
}

fn test_reward_calculation() {
    let mut consensus = setup_consensus_with_validators();
    
    // Create a custom reward schedule for testing
    consensus.reward_schedule = RewardSchedule {
        base_block_reward: 100,
        per_transaction_reward: 5,
        validation_reward: 10,
        halving_interval: 1000,
    };
    
    // Create a block with no transactions
    let empty_block = Block::new(
        1,
        DPoSConsensus::current_time_micros(),
        vec![0; 32],
        vec![0],
    );
    
    // Calculate reward for an empty block
    let empty_reward = consensus.calculate_block_reward(&empty_block);
    assert_eq!(empty_reward, 100); // Should be just the base reward
    println!("Empty block reward: {}", empty_reward);
    
    // Create a block with transactions
    let mut block_with_tx = Block::new(
        1,
        DPoSConsensus::current_time_micros(),
        vec![0; 32],
        vec![0],
    );
    
    // Add shard data with transactions
    let shard_data = ShardData {
        shard_id: 0,
        transactions: vec![vec![1, 2, 3], vec![4, 5, 6]], // 2 transactions
        execution_proof: Vec::new(),
        validator_signatures: Vec::new(),
    };
    
    block_with_tx.add_shard_data(shard_data).unwrap();
    
    // Calculate reward for block with transactions
    let tx_reward = consensus.calculate_block_reward(&block_with_tx);
    assert_eq!(tx_reward, 110); // Base 100 + (2 transactions * 5 per tx)
    println!("Block with transactions reward: {}", tx_reward);
    
    // Test reward halving
    let halving_block = Block::new(
        1500, // After first halving interval
        DPoSConsensus::current_time_micros(),
        vec![0; 32],
        vec![0],
    );
    
    // Test the halving reward calculation
    let halving_reward = consensus.calculate_block_reward(&halving_block);
    assert_eq!(halving_reward, 50); // Half of the base reward after first halving interval
    println!("Halving block reward: {}", halving_reward);
    
    println!("Reward calculation tests passed!");
}

fn main() {
    test_reward_calculation();
}
