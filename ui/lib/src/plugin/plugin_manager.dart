import 'dart:async';
import 'dart:io';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as path;
import 'package:path_provider/path_provider.dart';

import 'plugin_base.dart';
import 'plugin_manifest.dart';
import 'sample_plugin.dart';

/// Manages plugins for the SEBURE Blockchain desktop application
class PluginManager {
  // Singleton instance
  static final PluginManager _instance = PluginManager._internal();
  static PluginManager get instance => _instance;

  // Private constructor for singleton
  PluginManager._internal();

  // List of loaded plugins
  final List<SeburePlugin> _loadedPlugins = [];
  bool _isInitialized = false;

  // Getters
  List<SeburePlugin> get loadedPlugins => List.unmodifiable(_loadedPlugins);
  bool get isInitialized => _isInitialized;

  /// Initialize the plugin manager and load all enabled plugins
  Future<bool> initialize() async {
    if (_isInitialized) return true;

    try {
      // Get plugins directory
      final pluginsDir = await _getPluginsDirectory();
      final pluginsDirExists = await Directory(pluginsDir).exists();

      if (!pluginsDirExists) {
        await Directory(pluginsDir).create(recursive: true);
        debugPrint('Created plugins directory at: $pluginsDir');
        _isInitialized = true;
        return true; // No plugins to load yet
      }

      // Find and load all valid plugins
      final plugins = await _discoverPlugins(pluginsDir);
      for (final plugin in plugins) {
        if (plugin.isEnabled) {
          final loaded = await _loadPlugin(plugin);
          if (loaded != null) {
            _loadedPlugins.add(loaded);
            debugPrint('Loaded plugin: ${plugin.name} v${plugin.version}');
          }
        }
      }

      // Load sample plugin for demonstration purposes
      await _loadSamplePluginForDemo();

      _isInitialized = true;
      return true;
    } catch (e) {
      debugPrint('Error initializing plugin manager: $e');
      return false;
    }
  }

  /// Load a sample plugin for demonstration purposes
  Future<void> _loadSamplePluginForDemo() async {
    try {
      // Create a sample plugin instance using the factory
      final samplePlugin = SamplePluginFactory.createInstance();

      // Check if we already have this plugin loaded (avoid duplicates)
      if (!_loadedPlugins.any((p) => p.id == samplePlugin.id)) {
        // Initialize the plugin
        await samplePlugin.initialize();

        // Add to loaded plugins
        _loadedPlugins.add(samplePlugin);

        debugPrint(
          'Loaded demo plugin: ${samplePlugin.name} v${samplePlugin.version}',
        );
      }
    } catch (e) {
      debugPrint('Error loading sample plugin for demo: $e');
    }
  }

  /// Get the plugins directory path
  Future<String> _getPluginsDirectory() async {
    final appDir = await getApplicationDocumentsDirectory();
    return path.join(appDir.path, 'sebure_plugins');
  }

  /// Discover all plugin manifests in the plugins directory
  Future<List<PluginManifest>> _discoverPlugins(String pluginsDir) async {
    final List<PluginManifest> discovered = [];

    try {
      final dir = Directory(pluginsDir);
      final entities = await dir.list().toList();

      for (final entity in entities) {
        if (entity is Directory) {
          final manifestFile = File(path.join(entity.path, 'manifest.json'));
          if (await manifestFile.exists()) {
            try {
              final content = await manifestFile.readAsString();
              final json = jsonDecode(content);
              final manifest = PluginManifest.fromJson(json);
              manifest.path = entity.path;
              discovered.add(manifest);
            } catch (e) {
              debugPrint('Error parsing manifest in ${entity.path}: $e');
            }
          }
        }
      }
    } catch (e) {
      debugPrint('Error discovering plugins: $e');
    }

    return discovered;
  }

  /// Load a plugin using its manifest
  Future<SeburePlugin?> _loadPlugin(PluginManifest manifest) async {
    try {
      // In a real implementation, this would dynamically load Dart code
      // For now, we'll create a simple plugin instance
      final plugin = _createPluginInstance(manifest);

      if (plugin != null) {
        await plugin.initialize();
        return plugin;
      }
    } catch (e) {
      debugPrint('Error loading plugin ${manifest.name}: $e');
    }

    return null;
  }

  /// Create a plugin instance based on its type
  SeburePlugin? _createPluginInstance(PluginManifest manifest) {
    // In a real implementation, this would use reflection or a registry
    // to dynamically load plugins based on their manifest information

    // For demonstration purposes, we'll just create a sample plugin instance
    // when we detect a plugin with the ID 'sample-plugin'
    if (manifest.id == 'sample-plugin') {
      return SamplePlugin(manifest);
    } else if (manifest.id == 'debug-sample-plugin') {
      // Create using the factory as another example approach
      return SamplePluginFactory.createInstance();
    }

    // In a production implementation, we'd have a more sophisticated
    // plugin loading mechanism, possibly using code generation or
    // dynamic library loading
    return null;
  }

