import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
import '../ffi/sebure_ffi.dart';

/// Service for interacting with the SEBURE blockchain core through FFI
class BlockchainService {
  // Singleton instance
  static final BlockchainService _instance = BlockchainService._internal();
  static BlockchainService get instance => _instance;

  // Private constructor for singleton
  BlockchainService._internal();

  // FFI instance
  final _ffi = SebureFFI.instance;

  // Track if the node is running
  bool _isNodeRunning = false;

  // Path to data directory
  String? _dataDir;

  /// Initialize the blockchain service
  static Future<bool> initialize() async {
    debugPrint('Initializing SEBURE blockchain core...');
    final service = BlockchainService.instance;

    try {
      // Initialize FFI bindings
      final ffiInitialized = await service._ffi.initialize();
      if (!ffiInitialized) {
        debugPrint('Failed to initialize FFI bindings');
        return false;
      }

      // Create data directory
      final appDir = await getApplicationDocumentsDirectory();
      service._dataDir = '${appDir.path}/sebure_data';
      await Directory(service._dataDir!).create(recursive: true);

      // Initialize core components
      final coreResult = service._ffi.initCore();
      if (coreResult != ErrorCode.success) {
        debugPrint('Failed to initialize core: $coreResult');
        return false;
      }

      // Initialize storage
      final storageResult = service._ffi.initStorage(service._dataDir!);
      if (storageResult != ErrorCode.success &&
          storageResult != ErrorCode.alreadyInitialized) {
        debugPrint('Failed to initialize storage: $storageResult');
        return false;
      }

      debugPrint('SEBURE blockchain core initialized successfully');
      return true;
    } catch (e) {
      debugPrint('Error initializing blockchain service: $e');
      return false;
    }
  }

  /// Start the validation node
  Future<bool> startNode() async {
    if (_isNodeRunning) return true;

    debugPrint('Starting SEBURE node...');
    try {
      // Initialize network with a default listen address
      // In a real implementation, this would be configurable
      final networkResult = _ffi.initNetwork('127.0.0.1:9000');
      if (networkResult != ErrorCode.success &&
          networkResult != ErrorCode.alreadyInitialized) {
        debugPrint('Failed to initialize network: $networkResult');
        return false;
      }

      // Start network
      final startResult = _ffi.startNetwork();
      if (startResult != ErrorCode.success) {
        debugPrint('Failed to start network: $startResult');
        return false;
      }

      _isNodeRunning = true;
      return true;
    } catch (e) {
      debugPrint('Error starting node: $e');
      return false;
    }
  }

  /// Stop the validation node
  Future<bool> stopNode() async {
    if (!_isNodeRunning) return true;

    debugPrint('Stopping SEBURE node...');
    try {
      // In a production implementation, we would have a proper shutdown sequence
      // For now, we'll use the shutdown function that closes everything
      final result = _ffi.shutdown();
      if (result != ErrorCode.success) {
        debugPrint('Failed to stop node: $result');
        return false;
      }

      _isNodeRunning = false;
      return true;
    } catch (e) {
      debugPrint('Error stopping node: $e');
      return false;
    }
  }

  /// Get resource usage statistics
  Future<Map<String, double>> getResourceUsage() async {
    // In a real implementation, this would get actual resource usage
    // from the Rust core through FFI
    // For now, we'll return simulated data with some randomness to mimic changes

    // Simulate CPU usage between 5-25%
    final cpuUsage = 5.0 + (DateTime.now().millisecond % 20);

    // Simulate memory usage between 300-400 MB
    final memoryUsage = 300.0 + (DateTime.now().second % 100);

    // Simulate network usage between 0.5-2.0 MB/s
    final networkUsage = 0.5 + (DateTime.now().millisecond % 15) / 10;

    // Simulate disk usage
    final diskUsage =
        _dataDir != null ? await _calculateDiskUsage(_dataDir!) : 0.0;

    return {
      'cpu': cpuUsage,
      'memory': memoryUsage,
      'network': networkUsage,
      'disk': diskUsage,
    };
  }

  /// Calculate disk usage of a directory
  Future<double> _calculateDiskUsage(String dirPath) async {
    try {
      final dir = Directory(dirPath);
      if (!await dir.exists()) return 0.0;

      // For now, just return a simulated value
      // In a real implementation, we would recursively calculate directory size
      return 5.0 + (DateTime.now().second % 5);
    } catch (e) {
      debugPrint('Error calculating disk usage: $e');
      return 0.0;
    }
  }

  /// Get account balance
  Future<double> getBalance(String address) async {
    try {
      final result = _ffi.getAccountBalance(address);

      if (result.errorCode == ErrorCode.success && result.balance != null) {
        // Convert from smallest unit to main unit (similar to satoshi -> bitcoin)
        return result.balance! / 100000000;
      } else {
        debugPrint('Failed to get balance: ${result.errorCode}');

        // For development, return a mock balance
        return 100.0;
      }
    } catch (e) {
      debugPrint('Error getting balance: $e');
      return 0.0;
    }
  }

  /// Create a new account
  Future<Map<String, String>?> createAccount() async {
    try {
      final result = _ffi.createAccount();

      if (result.errorCode == ErrorCode.success &&
          result.publicKey != null &&
          result.privateKey != null) {
        return {
          'publicKey': result.publicKey!,
          'privateKey': result.privateKey!,
        };
      } else {
        debugPrint('Failed to create account: ${result.errorCode}');
        return null;
      }
    } catch (e) {
      debugPrint('Error creating account: $e');
      return null;
    }
  }

  /// Get network statistics
  Future<Map<String, dynamic>> getNetworkStats() async {
    // In a real implementation, this would get actual network stats
    // from the Rust core through FFI
    // For now, return simulated data with some variability

    // Simulate number of peers between 5-15
    final peers = 5 + (DateTime.now().second % 10);

    // Simulate increasing validated transactions
    final validatedTx =
        1000 + (DateTime.now().minute * 60 + DateTime.now().second);

    // Simulate increasing block height
    final blockHeight =
        4000 + (DateTime.now().minute * 2 + DateTime.now().second ~/ 30);

    // Simulate last block time between 5-60 seconds ago
    final lastBlockTime = DateTime.now().subtract(
      Duration(seconds: 5 + (DateTime.now().second % 55)),
    );

    return {
      'peers': peers,
      'validatedTransactions': validatedTx,
      'blockHeight': blockHeight,
      'lastBlockTime': lastBlockTime,
    };
  }
}
