import 'dart:convert';
import 'package:flutter/foundation.dart';

import 'plugin_base.dart';

/// Represents the metadata for a SEBURE blockchain plugin
class PluginManifest {
  /// Unique identifier for the plugin
  final String id;

  /// Human-readable name of the plugin
  final String name;

  /// Version string following semantic versioning
  final String version;

  /// Author of the plugin
  final String author;

  /// Brief description of the plugin
  final String description;

  /// Minimum required SEBURE version
  final String minSebureVersion;

  /// Type of plugin
  final PluginType type;

  /// Whether the plugin is enabled
  bool isEnabled;

  /// Path to the plugin directory (set after discovery)
  String? path;

  /// List of permissions required by the plugin
  final List<String> permissions;

  /// Map of plugin-specific configuration
  final Map<String, dynamic> config;

  /// Supported platforms
  final List<String> platforms;

  /// Plugin dependencies
  final List<String> dependencies;

  /// Plugin entry point
  final String entryPoint;

  /// Constructor
  PluginManifest({
    required this.id,
    required this.name,
    required this.version,
    required this.author,
    required this.description,
    required this.minSebureVersion,
    required this.type,
    this.isEnabled = false,
    this.path,
    this.permissions = const [],
    this.config = const {},
    this.platforms = const ["macos", "linux"],
    this.dependencies = const [],
    this.entryPoint = "main.dart",
  });

  /// Create from JSON
  factory PluginManifest.fromJson(Map<String, dynamic> json) {
    return PluginManifest(
      id: json['id'] as String,
      name: json['name'] as String,
      version: json['version'] as String,
      author: json['author'] as String,
      description: json['description'] as String,
      minSebureVersion: json['minSebureVersion'] as String? ?? '0.1.0',
      type: _typeFromString(json['type'] as String? ?? 'other'),
      isEnabled: json['enabled'] as bool? ?? false,
      permissions: List<String>.from(json['permissions'] ?? []),
      config: Map<String, dynamic>.from(json['config'] ?? {}),
      platforms: List<String>.from(json['platforms'] ?? ["macos", "linux"]),
      dependencies: List<String>.from(json['dependencies'] ?? []),
      entryPoint: json['entryPoint'] as String? ?? 'main.dart',
    );
  }

  /// Convert to JSON
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'version': version,
      'author': author,
      'description': description,
      'minSebureVersion': minSebureVersion,
      'type': _typeToString(type),
      'enabled': isEnabled,
      'permissions': permissions,
      'config': config,
      'platforms': platforms,
      'dependencies': dependencies,
      'entryPoint': entryPoint,
    };
  }

  /// Convert plugin type enum to string
  static String _typeToString(PluginType type) {
    switch (type) {
      case PluginType.visualization:
        return 'visualization';
      case PluginType.wallet:
        return 'wallet';
      case PluginType.validation:
        return 'validation';
      case PluginType.analysis:
        return 'analysis';
      case PluginType.security:
        return 'security';
      case PluginType.network:
        return 'network';
      case PluginType.ui:
        return 'ui';
      case PluginType.other:
      default:
        return 'other';
    }
  }

  /// Convert string to plugin type enum
  static PluginType _typeFromString(String typeStr) {
    switch (typeStr.toLowerCase()) {
      case 'visualization':
        return PluginType.visualization;
      case 'wallet':
        return PluginType.wallet;
      case 'validation':
        return PluginType.validation;
      case 'analysis':
        return PluginType.analysis;
      case 'security':
        return PluginType.security;
      case 'network':
        return PluginType.network;
      case 'ui':
        return PluginType.ui;
      case 'other':
      default:
        return PluginType.other;
    }
  }

  @override
  String toString() {
    return 'PluginManifest(id: $id, name: $name, version: $version, type: ${_typeToString(type)}, enabled: $isEnabled)';
  }
}
