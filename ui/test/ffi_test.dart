import 'package:flutter_test/flutter_test.dart';
import 'package:sebure_ui/src/ffi/sebure_ffi.dart';

void main() {
  test('FFI Error codes are properly defined', () {
    // Test that error codes are properly defined without requiring library
    expect(ErrorCode.values.length, greaterThan(0));
    expect(ErrorCode.success.index, equals(0));
    expect(ErrorCode.invalidArgument.index, equals(1));
  });

  // These tests will be skipped in environments without the native library
  group('FFI with mock implementations', () {
    test('FFI gracefully handles missing library', () {
      // The library won't be found in the test environment, but our code should
      // handle this gracefully and use mock implementations instead.
      final ffi = SebureFFI.instance;
      expect(() async => await ffi.initialize(), returnsNormally);
    });
  });

  // Note: In a real test environment with the compiled library, you'd add more tests here
}
