import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Service for managing and persisting application configuration
class ConfigService {
  // Singleton instance
  static final ConfigService _instance = ConfigService._internal();
  static ConfigService get instance => _instance;

  // Shared preferences instance
  late SharedPreferences _prefs;
  bool _isInitialized = false;

  // Configuration keys
  static const String _keyNodeEnabled = 'node_enabled';
  static const String _keyMaxCpuUsage = 'max_cpu_usage';
  static const String _keyMaxMemoryUsage = 'max_memory_usage';
  static const String _keyMaxNetworkUsage = 'max_network_usage';
  static const String _keyMaxDiskUsage = 'max_disk_usage';
  static const String _keyValidationSchedule = 'validation_schedule';
  static const String _keyListenAddress = 'listen_address';
  static const String _keyTheme = 'theme';
  static const String _keyActivatedFeatures = 'activated_features';
  static const String _keyCustomNetworkPeers = 'custom_network_peers';
  static const String _keyWalletAccounts = 'wallet_accounts';
  static const String _keySelectedAccount = 'selected_account';
  static const String _keyLogLevel = 'log_level';

  // Private constructor for singleton
  ConfigService._internal();

  /// Initialize the configuration service
  Future<bool> initialize() async {
    if (_isInitialized) return true;

    try {
      _prefs = await SharedPreferences.getInstance();
      _isInitialized = true;

      // Set default values if first run
      await _setDefaultsIfNeeded();

      return true;
    } catch (e) {
      debugPrint('Error initializing ConfigService: $e');
      return false;
    }
  }

  /// Set default values for all configuration options if they don't exist
  Future<void> _setDefaultsIfNeeded() async {
    // Node configuration defaults
    if (!_prefs.containsKey(_keyNodeEnabled)) {
      await _prefs.setBool(_keyNodeEnabled, false);
    }

    if (!_prefs.containsKey(_keyMaxCpuUsage)) {
      await _prefs.setDouble(_keyMaxCpuUsage, 20.0); // 20% max CPU usage
    }

    if (!_prefs.containsKey(_keyMaxMemoryUsage)) {
      await _prefs.setDouble(_keyMaxMemoryUsage, 500.0); // 500 MB max memory
    }

    if (!_prefs.containsKey(_keyMaxNetworkUsage)) {
      await _prefs.setDouble(_keyMaxNetworkUsage, 100.0); // 100 MB/hour
    }

    if (!_prefs.containsKey(_keyMaxDiskUsage)) {
      await _prefs.setDouble(_keyMaxDiskUsage, 10.0); // 10 GB max storage
    }

    if (!_prefs.containsKey(_keyListenAddress)) {
      await _prefs.setString(_keyListenAddress, '127.0.0.1:9000');
    }

    if (!_prefs.containsKey(_keyTheme)) {
      await _prefs.setString(_keyTheme, 'system'); // Follow system theme
    }

    if (!_prefs.containsKey(_keyLogLevel)) {
      await _prefs.setString(_keyLogLevel, 'info');
    }

    // Initialize empty lists/maps if needed
    if (!_prefs.containsKey(_keyValidationSchedule)) {
      await _prefs.setString(_keyValidationSchedule, jsonEncode([]));
    }

    if (!_prefs.containsKey(_keyActivatedFeatures)) {
      await _prefs.setString(_keyActivatedFeatures, jsonEncode([]));
    }

    if (!_prefs.containsKey(_keyCustomNetworkPeers)) {
      await _prefs.setString(_keyCustomNetworkPeers, jsonEncode([]));
    }

    if (!_prefs.containsKey(_keyWalletAccounts)) {
      await _prefs.setString(_keyWalletAccounts, jsonEncode([]));
    }

    if (!_prefs.containsKey(_keySelectedAccount)) {
      await _prefs.setString(_keySelectedAccount, '');
    }
  }

  // Node configuration getters and setters

  /// Get whether the node is enabled to start automatically
  bool get isNodeEnabled => _prefs.getBool(_keyNodeEnabled) ?? false;

  /// Set whether the node should start automatically
  Future<bool> setNodeEnabled(bool enabled) async {
    return await _prefs.setBool(_keyNodeEnabled, enabled);
  }

  /// Get the maximum CPU usage percentage
  double get maxCpuUsage => _prefs.getDouble(_keyMaxCpuUsage) ?? 20.0;

  /// Set the maximum CPU usage percentage
  Future<bool> setMaxCpuUsage(double percentage) async {
    return await _prefs.setDouble(_keyMaxCpuUsage, percentage);
  }

  /// Get the maximum memory usage in MB
  double get maxMemoryUsage => _prefs.getDouble(_keyMaxMemoryUsage) ?? 500.0;

  /// Set the maximum memory usage in MB
  Future<bool> setMaxMemoryUsage(double megabytes) async {
    return await _prefs.setDouble(_keyMaxMemoryUsage, megabytes);
  }

