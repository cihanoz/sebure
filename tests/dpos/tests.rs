use crate::tests::dpos::consensus::Consensus;
use crate::tests::dpos::test_helpers::*;

// Run all DPoS consensus tests
pub fn run_tests() {
    println!("Running DPoS consensus tests...");
    
    test_validator_assignment();
    test_block_production();
    test_block_validation();
    test_validator_scheduling();
    test_reward_calculation();
    
    println!("All DPoS consensus tests passed!");
}

// Test validator assignment to shards
fn test_validator_assignment() {
    let mut consensus = setup_consensus_with_validators();
    consensus.init().unwrap();
    
    // Verify all validators are assigned to shards
    let state = consensus.state.lock().unwrap();
    for shard in 0..consensus.config.shard_count {
        let validators_for_shard: Vec<_> = state.validators.get_all_validators()
            .iter()
            .filter(|v| v.is_assigned_to_shard(shard))
            .collect();
            
        // Ensure each shard has validators
        assert!(!validators_for_shard.is_empty(), "Shard {} has no validators", shard);
    }
    
    println!("✓ Validator assignment test passed");
}

// Test block production
fn test_block_production() {
    let mut consensus = setup_consensus_with_validators();
    consensus.init().unwrap();
    
    // Set local public key to one of the validators
    let local_pubkey = vec![101]; // First validator's public key
    consensus.set_local_public_key(local_pubkey);
    
    // Produce a block for shard 0
    let block = consensus.produce_block(0, 0).unwrap();
    
    // Verify block properties
    assert_eq!(block.header.index, 0);
    assert_eq!(block.header.shard_identifiers, vec![0]);
    assert_eq!(block.shard_data.len(), 1);
    assert_eq!(block.shard_data[0].shard_id, 0);
    
    println!("✓ Block production test passed");
}

// Test block validation
fn test_block_validation() {
    let mut consensus = setup_consensus_with_validators();
    consensus.init().unwrap();
    
    // Produce a block for shard 0
    let block = consensus.produce_block(0, 0).unwrap();
    
    // Validate the block
    let validation_result = consensus.validate_block(&block);
    
    // The validation should pass because we're at height 0 and this is block 0
    assert!(validation_result.is_err(), "Validation should fail for block with same height");
    
    // Update state height to 0
    {
        let mut state = consensus.state.lock().unwrap();
        state.height = 0;
    }
    
    // Validate again - now it should pass
    consensus.validate_block(&block).unwrap();
    
    println!("✓ Block validation test passed");
}

// Test validator scheduling
fn test_validator_scheduling() {
    let mut consensus = setup_consensus_with_validators();
    consensus.init().unwrap();
    
    // Check scheduling for shard 0 at different heights
    let validator1 = consensus.get_next_validator(0, 0).unwrap();
    let validator2 = consensus.get_next_validator(1, 0).unwrap();
    
    // They should be different due to rotation
    assert!(validator1.id != validator2.id, "Validators should rotate between blocks");
    
    println!("✓ Validator scheduling test passed");
}

// Test reward calculation
fn test_reward_calculation() {
    let consensus = setup_consensus_with_validators();
    
    // Create a block with 10 transactions
    let mut block = consensus.produce_block(0, 0).unwrap();
    
    // Add some mock transactions
    block.shard_data[0].transactions = vec![vec![1]; 10];
    
    // Calculate reward
    let reward = consensus.calculate_block_reward(&block);
    
    // Base reward + per transaction reward
    let expected_reward = consensus.reward_schedule.base_block_reward + 
                          10 * consensus.reward_schedule.per_transaction_reward;
                          
    assert_eq!(reward, expected_reward, "Reward calculation is incorrect");
    
    println!("✓ Reward calculation test passed");
}
