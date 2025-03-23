# SEBURE Blockchain Development Guide

This guide provides detailed information for developers who want to contribute to the SEBURE Blockchain project. It covers setting up a development environment, code organization, coding standards, testing practices, and the development workflow.

## Development Environment Setup

### Prerequisites

- **Rust**: Latest stable version (1.68.0+)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  
- **Flutter**: Latest stable version for UI development
  ```bash
  # macOS/Linux
  git clone https://github.com/flutter/flutter.git -b stable
  export PATH="$PATH:`pwd`/flutter/bin"
  flutter doctor
  ```

- **Git**: For version control
  ```bash
  # For Ubuntu/Debian
  sudo apt install git
  
  # For macOS (using Homebrew)
  brew install git
  ```
  
- **Cargo**: Rust's package manager (installed with Rust)

- **Additional dependencies**:
  - OpenSSL development libraries
  - pkg-config
  - CMake (for some dependencies)
  
  ```bash
  # For Ubuntu/Debian
  sudo apt install pkg-config libssl-dev cmake
  
  # For macOS
  brew install openssl cmake
  ```

### Getting the Source Code

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR-USERNAME/sebure-blockchain.git
   cd sebure-blockchain
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/sebure/sebure-blockchain.git
   ```

### Building the Project

1. Build in development mode:
   ```bash
   cargo build
   ```

2. Run tests:
   ```bash
   cargo test
   ```

3. Build in release mode (optimized):
   ```bash
   cargo build --release
   ```

### Development Tools

We recommend the following tools to enhance your development experience:

- **Rust Analyzer**: Provides IDE support for Rust
- **Clippy**: Lint tool for catching common mistakes
  ```bash
  rustup component add clippy
  cargo clippy
  ```
- **Rustfmt**: Code formatter
  ```bash
  rustup component add rustfmt
  cargo fmt
  ```

## Project Structure

The SEBURE Blockchain codebase is organized into several crates:

### Core Crate (`/core`)

Contains the fundamental blockchain components:

- `src/blockchain/`: Block and transaction implementation
  - `block.rs`: Block structure implementation
  - `transaction.rs`: Transaction structure implementation
  - `state.rs`: Account and blockchain state model
  - `mempool.rs`: Transaction mempool implementation
  - `mod.rs`: Blockchain management and validation
- `src/consensus/`: Consensus mechanism (DPoS)
  - `dpos.rs`: Delegated Proof-of-Stake implementation
  - `validator.rs`: Validator management and selection
  - `mod.rs`: Consensus interface and common functionality
- `src/crypto/`: Cryptographic utilities
  - `hash.rs`: Hashing utilities (SHA-256, BLAKE3)
  - `signature.rs`: Ed25519 signature generation and verification
  - `address.rs`: Address derivation and verification with Base58 encoding
  - `keystore.rs`: Secure key storage with encryption
  - `hdwallet.rs`: Hierarchical deterministic wallet implementation (BIP-32/BIP-44)
  - `bip39_wordlist.rs`: BIP-39 mnemonic wordlist for wallet recovery
  - `mod.rs`: Module exports and utility functions
- `src/network/`: P2P networking
  - `mod.rs`: Network module coordination and API
  - `message.rs`: Network message definition and serialization
    - Binary protocol implementation using bincode
    - Compact binary format for transactions
    - Length-prefixed encoding for variable-length data
    - Little-endian numeric values
    - Protocol versioning support
    - Comprehensive serialization tests
    - Fuzz testing for random input validation
    - Performance benchmarks
  - `protocol.rs`: P2P network protocol implementation
  - `peer.rs`: Peer connection management
  - `discovery.rs`: Peer discovery mechanisms
  - `transport.rs`: Message transport layer
  - `node_communication.rs`: Block and transaction broadcasting
  - `mesh_topology.rs`: Mesh network topology implementation
  - `bloom_filter.rs`: Bloom filter for efficient transaction propagation
  - `fast_path.rs`: Fast path routing for high-priority messages
  - `bandwidth_manager.rs`: Adaptive bandwidth allocation
- `src/storage/`: Blockchain data storage
  - `chain_store.rs`: Storage for blockchain data
  - `state_db.rs`: State database for account balances and contract state
- `src/types/`: Common types and utilities
- `src/utils/`: Utility functions
  - `serialization.rs`: Serialization/deserialization utilities
- `src/lib.rs`: Main library entry point
- `src/bin/`: Binary executables
  - `core_test.rs`: Test utility demonstrating core data structures
  - `network_test.rs`: Test utility for network components

### CLI Crate (`/cli`)

Command-line interface for interacting with the blockchain:

- `src/main.rs`: CLI entry point
- `src/commands/`: Implementations of CLI commands

### FFI Crate (`/ffi`)

Foreign Function Interface for integration with other languages:

- `src/lib.rs`: FFI entry point and bindings
  - Core blockchain interface
  - Network management
  - Storage access
  - Account operations
  - Error handling utilities

### UI Components (`/ui`)

Flutter-based user interfaces for desktop and mobile:

- `/lib`: Dart code for the UI
  - `main.dart`: Application entry point
  - `/src/ffi`: FFI bindings to the Rust core
  - `/src/models`: Data models for the UI
  - `/src/screens`: Application screens
    - `home_screen.dart`: Main dashboard screen
    - `validation_settings_screen.dart`: Validation service configuration
    - `preferences_screen.dart`: User preferences and settings
  - `/src/services`: Backend services
    - `blockchain_service.dart`: Interface to core blockchain functionality
    - `transaction_service.dart`: Transaction creation, signing, and management
    - `validation_service.dart`: Background validation service management
    - `config_service.dart`: User preferences and configuration management
  - `/src/plugin`: Plugin system architecture
  - `/src/utils`: Utility functions
  - `/src/widgets`: Reusable UI components
    - `resource_usage_chart.dart`: Charts for CPU, memory, network, and disk usage
    - `node_control_panel.dart`: Node status and control interface
    - `network_statistics.dart`: Network metrics and visualizations

## Coding Standards

### Rust Style Guidelines

1. **Formatting**: Follow the standard Rust style as enforced by `rustfmt`.
   ```bash
   cargo fmt
   ```

2. **Linting**: Use Clippy to catch common mistakes.
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```

