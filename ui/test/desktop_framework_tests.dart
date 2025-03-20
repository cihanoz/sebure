import 'package:flutter_test/flutter_test.dart';
import 'package:sebure_ui/src/services/config_service.dart';
import 'package:sebure_ui/src/services/blockchain_service.dart';
import 'package:sebure_ui/src/plugin/plugin_manager.dart';
import 'package:sebure_ui/src/plugin/plugin_manifest.dart';
import 'package:sebure_ui/src/plugin/plugin_base.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  // Set up new instances for each test to avoid state leakage
  setUp(() {
    // Clear and reset mocks between tests
    SharedPreferences.setMockInitialValues({});
  });

  group('Configuration Service Tests', () {
    test('ConfigService initializes with default values', () async {
      // Create a fresh instance for this test
      final configService = ConfigService.instance;
      await configService.initialize();

      // Check default values
      expect(configService.isNodeEnabled, equals(false));
      expect(configService.maxCpuUsage, equals(20.0));
      expect(configService.maxMemoryUsage, equals(500.0));
      expect(configService.maxNetworkUsage, equals(100.0));
      expect(configService.maxDiskUsage, equals(10.0));
      expect(configService.theme, equals('system'));
    });

    test('ConfigService can update and persist values', () async {
      // Create a fresh instance for this test
      SharedPreferences.setMockInitialValues({});
      final configService = ConfigService.instance;
      await configService.initialize();

      // Update values
      await configService.setMaxCpuUsage(30.0);
      await configService.setTheme('dark');
      await configService.setNodeEnabled(true);

      // Verify updated values
      expect(configService.maxCpuUsage, equals(30.0));
      expect(configService.theme, equals('dark'));
      expect(configService.isNodeEnabled, equals(true));

      // Test complex configuration
      final schedule = [
        {'day': 'Monday', 'startTime': '10:00', 'endTime': '16:00'},
        {'day': 'Wednesday', 'startTime': '14:00', 'endTime': '20:00'},
      ];
      await configService.setValidationSchedule(schedule);

      final retrievedSchedule = configService.validationSchedule;
      expect(retrievedSchedule.length, equals(2));
      expect(retrievedSchedule[0]['day'], equals('Monday'));
      expect(retrievedSchedule[1]['startTime'], equals('14:00'));
    });
  });

  group('Plugin Architecture Tests', () {
    test('PluginManifest correctly parses JSON', () {
      final json = {
        'id': 'test-plugin',
        'name': 'Test Plugin',
        'version': '1.0.0',
        'author': 'SEBURE Team',
        'description': 'A test plugin',
        'type': 'visualization',
        'enabled': true,
        'minSebureVersion': '0.1.0',
        'permissions': ['network', 'storage'],
        'platforms': ['macos', 'linux'],
      };

      final manifest = PluginManifest.fromJson(json);

      expect(manifest.id, equals('test-plugin'));
      expect(manifest.name, equals('Test Plugin'));
      expect(manifest.version, equals('1.0.0'));
      expect(manifest.author, equals('SEBURE Team'));
      expect(manifest.description, equals('A test plugin'));
      expect(manifest.isEnabled, isTrue);
      expect(manifest.permissions.length, equals(2));
      expect(manifest.permissions.contains('network'), isTrue);
      expect(manifest.platforms.length, equals(2));
    });

    test('Plugin base class implementation works', () {
      // Create a test plugin and manifest for testing
      final manifest = PluginManifest(
        id: 'test-plugin',
        name: 'Test Plugin',
        version: '1.0.0',
        author: 'SEBURE Team',
        description: 'A test plugin',
        minSebureVersion: '0.1.0',
        type: PluginType.other,
      );

      // Create a test implementation of SeburePlugin for testing
      final plugin = _TestPlugin(manifest);

      // Test basic functionality
      expect(plugin.id, equals('test-plugin'));
      expect(plugin.name, equals('Test Plugin'));
      expect(plugin.version, equals('1.0.0'));
      expect(plugin.author, equals('SEBURE Team'));
    });
  });

  group('BlockchainService Tests', () {
    test('BlockchainService mocks work correctly', () async {
      // This is just testing that our mocks function correctly without errors
      final service = BlockchainService.instance;

      // Resource usage should return some values
      final usage = await service.getResourceUsage();
      expect(usage, isNotEmpty);
      expect(usage['cpu'], isNotNull);
      expect(usage['memory'], isNotNull);
      expect(usage['network'], isNotNull);
      expect(usage['disk'], isNotNull);

      // Network stats should return some values
      final stats = await service.getNetworkStats();
      expect(stats['peers'], isNotNull);
      expect(stats['validatedTransactions'], isNotNull);
      expect(stats['blockHeight'], isNotNull);

      // Balance should return a value
      final balance = await service.getBalance('mock-address');
      expect(balance, isNotNull);
    });
  });
}

/// Test implementation of SeburePlugin for testing
class _TestPlugin extends SeburePlugin {
  _TestPlugin(super.manifest);

  bool _initialized = false;

  @override
  Future<void> initialize() async {
    _initialized = true;
  }

  @override
  Future<void> shutdown() async {
    _initialized = false;
  }

  bool get isInitialized => _initialized;
}
