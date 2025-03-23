import 'dart:async';
import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';
import '../ffi/sebure_ffi.dart';

/// Transaction model for the UI
class Transaction {
  /// Transaction ID
  final String id;

  /// Amount transferred
  final double amount;

  /// Timestamp of the transaction
  final DateTime timestamp;

  /// Whether this transaction is outgoing (sent) or incoming (received)
  final bool isOutgoing;

  /// Sender address
  final String sender;

  /// Recipient address
  final String recipient;

  /// Transaction fee
  final double fee;

  /// Transaction status
  final TransactionStatus status;

  /// Create a new transaction
  Transaction({
    required this.id,
    required this.amount,
    required this.timestamp,
    required this.isOutgoing,
    required this.sender,
    required this.recipient,
    required this.fee,
    this.status = TransactionStatus.confirmed,
  });
}

/// Transaction status
enum TransactionStatus {
  /// Transaction is pending confirmation
  pending,

  /// Transaction is confirmed
  confirmed,

  /// Transaction failed
  failed,
}

/// Transaction type
enum TransactionType {
  /// Transfer of tokens
  transfer,

  /// Smart contract deployment
  contractDeploy,

  /// Smart contract call
  contractCall,

  /// Validator registration
  validatorRegister,

  /// Validator unregistration
  validatorUnregister,

  /// Staking deposit
  stake,

  /// Staking withdrawal
  unstake,

  /// System transaction
  system,
}

/// Service for managing transactions
class TransactionService {
  // Singleton instance
  static final TransactionService _instance = TransactionService._internal();
  static TransactionService get instance => _instance;

  // Private constructor for singleton
  TransactionService._internal();

  // FFI instance
  final _ffi = SebureFFI.instance;

  // Track if the service is initialized
  bool _isInitialized = false;

  /// Initialize the transaction service
  Future<bool> initialize() async {
    if (_isInitialized) return true;

    debugPrint('Initializing transaction service...');
    try {
      // Initialize the transaction service through FFI
      final result = _ffi.initTransactionService();
      if (result != 0) {
        debugPrint('Failed to initialize transaction service: $result');
        return false;
      }

      _isInitialized = true;
      debugPrint('Transaction service initialized successfully');
      return true;
    } catch (e) {
      debugPrint('Error initializing transaction service: $e');
      return false;
    }
  }

  /// Create a transaction
  Future<String?> createTransaction({
    required String senderPublicKey,
    required int senderShard,
    required String recipientAddress,
    required int recipientShard,
    required double amount,
    required double fee,
    required TransactionType transactionType,
  }) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      // Convert amount to smallest unit (similar to satoshi in Bitcoin)
      final amountInSmallestUnit = (amount * 100000000).toInt();
      final feeInSmallestUnit = (fee * 100000000).toInt();

      // Create transaction through FFI
      final params = CreateTransactionParams(
        senderPublicKey: senderPublicKey,
        senderShard: senderShard,
        recipientAddress: recipientAddress,
        recipientShard: recipientShard,
        amount: amountInSmallestUnit,
        fee: feeInSmallestUnit,
        transactionType: transactionType.index,
      );

      final txId = await compute(_createTransaction, params);
      return txId;
    } catch (e) {
      debugPrint('Error creating transaction: $e');
      return null;
    }
  }

  /// Sign a transaction
  Future<bool> signTransaction({
    required String transactionId,
    required String privateKey,
  }) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      // Sign transaction through FFI
      final params = SignTransactionParams(
        transactionId: transactionId,
        privateKey: privateKey,
      );

      final result = await compute(_signTransaction, params);
      return result == 0;
    } catch (e) {
      debugPrint('Error signing transaction: $e');
      return false;
    }
  }

  /// Submit a transaction
  Future<String?> submitTransaction({
    required String senderPublicKey,
    required String senderPrivateKey,
    required int senderShard,
    required String recipientAddress,
    required int recipientShard,
    required double amount,
    required double fee,
  }) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      // Convert amount to smallest unit (similar to satoshi in Bitcoin)
      final amountInSmallestUnit = (amount * 100000000).toInt();
      final feeInSmallestUnit = (fee * 100000000).toInt();

      // Submit transaction through FFI
      final params = SubmitTransactionParams(
        senderPublicKey: senderPublicKey,
        senderPrivateKey: senderPrivateKey,
        senderShard: senderShard,
        recipientAddress: recipientAddress,
        recipientShard: recipientShard,
        amount: amountInSmallestUnit,
        fee: feeInSmallestUnit,
      );

      final txId = await compute(_submitTransaction, params);
      return txId;
    } catch (e) {
      debugPrint('Error submitting transaction: $e');
      return null;
    }
  }

  /// Estimate transaction fee
  Future<double> estimateFee({
    required TransactionType transactionType,
    required int dataSize,
  }) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      // Estimate fee through FFI
      final params = EstimateFeeParams(
        transactionType: transactionType.index,
        dataSize: dataSize,
      );

      final feeInSmallestUnit = await compute(_estimateFee, params);

      // Convert from smallest unit to main unit
      return feeInSmallestUnit / 100000000;
    } catch (e) {
      debugPrint('Error estimating fee: $e');
      return 0.0001; // Default fee
    }
  }

  /// Get transaction history for an address
  Future<List<Transaction>> getTransactionHistory(String address) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      // Get transaction history through FFI
      final history = await compute(_getTransactionHistory, address);
      return history;
    } catch (e) {
      debugPrint('Error getting transaction history: $e');

      // Return mock data for development
      return _getMockTransactionHistory(address);
    }
  }

  /// Get account balance
  Future<double> getBalance(String address) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      // Get balance through FFI
      final balanceInSmallestUnit = await compute(_getBalance, address);

      // Convert from smallest unit to main unit
      return balanceInSmallestUnit / 100000000;
    } catch (e) {
      debugPrint('Error getting balance: $e');
      return 0.0;
    }
  }

  /// Get mock transaction history for development
  List<Transaction> _getMockTransactionHistory(String address) {
    // Create some mock transactions
    final now = DateTime.now();

    return [
      Transaction(
        id: '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
        amount: 10.0,
        timestamp: now.subtract(const Duration(days: 1)),
        isOutgoing: false,
        sender: '0987654321fedcba0987654321fedcba0987654321',
        recipient: address,
        fee: 0.001,
      ),
      Transaction(
        id: 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
        amount: 5.0,
        timestamp: now.subtract(const Duration(days: 2)),
        isOutgoing: true,
        sender: address,
        recipient: 'fedcba0987654321fedcba0987654321fedcba09',
        fee: 0.001,
      ),
      Transaction(
        id: '7890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456',
        amount: 15.0,
        timestamp: now.subtract(const Duration(days: 3)),
        isOutgoing: false,
        sender: '4321fedcba0987654321fedcba0987654321fedcba',
        recipient: address,
        fee: 0.001,
      ),
    ];
  }
}

