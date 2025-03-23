import 'package:flutter/material.dart';
import '../services/config_service.dart';
import 'package:provider/provider.dart';
import '../models/app_state.dart';

/// Screen for managing user preferences and application settings
class PreferencesScreen extends StatefulWidget {
  const PreferencesScreen({super.key});

  @override
  State<PreferencesScreen> createState() => _PreferencesScreenState();
}

class _PreferencesScreenState extends State<PreferencesScreen> {
  final ConfigService _configService = ConfigService.instance;
  late String _selectedTheme;
  late String _selectedLogLevel;
  late double _maxCpuUsage;
  late double _maxMemoryUsage;
  late double _maxNetworkUsage;
  late double _maxDiskUsage;
  late String _listenAddress;
  late List<String> _customPeers;
  late bool _isAddingPeer;
  final TextEditingController _peerController = TextEditingController();
  final _formKey = GlobalKey<FormState>();

  @override
  void initState() {
    super.initState();
    _loadPreferences();
    _isAddingPeer = false;
  }

  void _loadPreferences() {
    _selectedTheme = _configService.theme;
    _selectedLogLevel = _configService.logLevel;
    _maxCpuUsage = _configService.maxCpuUsage;
    _maxMemoryUsage = _configService.maxMemoryUsage;
    _maxNetworkUsage = _configService.maxNetworkUsage;
    _maxDiskUsage = _configService.maxDiskUsage;
    _listenAddress = _configService.listenAddress;
    _customPeers = _configService.customNetworkPeers;
  }

  @override
  void dispose() {
    _peerController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Preferences')),
      body: Form(
        key: _formKey,
        child: ListView(
          padding: const EdgeInsets.all(16.0),
          children: [
            _buildAppearanceSection(),
            const Divider(),
            _buildResourceLimitsSection(),
            const Divider(),
            _buildNetworkSection(),
            const Divider(),
            _buildAdvancedSection(),
            const SizedBox(height: 32),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                OutlinedButton(
                  onPressed: () {
                    _loadPreferences();
                    setState(() {});
                  },
                  child: const Text('Reset Changes'),
                ),
                ElevatedButton(
                  onPressed: _savePreferences,
                  child: const Text('Save Preferences'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAppearanceSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Appearance',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 16),
        const Text('Theme'),
        const SizedBox(height: 8),
        SegmentedButton<String>(
          segments: const [
            ButtonSegment<String>(
              value: 'system',
              label: Text('System'),
              icon: Icon(Icons.brightness_auto),
            ),
            ButtonSegment<String>(
              value: 'light',
              label: Text('Light'),
              icon: Icon(Icons.light_mode),
            ),
            ButtonSegment<String>(
              value: 'dark',
              label: Text('Dark'),
              icon: Icon(Icons.dark_mode),
            ),
          ],
          selected: {_selectedTheme},
          onSelectionChanged: (Set<String> selection) {
            setState(() {
              _selectedTheme = selection.first;
            });
          },
        ),
      ],
    );
  }

  Widget _buildResourceLimitsSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Resource Limits',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 16),
        const Text('CPU Usage Limit (%)'),
        Slider(
          value: _maxCpuUsage,
          min: 5,
          max: 90,
          divisions: 17,
          label: _maxCpuUsage.round().toString(),
          onChanged: (value) {
            setState(() {
              _maxCpuUsage = value;
            });
          },
        ),
        const SizedBox(height: 16),
        const Text('Memory Usage Limit (MB)'),
        Slider(
          value: _maxMemoryUsage,
          min: 100,
          max: 2000,
          divisions: 19,
          label: _maxMemoryUsage.round().toString(),
          onChanged: (value) {
            setState(() {
              _maxMemoryUsage = value;
            });
          },
        ),
        const SizedBox(height: 16),
        const Text('Network Bandwidth Limit (MB/hour)'),
        Slider(
          value: _maxNetworkUsage,
          min: 10,
          max: 500,
          divisions: 49,
          label: _maxNetworkUsage.round().toString(),
          onChanged: (value) {
            setState(() {
              _maxNetworkUsage = value;
            });
          },
        ),
        const SizedBox(height: 16),
        const Text('Disk Space Limit (GB)'),
        Slider(
          value: _maxDiskUsage,
          min: 1,
          max: 50,
          divisions: 49,
          label: _maxDiskUsage.round().toString(),
          onChanged: (value) {
            setState(() {
              _maxDiskUsage = value;
            });
          },
        ),
      ],
    );
  }