3. **Naming Conventions**:
   - Use `snake_case` for variables, functions, and file names
   - Use `CamelCase` for types, traits, and enums
   - Use `SCREAMING_SNAKE_CASE` for constants

4. **Documentation**: Document all public items with doc comments.
   - Every public function, struct, trait, and module should have a doc comment
   - Include examples in doc comments where appropriate

### Code Organization

1. **Modular Design**: 
   - Keep modules focused on a single responsibility
   - Use clear boundaries between components
   - Minimize dependencies between modules

2. **Error Handling**:
   - Use the `Result` type for operations that can fail
   - Provide meaningful error messages
   - Use custom error types where appropriate

3. **Configuration**:
   - Use structs with builder patterns for configurable components
   - Provide sensible defaults

4. **UI Design Pattern**:
   - Utilize Provider for state management
   - Separate UI from business logic through services
   - Use dependency injection for service management
   - Follow the Material Design guidelines

5. **FFI Architecture**:
   - Create well-defined bindings using FFI
   - Handle memory management carefully
   - Use specific error codes for cross-language error handling
   - Wrap unsafe FFI calls in safe Dart interfaces

## Testing Practices

### Unit Tests

- Write unit tests for all modules
- Place tests in a `tests` submodule within each module
- Use the `#[test]` attribute for test functions
- Aim for high test coverage (80%+)

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_creation() {
        // Test code here
    }
}
```

### Integration Tests

- Place integration tests in the `tests/` directory
- Focus on testing component interactions
- Use realistic scenarios

### DPoS Consensus Testing Framework

The project includes a comprehensive testing framework for the Delegated Proof-of-Stake consensus mechanism in the `tests/dpos/` directory:

- **Core Test Components**:
  - `block.rs`: Block creation and validation tests
  - `consensus.rs`: Consensus mechanism interface testing
  - `consensus_state.rs`: State management and transitions
  - `dpos_consensus.rs`: Complete DPoS implementation tests
  - `reward.rs`: Reward calculation and distribution tests
  - `test_helpers.rs`: Utilities for creating test validators and consensus setups
  - `tests.rs`: Main test runner with individual test functions
  - `types.rs`: Common types for testing
  - `validator.rs`: Validator management and selection testing

- **Running Consensus Tests**:
  ```bash
  cargo run --bin sebure_test
  ```

- **Writing DPoS Tests**:
  - Use the test helper functions to set up consensus instances
  - Create test validators with different stake amounts
  - Test validator assignment to shards
  - Validate block production and verification
  - Test reward calculation with various transaction counts

### Performance Tests

- Write benchmarks for performance-critical code
- Use the `criterion` crate for benchmarking
- Monitor performance changes over time

## Flutter UI Development

### Desktop UI Dashboard

The Desktop UI Dashboard provides a comprehensive interface for monitoring and controlling the blockchain node:

1. **Node Status Dashboard**:
   - Real-time node status indicators (running, stopped, syncing)
   - Start/stop controls for node operation
   - Connected peers and transaction count display
   - Auto-start configuration option

2. **Resource Usage Monitoring**:
   - Interactive charts for CPU, memory, network, and disk usage
   - Real-time data visualization with FL Chart library
   - Resource usage trends and metrics
   - Visual indicators for resource limits

3. **Network Statistics Visualization**:
   - Transaction volume charts with historical data
   - Connected peers metrics and visualization
   - Block height and validation statistics
   - Network health indicators

4. **Validation Settings Controls**:
   - Configuration interface for validation service parameters
   - CPU and memory usage limit controls
   - Task queue and batch size configuration
   - Service status monitoring with visual indicators
   - Manual control options (start, stop, pause, resume)

5. **User Preferences System**:
   - Theme selection (light, dark, system)
   - Resource limit configuration
   - Network settings management
   - Custom peer configuration
   - Log level selection

### Desktop Application Framework

The desktop application framework consists of several key components:

1. **FFI Layer**:
   - **`sebure_ffi.dart`**: Provides type-safe bindings to the Rust code
   - Error handling with specific error codes
   - Resource management for memory-safe FFI calls

2. **Service Layer**:
   - **`BlockchainService`**: Interface to core blockchain features
   - **`ConfigService`**: Persistent storage for application settings
   - **`ValidationService`**: Background processing service management
   - **`TransactionService`**: Transaction creation and management
   - **`ContactService`**: Contact management for the wallet
   - **`QrService`**: QR code generation and scanning
   - Singleton pattern for shared service instances
   - Asynchronous APIs with proper error handling

3. **Plugin Architecture**:
   - **`PluginManager`**: Handles plugin discovery, loading, and lifecycle
   - **`PluginManifest`**: Metadata for plugins (ID, version, dependencies)
   - **`SeburePlugin`**: Base class for all plugins
   - Plugin installation/uninstallation management

4. **Application Lifecycle**:
   - Structured initialization sequence
   - Resource cleanup on shutdown
   - State management throughout application lifecycle
   - Error handling and recovery

5. **Background Validation Service**:
   - Resource-controlled transaction processing
   - Configurable CPU and memory usage limits
   - Task scheduling with priority-based queuing
   - Health monitoring and automatic recovery
   - Statistics collection and reporting
   - User interface for configuration and monitoring

6. **Transaction Service**:
   - Transaction creation with comprehensive metadata
   - Cryptographic signing using Ed25519
   - Fee estimation with multiple models
   - Transaction history tracking and management
   - Balance calculation and monitoring
   - Transaction validation and submission
   - FFI integration for cross-language access
   - Flutter service layer with reactive state management
   - Background processing with compute isolates
   - Mock data support for development and testing

7. **Resource Management System**:
   - System resource monitoring (CPU, memory, network, disk)
   - Resource usage control with configurable limits
   - Adaptive batch processing based on available resources
   - Resource status reporting and recommendations
   - Resource reservation for critical operations
   - Integration with validation service for optimized processing

8. **Wallet UI Components**:
   - **Balance Display**: Shows current balance with token denomination
   - **Send Transaction Interface**: Form for creating and sending transactions
   - **Receive Interface**: QR code generation for receiving funds
   - **Transaction History**: Filterable list of past transactions
   - **Contact Management**: System for storing and managing recipient addresses

### Working with FFI

When extending or modifying the FFI layer:

1. Add Rust functions in `ffi/src/lib.rs` with `#[no_mangle]` and appropriate C-compatible types
2. Update `ui/lib/src/ffi/sebure_ffi.dart` to add the corresponding Dart bindings
3. Create type-safe wrappers in service classes that handle errors and memory management
4. Use the services in the UI rather than calling FFI directly

