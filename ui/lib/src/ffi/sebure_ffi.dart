import 'dart:ffi';
import 'dart:io';
import 'package:ffi/ffi.dart';
import 'package:path/path.dart' as path;

/// ErrorCode enum to match the Rust FFI error codes
enum ErrorCode {
  success,
  invalidArgument,
  ioError,
  networkError,
  consensusError,
  storageError,
  alreadyInitialized,
  notInitialized,
  internalError,
  unknown,
}

/// Validation service status enum
enum ValidationServiceStatus {
  stopped,
  starting,
  running,
  paused,
  recovering,
  failed,
  shuttingDown,
}

/// Statistics for the validation service
class ValidationServiceStats {
  /// Number of transactions processed
  final int transactionsProcessed;

  /// Number of blocks validated
  final int blocksValidated;

  /// Number of blocks generated
  final int blocksGenerated;

  /// Number of validation errors
  final int validationErrors;

  /// Current task queue length
  final int queueLength;

  /// Average transaction processing time in milliseconds
  final double avgTransactionTimeMs;

  /// Service uptime in seconds
  final int uptimeSeconds;

  /// CPU usage percentage (0-100)
  final double cpuUsage;

  /// Memory usage in MB
  final double memoryUsage;

  /// Create a new validation service statistics object
  ValidationServiceStats({
    required this.transactionsProcessed,
    required this.blocksValidated,
    required this.blocksGenerated,
    required this.validationErrors,
    required this.queueLength,
    required this.avgTransactionTimeMs,
    required this.uptimeSeconds,
    required this.cpuUsage,
    required this.memoryUsage,
  });
}

/// FFI class to interface with the SEBURE Blockchain Rust core
class SebureFFI {
  // Singleton instance
  static final SebureFFI _instance = SebureFFI._internal();
  static SebureFFI get instance => _instance;

  // Library name based on platform
  static const String _libName = 'sebure_ffi';
  late final DynamicLibrary _dylib;
  bool _isInitialized = false;

  // Private constructor
  SebureFFI._internal();

  /// Initialize the FFI bindings
  Future<bool> initialize() async {
    if (_isInitialized) return true;

    try {
      _dylib = _loadLibrary();
      _setupBindings();
      _isInitialized = true;
      return true;
    } catch (e) {
      print('Failed to initialize SEBURE FFI: $e');
      return false;
    }
  }

  // Load the dynamic library based on the platform
  DynamicLibrary _loadLibrary() {
    if (Platform.isWindows) {
      return DynamicLibrary.open('$_libName.dll');
    } else if (Platform.isMacOS) {
      return DynamicLibrary.open('lib$_libName.dylib');
    } else if (Platform.isLinux) {
      return DynamicLibrary.open('lib$_libName.so');
    } else {
      throw UnsupportedError(
        'Unsupported platform: ${Platform.operatingSystem}',
      );
    }
  }

  // Core FFI function typedefs
  late final int Function() _sebureInit;
  late final int Function() _sebureBlockchainInit;
  late final int Function(Pointer<Utf8>) _sebureStorageInit;
  late final int Function(Pointer<Utf8>) _sebureNetworkInit;
  late final int Function() _sebureNetworkStart;
  late final int Function(Pointer<Pointer<Utf8>>, Pointer<Pointer<Utf8>>)
  _sebureCreateAccount;
  late final int Function(Pointer<Utf8>, Pointer<Uint64>)
  _sebureGetAccountBalance;
  late final void Function(Pointer<Utf8>) _sebureFreString;
  late final int Function() _sebureShutdown;

