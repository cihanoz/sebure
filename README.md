# SEBURE Blockchain

SEBURE is a next-generation blockchain platform designed with adaptive scaling, environmental efficiency, and user-centric design. It aims to deliver a blockchain solution that is both powerful for desktop validators and simple for mobile wallet users.

## Project Overview

SEBURE is designed as a dual-interface blockchain platform:
- Desktop applications (Mac and Linux) serving as full validation nodes
- Mobile applications (iOS and Android) functioning as lightweight wallet clients

The platform introduces innovative architectural features while maintaining simplicity and usability for end users.

## Key Features

- High-performance, Rust-based blockchain core
- Delegated Proof-of-Stake (DPoS) consensus mechanism
- Multi-layered architecture with parallel processing
- Sharded state database for scalability
- Environmental efficiency through optimized validation
- User-friendly interfaces for both node operators and wallet users

## Repository Structure

- `/core` - Core blockchain components written in Rust
- `/cli` - Command-line interface for blockchain interaction
- `/ffi` - Foreign Function Interface (FFI) bindings for integration with other languages
- `/docs` - Documentation resources
- `/ui` - User interface components built with Flutter for desktop and mobile
- `/.github` - GitHub workflows for CI/CD automation

## Getting Started

### Prerequisites

- Rust 1.68.0 or later
- Cargo package manager
- Git

### Building from Source

1. Clone the repository:
   ```
   git clone https://github.com/sebure/sebure-blockchain.git
   cd sebure-blockchain
   ```

2. Build the project:
   ```
   cargo build
   ```

3. Run the CLI:
   ```
   cargo run --bin sebure
   ```

### CLI Commands

- Initialize a new blockchain:
  ```
  sebure init --network-id sebure-dev --shard-count 4
  ```

- Start a node:
  ```
  sebure start --listen-addr 127.0.0.1:8765
  ```

- Create a new account:
  ```
  sebure create-account
  ```

- Show account information:
  ```
  sebure show-account <address>
  ```

- Send a transaction:
  ```
  sebure send-transaction --from <sender> --to <recipient> --amount <amount>
  ```

## Development Status

SEBURE Blockchain is currently in early development. The core components and architecture are being established.

### Completed Tasks

- [x] Project setup and architecture design
- [x] Core blockchain data structures
- [x] Comprehensive cryptography foundation
  - Ed25519 signature generation and verification
  - SHA-256 and BLAKE3 hashing utilities
  - Secure key generation and management
  - Address derivation with Base58 encoding
  - Key storage with encryption
- [x] Basic chain management
  - Chain validation logic
  - Genesis block generation
  - Block linking and verification
  - Transaction mempool implementation
  - Blockchain storage interface
- [x] Simplified consensus mechanism
  - Delegated Proof-of-Stake (DPoS) implementation
  - Validator selection and rotation
  - Block production scheduling
  - Validation rules
  - Reward distribution
- [x] Minimal network layer
  - P2P network protocol design
  - Node discovery mechanism
  - Message passing between nodes
  - Block propagation
  - Transaction broadcasting
- [x] Simple command-line interface
  - CLI node implementation for development testing
  - Basic node control commands
  - Transaction submission via CLI
  - Blockchain explorer functionality
  - Comprehensive DPoS consensus testing framework
- [x] Complete blockchain data structure implementation
- [x] State and account model implementation
- [x] Serialization/deserialization utilities
- [x] Desktop application framework
  - Flutter desktop application structure
  - Rust-Dart FFI bindings
  - Application lifecycle management
  - Configuration storage system
  - Plugin architecture for extensibility
- [x] Background validation service
  - Priority-based task scheduling system
  - Resource-controlled transaction validation
  - Inter-Process Communication (IPC) via FFI
  - Automatic service recovery with health monitoring
  - Comprehensive service diagnostics and statistics
  - UI for configuration and management

### In Progress

- [x] Resource management system
  - CPU usage monitoring and control
  - Memory allocation management
  - Network bandwidth controls
  - Disk space management
  - Resource allocation settings interface
- [x] Desktop UI Dashboard
  - Node status dashboard with real-time indicators
  - Resource usage monitoring displays with interactive charts
  - Network statistics visualization
  - Validation settings controls
  - User preference system
- [x] Transaction Services
  - Transaction creation and signing functionality
  - Fee estimation algorithm with multiple models
  - Transaction history tracking and caching
  - Balance calculation and management
  - Transaction validation and submission to mempool
  - FFI bindings for cross-language integration
  - Flutter service layer for UI integration
- [x] Wallet Key Management
  - Secure key generation with BIP-39 mnemonic support
  - Encrypted storage for private keys
  - Key backup and recovery mechanism using BIP-39 mnemonics
  - Multi-signature capability with M-of-N schemes
  - Hierarchical deterministic wallet support (BIP-32/BIP-44)
- [ ] State database implementation

For a complete list of planned tasks, see [tasks.md](tasks.md).

## Documentation

- [Product Requirements Document (PRD)](PRD.md)
- [Architecture Overview](ARCHITECTURE.md)
- [Development Guide](DEVELOPMENT.md)
- [Contribution Guidelines](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

This project is licensed under [LICENSE INFORMATION].

## Contact

For more information, please contact the SEBURE team at [CONTACT INFORMATION].
