import 'dart:async';
import 'dart:ffi';
import 'package:flutter/foundation.dart';
import '../ffi/sebure_ffi.dart';
import 'blockchain_service.dart';

/// Service for managing the background validation process
class ValidationService {
  // Singleton instance
  static final ValidationService _instance = ValidationService._internal();
  static ValidationService get instance => _instance;

  // Private constructor for singleton
  ValidationService._internal();

  // Service ID for the currently running validation service
  int? _serviceId;

  // Timer for periodically updating stats
  Timer? _statsTimer;

  // Latest service statistics
  ValidationServiceStats _stats = ValidationServiceStats(
    transactionsProcessed: 0,
    blocksValidated: 0,
    blocksGenerated: 0,
    validationErrors: 0,
    queueLength: 0,
    avgTransactionTimeMs: 0.0,
    uptimeSeconds: 0,
    cpuUsage: 0.0,
    memoryUsage: 0.0,
  );

  // Service status
  ValidationServiceStatus _status = ValidationServiceStatus.stopped;

  // Stream controllers for stats and status updates
  final _statsStreamController =
      StreamController<ValidationServiceStats>.broadcast();
  final _statusStreamController =
      StreamController<ValidationServiceStatus>.broadcast();

  // Configuration
  late ValidationServiceConfig _config;

  /// Stream of statistics updates
  Stream<ValidationServiceStats> get statsStream =>
      _statsStreamController.stream;

  /// Stream of status updates
  Stream<ValidationServiceStatus> get statusStream =>
      _statusStreamController.stream;

  /// The latest service statistics
  ValidationServiceStats get stats => _stats;

  /// The current service status
  ValidationServiceStatus get status => _status;

  /// Initialize the validation service
  Future<bool> initialize({ValidationServiceConfig? config}) async {
    debugPrint('Initializing validation service...');
    try {
      // Make sure blockchain service is initialized
      final blockchainInitialized = await BlockchainService.initialize();
      if (!blockchainInitialized) {
        debugPrint('Failed to initialize blockchain service');
        return false;
      }

      // Set default configuration if not provided
      _config = config ?? ValidationServiceConfig.defaultConfig();

      // Create validation service
      _serviceId = SebureFFI.instance.createValidationService(
        maxCpuUsage: _config.maxCpuUsage,
        maxMemoryUsage: _config.maxMemoryUsage,
        queueSizeLimit: _config.queueSizeLimit,
        processingTimeSlotMs: _config.processingTimeSlotMs,
        batchSize: _config.batchSize,
      );

      if (_serviceId == null || _serviceId == 0) {
        debugPrint('Failed to create validation service');
        return false;
      }

      // Start a timer to update stats periodically
      _startStatsTimer();

      debugPrint(
        'Validation service initialized successfully with ID: $_serviceId',
      );
      return true;
    } catch (e) {
      debugPrint('Error initializing validation service: $e');
      return false;
    }
  }

  /// Start the validation service
  Future<bool> start() async {
    if (_serviceId == null) {
      debugPrint('Validation service not initialized');
      return false;
    }

    try {
      final result = SebureFFI.instance.startValidationService(_serviceId!);
      if (result != 0) {
        debugPrint('Failed to start validation service: $result');
        return false;
      }

      _updateStatus();
      debugPrint('Validation service started');
      return true;
    } catch (e) {
      debugPrint('Error starting validation service: $e');
      return false;
    }
  }

  /// Stop the validation service
  Future<bool> stop() async {
    if (_serviceId == null) {
      debugPrint('Validation service not initialized');
      return false;
    }

    try {
      final result = SebureFFI.instance.stopValidationService(_serviceId!);
      if (result != 0) {
        debugPrint('Failed to stop validation service: $result');
        return false;
      }

      _updateStatus();
      debugPrint('Validation service stopped');
      return true;
    } catch (e) {
      debugPrint('Error stopping validation service: $e');
      return false;
    }
  }