  // Validation service FFI function typedefs
  late final int Function(int, int, int, int, int)
  _sebureValidationServiceCreate;
  late final int Function(int) _sebureValidationServiceStart;
  late final int Function(int) _sebureValidationServiceStop;
  late final int Function(int) _sebureValidationServiceDestroy;
  late final int Function(int) _sebureValidationServiceStatus;
  late final int Function(int) _sebureValidationServicePause;
  late final int Function(int) _sebureValidationServiceResume;
  late final int Function(
    int,
    Pointer<Uint64>,
    Pointer<Uint64>,
    Pointer<Uint64>,
    Pointer<Uint64>,
    Pointer<Uint32>,
    Pointer<Double>,
    Pointer<Uint64>,
    Pointer<Float>,
    Pointer<Float>,
  )
  _sebureValidationServiceGetStats;
  late final int Function(int, int, int, int, int, int)
  _sebureValidationServiceUpdateConfig;
  late final int Function(int, int, int, Pointer<Utf8>, Pointer<Uint64>)
  _sebureValidationServiceAddTask;

  // Set up all the function bindings
  void _setupBindings() {
    // Core bindings
    _sebureInit =
        _dylib
            .lookup<NativeFunction<Int32 Function()>>('sebure_init')
            .asFunction();

    _sebureBlockchainInit =
        _dylib
            .lookup<NativeFunction<Int32 Function()>>('sebure_blockchain_init')
            .asFunction();

    _sebureStorageInit =
        _dylib
            .lookup<NativeFunction<Int32 Function(Pointer<Utf8>)>>(
              'sebure_storage_init',
            )
            .asFunction();

    _sebureNetworkInit =
        _dylib
            .lookup<NativeFunction<Int32 Function(Pointer<Utf8>)>>(
              'sebure_network_init',
            )
            .asFunction();

    _sebureNetworkStart =
        _dylib
            .lookup<NativeFunction<Int32 Function()>>('sebure_network_start')
            .asFunction();

    _sebureCreateAccount =
        _dylib
            .lookup<
              NativeFunction<
                Int32 Function(Pointer<Pointer<Utf8>>, Pointer<Pointer<Utf8>>)
              >
            >('sebure_create_account')
            .asFunction();

    _sebureGetAccountBalance =
        _dylib
            .lookup<
              NativeFunction<Int32 Function(Pointer<Utf8>, Pointer<Uint64>)>
            >('sebure_get_account_balance')
            .asFunction();

    _sebureFreString =
        _dylib
            .lookup<NativeFunction<Void Function(Pointer<Utf8>)>>(
              'sebure_free_string',
            )
            .asFunction();

    _sebureShutdown =
        _dylib
            .lookup<NativeFunction<Int32 Function()>>('sebure_shutdown')
            .asFunction();

    // Validation service bindings
    _sebureValidationServiceCreate =
        _dylib
            .lookup<
              NativeFunction<
                Uint32 Function(Uint32, Uint32, Uint32, Uint32, Uint32)
              >
            >('sebure_validation_service_create')
            .asFunction();

    _sebureValidationServiceStart =
        _dylib
            .lookup<NativeFunction<Int32 Function(Uint32)>>(
              'sebure_validation_service_start',
            )
            .asFunction();

    _sebureValidationServiceStop =
        _dylib
            .lookup<NativeFunction<Int32 Function(Uint32)>>(
              'sebure_validation_service_stop',
            )
            .asFunction();

    _sebureValidationServiceDestroy =
        _dylib
            .lookup<NativeFunction<Int32 Function(Uint32)>>(
              'sebure_validation_service_destroy',
            )
            .asFunction();

    _sebureValidationServiceStatus =
        _dylib
            .lookup<NativeFunction<Int32 Function(Uint32)>>(
              'sebure_validation_service_status',
            )
            .asFunction();

    _sebureValidationServicePause =
        _dylib
            .lookup<NativeFunction<Int32 Function(Uint32)>>(
              'sebure_validation_service_pause',
            )
            .asFunction();

    _sebureValidationServiceResume =
        _dylib
            .lookup<NativeFunction<Int32 Function(Uint32)>>(
              'sebure_validation_service_resume',
            )
            .asFunction();

    _sebureValidationServiceGetStats =
        _dylib
            .lookup<
              NativeFunction<
                Int32 Function(
                  Uint32,
                  Pointer<Uint64>,
                  Pointer<Uint64>,
                  Pointer<Uint64>,
                  Pointer<Uint64>,
                  Pointer<Uint32>,
                  Pointer<Double>,
                  Pointer<Uint64>,
                  Pointer<Float>,
                  Pointer<Float>,
                )
              >
            >('sebure_validation_service_get_stats')
            .asFunction();

    _sebureValidationServiceUpdateConfig =
        _dylib
            .lookup<
              NativeFunction<
                Int32 Function(Uint32, Uint32, Uint32, Uint32, Uint32, Uint32)
              >
            >('sebure_validation_service_update_config')
            .asFunction();

    _sebureValidationServiceAddTask =
        _dylib
            .lookup<
              NativeFunction<
                Int32 Function(
                  Uint32,
                  Int32,
                  Int32,
                  Pointer<Utf8>,
                  Pointer<Uint64>,
                )
              >
            >('sebure_validation_service_add_task')
            .asFunction();
  }

