# SEBURE Blockchain Context

## Project Overview
SEBURE is a next-generation blockchain platform designed with:
- Adaptive scaling for high performance
- Environmental efficiency through optimized validation
- User-centric design for both desktop validators and mobile wallet users

Key features:
- Rust-based blockchain core
- Delegated Proof-of-Stake (DPoS) consensus
- Multi-layered architecture with parallel processing
- Sharded state database for scalability
- Desktop (Mac/Linux) and mobile (iOS/Android) interfaces

## Core Architecture

### System Components
1. **Blockchain Core**
   - Immutable ledger with hash-based validation
   - Transaction handling and verification
   - Multi-layered architecture with specialized execution paths

2. **Consensus Mechanism**
   - Delegated Proof-of-Stake (DPoS)
   - Validator pools for high throughput
   - Parallel block validation

3. **Network Layer**
   - Mesh network topology
   - Bloom-filter-based transaction propagation
   - Adaptive bandwidth allocation
   - Hierarchical node organization with supernodes
   - Node role assignment (validator, block producer, router)
   - Tiered routing with optimized paths
   - Reputation management system

4. **State Management**
   - Sharded state database
   - Parallel state computation
   - Multi-level caching

### Node Architecture (Desktop)
- Background validation engine
- Resource management system
- Node storage with efficient indexing
- Desktop UI dashboard with:
  - Node status indicators
  - Resource usage monitoring
  - Validation controls
  - Network statistics

### Wallet Architecture (Mobile)
- Secure key management
- Transaction services:
  - Creation and signing
  - Fee estimation
  - History tracking
- UI components:
  - Balance display
  - Send/receive interface
  - QR code support

## Development Status

### Completed Features
- Core blockchain data structures
- Cryptographic foundation (Ed25519, SHA-256, BLAKE3)
- Basic chain management
- DPoS consensus mechanism
- Advanced network layer with:
  - Hierarchical node organization
  - Role assignment system
  - Tiered routing
  - Reputation management
- CLI interface
- Desktop application framework
- Background validation service
- Wallet key management
- Sharding implementation with:
  - State synchronization
  - Cross-shard communication
  - State verification
  - Validator signature verification

### In Progress
- Advanced network topology
- Resource management system
- Desktop UI dashboard
- Transaction services
- Wallet UI components

### Future Milestones
1. State database implementation
2. Layer-2 integration
3. Smart contract foundation
4. Advanced governance features

## Technical Specifications

### Performance Requirements
- **Transaction Throughput**: 10,000+ TPS
- **Block Time**: 2 seconds
- **Confirmation Time**: <6 seconds
- **Finality**: 6-9 seconds

### Security Features
- Ed25519 signatures
- SHA-256 hashing
- Secure key storage
- Node authentication
- DDoS protection

## Development Environment

### Core Technologies
- **Primary Language**: Rust
- **UI Framework**: Flutter
- **Database**: LevelDB, LMDB
- **Network Protocol**: Custom UDP-based

### Development Tools
- Rust Analyzer
- Clippy (linting)
- Rustfmt (formatting)
- Criterion (benchmarking)

## Key Implementation Details

### Blockchain Core
- Block structure with standard headers
- Transaction validation and linking
- State management with sharding
- Parallel processing capabilities

### Consensus Mechanism
- Validator rotation and scheduling
- Reward distribution system
- Checkpoint-based validation
- Asynchronous finality confirmation

### Network Layer
- Peer discovery and management
- Message passing protocols
- Block and transaction propagation
- Network health monitoring

## Current Development Tasks

### Active Development Areas
- Advanced network topology
- Resource management system
- Desktop UI dashboard
- Transaction services
- Wallet key management
- Wallet UI components

### Upcoming Features
- State database implementation
- Dynamic sharding
- Layer-2 solutions
- Smart contract support
- Advanced governance

## Documentation References
- [Product Requirements Document](PRD.md)
- [Architecture Overview](ARCHITECTURE.md) 
- [Development Guide](DEVELOPMENT.md)
- [Task List](tasks.md)
- [Changelog](CHANGELOG.md)

## Task Documentation
For each development task, a corresponding TASK-X-CHANGES.md file will be automatically created in the donetasks folder to track:
- Files modified during the task
- Implementation details
- Testing results
- Relevant notes and observations
Example: When working on task 3.2, a donetasks/TASK-3.2-CHANGES.md file will be automatically created and maintained
