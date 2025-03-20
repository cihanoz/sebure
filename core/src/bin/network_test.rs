//! Test the network functionality
//!
//! This is a simple test script to verify that our network layer works correctly

use sebure_core::network::{Network, NetworkConfig, MessageType, Message};
use sebure_core::types::Priority;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::time::Duration;
use std::thread;

/// Create a test message
fn create_test_message() -> Message {
    Message::new(
        MessageType::NetworkHealth,
        vec![1, 2, 3, 4], // Sample data
        None,
        Priority::Normal,
        vec![10, 11, 12], // Sample sender ID
    )
}

fn main() {
    // Initialize logger
    env_logger::init();
    
    println!("Starting SEBURE Network Layer Test");
    
    // Create peer addresses
    let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8765);
    let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8766);
    
    // Create network configs
    let mut config1 = NetworkConfig::default();
    config1.listen_addr = addr1;
    config1.bootstrap_peers = vec![addr2];
    
    let mut config2 = NetworkConfig::default();
    config2.listen_addr = addr2;
    config2.bootstrap_peers = vec![addr1];
    
    // Create networks
    let mut network1 = Network::new(config1);
    let mut network2 = Network::new(config2);
    
    // Start networks
    if let Err(e) = network1.start() {
        println!("Failed to start network1: {}", e);
        return;
    }
    
    if let Err(e) = network2.start() {
        println!("Failed to start network2: {}", e);
        return;
    }
    
    println!("Networks started successfully");
    
    // Give them some time to connect
    println!("Waiting for peer discovery...");
    thread::sleep(Duration::from_secs(3));
    
    // Process networks to discover peers
    if let Err(e) = network1.process() {
        println!("Failed to process network1: {}", e);
        return;
    }
    
    if let Err(e) = network2.process() {
        println!("Failed to process network2: {}", e);
        return;
    }
    
    // Wait for connections to establish
    thread::sleep(Duration::from_secs(1));
    
    // Check if peers are connected
    println!("Network1 peer count: {}", network1.peer_count());
    println!("Network2 peer count: {}", network2.peer_count());
    
    // Create a test message
    let message = create_test_message();
    
    // Broadcast the message from network1
    println!("Broadcasting message from network1...");
    if let Err(e) = network1.broadcast(message) {
        println!("Failed to broadcast message: {}", e);
    } else {
        println!("Message broadcasted successfully");
    }
    
    // Wait for message processing
    thread::sleep(Duration::from_secs(1));
    
    // Clean up
    if let Err(e) = network1.stop() {
        println!("Failed to stop network1: {}", e);
    }
    
    if let Err(e) = network2.stop() {
        println!("Failed to stop network2: {}", e);
    }
    
    println!("Networks stopped successfully");
    println!("Test completed");
}