  /// Initialize the SEBURE blockchain core
  ErrorCode initCore() {
    final result = _sebureInit();
    return ErrorCode.values[result];
  }

  /// Initialize the blockchain with default configuration
  ErrorCode initBlockchain() {
    final result = _sebureBlockchainInit();
    return ErrorCode.values[result];
  }

  /// Initialize storage with the provided data directory
  ErrorCode initStorage(String dataDir) {
    final dataDirUtf8 = dataDir.toNativeUtf8();
    try {
      final result = _sebureStorageInit(dataDirUtf8);
      return ErrorCode.values[result];
    } finally {
      calloc.free(dataDirUtf8);
    }
  }

  /// Initialize network with the provided listen address
  ErrorCode initNetwork(String listenAddr) {
    final listenAddrUtf8 = listenAddr.toNativeUtf8();
    try {
      final result = _sebureNetworkInit(listenAddrUtf8);
      return ErrorCode.values[result];
    } finally {
      calloc.free(listenAddrUtf8);
    }
  }

  /// Start the network service
  ErrorCode startNetwork() {
    final result = _sebureNetworkStart();
    return ErrorCode.values[result];
  }

  /// Create a new account
  ({ErrorCode errorCode, String? publicKey, String? privateKey})
  createAccount() {
    final publicKeyOut = calloc<Pointer<Utf8>>();
    final privateKeyOut = calloc<Pointer<Utf8>>();

    try {
      final result = _sebureCreateAccount(publicKeyOut, privateKeyOut);

      if (result == ErrorCode.success.index) {
        final publicKey = publicKeyOut.value.toDartString();
        final privateKey = privateKeyOut.value.toDartString();

        // Free the strings allocated by Rust
        _sebureFreString(publicKeyOut.value);
        _sebureFreString(privateKeyOut.value);

        return (
          errorCode: ErrorCode.values[result],
          publicKey: publicKey,
          privateKey: privateKey,
        );
      } else {
        return (
          errorCode: ErrorCode.values[result],
          publicKey: null,
          privateKey: null,
        );
      }
    } finally {
      calloc.free(publicKeyOut);
      calloc.free(privateKeyOut);
    }
  }

  /// Get account balance
  ({ErrorCode errorCode, int? balance}) getAccountBalance(String address) {
    final addressUtf8 = address.toNativeUtf8();
    final balanceOut = calloc<Uint64>();

    try {
      final result = _sebureGetAccountBalance(addressUtf8, balanceOut);

      if (result == ErrorCode.success.index) {
        return (errorCode: ErrorCode.values[result], balance: balanceOut.value);
      } else {
        return (errorCode: ErrorCode.values[result], balance: null);
      }
    } finally {
      calloc.free(addressUtf8);
      calloc.free(balanceOut);
    }
  }

  /// Shutdown the SEBURE blockchain core
  ErrorCode shutdown() {
    final result = _sebureShutdown();
    return ErrorCode.values[result];
  }

  /// Create and initialize a validation service
  int createValidationService({
    required int maxCpuUsage,
    required int maxMemoryUsage,
    required int queueSizeLimit,
    required int processingTimeSlotMs,
    required int batchSize,
  }) {
    return _sebureValidationServiceCreate(
      maxCpuUsage,
      maxMemoryUsage,
      queueSizeLimit,
      processingTimeSlotMs,
      batchSize,
    );
  }

