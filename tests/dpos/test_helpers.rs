use crate::tests::dpos::validator::Validator;
use crate::tests::dpos::dpos_consensus::DPoSConsensus;
use crate::tests::dpos::consensus::ConsensusConfig;

// Test helpers

/// Create a test validator with default settings
pub fn create_test_validator(id: u8, stake: u64) -> Validator {
    Validator::new(
        vec![id],
        vec![100 + id],
        vec![200 + id],
        stake,
    )
}

/// Setup consensus with test validators
pub fn setup_consensus_with_validators() -> DPoSConsensus {
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