### Plugin Development

To create a new plugin:

1. Create a class that extends `SeburePlugin`
2. Implement the `initialize()` and `shutdown()` methods
3. Create a manifest JSON file with plugin metadata
4. Install the plugin using the `PluginManager.installPlugin()` method

Example plugin structure:
```
my-plugin/
  |- manifest.json
  |- main.dart
  |- assets/
```

### Configuration Management

The configuration system provides:

1. Persistent storage of application settings
2. Type-safe access to configuration values
3. Default values for first-time initialization
4. Advanced configuration with JSON for complex structures

When adding new settings:

1. Add keys in the `ConfigService` class
2. Create appropriate getters and setters
3. Add default values in the `_setDefaultsIfNeeded` method
4. Update UI to use the new configuration options

## Development Workflow

### Feature Development

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Implement the feature**:
   - Write tests first (TDD approach)
   - Implement the feature
   - Ensure all tests pass
   - Format and lint your code

3. **Commit your changes**:
   - Use meaningful commit messages
   - Follow the format: `[Component] Brief description of change`
   - Example: `[Consensus] Implement validator rotation mechanism`

4. **Keep your branch updated**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

5. **Push your changes**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create a pull request**:
   - Provide a clear description of the changes
   - Reference any relevant issues
   - Ensure CI checks pass