  /// Start a validation service
  int startValidationService(int serviceId) {
    return _sebureValidationServiceStart(serviceId);
  }

  /// Stop a validation service
  int stopValidationService(int serviceId) {
    return _sebureValidationServiceStop(serviceId);
  }

  /// Destroy a validation service
  int destroyValidationService(int serviceId) {
    return _sebureValidationServiceDestroy(serviceId);
  }

  /// Get the status of a validation service
  int getValidationServiceStatus(int serviceId) {
    return _sebureValidationServiceStatus(serviceId);
  }

  /// Pause a validation service
  int pauseValidationService(int serviceId) {
    return _sebureValidationServicePause(serviceId);
  }

  /// Resume a validation service
  int resumeValidationService(int serviceId) {
    return _sebureValidationServiceResume(serviceId);
  }

  /// Get statistics from a validation service
  ValidationServiceStats? getValidationServiceStats(int serviceId) {
    final transactionsProcessed = calloc<Uint64>();
    final blocksValidated = calloc<Uint64>();
    final blocksGenerated = calloc<Uint64>();
    final validationErrors = calloc<Uint64>();
    final queueLength = calloc<Uint32>();
    final avgTransactionTimeMs = calloc<Double>();
    final uptimeSeconds = calloc<Uint64>();
    final cpuUsage = calloc<Float>();
    final memoryUsage = calloc<Float>();

    try {
      final result = _sebureValidationServiceGetStats(
        serviceId,
        transactionsProcessed,
        blocksValidated,
        blocksGenerated,
        validationErrors,
        queueLength,
        avgTransactionTimeMs,
        uptimeSeconds,
        cpuUsage,
        memoryUsage,
      );

      if (result == 0) {
        return ValidationServiceStats(
          transactionsProcessed: transactionsProcessed.value,
          blocksValidated: blocksValidated.value,
          blocksGenerated: blocksGenerated.value,
          validationErrors: validationErrors.value,
          queueLength: queueLength.value,
          avgTransactionTimeMs: avgTransactionTimeMs.value,
          uptimeSeconds: uptimeSeconds.value,
          cpuUsage: cpuUsage.value.toDouble(),
          memoryUsage: memoryUsage.value.toDouble(),
        );
      } else {
        return null;
      }
    } finally {
      calloc.free(transactionsProcessed);
      calloc.free(blocksValidated);
      calloc.free(blocksGenerated);
      calloc.free(validationErrors);
      calloc.free(queueLength);
      calloc.free(avgTransactionTimeMs);
      calloc.free(uptimeSeconds);
      calloc.free(cpuUsage);
      calloc.free(memoryUsage);
    }
  }

  /// Update configuration for a validation service
  int updateValidationServiceConfig(
    int serviceId, {
    required int maxCpuUsage,
    required int maxMemoryUsage,
    required int queueSizeLimit,
    required int processingTimeSlotMs,
    required int batchSize,
  }) {
    return _sebureValidationServiceUpdateConfig(
      serviceId,
      maxCpuUsage,
      maxMemoryUsage,
      queueSizeLimit,
      processingTimeSlotMs,
      batchSize,
    );
  }

  /// Add a task to the validation service
  ({int errorCode, int? taskId}) addValidationServiceTask(
    int serviceId, {
    required int taskType,
    required int priority,
    String? data,
  }) {
    final taskIdOut = calloc<Uint64>();
    Pointer<Utf8>? dataUtf8;

    try {
      if (data != null) {
        dataUtf8 = data.toNativeUtf8();
      }

      final result = _sebureValidationServiceAddTask(
        serviceId,
        taskType,
        priority,
        dataUtf8 ?? Pointer<Utf8>.fromAddress(0),
        taskIdOut,
      );

      if (result == 0) {
        return (errorCode: 0, taskId: taskIdOut.value);
      } else {
        return (errorCode: result, taskId: null);
      }
    } finally {
      calloc.free(taskIdOut);
      if (dataUtf8 != null) {
        calloc.free(dataUtf8);
      }
    }
  }
}
