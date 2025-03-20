# Testing the SEBURE Desktop Application Framework

This document provides instructions for testing the components implemented in Task 2.1 (Desktop Application Framework) to ensure they work correctly and don't break existing functionality.

## Test Categories

There are several levels of testing available:

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test how components work together
3. **Manual Tests**: UI interactions and FFI functionality that's difficult to automate

## Running the Tests

### Unit Tests

To run all the unit tests:

```bash
cd ui
flutter test
```

To run specific test files:

```bash
flutter test test/desktop_framework_tests.dart
```

### Integration Tests

Integration tests require a device (or simulator) to run:

```bash
cd ui
flutter test integration_test/app_test.dart
```

## Test Coverage

### FFI Layer Tests

The FFI layer tests verify that:
- The SebureFFI class initializes correctly
- Error codes are properly mapped
- FFI calls don't crash the application

**Note:** Some FFI tests may be skipped if the native library is not available, which is expected in a test environment without a compiled Rust library.

### Configuration Service Tests

The ConfigService tests verify that:
- Default values are correctly initialized
- Configuration can be updated and persisted
- Complex configuration structures (JSON) can be stored and retrieved

### Plugin Architecture Tests

The plugin system tests verify that:
- Plugin manifests can be parsed from JSON
- The PluginManager initializes correctly
- Plugins can be discovered, loaded, enabled, and disabled

### Blockchain Service Tests

These tests verify that:
- The service layer provides proper abstractions over the FFI layer
- Mock implementations work correctly for development and testing
- Resource usage and network stats are properly formatted

## Manual Testing

Some aspects are better tested manually:

1. **Application Startup and Shutdown**:
   ```bash
   cd ui
   flutter run -d macos  # or linux, depending on your platform
   ```
   Verify that:
   - The application starts without errors
   - The splash screen appears and transitions to the main UI
   - The application can be closed properly without errors

2. **Resource Monitoring**:
   - Start the application
   - Toggle the node on/off
   - Verify that resource usage statistics update correctly

3. **Configuration Changes**:
   - Change settings in the application
   - Restart the application
   - Verify that settings are persisted

4. **Plugin System**:
   - The sample plugin should load automatically 
   - You should see log messages about the plugin being loaded

## Troubleshooting Common Issues

### FFI Library Not Found

If you see errors about missing FFI libraries:

```
Error: Failed to initialize the SEBURE FFI bindings (library not found)
```

This is expected if the Rust FFI library hasn't been compiled or isn't in the correct location. The application will use mock implementations for development.

### SharedPreferences Issues

If tests related to ConfigService fail with SharedPreferences errors, make sure you're setting up the mock properly:

```dart
SharedPreferences.setMockInitialValues({});
```

This should be done in the setUp function of the test.

### Plugin Discovery Problems

If plugin tests fail, verify that:
- The plugin directory exists with correct permissions
- The manifest.json file is valid JSON
- You're running the test with the right working directory

## Verifying Test Results

After running tests, look for:
- Green check marks for passing tests
- Detailed error messages for failing tests
- Test coverage reports (if enabled)

A common output for successful tests looks like:

```
✓ ConfigService initializes with default values
✓ ConfigService can update and persist values
✓ PluginManifest correctly parses JSON
✓ PluginManager initializes correctly
✓ BlockchainService mocks work correctly

All tests passed!
```

## Adding New Tests

When adding new components or modifying existing ones, consider:
1. Adding new test cases to existing test files
2. Creating new test files for significant new features
3. Updating integration tests for UI changes

Follow the existing patterns for writing tests:
- Group related tests with `group()`
- Use descriptive test names
- Mock external dependencies
- Test both success and failure cases
