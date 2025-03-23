import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'dart:async'; // For Timer
import '../models/app_state.dart';
import '../services/blockchain_service.dart';
import 'validation_settings_screen.dart';
import 'preferences_screen.dart';
import 'wallet_screen.dart';
import '../widgets/resource_usage_chart.dart';
import '../widgets/node_control_panel.dart';
import '../widgets/network_statistics.dart';

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
              Navigator.push(
                context,
                MaterialPageRoute(builder: (context) => const WalletScreen()),
              );
            },
            tooltip: 'Wallet',
          ),
          IconButton(
            icon: const Icon(Icons.tune),
            onPressed: () {
              Navigator.push(
                context,
                MaterialPageRoute(
                  builder: (context) => const ValidationSettingsScreen(),
                ),
              );
            },
            tooltip: 'Validation Settings',
          ),
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () {
              Navigator.push(
                context,
                MaterialPageRoute(
                  builder: (context) => const PreferencesScreen(),
                ),
              );
            },
            tooltip: 'Settings',
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: ListView(
            children: [
              const Text(
                'Node Dashboard',
                style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 16),

              // Node status and controls
              NodeControlPanel(),

              const SizedBox(height: 24),

              // Resource usage section
              const Text(
                'Resource Usage',
                style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 16),
              const ResourceUsageGrid(),

              const SizedBox(height: 24),

              // Network statistics section
              const Text(
                'Network',
                style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 16),
              const NetworkStatistics(),
            ],
          ),
        ),
      ),
    );
  }
}
