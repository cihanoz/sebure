// Module exports
pub mod types;
pub mod block;
pub mod validator;
pub mod consensus_state;
pub mod consensus;
pub mod reward;
pub mod dpos_consensus;
pub mod test_helpers;
pub mod tests;

// Re-export main test function
pub use tests::run_tests;
