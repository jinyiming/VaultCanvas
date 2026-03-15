import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:go_router/go_router.dart';

import '../features/crypto/crypto_page.dart';
import '../features/passwords/passwords_page.dart';
import '../features/stego/stego_page.dart';
import '../shared/app_shell.dart';
import '../shared/theme.dart';

class VaultCanvasApp extends StatelessWidget {
  const VaultCanvasApp({super.key});

  @override
  Widget build(BuildContext context) {
    final router = GoRouter(
      initialLocation: '/passwords',
      routes: [
        ShellRoute(
          builder: (context, state, child) => AppShell(child: child),
          routes: [
            GoRoute(
              path: '/passwords',
              builder: (context, state) => const PasswordsPage(),
            ),
            GoRoute(
              path: '/crypto',
              builder: (context, state) => const CryptoPage(),
            ),
            GoRoute(
              path: '/stego',
              builder: (context, state) => const StegoPage(),
            ),
          ],
        ),
      ],
    );

    return MaterialApp.router(
      debugShowCheckedModeBanner: false,
      title: 'VaultCanvas',
      theme: buildAppTheme(),
      routerConfig: router,
      locale: const Locale('zh', 'CN'),
      supportedLocales: const [
        Locale('zh', 'CN'),
      ],
      localizationsDelegates: const [
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
    );
  }
}