  /// Pause the validation service
  Future<bool> pause() async {
    if (_serviceId == null) {
      debugPrint('Validation service not initialized');
      return false;
    }

    try {
      final result = SebureFFI.instance.pauseValidationService(_serviceId!);
      if (result != 0) {
        debugPrint('Failed to pause validation service: $result');
        return false;
      }

      _updateStatus();
      debugPrint('Validation service paused');
      return true;
    } catch (e) {
      debugPrint('Error pausing validation service: $e');
      return false;
    }
  }

  /// Resume the validation service
  Future<bool> resume() async {
    if (_serviceId == null) {
      debugPrint('Validation service not initialized');
      return false;
    }

    try {
      final result = SebureFFI.instance.resumeValidationService(_serviceId!);
      if (result != 0) {
        debugPrint('Failed to resume validation service: $result');
        return false;
      }

      _updateStatus();
      debugPrint('Validation service resumed');
      return true;
    } catch (e) {
      debugPrint('Error resuming validation service: $e');
      return false;
    }
  }

  /// Update the service configuration
  Future<bool> updateConfig(ValidationServiceConfig config) async {
    if (_serviceId == null) {
      debugPrint('Validation service not initialized');
      return false;
    }

    try {
      final result = SebureFFI.instance.updateValidationServiceConfig(
        _serviceId!,
        maxCpuUsage: config.maxCpuUsage,
        maxMemoryUsage: config.maxMemoryUsage,
        queueSizeLimit: config.queueSizeLimit,
        processingTimeSlotMs: config.processingTimeSlotMs,
        batchSize: config.batchSize,
      );
      if (result != 0) {
        debugPrint('Failed to update validation service config: $result');
        return false;
      }

      _config = config;
      debugPrint('Validation service configuration updated');
      return true;
    } catch (e) {
      debugPrint('Error updating validation service config: $e');
      return false;
    }
  }

  /// Add a custom task to the validation service
  Future<int?> addCustomTask(
    String name, {
    TaskPriority priority = TaskPriority.medium,
  }) async {
    if (_serviceId == null) {
      debugPrint('Validation service not initialized');
      return null;
    }

    try {
      final result = SebureFFI.instance.addValidationServiceTask(
        _serviceId!,
        taskType: 1, // Custom task
        priority: priority.index,
        data: name,
      );

      if (result.errorCode != 0) {
        debugPrint('Failed to add custom task: ${result.errorCode}');
        return null;
      }

      return result.taskId;
    } catch (e) {
      debugPrint('Error adding custom task: $e');
      return null;
    }
  }

  /// Add a block generation task to the validation service
  Future<int?> addBlockGenerationTask({
    TaskPriority priority = TaskPriority.high,
  }) async {
    if (_serviceId == null) {
      debugPrint('Validation service not initialized');
      return null;
    }

    try {
      final result = SebureFFI.instance.addValidationServiceTask(
        _serviceId!,
        taskType: 0, // Block generation task
        priority: priority.index,
      );

      if (result.errorCode != 0) {
        debugPrint('Failed to add block generation task: ${result.errorCode}');
        return null;
      }

      return result.taskId;
    } catch (e) {
      debugPrint('Error adding block generation task: $e');
      return null;
    }
  }

  /// Start a timer to periodically update statistics
  void _startStatsTimer() {
    // Stop existing timer if any
    _statsTimer?.cancel();

    // Create a new timer that fires every second
    _statsTimer = Timer.periodic(const Duration(seconds: 1), (timer) {
      _updateStats();
      _updateStatus();
    });
  }

  /// Update service statistics from the Rust core
  void _updateStats() {
    if (_serviceId == null) return;

    try {
      final stats = SebureFFI.instance.getValidationServiceStats(_serviceId!);
      if (stats != null) {
        _stats = stats;
        _statsStreamController.add(_stats);
      }
    } catch (e) {
      debugPrint('Error updating validation service stats: $e');
    }
  }

