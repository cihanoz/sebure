import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'plugin_base.dart';
import 'plugin_manifest.dart';

/// A sample plugin to demonstrate the plugin architecture
class SamplePlugin extends SeburePlugin {
  bool _isRunning = false;
  Timer? _statsTimer;

  SamplePlugin(PluginManifest manifest) : super(manifest);

  @override
  Future<void> initialize() async {
    debugPrint('Initializing $name v$version');

    // In a real plugin, we might load configuration, connect to services, etc.
    _isRunning = true;

    // Example: Start a timer that might collect statistics or perform periodic tasks
    _statsTimer = Timer.periodic(const Duration(minutes: 5), _collectStats);

    debugPrint('$name initialized successfully');
  }

  @override
  Future<void> shutdown() async {
    debugPrint('Shutting down $name');

    // Clean up resources
    _statsTimer?.cancel();
    _isRunning = false;

    debugPrint('$name shut down successfully');
  }

  /// Example of a plugin method that might be called by the application
  String getStatus() {
    return _isRunning ? 'Running' : 'Stopped';
  }

  /// Example widget that the plugin might provide to the UI
  Widget buildDashboardWidget(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              children: [
                Icon(
                  Icons.extension,
                  color: _isRunning ? Colors.green : Colors.grey,
                ),
                const SizedBox(width: 8),
                Text(
                  name,
                  style: const TextStyle(
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const Spacer(),
                Text(
                  'v$version',
                  style: TextStyle(color: Colors.grey[600], fontSize: 12),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(description),
            const SizedBox(height: 16),
            Text('Status: ${getStatus()}'),
            const SizedBox(height: 8),
            Text('Author: $author'),
          ],
        ),
      ),
    );
  }

  /// Example of a periodic task the plugin might perform
  void _collectStats(Timer timer) {
    if (!_isRunning) return;

    // In a real plugin, this might collect statistics, perform maintenance, etc.
    debugPrint('$name collecting statistics...');
  }
}

/// Factory for creating sample plugins
class SamplePluginFactory {
  /// Create a sample plugin instance
  static SeburePlugin createInstance() {
    // Create a manifest for the sample plugin
    final manifest = PluginManifest(
      id: 'sample-plugin',
      name: 'Sample Plugin',
      version: '1.0.0',
      author: 'SEBURE Team',
      description: 'A sample plugin to demonstrate the plugin architecture',
      minSebureVersion: '0.1.0',
      type: PluginType.other,
      isEnabled: true,
    );

    // Create and return the plugin instance
    return SamplePlugin(manifest);
  }
}
