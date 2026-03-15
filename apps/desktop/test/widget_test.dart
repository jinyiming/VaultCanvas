import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:vaultcanvas_desktop/app/app.dart';

void main() {
  testWidgets('renders navigation shell', (WidgetTester tester) async {
    tester.view.physicalSize = const Size(1200, 800);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(const VaultCanvasApp());
    await tester.pumpAndSettle();

    expect(find.text('VaultCanvas'), findsOneWidget);
    expect(find.text('密码'), findsOneWidget);
    expect(find.text('加解密'), findsOneWidget);
    expect(find.text('隐写'), findsOneWidget);
  });
}
