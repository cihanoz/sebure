# SEBURE Blockchain Technical Reference

This technical reference document provides detailed information about the SEBURE Blockchain implementation, including data structures, algorithms, protocols, and APIs.

## Core Data Structures

### Block

The fundamental unit of the blockchain, containing a set of transactions and metadata.

```rust
struct BlockHeader {
    index: u64,                     // Block height
    timestamp: u64,                 // Microseconds since UNIX epoch
    previous_hash: Vec<u8>,         // Hash of the previous block (32 bytes)
    state_root: Vec<u8>,            // Merkle root of the state (32 bytes)
    transaction_root: Vec<u8>,      // Merkle root of transactions (32 bytes)
    receipt_root: Vec<u8>,          // Merkle root of transaction receipts (32 bytes)
    validator_merkle: Vec<u8>,      // Merkle root of validator set (32 bytes)
    shard_identifiers: Vec<u16>,    // IDs of shards included in this block
    aggregated_signature: Vec<u8>,  // BLS aggregated signature (96 bytes)
}

struct ShardData {
    shard_id: u16,                  // Shard identifier
    transactions: Vec<Transaction>, // Transactions in this shard
    execution_proof: Vec<u8>,       // Proof of execution
    validator_signatures: Vec<Vec<u8>>, // Signatures from shard validators
}

struct Block {
    header: BlockHeader,            // Block header
    shard_data: Vec<ShardData>,     // Data for each shard
    cross_shard_receipts: Vec<Receipt>, // Receipts for cross-shard transactions
    validator_set: Vec<ValidatorRef>,    // Current set of validators
}
```

### Transaction

A record of value transfer or smart contract execution.

```rust
enum TransactionType {
    Transfer,           // Simple token transfer
    ContractDeployment, // Deploy a smart contract
    ContractExecution,  // Execute a smart contract function
    ValidatorAction,    // Validator-related actions (stake, unstake, etc.)
    Governance,         // Governance-related actions (voting, proposals)
    System,             // System-level operations
}

struct Transaction {
    id: Vec<u8>,                   // Transaction hash/ID (32 bytes)
    version: u8,                   // Transaction format version
    type: TransactionType,         // Type of transaction
    sender_public_key: Vec<u8>,    // Sender's public key (32 bytes)
    sender_shard: u16,             // Sender's shard
    recipient_address: Vec<u8>,    // Recipient's address (20 bytes)
    recipient_shard: u16,          // Recipient's shard
    amount: u64,                   // Amount to transfer
    fee: u32,                      // Transaction fee
    gas_limit: u32,                // Maximum gas allowed
    nonce: u64,                    // Sender's transaction count
    timestamp: u64,                // Microseconds since UNIX epoch
    data: Vec<u8>,                 // Transaction-specific data
    dependencies: Vec<Vec<u8>>,    // IDs of dependent transactions
    signature: Vec<u8>,            // Transaction signature (64 bytes)
    execution_priority: u8,        // Execution priority (0-255)
}
```

### Receipt

The result of a transaction execution.

```rust
struct Receipt {
    transaction_id: Vec<u8>,        // Transaction hash/ID (32 bytes)
    status: u8,                     // Status code (0 = success, others = various failures)
    gas_used: u32,                  // Gas consumed by execution
    return_data: Vec<u8>,           // Data returned from execution
    logs: Vec<Log>,                 // Event logs generated
    shard_id: u16,                  // Shard where execution occurred
}

struct Log {
    address: Vec<u8>,              // Contract address (20 bytes)
    topics: Vec<Vec<u8>>,          // Indexed topics (32 bytes each)
    data: Vec<u8>,                 // Log data
}
```

## Consensus Mechanism

SEBURE uses a Delegated Proof-of-Stake (DPoS) consensus mechanism with the following characteristics:

### Validator Selection

- Validators are selected based on their stake amount
- A total of 21 validators are active per shard
- Validators are rotated at fixed intervals (epochs)
- A validator must have a minimum stake of 1,000 SEBURE tokens

### Block Production

- Block time is 2 seconds
- Block producers are scheduled deterministically
- Production follows a round-robin pattern within validator sets
- Blocks contain transactions from specific shards

### Finality

- A block is considered final after 3 confirmations (~6 seconds)
- Finality is achieved when 2/3+ of validators sign the block
- Checkpoints are created every 100 blocks for long-term finality

### Rewards and Penalties

- Block producers receive rewards for each block produced
- Validators who miss their scheduled blocks are penalized
- Malicious validators can be slashed (lose stake)
- Rewards are proportional to stake amount