  /// Get the maximum network usage in MB per hour
  double get maxNetworkUsage => _prefs.getDouble(_keyMaxNetworkUsage) ?? 100.0;

  /// Set the maximum network usage in MB per hour
  Future<bool> setMaxNetworkUsage(double megabytesPerHour) async {
    return await _prefs.setDouble(_keyMaxNetworkUsage, megabytesPerHour);
  }

  /// Get the maximum disk usage in GB
  double get maxDiskUsage => _prefs.getDouble(_keyMaxDiskUsage) ?? 10.0;

  /// Set the maximum disk usage in GB
  Future<bool> setMaxDiskUsage(double gigabytes) async {
    return await _prefs.setDouble(_keyMaxDiskUsage, gigabytes);
  }

  /// Get the network listen address
  String get listenAddress =>
      _prefs.getString(_keyListenAddress) ?? '127.0.0.1:9000';

  /// Set the network listen address
  Future<bool> setListenAddress(String address) async {
    return await _prefs.setString(_keyListenAddress, address);
  }

  /// Get the theme setting (light, dark, system)
  String get theme => _prefs.getString(_keyTheme) ?? 'system';

  /// Set the theme setting
  Future<bool> setTheme(String theme) async {
    return await _prefs.setString(_keyTheme, theme);
  }

  /// Get the log level
  String get logLevel => _prefs.getString(_keyLogLevel) ?? 'info';

  /// Set the log level
  Future<bool> setLogLevel(String level) async {
    return await _prefs.setString(_keyLogLevel, level);
  }

  // Complex configuration getters and setters

  /// Get the validation schedule
  List<Map<String, dynamic>> get validationSchedule {
    final String json = _prefs.getString(_keyValidationSchedule) ?? '[]';
    try {
      final List<dynamic> decoded = jsonDecode(json);
      return List<Map<String, dynamic>>.from(decoded);
    } catch (e) {
      debugPrint('Error parsing validation schedule: $e');
      return [];
    }
  }

  /// Set the validation schedule
  Future<bool> setValidationSchedule(
    List<Map<String, dynamic>> schedule,
  ) async {
    try {
      final String json = jsonEncode(schedule);
      return await _prefs.setString(_keyValidationSchedule, json);
    } catch (e) {
      debugPrint('Error saving validation schedule: $e');
      return false;
    }
  }

  /// Get the list of activated features
  List<String> get activatedFeatures {
    final String json = _prefs.getString(_keyActivatedFeatures) ?? '[]';
    try {
      final List<dynamic> decoded = jsonDecode(json);
      return List<String>.from(decoded);
    } catch (e) {
      debugPrint('Error parsing activated features: $e');
      return [];
    }
  }

  /// Set the list of activated features
  Future<bool> setActivatedFeatures(List<String> features) async {
    try {
      final String json = jsonEncode(features);
      return await _prefs.setString(_keyActivatedFeatures, json);
    } catch (e) {
      debugPrint('Error saving activated features: $e');
      return false;
    }
  }

  /// Get the list of custom network peers
  List<String> get customNetworkPeers {
    final String json = _prefs.getString(_keyCustomNetworkPeers) ?? '[]';
    try {
      final List<dynamic> decoded = jsonDecode(json);
      return List<String>.from(decoded);
    } catch (e) {
      debugPrint('Error parsing custom network peers: $e');
      return [];
    }
  }

  /// Set the list of custom network peers
  Future<bool> setCustomNetworkPeers(List<String> peers) async {
    try {
      final String json = jsonEncode(peers);
      return await _prefs.setString(_keyCustomNetworkPeers, json);
    } catch (e) {
      debugPrint('Error saving custom network peers: $e');
      return false;
    }
  }

  /// Get the wallet accounts
  List<Map<String, dynamic>> get walletAccounts {
    final String json = _prefs.getString(_keyWalletAccounts) ?? '[]';
    try {
      final List<dynamic> decoded = jsonDecode(json);
      return List<Map<String, dynamic>>.from(decoded);
    } catch (e) {
      debugPrint('Error parsing wallet accounts: $e');
      return [];
    }
  }

  /// Set the wallet accounts
  Future<bool> setWalletAccounts(List<Map<String, dynamic>> accounts) async {
    try {
      final String json = jsonEncode(accounts);
      return await _prefs.setString(_keyWalletAccounts, json);
    } catch (e) {
      debugPrint('Error saving wallet accounts: $e');
      return false;
    }
  }

  /// Get the currently selected account address
  String get selectedAccount => _prefs.getString(_keySelectedAccount) ?? '';

  /// Set the currently selected account address
  Future<bool> setSelectedAccount(String address) async {
    return await _prefs.setString(_keySelectedAccount, address);
  }

  /// Clear all configuration data (for testing or resets)
  Future<bool> clearAllConfig() async {
    try {
      return await _prefs.clear();
    } catch (e) {
      debugPrint('Error clearing configuration: $e');
      return false;
    }
  }
}
