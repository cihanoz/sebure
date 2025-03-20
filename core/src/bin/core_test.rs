//! # Core Test Utility
//! 
//! This utility demonstrates the core data structures and serialization capabilities.

use sebure_core::{
    Block, Transaction, Account, AccountType, GlobalState,
    serialize, deserialize, to_json, from_json, SerializationFormat,
};
use sebure_core::types::{ShardId, Timestamp};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time_micros() -> Timestamp {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    
    // Convert to microseconds
    since_epoch.as_secs() * 1_000_000 + since_epoch.subsec_micros() as u64
}

fn print_separator() {
    println!("\n{}\n", "-".repeat(80));
}

fn main() {
    println!("SEBURE Blockchain Core Data Structures Test");
    print_separator();
    
    // Create and test a block
    let timestamp = current_time_micros();
    let previous_hash = vec![0; 32];
    let shard_ids = vec![0, 1];
    
    let block = Block::new(1, timestamp, previous_hash, shard_ids);
    
    println!("Block created:");
    println!("  Index: {}", block.header.index);
    println!("  Timestamp: {}", block.header.timestamp);
    println!("  Shards: {:?}", block.header.shard_identifiers);
    
    print_separator();
    
    // Serialize and deserialize the block
    let serialized_block = serialize(&block, SerializationFormat::Binary).unwrap();
    println!("Block serialized to {} bytes", serialized_block.len());
    
    let deserialized_block: Block = deserialize(&serialized_block, SerializationFormat::Binary).unwrap();
    println!("Block deserialized successfully");
    println!("  Index matches: {}", deserialized_block.header.index == block.header.index);
    
    // JSON representation
    let block_json = to_json(&block).unwrap();
    println!("\nBlock JSON representation:");
    println!("{}", block_json);
    
    print_separator();
    
    // Create and test a transaction
    let sender_key = vec![1; 32];
    let recipient_addr = vec![2; 20];
    
    let tx = Transaction::new_transfer(
        sender_key,
        0, // sender shard
        recipient_addr,
        1, // recipient shard
        1000, // amount
        10, // fee
        0, // nonce
    );
    
    println!("Transaction created:");
    println!("  Type: {:?}", tx.transaction_type);
    println!("  Amount: {}", tx.amount);
    println!("  Fee: {}", tx.fee);
    
    // Serialize and deserialize the transaction
    let serialized_tx = serialize(&tx, SerializationFormat::Binary).unwrap();
    println!("\nTransaction serialized to {} bytes", serialized_tx.len());
    
    let deserialized_tx: Transaction = deserialize(&serialized_tx, SerializationFormat::Binary).unwrap();
    println!("Transaction deserialized successfully");
    println!("  Amount matches: {}", deserialized_tx.amount == tx.amount);
    
    print_separator();
    
    // Create and test an account
    let account_addr = vec![3; 20];
    let account = Account::new_user(account_addr.clone(), 0, timestamp);
    
    println!("Account created:");
    println!("  Type: {:?}", account.account_type);
    println!("  Balance: {}", account.balance);
    println!("  Nonce: {}", account.nonce);
    
    // Create a contract account
    let contract_addr = vec![4; 20];
    let contract_code = vec![5, 6, 7, 8];
    let contract = Account::new_contract(contract_addr.clone(), contract_code, 0, timestamp);
    
    println!("\nContract account created:");
    println!("  Type: {:?}", contract.account_type);
    println!("  Is contract: {}", contract.is_contract());
    
    print_separator();
    
    // Create and test global state
    let mut global_state = GlobalState::new();
    
    println!("Global state created:");
    println!("  Block height: {}", global_state.block_height);
    println!("  Total accounts: {}", global_state.total_accounts);
    
    println!("\nComplete test finished successfully!");
}
