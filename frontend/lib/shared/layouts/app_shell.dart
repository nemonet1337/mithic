import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:lucide_icons/lucide_icons.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/shared/widgets/spine_sidebar.dart';
import 'package:mithic/shared/widgets/right_rail_widget.dart';

// ブレークポイント
const _kDesktop = 1200.0;
const _kTablet  = 700.0;

/// Web 向け 3カラムシェル。
/// ShellRoute の builder から child を受け取り、レイアウトを適用する。
class AppShell extends ConsumerWidget {
  final Widget child;
  const AppShell({super.key, required this.child});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final w = constraints.maxWidth;

        if (w >= _kDesktop) {
          return _DesktopShell(child: child);
        } else if (w >= _kTablet) {
          return _TabletShell(child: child);
        } else {
          return _MobileShell(child: child);
        }
      },
    );
  }
}

// ── Desktop: 3-column ─────────────────────────────────────────────────────────
class _DesktopShell extends StatelessWidget {
  final Widget child;
  const _DesktopShell({required this.child});

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SpineSidebar(compact: false),
        Expanded(child: child),
        const RightRailWidget(),
      ],
    );
  }
}

// ── Tablet: compact sidebar + content ────────────────────────────────────────
class _TabletShell extends StatelessWidget {
  final Widget child;
  const _TabletShell({required this.child});

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SpineSidebar(compact: true),
        Expanded(child: child),
      ],
    );
  }
}

// ── Mobile: bottom nav bar ─────────────────────────────────────────────────────
class _MobileShell extends StatelessWidget {
  final Widget child;
  const _MobileShell({required this.child});

  @override
  Widget build(BuildContext context) {
    final location = GoRouterState.of(context).uri.toString();
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final bg   = isDark ? MithicColors.paperDark : MithicColors.paper;
    final ink3 = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;

    final selectedIndex = _selectedIndex(location);

    return Scaffold(
      backgroundColor: bg,
      body: child,
      bottomNavigationBar: Container(
        decoration: BoxDecoration(
          color: bg,
          border: Border(
            top: BorderSide(
              color: isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft,
              width: 1.25,
            ),
          ),
        ),
        child: BottomNavigationBar(
          currentIndex: selectedIndex,
          backgroundColor: bg,
          selectedItemColor: MithicColors.accent,
          unselectedItemColor: ink3,
          type: BottomNavigationBarType.fixed,
          elevation: 0,
          selectedFontSize: 10,
          unselectedFontSize: 10,
          onTap: (i) => _onTap(context, i),
          items: [
            BottomNavigationBarItem(
              icon: Icon(LucideIcons.home, size: 20),
              label: 'ホーム',
            ),
            BottomNavigationBarItem(
              icon: Icon(LucideIcons.search, size: 20),
              label: '検索',
            ),
            BottomNavigationBarItem(
              icon: Icon(LucideIcons.bell, size: 20),
              label: '通知',
            ),
            BottomNavigationBarItem(
              icon: Icon(LucideIcons.mail, size: 20),
              label: 'DM',
            ),
            BottomNavigationBarItem(
              icon: Icon(LucideIcons.user, size: 20),
              label: 'プロフィール',
            ),
          ],
        ),
      ),
    );
  }

  int _selectedIndex(String location) {
    if (location.startsWith('/search'))        return 1;
    if (location.startsWith('/notifications')) return 2;
    if (location.startsWith('/messages'))      return 3;
    if (location.startsWith('/profile'))       return 4;
    return 0;
  }

  void _onTap(BuildContext context, int index) {
    switch (index) {
      case 0: context.go('/');
      case 1: context.go('/search');
      case 2: context.go('/notifications');
      case 3: context.go('/messages');
      case 4: context.go('/profile');
    }
  }
}
