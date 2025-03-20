import 'dart:async';
import 'package:flutter/material.dart';
import '../services/validation_service.dart';
import '../ffi/sebure_ffi.dart' hide ValidationServiceStatus;

/// Screen for controlling validation service settings and monitoring status
class ValidationSettingsScreen extends StatefulWidget {
  const ValidationSettingsScreen({super.key});

  @override
  State<ValidationSettingsScreen> createState() =>
      _ValidationSettingsScreenState();
}

class _ValidationSettingsScreenState extends State<ValidationSettingsScreen> {
  // Reference to the validation service
  final _validationService = ValidationService.instance;

  // Streams for receiving updates
  StreamSubscription<ValidationServiceStats>? _statsSubscription;
  StreamSubscription<ValidationServiceStatus>? _statusSubscription;

  // Current state
  ValidationServiceStats? _stats;
  ValidationServiceStatus _status = ValidationServiceStatus.stopped;
  bool _isInitialized = false;

  // Configuration form controller
  final _formKey = GlobalKey<FormState>();

  // Text controllers for form fields
  final _cpuController = TextEditingController();
  final _memoryController = TextEditingController();
  final _queueController = TextEditingController();
  final _timeSlotController = TextEditingController();
  final _batchSizeController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _initializeService();
  }

  Future<void> _initializeService() async {
    // Initialize the validation service
    final initialized = await _validationService.initialize();
    if (mounted) {
      setState(() {
        _isInitialized = initialized;
      });
    }

    if (initialized) {
      // Get the current service configuration
      _updateFormFields();

      // Subscribe to service status updates
      _statusSubscription = _validationService.statusStream.listen((status) {
        if (mounted) {
          setState(() {
            _status = status;
          });
        }
      });

      // Subscribe to service stats updates
      _statsSubscription = _validationService.statsStream.listen((stats) {
        if (mounted) {
          setState(() {
            _stats = stats;
          });
        }
      });

      // Get current service status and stats
      _status = _validationService.status;
      _stats = _validationService.stats;
    }
  }

  void _updateFormFields() {
    // Set text controllers with current configuration values
    final config =
        ValidationServiceConfig.defaultConfig(); // Ideally get from service
    _cpuController.text = config.maxCpuUsage.toString();
    _memoryController.text = config.maxMemoryUsage.toString();
    _queueController.text = config.queueSizeLimit.toString();
    _timeSlotController.text = config.processingTimeSlotMs.toString();
    _batchSizeController.text = config.batchSize.toString();
  }

  Future<void> _updateConfiguration() async {
    if (_formKey.currentState!.validate()) {
      // Create a new configuration from form values
      final config = ValidationServiceConfig(
        maxCpuUsage: int.parse(_cpuController.text),
        maxMemoryUsage: int.parse(_memoryController.text),
        queueSizeLimit: int.parse(_queueController.text),
        processingTimeSlotMs: int.parse(_timeSlotController.text),
        batchSize: int.parse(_batchSizeController.text),
      );

      // Update the service configuration
      final success = await _validationService.updateConfig(config);

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              success
                  ? 'Configuration updated successfully'
                  : 'Failed to update configuration',
            ),
            backgroundColor: success ? Colors.green : Colors.red,
          ),
        );
      }
    }
  }

  Future<void> _startValidation() async {
    final success = await _validationService.start();
    if (mounted && !success) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to start validation service'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _stopValidation() async {
    final success = await _validationService.stop();
    if (mounted && !success) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to stop validation service'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _pauseValidation() async {
    final success = await _validationService.pause();
    if (mounted && !success) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to pause validation service'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _resumeValidation() async {
    final success = await _validationService.resume();
    if (mounted && !success) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Failed to resume validation service'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

  Future<void> _addBlockGenerationTask() async {
    final taskId = await _validationService.addBlockGenerationTask();
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            taskId != null
                ? 'Added block generation task with ID: $taskId'
                : 'Failed to add block generation task',
          ),
          backgroundColor: taskId != null ? Colors.green : Colors.red,
        ),
      );
    }
  }

  @override
  void dispose() {
    // Cancel stream subscriptions
    _statsSubscription?.cancel();
    _statusSubscription?.cancel();

    // Dispose text controllers
    _cpuController.dispose();
    _memoryController.dispose();
    _queueController.dispose();
    _timeSlotController.dispose();
    _batchSizeController.dispose();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!_isInitialized) {
      return Scaffold(
        appBar: AppBar(title: const Text('Validation Settings')),
        body: const Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CircularProgressIndicator(),
              SizedBox(height: 16),
              Text('Initializing validation service...'),
            ],
          ),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(title: const Text('Validation Settings')),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // Status card
              _buildStatusCard(),

              const SizedBox(height: 24),

              // Statistics card
              if (_stats != null) _buildStatsCard(),

              const SizedBox(height: 24),

              // Configuration form
              _buildConfigurationForm(),

              const SizedBox(height: 24),

              // Control buttons
              _buildControlButtons(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildStatusCard() {
    final statusColor = _getStatusColor(_status);

    return Card(
      elevation: 4,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Validation Service Status',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Icon(Icons.circle, color: statusColor, size: 16),
                const SizedBox(width: 8),
                Text(
                  _status.toString().split('.').last,
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: statusColor,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatsCard() {
    return Card(
      elevation: 4,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Validation Statistics',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            _buildStatRow(
              'Transactions Processed',
              _stats!.transactionsProcessed.toString(),
            ),
            _buildStatRow(
              'Blocks Validated',
              _stats!.blocksValidated.toString(),
            ),
            _buildStatRow(
              'Blocks Generated',
              _stats!.blocksGenerated.toString(),
            ),
            _buildStatRow(
              'Validation Errors',
              _stats!.validationErrors.toString(),
            ),
            _buildStatRow('Queue Length', _stats!.queueLength.toString()),
            _buildStatRow(
              'Avg. Transaction Time',
              '${_stats!.avgTransactionTimeMs.toStringAsFixed(2)} ms',
            ),
            _buildStatRow('Uptime', _formatUptime(_stats!.uptimeSeconds)),
            _buildResourceMeters(),
          ],
        ),
      ),
    );
  }

  Widget _buildStatRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label),
          Text(value, style: const TextStyle(fontWeight: FontWeight.bold)),
        ],
      ),
    );
  }

  Widget _buildResourceMeters() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 16),
        const Text(
          'Resource Usage',
          style: TextStyle(fontWeight: FontWeight.bold),
        ),
        const SizedBox(height: 8),
        _buildResourceMeter(
          'CPU',
          _stats!.cpuUsage / 100,
          '${_stats!.cpuUsage.toStringAsFixed(1)}%',
        ),
        const SizedBox(height: 8),
        _buildResourceMeter(
          'Memory',
          _stats!.memoryUsage / 1000,
          '${_stats!.memoryUsage.toStringAsFixed(1)} MB',
        ),
      ],
    );
  }

  Widget _buildResourceMeter(String label, double value, String valueText) {
    final color =
        value < 0.7 ? Colors.green : (value < 0.9 ? Colors.orange : Colors.red);

    return Row(
      children: [
        SizedBox(width: 60, child: Text(label)),
        Expanded(
          child: LinearProgressIndicator(
            value: value.clamp(0.0, 1.0),
            backgroundColor: Colors.grey[200],
            valueColor: AlwaysStoppedAnimation<Color>(color),
          ),
        ),
        const SizedBox(width: 8),
        SizedBox(width: 60, child: Text(valueText, textAlign: TextAlign.right)),
      ],
    );
  }

  Widget _buildConfigurationForm() {
    return Form(
      key: _formKey,
      child: Card(
        elevation: 4,
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Service Configuration',
                style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: _buildNumberField(
                      controller: _cpuController,
                      labelText: 'Max CPU Usage (%)',
                      helperText: 'From 0 to 100',
                      validator: (value) {
                        final intValue = int.tryParse(value ?? '');
                        if (intValue == null ||
                            intValue < 0 ||
                            intValue > 100) {
                          return 'Enter a value between 0 and 100';
                        }
                        return null;
                      },
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildNumberField(
                      controller: _memoryController,
                      labelText: 'Max Memory (MB)',
                      helperText: 'Recommended: 250-1000',
                      validator: (value) {
                        final intValue = int.tryParse(value ?? '');
                        if (intValue == null || intValue < 50) {
                          return 'Min 50MB required';
                        }
                        return null;
                      },
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Row(
                children: [
                  Expanded(
                    child: _buildNumberField(
                      controller: _queueController,
                      labelText: 'Queue Size Limit',
                      helperText: 'Recommended: 5000-20000',
                      validator: (value) {
                        final intValue = int.tryParse(value ?? '');
                        if (intValue == null || intValue < 100) {
                          return 'Min 100 required';
                        }
                        return null;
                      },
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _buildNumberField(
                      controller: _timeSlotController,
                      labelText: 'Processing Time Slot (ms)',
                      helperText: 'Recommended: 100-300',
                      validator: (value) {
                        final intValue = int.tryParse(value ?? '');
                        if (intValue == null || intValue < 50) {
                          return 'Min 50ms required';
                        }
                        return null;
                      },
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              _buildNumberField(
                controller: _batchSizeController,
                labelText: 'Batch Size',
                helperText: 'Number of transactions to process in a batch',
                validator: (value) {
                  final intValue = int.tryParse(value ?? '');
                  if (intValue == null || intValue < 1) {
                    return 'Must be at least 1';
                  }
                  return null;
                },
              ),
              const SizedBox(height: 24),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  OutlinedButton.icon(
                    onPressed: () => _updateFormFields(),
                    icon: const Icon(Icons.restore),
                    label: const Text('Reset'),
                  ),
                  ElevatedButton.icon(
                    onPressed: () => _updateConfiguration(),
                    icon: const Icon(Icons.save),
                    label: const Text('Update Configuration'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildNumberField({
    required TextEditingController controller,
    required String labelText,
    required String helperText,
    required String? Function(String?) validator,
  }) {
    return TextFormField(
      controller: controller,
      decoration: InputDecoration(
        labelText: labelText,
        helperText: helperText,
        border: const OutlineInputBorder(),
      ),
      keyboardType: TextInputType.number,
      validator: validator,
    );
  }

  Widget _buildControlButtons() {
    return Card(
      elevation: 4,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Service Controls',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                _buildControlButton(
                  label: 'Start',
                  icon: Icons.play_arrow,
                  onPressed:
                      _status == ValidationServiceStatus.stopped
                          ? _startValidation
                          : null,
                  color: Colors.green,
                ),
                _buildControlButton(
                  label: 'Stop',
                  icon: Icons.stop,
                  onPressed:
                      (_status == ValidationServiceStatus.running ||
                              _status == ValidationServiceStatus.paused)
                          ? _stopValidation
                          : null,
                  color: Colors.red,
                ),
                _buildControlButton(
                  label: 'Pause',
                  icon: Icons.pause,
                  onPressed:
                      _status == ValidationServiceStatus.running
                          ? _pauseValidation
                          : null,
                  color: Colors.orange,
                ),
                _buildControlButton(
                  label: 'Resume',
                  icon: Icons.play_arrow,
                  onPressed:
                      _status == ValidationServiceStatus.paused
                          ? _resumeValidation
                          : null,
                  color: Colors.blue,
                ),
              ],
            ),
            const SizedBox(height: 16),
            Divider(),
            const SizedBox(height: 16),
            const Text(
              'Testing',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            OutlinedButton.icon(
              onPressed:
                  (_status == ValidationServiceStatus.running)
                      ? _addBlockGenerationTask
                      : null,
              icon: const Icon(Icons.add_task),
              label: const Text('Add Block Generation Task'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildControlButton({
    required String label,
    required IconData icon,
    required VoidCallback? onPressed,
    required Color color,
  }) {
    return ElevatedButton.icon(
      onPressed: onPressed,
      icon: Icon(icon),
      label: Text(label),
      style: ElevatedButton.styleFrom(
        backgroundColor: onPressed != null ? color : Colors.grey,
        foregroundColor: Colors.white,
      ),
    );
  }

  Color _getStatusColor(ValidationServiceStatus status) {
    switch (status) {
      case ValidationServiceStatus.running:
        return Colors.green;
      case ValidationServiceStatus.paused:
        return Colors.orange;
      case ValidationServiceStatus.failed:
        return Colors.red;
      case ValidationServiceStatus.recovering:
        return Colors.amber;
      case ValidationServiceStatus.starting:
        return Colors.blue;
      case ValidationServiceStatus.shuttingDown:
        return Colors.purple;
      case ValidationServiceStatus.stopped:
      default:
        return Colors.grey;
    }
  }

  String _formatUptime(int seconds) {
    final hours = seconds ~/ 3600;
    final minutes = (seconds % 3600) ~/ 60;
    final remainingSeconds = seconds % 60;

    final parts = <String>[];
    if (hours > 0) parts.add('${hours}h');
    if (minutes > 0) parts.add('${minutes}m');
    parts.add('${remainingSeconds}s');

    return parts.join(' ');
  }
}
