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
  unknown,
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

  // FFI function typedefs
  late final int Function() _sebureInit;
  late final int Function(Pointer<Utf8>) _sebureStorageInit;
  late final int Function(Pointer<Utf8>) _sebureNetworkInit;
  late final int Function() _sebureNetworkStart;
  late final int Function(Pointer<Pointer<Utf8>>, Pointer<Pointer<Utf8>>)
  _sebureCreateAccount;
  late final int Function(Pointer<Utf8>, Pointer<Uint64>)
  _sebureGetAccountBalance;
  late final void Function(Pointer<Utf8>) _sebureFreString;
  late final int Function() _sebureShutdown;

  // Set up all the function bindings
  void _setupBindings() {
    _sebureInit =
        _dylib
            .lookup<NativeFunction<Int32 Function()>>('sebure_init')
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
  }

  /// Initialize the SEBURE blockchain core
  ErrorCode initCore() {
    final result = _sebureInit();
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
}
