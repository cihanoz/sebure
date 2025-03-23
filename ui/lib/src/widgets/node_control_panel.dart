import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/app_state.dart';
import '../services/blockchain_service.dart';
import '../services/config_service.dart';

/// Widget for controlling node operations and displaying status
class NodeControlPanel extends StatelessWidget {
  final BlockchainService _blockchainService = BlockchainService.instance;
  final ConfigService _configService = ConfigService.instance;

  NodeControlPanel({super.key});

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 2,
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Node Status',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 16),
            Consumer<AppState>(
              builder: (context, appState, child) {
                return Column(
                  children: [
                    _buildStatusIndicator(appState),
                    const SizedBox(height: 16),
                    _buildControlButtons(context, appState),
                    const SizedBox(height: 16),
                    _buildAutoStartSwitch(context, appState),
                  ],
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusIndicator(AppState appState) {
    final isRunning = appState.isNodeRunning;
    final statusColor = isRunning ? Colors.green : Colors.red;
    final statusText = isRunning ? 'Running' : 'Stopped';
    final statusIcon = isRunning ? Icons.check_circle : Icons.cancel;

    return Row(
      children: [
        Icon(statusIcon, color: statusColor, size: 24),
        const SizedBox(width: 8),
        Text(
          statusText,
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.bold,
            color: statusColor,
          ),
        ),
        const Spacer(),
        if (isRunning)
          Row(
            children: [
              const Icon(Icons.people, size: 16),
              const SizedBox(width: 4),
              Text('${appState.connectedPeers} peers'),
              const SizedBox(width: 16),
              const Icon(Icons.receipt_long, size: 16),
              const SizedBox(width: 4),
              Text('${appState.transactionsValidated} txs'),
            ],
          ),
      ],
    );
  }

  Widget _buildControlButtons(BuildContext context, AppState appState) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Expanded(
          child: ElevatedButton.icon(
            onPressed:
                appState.isNodeRunning
                    ? null
                    : () async {
                      final success = await _blockchainService.startNode();
                      if (success) {
                        // ignore: use_build_context_synchronously
                        Provider.of<AppState>(
                          context,
                          listen: false,
                        ).updateNodeStatus(isRunning: true);
                      } else {
                        // ignore: use_build_context_synchronously
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                            content: Text('Failed to start node'),
                            backgroundColor: Colors.red,
                          ),
                        );
                      }
                    },
            icon: const Icon(Icons.play_arrow),
            label: const Text('Start'),
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.green,
              foregroundColor: Colors.white,
              disabledBackgroundColor: Colors.grey.shade300,
            ),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: ElevatedButton.icon(
            onPressed:
                !appState.isNodeRunning
                    ? null
                    : () async {
                      final success = await _blockchainService.stopNode();
                      if (success) {
                        // ignore: use_build_context_synchronously
                        Provider.of<AppState>(
                          context,
                          listen: false,
                        ).updateNodeStatus(isRunning: false);
                      } else {
                        // ignore: use_build_context_synchronously
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                            content: Text('Failed to stop node'),
                            backgroundColor: Colors.red,
                          ),
                        );
                      }
                    },
            icon: const Icon(Icons.stop),
            label: const Text('Stop'),
            style: ElevatedButton.styleFrom(
              backgroundColor: Colors.red,
              foregroundColor: Colors.white,
              disabledBackgroundColor: Colors.grey.shade300,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildAutoStartSwitch(BuildContext context, AppState appState) {
    return Row(
      children: [
        const Text('Auto-start node on application launch'),
        const Spacer(),
        Switch(
          value: _configService.isNodeEnabled,
          onChanged: (value) async {
            await _configService.setNodeEnabled(value);
            // Force rebuild to reflect the new setting
            // ignore: use_build_context_synchronously
            Provider.of<AppState>(
              context,
              listen: false,
            ).updateNodeStatus(isRunning: appState.isNodeRunning);
          },
          activeColor: Colors.green,
        ),
      ],
    );
  }
}