  /// Update service status from the Rust core
  void _updateStatus() {
    if (_serviceId == null) return;

    try {
      final statusCode = SebureFFI.instance.getValidationServiceStatus(
        _serviceId!,
      );
      if (statusCode >= 0 &&
          statusCode < ValidationServiceStatus.values.length) {
        _status = ValidationServiceStatus.values[statusCode];
        _statusStreamController.add(_status);
      }
    } catch (e) {
      debugPrint('Error updating validation service status: $e');
    }
  }

  /// Dispose of resources
  Future<void> dispose() async {
    // Stop the stats timer
    _statsTimer?.cancel();
    _statsTimer = null;

    // If we have a service ID, destroy the service
    if (_serviceId != null) {
      try {
        // Stop the service if it's running
        if (_status != ValidationServiceStatus.stopped) {
          await stop();
        }

        // Destroy the service
        SebureFFI.instance.destroyValidationService(_serviceId!);
        _serviceId = null;
      } catch (e) {
        debugPrint('Error disposing validation service: $e');
      }
    }

    // Close the stream controllers
    await _statsStreamController.close();
    await _statusStreamController.close();
  }
}

/// Configuration for the validation service
class ValidationServiceConfig {
  /// Maximum CPU usage percentage (0-100)
  final int maxCpuUsage;

  /// Maximum memory usage in MB
  final int maxMemoryUsage;

  /// Task queue size limit
  final int queueSizeLimit;

  /// Time slot for transaction processing in milliseconds
  final int processingTimeSlotMs;

  /// Number of transactions to process in a batch
  final int batchSize;

  /// Create a new validation service configuration
  ValidationServiceConfig({
    required this.maxCpuUsage,
    required this.maxMemoryUsage,
    required this.queueSizeLimit,
    required this.processingTimeSlotMs,
    required this.batchSize,
  });

  /// Create a configuration with default values
  factory ValidationServiceConfig.defaultConfig() => ValidationServiceConfig(
    maxCpuUsage: 20,
    maxMemoryUsage: 500,
    queueSizeLimit: 10000,
    processingTimeSlotMs: 200,
    batchSize: 100,
  );

  /// Create a low-resource configuration
  factory ValidationServiceConfig.lowResource() => ValidationServiceConfig(
    maxCpuUsage: 10,
    maxMemoryUsage: 250,
    queueSizeLimit: 5000,
    processingTimeSlotMs: 300,
    batchSize: 50,
  );

  /// Create a high-performance configuration
  factory ValidationServiceConfig.highPerformance() => ValidationServiceConfig(
    maxCpuUsage: 50,
    maxMemoryUsage: 1000,
    queueSizeLimit: 20000,
    processingTimeSlotMs: 100,
    batchSize: 200,
  );
}

/// Statistics for the validation service
// Using ValidationServiceStats from FFI module

/// Extension to add formatting to ValidationServiceStats
extension ValidationServiceStatsExtension on ValidationServiceStats {
  /// Format the statistics as a readable string
  String format() =>
      'ValidationServiceStats{'
      'transactionsProcessed: $transactionsProcessed, '
      'blocksValidated: $blocksValidated, '
      'blocksGenerated: $blocksGenerated, '
      'validationErrors: $validationErrors, '
      'queueLength: $queueLength, '
      'avgTransactionTimeMs: ${avgTransactionTimeMs.toStringAsFixed(2)}, '
      'uptimeSeconds: $uptimeSeconds, '
      'cpuUsage: ${cpuUsage.toStringAsFixed(1)}%, '
      'memoryUsage: ${memoryUsage.toStringAsFixed(1)} MB'
      '}';
}

/// Status of the validation service
enum ValidationServiceStatus {
  /// Service is not started
  stopped,

  /// Service is starting up
  starting,

  /// Service is running normally
  running,

  /// Service is paused
  paused,

  /// Service is currently recovering from an error
  recovering,

  /// Service has encountered a fatal error
  failed,

  /// Service is shutting down
  shuttingDown,
}

/// Priority levels for tasks
enum TaskPriority {
  /// Low priority
  low,

  /// Medium priority
  medium,

  /// High priority
  high,

  /// Critical priority
  critical,
}
