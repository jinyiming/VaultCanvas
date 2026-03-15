import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class AppShell extends StatelessWidget {
  const AppShell({required this.child, super.key});

  final Widget child;

  static const _items = <({String route, IconData icon, String label})>[
    (route: '/passwords', icon: Icons.key_rounded, label: '密码'),
    (route: '/crypto', icon: Icons.lock_rounded, label: '加解密'),
    (route: '/stego', icon: Icons.layers_rounded, label: '隐写'),
  ];

  @override
  Widget build(BuildContext context) {
    final current = GoRouterState.of(context).uri.path;

    return Scaffold(
      body: LayoutBuilder(
        builder: (context, constraints) {
          final tiny = constraints.maxWidth < 620;

          return Container(
            decoration: const BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [Color(0xFFF7FAFB), Color(0xFFEFF4F5)],
              ),
            ),
            child: SafeArea(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(6, 6, 6, 6),
                child: Column(
                  children: [
                    Container(
                      padding: EdgeInsets.symmetric(horizontal: tiny ? 5 : 6, vertical: tiny ? 4 : 5),
                      decoration: BoxDecoration(
                        color: const Color(0xFF12323C),
                        borderRadius: BorderRadius.circular(11),
                        boxShadow: const [
                          BoxShadow(
                            color: Color(0x120A2A33),
                            blurRadius: 10,
                            offset: Offset(0, 4),
                          ),
                        ],
                      ),
                      child: Row(
                        children: [
                          Container(
                            width: 20,
                            height: 20,
                            decoration: BoxDecoration(
                              color: Colors.white.withValues(alpha: 0.1),
                              borderRadius: BorderRadius.circular(6),
                            ),
                            child: const Icon(Icons.shield_moon_rounded, color: Colors.white, size: 11),
                          ),
                          const SizedBox(width: 5),
                          if (!tiny)
                            const Expanded(
                              child: Text(
                                'VaultCanvas',
                                overflow: TextOverflow.ellipsis,
                                style: TextStyle(
                                  color: Colors.white,
                                  fontSize: 9.5,
                                  fontWeight: FontWeight.w700,
                                  letterSpacing: 0.1,
                                ),
                              ),
                            )
                          else
                            const Spacer(),
                          for (final item in _items)
                            Padding(
                              padding: const EdgeInsets.only(left: 4),
                              child: _TopTab(
                                icon: item.icon,
                                label: item.label,
                                selected: current == item.route,
                                compact: tiny,
                                onTap: () => context.go(item.route),
                              ),
                            ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 5),
                    Expanded(child: child),
                  ],
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}

class _TopTab extends StatelessWidget {
  const _TopTab({
    required this.icon,
    required this.label,
    required this.selected,
    required this.compact,
    required this.onTap,
  });

  final IconData icon;
  final String label;
  final bool selected;
  final bool compact;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final bg = selected ? Colors.white : Colors.white.withValues(alpha: 0.06);
    final fg = selected ? const Color(0xFF14353F) : Colors.white;

    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(8),
        onTap: onTap,
        child: Ink(
          padding: EdgeInsets.symmetric(horizontal: compact ? 6 : 7, vertical: compact ? 5 : 6),
          decoration: BoxDecoration(
            color: bg,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, color: fg, size: compact ? 11 : 12),
              if (!compact) ...[
                const SizedBox(width: 4),
                Text(
                  label,
                  style: TextStyle(
                    color: fg,
                    fontSize: 8.5,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
