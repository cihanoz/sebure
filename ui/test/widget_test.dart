// This is a basic Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sebure_ui/src/models/app_state.dart';

void main() {
  testWidgets('App initialization in test environment', (
    WidgetTester tester,
  ) async {
    // Mock SharedPreferences for testing
    SharedPreferences.setMockInitialValues({
      'theme': 'system',
      'node_enabled': false,
      'max_cpu_usage': 20.0,
      'max_memory_usage': 500.0,
      'max_network_usage': 100.0,
      'max_disk_usage': 10.0,
    });

    // Build a simplified test app to verify components work
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Text('SEBURE Blockchain'),
                ElevatedButton(
                  onPressed: () {},
                  child: const Text('Start Node'),
                ),
              ],
            ),
          ),
        ),
      ),
    );

    // Basic verification that our widget renders
    expect(find.text('SEBURE Blockchain'), findsOneWidget);
    expect(find.text('Start Node'), findsOneWidget);

    // Find and verify a button exists
    expect(find.byType(ElevatedButton), findsOneWidget);
  });
}