### Code Review Process

- All code changes require review
- Address review comments promptly
- Ensure all tests pass and CI checks are green
- Updates may be requested before merging

### CI/CD Pipeline

The project uses a CI/CD pipeline that:
- Builds the project on multiple platforms
- Runs unit and integration tests
- Checks formatting and linting
- Calculates test coverage
- Performs security analysis

## Debugging Tips

### Logging

The project uses the `log` crate for logging:
- Use appropriate log levels (`error`, `warn`, `info`, `debug`, `trace`)
- Enable logging in applications using the `env_logger` crate

Example:
```rust
use log::{info, debug, error};

fn process_block(block: &Block) -> Result<()> {
    info!("Processing block #{}", block.header.index);
    
    debug!("Block details: {:?}", block);
    
    if let Err(e) = validate_block(block) {
        error!("Block validation failed: {}", e);
        return Err(e.into());
    }
    
    Ok(())
}
```

### Common Issues

1. **Build Failures**:
   - Check for missing dependencies
   - Ensure you have the latest Rust version
   - Run `cargo clean` and try building again

2. **Test Failures**:
   - Use `cargo test -- --nocapture` to see output
   - Debug specific tests with `cargo test test_name -- --nocapture`

3. **Performance Issues**:
   - Use the Rust profiler: `cargo profiler callgrind --bin target`
   - Check memory usage with `valgrind`

## Documentation

### Code Documentation

- Document all public items with doc comments
- Include examples in doc comments
- Run `cargo doc --open` to generate and view documentation

### External Documentation

- Update relevant markdown files when making significant changes
- Keep diagrams up-to-date with the architecture

## Release Process

1. **Version Bumping**:
   - Update version in `Cargo.toml` files
   - Follow semantic versioning (MAJOR.MINOR.PATCH)

2. **Changelog Updates**:
   - Document all significant changes in the CHANGELOG.md file
   - Group changes by type (New Features, Improvements, Bug Fixes)

3. **Release Tagging**:
   - Create a git tag for each release
   - Tag format: `v1.0.0`

4. **Publishing**:
   - Publish crates to crates.io if applicable
   - Create GitHub releases

## Getting Help

- Check the existing issues on GitHub
- Ask questions in the project's discussion forum
- Join the developer chat room (details in README)

---

Happy coding! Your contributions are valuable to making SEBURE Blockchain a successful project.
