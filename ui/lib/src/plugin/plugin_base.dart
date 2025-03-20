import 'dart:async';
import 'package:flutter/foundation.dart';
import 'plugin_manifest.dart';

/// Base class for all SEBURE blockchain plugins
abstract class SeburePlugin {
  /// The plugin manifest containing metadata
  final PluginManifest manifest;

  /// Constructor
  SeburePlugin(this.manifest);

  /// Initialize the plugin
  Future<void> initialize();

  /// Shutdown the plugin
  Future<void> shutdown();

  /// Get plugin ID
  String get id => manifest.id;

  /// Get plugin name
  String get name => manifest.name;

  /// Get plugin version
  String get version => manifest.version;

  /// Get plugin author
  String get author => manifest.author;

  /// Get plugin description
  String get description => manifest.description;

  /// Check if the plugin is enabled
  bool get isEnabled => manifest.isEnabled;

  @override
  String toString() => 'SeburePlugin(id: $id, name: $name, version: $version)';
}

/// Plugin lifecycle events
enum PluginLifecycleEvent {
  initialized,
  starting,
  started,
  stopping,
  stopped,
  updated,
  error,
}

/// Plugin type
enum PluginType {
  visualization,
  wallet,
  validation,
  analysis,
  security,
  network,
  ui,
  other,
}
