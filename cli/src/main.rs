//! # SEBURE Blockchain CLI
//!
//! Command-line interface for interacting with the SEBURE blockchain.

use clap::{Parser, Subcommand};
use colored::Colorize;
use sebure_core::{self, Consensus, ConsensusConfig, Network, NetworkConfig, Storage, StorageConfig};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// SEBURE Blockchain CLI
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Optional config file path
    #[clap(short, long, value_parser)]
    config: Option<PathBuf>,
    
    /// Data directory path
    #[clap(short, long, value_parser)]
    data_dir: Option<PathBuf>,
    
    /// Subcommands
    #[clap(subcommand)]
    command: Commands,
}

/// CLI Subcommands
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new blockchain
    Init {
        /// Network ID
        #[clap(long, default_value = "sebure-dev")]
        network_id: String,
        
        /// Shard count
        #[clap(long, default_value_t = 4)]
        shard_count: u16,
    },
    
    /// Start a node
    Start {
        /// Run as validator
        #[clap(long)]
        validator: bool,
        
        /// Listen address for P2P connections
        #[clap(long, default_value = "127.0.0.1:8765")]
        listen_addr: String,
        
        /// Bootstrap peers to connect to
        #[clap(long)]
        peers: Vec<String>,
    },
    
    /// Create a new account
    CreateAccount,
    
    /// Show account information
    ShowAccount {
        /// Account address
        address: String,
    },
    
    /// Send a transaction
    SendTransaction {
        /// Sender account address or index
        from: String,
        
        /// Recipient account address
        to: String,
        
        /// Amount to send
        amount: u64,
        
        /// Optional data to include
        #[clap(long)]
        data: Option<String>,
    },
    
    /// Show blockchain status
    Status,
    
    /// Show blockchain configuration
    Config,
}

/// Run the CLI
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize core library
    sebure_core::init()?;
    
    // Process command
    match cli.command {
        Commands::Init { network_id, shard_count } => {
            init_blockchain(cli.data_dir, network_id, shard_count)?;
        },
        Commands::Start { validator, listen_addr, peers } => {
            start_node(cli.data_dir, validator, listen_addr, peers)?;
        },
        Commands::CreateAccount => {
            create_account(cli.data_dir)?;
        },
        Commands::ShowAccount { address } => {
            show_account(cli.data_dir, address)?;
        },
        Commands::SendTransaction { from, to, amount, data } => {
            send_transaction(cli.data_dir, from, to, amount, data)?;
        },
        Commands::Status => {
            show_status(cli.data_dir)?;
        },
        Commands::Config => {
            show_config(cli.data_dir)?;
        },
    }
    
    Ok(())
}

/// Initialize a new blockchain
fn init_blockchain(
    data_dir: Option<PathBuf>,
    network_id: String,
    shard_count: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Initializing SEBURE blockchain...".bright_blue());
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from(".sebure"));
    println!("Data directory: {}", data_dir.display());
    
    // Create storage
    let storage_config = StorageConfig {
        data_dir: data_dir.to_string_lossy().to_string(),
        ..StorageConfig::default()
    };
    
    let storage = Storage::new(storage_config)?;
    
    // Create consensus
    let consensus_config = ConsensusConfig {
        shard_count,
        ..ConsensusConfig::default()
    };
    
    let mut consensus = sebure_core::consensus::ConsensusFactory::create(consensus_config);
    consensus.init()?;
    
    println!("{} blockchain initialized with:", "SEBURE".green());
    println!("  Network ID: {}", network_id);
    println!("  Shard count: {}", shard_count);
    
    Ok(())
}

/// Start a blockchain node
fn start_node(
    data_dir: Option<PathBuf>,
    validator: bool,
    listen_addr: String,
    peers: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Starting SEBURE blockchain node...".bright_blue());
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from(".sebure"));
    println!("Data directory: {}", data_dir.display());
    
    // Parse peer addresses
    let mut bootstrap_peers = vec![];
    for peer in peers {
        let peer_addr = peer.parse()
            .map_err(|e| format!("Invalid peer address {}: {}", peer, e))?;
        bootstrap_peers.push(peer_addr);
    }
    
    // Create network configuration
    let listen_addr = listen_addr.parse()
        .map_err(|e| format!("Invalid listen address: {}", e))?;
    
    let network_config = NetworkConfig {
        listen_addr,
        bootstrap_peers,
        ..NetworkConfig::default()
    };
    
    // Create storage configuration
    let storage_config = StorageConfig {
        data_dir: data_dir.to_string_lossy().to_string(),
        ..StorageConfig::default()
    };
    
    // Create and start components
    let network = Network::new(network_config);
    let storage = Storage::new(storage_config)?;
    
    // Start network
    network.start()?;
    
    println!("{} node started:", "SEBURE".green());
    println!("  Listening on: {}", listen_addr);
    println!("  Running as validator: {}", validator);
    println!("  Connected peers: {}", network.peer_count());
    
    // Keep the node running
    println!("Press Ctrl+C to stop the node.");
    let start_time = Instant::now();
    
    loop {
        // Simple blocking loop to keep the node running
        std::thread::sleep(Duration::from_secs(1));
        
        // Show runtime in minutes:seconds format
        let runtime = Instant::now().duration_since(start_time);
        let minutes = runtime.as_secs() / 60;
        let seconds = runtime.as_secs() % 60;
        print!("\rRuntime: {:02}:{:02}  Peers: {}  ", 
               minutes, seconds, network.peer_count());
        
        // Flush output
        std::io::Write::flush(&mut std::io::stdout())?;
    }
}