## Network Protocol

### Message Types

```rust
enum MessageType {
    BlockAnnouncement,       // Announce new block availability
    BlockHeader,             // Send block header
    BlockBody,               // Send block body
    TransactionAnnouncement, // Announce new transaction
    TransactionBatch,        // Send batch of transactions
    ShardSyncRequest,        // Request shard state sync
    ShardStateResponse,      // Respond with shard state
    ValidatorHandshake,      // Validator peer handshake
    PeerDiscovery,           // Discover new peers
    PeerExchange,            // Exchange peer information
    StateSnapshot,           // State snapshot for syncing
    CheckpointVote,          // Vote for checkpoint finality
    NetworkHealth,           // Network health monitoring
}

struct Message {
    version: u8,             // Protocol version
    compression: bool,       // Whether data is compressed
    encryption: bool,        // Whether data is encrypted
    priority: u8,            // Message priority (0-255)
    type: MessageType,       // Message type
    shard_id: Option<u16>,   // Optional shard ID
    data: Vec<u8>,           // Message data (possibly compressed)
    checksum: [u8; 4],       // CRC32 checksum
    sender: Vec<u8>,         // Sender's node ID
    signature: Vec<u8>,      // Message signature
}
```

### Peer Management

- Peers are discovered via DNS seeds and peer exchange
- Maximum connections per node is configurable (default: 50)
- Peers are scored based on behavior (reliability, responsiveness)
- Low-scoring peers are disconnected
- Persistent peer connections are maintained with validators

## Cryptographic Primitives

### Signatures

- Ed25519 for transaction signatures
- BLS signatures for validator aggregation
- Schnorr signatures for advanced privacy features (future)

### Hashing

- SHA-256 for general purpose hashing
- BLAKE3 for high-performance operations
- Keccak-256 for smart contract compatibility

### Key Derivation

- PBKDF2 for key derivation from passphrases
- BIP-32/39/44 for hierarchical deterministic wallet support

## Sharding Architecture

Sharding divides the network state into multiple partitions to improve scalability.

### Shard Types

- **Transaction Shards**: Process different sets of transactions
- **State Shards**: Store different parts of the global state
- **Validator Shards**: Different validator sets for different shards

### Cross-Shard Communication

- Atomic cross-shard transactions using a two-phase commit protocol
- Cross-shard receipts for transaction verification
- Merkle proofs for cross-shard state validation

### Dynamic Resharding

- Shards can be split or merged based on load
- Resharding occurs at epoch boundaries
- State migration protocols ensure consistency

## Storage Layer

### Chain Store

Stores blockchain data including blocks and transactions.

- Block headers stored separately from bodies for quick access
- Transaction index for fast lookups
- LevelDB as the underlying key-value store

### State Database

Stores the current state including account balances and contract storage.

- Account state trie (Merkle-Patricia Trie)
- Contract storage tries
- LMDB for memory-mapped access to state data

## Smart Contract Support

_Note: Smart contract functionality is planned for future implementation._

- WebAssembly (WASM) execution environment
- Gas metering for resource control
- Deterministic execution guarantees
- Template-based contract creation

## API Reference

### RPC API

SEBURE provides a JSON-RPC API for interacting with the blockchain:

```
POST /v1/jsonrpc
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "blockchain.getBlockByHeight",
  "params": {"height": 12345},
  "id": 1
}
```

#### Chain Methods

- `blockchain.getBlockByHash(hash)`: Get block by hash
- `blockchain.getBlockByHeight(height)`: Get block by height
- `blockchain.getTransaction(id)`: Get transaction by ID
- `blockchain.getLatestBlock()`: Get the latest block
- `blockchain.getTransactionReceipt(id)`: Get receipt for transaction

#### Account Methods

- `account.getBalance(address)`: Get account balance
- `account.getNonce(address)`: Get account nonce
- `account.getHistory(address, limit, offset)`: Get transaction history

#### Transaction Methods

- `transaction.send(tx)`: Submit a signed transaction
- `transaction.estimateFee(tx)`: Estimate fee for a transaction
- `transaction.getStatus(id)`: Get transaction status

#### Validator Methods

- `validator.getList()`: Get list of active validators
- `validator.getInfo(address)`: Get validator information
- `validator.getRewards(address)`: Get validator rewards

## Performance Characteristics

- **Transaction Throughput**: 10,000+ TPS (target)
- **Block Time**: 2 seconds
- **Confirmation Time**: ~6 seconds
- **Storage Efficiency**: Optimized for minimal disk usage
- **Network Bandwidth**: <100MB/hour for full nodes
