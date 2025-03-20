import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'src/screens/home_screen.dart';
import 'src/services/blockchain_service.dart';
import 'src/services/config_service.dart';
import 'src/plugin/plugin_manager.dart';
import 'src/models/app_state.dart';

void main() async {
  // Ensure Flutter is initialized
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize services
  final configService = ConfigService.instance;
  await configService.initialize();

  // Initialize plugin manager
  final pluginManager = PluginManager.instance;
  await pluginManager.initialize();

  // Initialize blockchain service
  // (this may take a moment so we do it in the splash screen)

  runApp(
    MultiProvider(
      providers: [ChangeNotifierProvider(create: (_) => AppState())],
      child: const SebureApp(),
    ),
  );
}

class SebureApp extends StatelessWidget {
  const SebureApp({super.key});

  @override
  Widget build(BuildContext context) {
    // Read the theme setting from ConfigService
    final configService = ConfigService.instance;
    final themeSetting = configService.theme;

    ThemeMode themeMode;
    switch (themeSetting) {
      case 'light':
        themeMode = ThemeMode.light;
        break;
      case 'dark':
        themeMode = ThemeMode.dark;
        break;
      case 'system':
      default:
        themeMode = ThemeMode.system;
        break;
    }

    return MaterialApp(
      title: 'SEBURE Blockchain',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF124191),
          brightness: Brightness.light,
        ),
        useMaterial3: true,
        visualDensity: VisualDensity.adaptivePlatformDensity,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF124191),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
        visualDensity: VisualDensity.adaptivePlatformDensity,
      ),
      themeMode: themeMode,
      home: const AppStartupHandler(),
    );
  }
}

/// Handles the application startup sequence
class AppStartupHandler extends StatefulWidget {
  const AppStartupHandler({super.key});

  @override
  State<AppStartupHandler> createState() => _AppStartupHandlerState();
}

class _AppStartupHandlerState extends State<AppStartupHandler> {
  late Future<bool> _initializationFuture;

  @override
  void initState() {
    super.initState();
    _initializationFuture = _initializeApp();
  }

  Future<bool> _initializeApp() async {
    // Initialize blockchain service
    final initialized = await BlockchainService.initialize();

    // Update app state with initial data
    if (initialized && mounted) {
      final appState = Provider.of<AppState>(context, listen: false);
      final blockchainService = BlockchainService.instance;

      // Initial resource usage stats
      final resourceUsage = await blockchainService.getResourceUsage();
      appState.updateResourceUsage(
        cpu: resourceUsage['cpu'],
        memory: resourceUsage['memory'],
        network: resourceUsage['network'],
        disk: resourceUsage['disk'],
      );

      // Initial network stats
      final networkStats = await blockchainService.getNetworkStats();
      appState.updateNetworkStats(
        peers: networkStats['peers'],
        validatedTransactions: networkStats['validatedTransactions'],
      );

      // Auto-start node if enabled in config
      final configService = ConfigService.instance;
      if (configService.isNodeEnabled) {
        final started = await blockchainService.startNode();
        appState.updateNodeStatus(isRunning: started);
      }
    }

    return initialized;
  }

  @override
  void dispose() {
    // Shutdown plugin system when app is closed
    PluginManager.instance.shutdownAll();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<bool>(
      future: _initializationFuture,
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return Scaffold(
            body: Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Image.asset(
                    'assets/images/sebure_logo.png',
                    width: 150,
                    height: 150,
                    errorBuilder: (context, error, stackTrace) {
                      // If logo image isn't available yet, show a placeholder
                      return const Icon(
                        Icons.account_balance,
                        size: 100,
                        color: Color(0xFF124191),
                      );
                    },
                  ),
                  const SizedBox(height: 24),
                  const Text(
                    'SEBURE Blockchain',
                    style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 24),
                  const CircularProgressIndicator(),
                  const SizedBox(height: 16),
                  const Text('Initializing blockchain core...'),
                ],
              ),
            ),
          );
        } else if (snapshot.hasError || snapshot.data == false) {
          return Scaffold(
            body: Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.error_outline, size: 64, color: Colors.red),
                  const SizedBox(height: 16),
                  const Text(
                    'Failed to initialize',
                    style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    snapshot.error?.toString() ?? 'Unknown error',
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 24),
                  ElevatedButton(
                    onPressed: () {
                      setState(() {
                        _initializationFuture = _initializeApp();
                      });
                    },
                    child: const Text('Retry'),
                  ),
                ],
              ),
            ),
          );
        }

        // Successfully initialized
        return const HomeScreen();
      },
    );
  }
}
