// This is a basic Flutter widget test.
//
// To perform an interaction with a widget in your test, use the WidgetTester
// utility in the flutter_test package. For example, you can send tap and scroll
// gestures. You can also use WidgetTester to find child widgets in the widget
// tree, read text, and verify that the values of widget properties are correct.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:sebure_ui/main.dart';
import 'package:sebure_ui/src/models/app_state.dart';

void main() {
  testWidgets('App smoke test', (WidgetTester tester) async {
    // Build our app and trigger a frame
    await tester.pumpWidget(
      MultiProvider(
        providers: [ChangeNotifierProvider(create: (_) => AppState())],
        child: const SebureApp(),
      ),
    );

    // Verify that the app title is displayed
    expect(find.text('SEBURE Blockchain Node'), findsOneWidget);

    // Verify that Node Dashboard text is shown
    expect(find.text('Node Dashboard'), findsOneWidget);

    // Verify that Node Status text is shown
    expect(find.text('Node Status'), findsOneWidget);

    // Verify that Resource Usage text is shown
    expect(find.text('Resource Usage'), findsOneWidget);

    // Verify that Start Node button is present (initially node is stopped)
    expect(find.text('Start Node'), findsOneWidget);
  });
}