  Widget _buildNetworkSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Network Settings',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 16),
        TextFormField(
          initialValue: _listenAddress,
          decoration: const InputDecoration(
            labelText: 'Listen Address',
            helperText: 'Format: IP:Port (e.g., 127.0.0.1:9000)',
            border: OutlineInputBorder(),
          ),
          validator: (value) {
            if (value == null || value.isEmpty) {
              return 'Please enter a listen address';
            }
            if (!RegExp(r'^.+:\d+$').hasMatch(value)) {
              return 'Invalid format. Use IP:Port';
            }
            return null;
          },
          onChanged: (value) {
            _listenAddress = value;
          },
        ),
        const SizedBox(height: 24),
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text(
              'Custom Network Peers',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
            ),
            IconButton(
              icon: const Icon(Icons.add),
              onPressed: () {
                setState(() {
                  _isAddingPeer = true;
                });
              },
            ),
          ],
        ),
        const SizedBox(height: 8),
        if (_isAddingPeer)
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  controller: _peerController,
                  decoration: const InputDecoration(
                    labelText: 'Peer Address',
                    helperText: 'Format: IP:Port',
                    border: OutlineInputBorder(),
                  ),
                  validator: (value) {
                    if (value == null || value.isEmpty) {
                      return 'Please enter a peer address';
                    }
                    if (!RegExp(r'^.+:\d+$').hasMatch(value)) {
                      return 'Invalid format. Use IP:Port';
                    }
                    return null;
                  },
                ),
              ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.check),
                onPressed: () {
                  if (_peerController.text.isNotEmpty &&
                      RegExp(r'^.+:\d+$').hasMatch(_peerController.text)) {
                    setState(() {
                      _customPeers.add(_peerController.text);
                      _peerController.clear();
                      _isAddingPeer = false;
                    });
                  }
                },
              ),
              IconButton(
                icon: const Icon(Icons.close),
                onPressed: () {
                  setState(() {
                    _peerController.clear();
                    _isAddingPeer = false;
                  });
                },
              ),
            ],
          ),
        const SizedBox(height: 8),
        if (_customPeers.isEmpty)
          const Padding(
            padding: EdgeInsets.all(8.0),
            child: Text('No custom peers added'),
          )
        else
          ListView.builder(
            shrinkWrap: true,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: _customPeers.length,
            itemBuilder: (context, index) {
              return ListTile(
                title: Text(_customPeers[index]),
                trailing: IconButton(
                  icon: const Icon(Icons.delete),
                  onPressed: () {
                    setState(() {
                      _customPeers.removeAt(index);
                    });
                  },
                ),
              );
            },
          ),
      ],
    );
  }

  Widget _buildAdvancedSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text(
          'Advanced Settings',
          style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 16),
        const Text('Log Level'),
        const SizedBox(height: 8),
        DropdownButtonFormField<String>(
          value: _selectedLogLevel,
          decoration: const InputDecoration(border: OutlineInputBorder()),
          items: const [
            DropdownMenuItem(value: 'error', child: Text('Error')),
            DropdownMenuItem(value: 'warn', child: Text('Warning')),
            DropdownMenuItem(value: 'info', child: Text('Info')),
            DropdownMenuItem(value: 'debug', child: Text('Debug')),
            DropdownMenuItem(value: 'trace', child: Text('Trace')),
          ],
          onChanged: (value) {
            if (value != null) {
              setState(() {
                _selectedLogLevel = value;
              });
            }
          },
        ),
        const SizedBox(height: 24),
        OutlinedButton.icon(
          onPressed: () {
            showDialog(
              context: context,
              builder:
                  (context) => AlertDialog(
                    title: const Text('Reset All Settings'),
                    content: const Text(
                      'This will reset all preferences to their default values. '
                      'This action cannot be undone.',
                    ),
                    actions: [
                      TextButton(
                        onPressed: () => Navigator.of(context).pop(),
                        child: const Text('Cancel'),
                      ),
                      TextButton(
                        onPressed: () async {
                          await _configService.clearAllConfig();
                          _loadPreferences();
                          // ignore: use_build_context_synchronously
                          Navigator.of(context).pop();
                          setState(() {});
                          // ignore: use_build_context_synchronously
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(
                              content: Text('All settings have been reset'),
                            ),
                          );
                        },
                        style: TextButton.styleFrom(
                          foregroundColor: Colors.red,
                        ),
                        child: const Text('Reset All'),
                      ),
                    ],
                  ),
            );
          },
          icon: const Icon(Icons.restore),
          label: const Text('Reset All Settings'),
          style: OutlinedButton.styleFrom(foregroundColor: Colors.red),
        ),
      ],
    );
  }

  Future<void> _savePreferences() async {
    if (_formKey.currentState!.validate()) {
      // Save theme
      await _configService.setTheme(_selectedTheme);

      // Save resource limits
      await _configService.setMaxCpuUsage(_maxCpuUsage);
      await _configService.setMaxMemoryUsage(_maxMemoryUsage);
      await _configService.setMaxNetworkUsage(_maxNetworkUsage);
      await _configService.setMaxDiskUsage(_maxDiskUsage);

      // Save network settings
      await _configService.setListenAddress(_listenAddress);
      await _configService.setCustomNetworkPeers(_customPeers);

      // Save advanced settings
      await _configService.setLogLevel(_selectedLogLevel);

      // Update app state to reflect theme changes
      if (mounted) {
        Provider.of<AppState>(context, listen: false).updateNodeStatus(
          isRunning:
              Provider.of<AppState>(context, listen: false).isNodeRunning,
        );

        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Preferences saved successfully'),
            backgroundColor: Colors.green,
          ),
        );
      }
    }
  }
}