  /// Enable a plugin by name
  Future<bool> enablePlugin(String pluginId) async {
    try {
      final pluginsDir = await _getPluginsDirectory();
      final potentialPlugins = await _discoverPlugins(pluginsDir);

      for (final manifest in potentialPlugins) {
        if (manifest.id == pluginId && !manifest.isEnabled) {
          // Update the manifest
          manifest.isEnabled = true;
          await _saveManifest(manifest);

          // Load the plugin
          final plugin = await _loadPlugin(manifest);
          if (plugin != null) {
            _loadedPlugins.add(plugin);
            return true;
          }
        }
      }

      return false;
    } catch (e) {
      debugPrint('Error enabling plugin $pluginId: $e');
      return false;
    }
  }

  /// Disable a plugin by name
  Future<bool> disablePlugin(String pluginId) async {
    try {
      // Find the plugin in loaded plugins
      final pluginIndex = _loadedPlugins.indexWhere(
        (p) => p.manifest.id == pluginId,
      );

      if (pluginIndex >= 0) {
        // Get the plugin
        final plugin = _loadedPlugins[pluginIndex];

        // Update the manifest
        plugin.manifest.isEnabled = false;
        await _saveManifest(plugin.manifest);

        // Shutdown the plugin
        await plugin.shutdown();

        // Remove from loaded plugins
        _loadedPlugins.removeAt(pluginIndex);

        return true;
      }

      return false;
    } catch (e) {
      debugPrint('Error disabling plugin $pluginId: $e');
      return false;
    }
  }

  /// Save a plugin manifest back to disk
  Future<void> _saveManifest(PluginManifest manifest) async {
    if (manifest.path == null) return;

    try {
      final manifestFile = File(path.join(manifest.path!, 'manifest.json'));
      final json = jsonEncode(manifest.toJson());
      await manifestFile.writeAsString(json);
    } catch (e) {
      debugPrint('Error saving manifest for ${manifest.name}: $e');
    }
  }

  /// Install a plugin from a directory
  Future<bool> installPlugin(String sourceDir) async {
    try {
      final pluginsDir = await _getPluginsDirectory();
      final sourceDirectory = Directory(sourceDir);

      // Check if source directory exists and contains a manifest
      if (!await sourceDirectory.exists()) {
        debugPrint('Source directory does not exist: $sourceDir');
        return false;
      }

      final manifestFile = File(path.join(sourceDir, 'manifest.json'));
      if (!await manifestFile.exists()) {
        debugPrint('No manifest file found in source directory');
        return false;
      }

      // Parse the manifest
      final content = await manifestFile.readAsString();
      final json = jsonDecode(content);
      final manifest = PluginManifest.fromJson(json);

      // Create destination directory
      final destinationDir = path.join(pluginsDir, manifest.id);
      final destDirectory = Directory(destinationDir);

      if (await destDirectory.exists()) {
        // Plugin already installed, could implement update logic here
        debugPrint('Plugin ${manifest.id} already installed');
        return false;
      }

      // Copy plugin files
      await destDirectory.create(recursive: true);
      await _copyDirectory(sourceDirectory, destDirectory);

      // Enable the plugin if it's set to be enabled
      if (manifest.isEnabled) {
        return await enablePlugin(manifest.id);
      }

      return true;
    } catch (e) {
      debugPrint('Error installing plugin: $e');
      return false;
    }
  }

  /// Recursively copy a directory
  Future<void> _copyDirectory(Directory source, Directory destination) async {
    await for (final entity in source.list(recursive: false)) {
      if (entity is Directory) {
        final newDirectory = Directory(
          path.join(destination.path, path.basename(entity.path)),
        );
        await newDirectory.create();
        await _copyDirectory(entity, newDirectory);
      } else if (entity is File) {
        await entity.copy(
          path.join(destination.path, path.basename(entity.path)),
        );
      }
    }
  }

  /// Uninstall a plugin by ID
  Future<bool> uninstallPlugin(String pluginId) async {
    try {
      // Disable the plugin first if it's loaded
      await disablePlugin(pluginId);

      // Remove the plugin directory
      final pluginsDir = await _getPluginsDirectory();
      final pluginDir = path.join(pluginsDir, pluginId);

      final directory = Directory(pluginDir);
      if (await directory.exists()) {
        await directory.delete(recursive: true);
        return true;
      }

      return false;
    } catch (e) {
      debugPrint('Error uninstalling plugin $pluginId: $e');
      return false;
    }
  }

  /// List all available plugins (both enabled and disabled)
  Future<List<PluginManifest>> listAvailablePlugins() async {
    try {
      final pluginsDir = await _getPluginsDirectory();
      return await _discoverPlugins(pluginsDir);
    } catch (e) {
      debugPrint('Error listing available plugins: $e');
      return [];
    }
  }

  /// Get a plugin instance by ID
  SeburePlugin? getPluginById(String pluginId) {
    try {
      return _loadedPlugins.firstWhere((p) => p.manifest.id == pluginId);
    } catch (e) {
      return null;
    }
  }

  /// Shutdown all plugins
  Future<void> shutdownAll() async {
    for (final plugin in _loadedPlugins) {
      try {
        await plugin.shutdown();
      } catch (e) {
        debugPrint('Error shutting down plugin ${plugin.manifest.name}: $e');
      }
    }

    _loadedPlugins.clear();
  }
}
