import 'package:flutter/foundation.dart';
import '../services/transaction_service.dart';

/// Main application state management class
class AppState extends ChangeNotifier {
  bool _isNodeRunning = false;
  double _cpuUsage = 0.0;
  double _memoryUsage = 0.0;
  double _networkUsage = 0.0;
  double _diskUsage = 0.0;
  int _connectedPeers = 0;
  int _transactionsValidated = 0;
  double _balance = 0.0;
  String _currentAddress = '';
  List<Transaction> _transactions = [];

  // Getters
  bool get isNodeRunning => _isNodeRunning;
  double get cpuUsage => _cpuUsage;
  double get memoryUsage => _memoryUsage;
  double get networkUsage => _networkUsage;
  double get diskUsage => _diskUsage;
  int get connectedPeers => _connectedPeers;
  int get transactionsValidated => _transactionsValidated;
  double get balance => _balance;
  String get currentAddress => _currentAddress;
  List<Transaction> get transactions => List.unmodifiable(_transactions);

  // Methods to update state
  void updateNodeStatus({required bool isRunning}) {
    _isNodeRunning = isRunning;
    notifyListeners();
  }

  void updateResourceUsage({
    double? cpu,
    double? memory,
    double? network,
    double? disk,
  }) {
    if (cpu != null) _cpuUsage = cpu;
    if (memory != null) _memoryUsage = memory;
    if (network != null) _networkUsage = network;
    if (disk != null) _diskUsage = disk;
    notifyListeners();
  }

  void updateNetworkStats({int? peers, int? validatedTransactions}) {
    if (peers != null) _connectedPeers = peers;
    if (validatedTransactions != null)
      _transactionsValidated = validatedTransactions;
    notifyListeners();
  }

  void updateBalance(double newBalance) {
    _balance = newBalance;
    notifyListeners();
  }

  void setCurrentAddress(String address) {
    _currentAddress = address;
    notifyListeners();
  }

  void updateTransactions(List<Transaction> transactions) {
    _transactions = transactions;
    notifyListeners();
  }

  void addTransaction(Transaction transaction) {
    _transactions.add(transaction);
    notifyListeners();
  }
}
