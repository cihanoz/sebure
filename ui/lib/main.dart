import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'src/screens/home_screen.dart';
import 'src/services/blockchain_service.dart';
import 'src/models/app_state.dart';

void main() {
  // Ensure Flutter is initialized
  WidgetsFlutterBinding.ensureInitialized();

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
      themeMode: ThemeMode.system,
      home: FutureBuilder<bool>(
        future: BlockchainService.initialize(),
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return const MaterialApp(
              home: Scaffold(body: Center(child: CircularProgressIndicator())),
            );
          }

          return const HomeScreen();
        },
      ),
    );
  }
}