/// Create a new account
fn create_account(data_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Creating a new account...".bright_blue());
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from(".sebure"));
    
    // Generate a new key pair
    let keypair = sebure_core::crypto::signature::KeyPair::generate();
    let public_key = keypair.public_key();
    
    // For now, just print the public key as a hex string
    println!("{} new account created:", "SEBURE".green());
    println!("  Public key: 0x{}", hex::encode(&public_key));
    println!("  Private key: <sensitive information, not shown>");
    println!("\n{}", "WARNING: In a real implementation, the private key would be stored securely.".yellow());
    
    Ok(())
}

/// Show account information
fn show_account(data_dir: Option<PathBuf>, address: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Retrieving account information...".bright_blue());
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from(".sebure"));
    
    // Parse address
    let address_bytes = match address.strip_prefix("0x") {
        Some(hex_str) => hex::decode(hex_str)?,
        None => hex::decode(address)?,
    };
    
    // Create storage
    let storage_config = StorageConfig {
        data_dir: data_dir.to_string_lossy().to_string(),
        ..StorageConfig::default()
    };
    
    let storage = Storage::new(storage_config)?;
    
    // Retrieve account information
    let account_info = storage.state_db().get_account_info(&address_bytes)?;
    
    println!("{} account information:", "SEBURE".green());
    println!("  Address: 0x{}", hex::encode(&address_bytes));
    println!("  Balance: {} tokens", account_info.balance);
    println!("  Nonce: {}", account_info.nonce);
    println!("  Is contract: {}", account_info.is_contract);
    
    Ok(())
}

/// Send a transaction
fn send_transaction(
    data_dir: Option<PathBuf>,
    from: String,
    to: String,
    amount: u64,
    data: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Creating and sending transaction...".bright_blue());
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from(".sebure"));
    
    // Parse addresses
    let from_bytes = match from.strip_prefix("0x") {
        Some(hex_str) => hex::decode(hex_str)?,
        None => hex::decode(from)?,
    };
    
    let to_bytes = match to.strip_prefix("0x") {
        Some(hex_str) => hex::decode(hex_str)?,
        None => hex::decode(to)?,
    };
    
    // Create storage
    let storage_config = StorageConfig {
        data_dir: data_dir.to_string_lossy().to_string(),
        ..StorageConfig::default()
    };
    
    let storage = Storage::new(storage_config)?;
    
    // Create a transaction
    // In a real implementation, this would use the actual sender's key to sign
    let transaction = sebure_core::blockchain::Transaction::new_transfer(
        from_bytes.clone(), // sender public key
        0,                  // sender shard
        to_bytes.clone(),   // recipient address
        0,                  // recipient shard
        amount,
        10,                 // fee
        0,                  // nonce
    );
    
    // Show transaction details
    println!("{} transaction created:", "SEBURE".green());
    println!("  From: 0x{}", hex::encode(&from_bytes));
    println!("  To: 0x{}", hex::encode(&to_bytes));
    println!("  Amount: {} tokens", amount);
    println!("  Fee: 10 tokens");
    
    if let Some(data_str) = data {
        println!("  Data: {}", data_str);
    }
    
    println!("\n{}", "NOTE: In a real implementation, this transaction would be signed and broadcast to the network.".yellow());
    
    Ok(())
}

/// Show blockchain status
fn show_status(data_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Retrieving blockchain status...".bright_blue());
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from(".sebure"));
    
    // Create storage
    let storage_config = StorageConfig {
        data_dir: data_dir.to_string_lossy().to_string(),
        ..StorageConfig::default()
    };
    
    let storage = Storage::new(storage_config)?;
    
    // Get blockchain information
    let latest_height = storage.chain_store().get_latest_height().unwrap_or(0);
    let latest_hash = storage.chain_store().get_latest_hash()
        .map(|h| hex::encode(h))
        .unwrap_or_else(|| "N/A".to_string());
    
    println!("{} blockchain status:", "SEBURE".green());
    println!("  Latest height: {}", latest_height);
    println!("  Latest block hash: 0x{}", latest_hash);
    
    Ok(())
}

/// Show blockchain configuration
fn show_config(data_dir: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Retrieving blockchain configuration...".bright_blue());
    
    let data_dir = data_dir.unwrap_or_else(|| PathBuf::from(".sebure"));
    
    // Show default configurations
    let storage_config = StorageConfig::default();
    let network_config = NetworkConfig::default();
    let consensus_config = ConsensusConfig::default();
    
    println!("{} blockchain configuration:", "SEBURE".green());
    println!("  Data directory: {}", data_dir.display());
    
    println!("\nStorage configuration:");
    println!("  Chain DB path: {}", storage_config.chain_path);
    println!("  State DB path: {}", storage_config.state_path);
    println!("  Max open files: {}", storage_config.max_open_files);
    println!("  Cache size: {} MB", storage_config.cache_size);
    
    println!("\nNetwork configuration:");
    println!("  Default listen address: {}", network_config.listen_addr);
    println!("  Max peers: {}", network_config.max_peers);
    println!("  Announce interval: {} seconds", network_config.announce_interval);
    println!("  Connection timeout: {} seconds", network_config.connection_timeout);
    
    println!("\nConsensus configuration:");
    println!("  Validators per pool: {}", consensus_config.validators_per_pool);
    println!("  Blocks per epoch: {}", consensus_config.blocks_per_epoch);
    println!("  Block interval: {} ms", consensus_config.block_interval_ms);
    println!("  Min stake: {}", consensus_config.min_stake);
    println!("  Shard count: {}", consensus_config.shard_count);
    println!("  Optimistic validation: {}", consensus_config.optimistic_validation);
    println!("  Finality confirmations: {}", consensus_config.finality_confirmations);
    
    Ok(())
}
