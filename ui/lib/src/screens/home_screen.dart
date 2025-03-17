import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:fl_chart/fl_chart.dart';
import 'dart:async'; // For Timer
import '../models/app_state.dart';
import '../services/blockchain_service.dart';
// These widgets will be implemented in future tasks
// import '../widgets/resource_usage_chart.dart';
// import '../widgets/node_control_panel.dart';
// import '../widgets/transaction_history.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final _blockchainService = BlockchainService.instance;
  // Will be used in the future to update stats periodically
  // late Timer _statsTimer;

  @override
  void initState() {
    super.initState();
    _updateStats();

    // We'll implement this in the real app to periodically update stats
    // _statsTimer = Timer.periodic(const Duration(seconds: 5), (_) => _updateStats());
  }

  @override
  void dispose() {
    // We'll uncomment this in the real app
    // _statsTimer.cancel();
    super.dispose();
  }

  Future<void> _updateStats() async {
    final appState = Provider.of<AppState>(context, listen: false);

    // Get resource usage
    final resourceUsage = await _blockchainService.getResourceUsage();
    appState.updateResourceUsage(
      cpu: resourceUsage['cpu'],
      memory: resourceUsage['memory'],
      network: resourceUsage['network'],
      disk: resourceUsage['disk'],
    );

    // Get network stats
    final networkStats = await _blockchainService.getNetworkStats();
    appState.updateNetworkStats(
      peers: networkStats['peers'],
      validatedTransactions: networkStats['validatedTransactions'],
    );

    // Get balance
    final balance = await _blockchainService.getBalance('mock-address');
    appState.updateBalance(balance);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('SEBURE Blockchain Node'),
        actions: [
          IconButton(
            icon: const Icon(Icons.account_balance_wallet),
            onPressed: () {
              // Navigate to wallet screen in the future
            },
            tooltip: 'Wallet',
          ),
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () {
              // Navigate to settings screen in the future
            },
            tooltip: 'Settings',
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // In a real implementation, we would use the actual widgets
              // that we'll create in future tasks
              const Text(
                'Node Dashboard',
                style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 16),

              // Node status and controls
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const Text(
                        'Node Status',
                        style: TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Consumer<AppState>(
                        builder: (context, appState, child) {
                          return Row(
                            children: [
                              Icon(
                                appState.isNodeRunning
                                    ? Icons.check_circle
                                    : Icons.cancel,
                                color:
                                    appState.isNodeRunning
                                        ? Colors.green
                                        : Colors.red,
                              ),
                              const SizedBox(width: 8),
                              Text(
                                appState.isNodeRunning ? 'Running' : 'Stopped',
                                style: const TextStyle(fontSize: 16),
                              ),
                              const Spacer(),
                              ElevatedButton(
                                onPressed: () async {
                                  final appState = Provider.of<AppState>(
                                    context,
                                    listen: false,
                                  );
                                  if (appState.isNodeRunning) {
                                    await _blockchainService.stopNode();
                                    appState.updateNodeStatus(isRunning: false);
                                  } else {
                                    await _blockchainService.startNode();
                                    appState.updateNodeStatus(isRunning: true);
                                  }
                                },
                                child: Text(
                                  Provider.of<AppState>(context).isNodeRunning
                                      ? 'Stop Node'
                                      : 'Start Node',
                                ),
                              ),
                            ],
                          );
                        },
                      ),
                    ],
                  ),
                ),
              ),

              const SizedBox(height: 16),

              // Resource usage section - placeholder
              Expanded(
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          'Resource Usage',
                          style: TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        const SizedBox(height: 16),
                        Expanded(
                          child: Center(
                            child: Text(
                              'Resource usage charts will be displayed here',
                              style: TextStyle(
                                color: Colors.grey[600],
                                fontSize: 16,
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