/// Parameters for creating a transaction
class CreateTransactionParams {
  final String senderPublicKey;
  final int senderShard;
  final String recipientAddress;
  final int recipientShard;
  final int amount;
  final int fee;
  final int transactionType;

  CreateTransactionParams({
    required this.senderPublicKey,
    required this.senderShard,
    required this.recipientAddress,
    required this.recipientShard,
    required this.amount,
    required this.fee,
    required this.transactionType,
  });
}

/// Parameters for signing a transaction
class SignTransactionParams {
  final String transactionId;
  final String privateKey;

  SignTransactionParams({
    required this.transactionId,
    required this.privateKey,
  });
}

/// Parameters for submitting a transaction
class SubmitTransactionParams {
  final String senderPublicKey;
  final String senderPrivateKey;
  final int senderShard;
  final String recipientAddress;
  final int recipientShard;
  final int amount;
  final int fee;

  SubmitTransactionParams({
    required this.senderPublicKey,
    required this.senderPrivateKey,
    required this.senderShard,
    required this.recipientAddress,
    required this.recipientShard,
    required this.amount,
    required this.fee,
  });
}

/// Parameters for estimating a transaction fee
class EstimateFeeParams {
  final int transactionType;
  final int dataSize;

  EstimateFeeParams({required this.transactionType, required this.dataSize});
}

// Isolate functions for FFI calls

/// Initialize the transaction service in an isolate
int _initTransactionService(_) {
  final ffi = SebureFFI.instance;
  return ffi.initTransactionService();
}

/// Create a transaction in an isolate
String? _createTransaction(CreateTransactionParams params) {
  final ffi = SebureFFI.instance;
  return ffi.createTransaction(
    senderPublicKey: params.senderPublicKey,
    senderShard: params.senderShard,
    recipientAddress: params.recipientAddress,
    recipientShard: params.recipientShard,
    amount: params.amount,
    fee: params.fee,
    transactionType: params.transactionType,
  );
}

/// Sign a transaction in an isolate
int _signTransaction(SignTransactionParams params) {
  final ffi = SebureFFI.instance;
  return ffi.signTransaction(
    transactionId: params.transactionId,
    privateKey: params.privateKey,
  );
}

/// Submit a transaction in an isolate
String? _submitTransaction(SubmitTransactionParams params) {
  final ffi = SebureFFI.instance;
  return ffi.submitTransaction(
    senderPublicKey: params.senderPublicKey,
    senderPrivateKey: params.senderPrivateKey,
    senderShard: params.senderShard,
    recipientAddress: params.recipientAddress,
    recipientShard: params.recipientShard,
    amount: params.amount,
    fee: params.fee,
  );
}

/// Estimate a transaction fee in an isolate
int _estimateFee(EstimateFeeParams params) {
  final ffi = SebureFFI.instance;
  return ffi.estimateFee(
    transactionType: params.transactionType,
    dataSize: params.dataSize,
  );
}

/// Get transaction history in an isolate
List<Transaction> _getTransactionHistory(String address) {
  final ffi = SebureFFI.instance;

  // Use the mock data for now since the FFI implementation has issues
  final now = DateTime.now();

  return [
    Transaction(
      id: '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
      amount: 10.0,
      timestamp: now.subtract(const Duration(days: 1)),
      isOutgoing: false,
      sender: '0987654321fedcba0987654321fedcba0987654321',
      recipient: address,
      fee: 0.001,
    ),
    Transaction(
      id: 'abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
      amount: 5.0,
      timestamp: now.subtract(const Duration(days: 2)),
      isOutgoing: true,
      sender: address,
      recipient: 'fedcba0987654321fedcba0987654321fedcba09',
      fee: 0.001,
    ),
    Transaction(
      id: '7890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456',
      amount: 15.0,
      timestamp: now.subtract(const Duration(days: 3)),
      isOutgoing: false,
      sender: '4321fedcba0987654321fedcba0987654321fedcba',
      recipient: address,
      fee: 0.001,
    ),
  ];
}

/// Get account balance in an isolate
int _getBalance(String address) {
  final ffi = SebureFFI.instance;
  return ffi.getBalance(address);
}
