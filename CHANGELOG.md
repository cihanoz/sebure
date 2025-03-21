# Changelog

All notable changes to the SEBURE Blockchain project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Project structure and baseline architecture
- Core module with blockchain data structures
- Comprehensive cryptography module with:
  - Ed25519 signature generation and verification
  - SHA-256 and BLAKE3 hashing utilities
  - Secure key generation and management
  - Key storage with encryption
  - Address derivation and validation with Base58 encoding
- Transaction mempool with:
  - Transaction prioritization by fee and type
  - Dependency tracking and management
  - Automatic expiration and cleanup
  - Shard-aware transaction organization
- Enhanced blockchain core with:
  - Chain validation logic for blocks and transactions
  - Genesis block generation with customizable parameters
  - Comprehensive block linking and verification
  - Transaction lifecycle management 
  - Cryptographic hash-based chain integrity verification
- Delegated Proof-of-Stake (DPoS) consensus mechanism with:
  - Validator selection algorithm based on stake and performance
  - Epoch-based validator rotation for block production
  - Block scheduling and production timing
  - Validator pool management with shard assignments
  - Transaction fee distribution and block rewards
  - Reward halving mechanism for inflation control
- Complete P2P network layer implementation with:
  - Structured message format with various message types
  - Efficient peer discovery with multiple methods (manual, DNS seeds, peer exchange)
  - Reliable TCP-based transport for message passing between nodes
  - Block announcement and propagation system
  - Transaction broadcasting with batching support
  - Peer management with connection tracking
- Storage module with chain and state databases including:
  - Block and transaction persistence
  - State database with account information
  - Efficient lookup and retrieval mechanisms
- CLI interface for basic blockchain operations including:
  - Node control commands
  - Transaction submission functionality
  - Blockchain explorer features
  - Development testing interfaces
- DPoS consensus testing framework with:
  - Validator management and rotation testing
  - Block production and validation simulation
  - Reward calculation verification
  - Consensus state management tests
  - Shard assignment validation
- FFI bindings for integration with other languages
- Flutter UI project structure for desktop and mobile interfaces
- CI/CD pipeline using GitHub Actions for automated testing and building
- Initial documentation (README, ARCHITECTURE, DEVELOPMENT, CONTRIBUTING)
- Complete implementation of core blockchain data structures (Block, Transaction)
- Blockchain state data model with account management
- Serialization utilities for binary and JSON formats
- Cross-shard state management capabilities
- Desktop application framework with:
  - Rust-Dart FFI bindings for core blockchain integration
  - Configuration storage system using SharedPreferences
  - Application lifecycle management with startup and shutdown sequences
  - Plugin architecture for extensibility with:
    - Plugin discovery and loading mechanism
    - Plugin manifest system for metadata
    - Plugin installation and management
    - Demonstration plugin implementation
- Background validation service with:
  - Multi-threaded architecture for transaction validation
  - Priority-based task scheduling system
  - Resource-controlled processing (CPU, memory limits)
  - Automatic service recovery with health monitoring
  - Comprehensive statistics collection and reporting
  - FFI bridge for cross-language communication
  - Flutter UI for service configuration and monitoring:
    - Real-time service status dashboard
    - Configuration interface for resource limits
    - Statistics visualization for performance metrics
    - Manual service control options

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - 2025-03-18

### Added
- Initial project setup
- Basic project structure
- Core data structure definitions
- Cryptographic utility implementations
- Consensus mechanism foundation
- Command-line interface skeleton
- FFI layer basics

## Types of changes
- **Added** for new features.
- **Changed** for changes in existing functionality.
- **Deprecated** for soon-to-be removed features.
- **Removed** for now removed features.
- **Fixed** for any bug fixes.
- **Security** in case of vulnerabilities.
