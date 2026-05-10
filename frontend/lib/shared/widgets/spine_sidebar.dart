import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:lucide_icons/lucide_icons.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/auth/providers/auth_provider.dart';

enum _NavItem {
  home,
  search,
  notifications,
  messages,
  profile,
  settings,
}

class SpineSidebar extends ConsumerWidget {
  final bool compact;

  const SpineSidebar({super.key, this.compact = false});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final location = GoRouterState.of(context).uri.toString();
    final isDark = Theme.of(context).brightness == Brightness.dark;

    final bg        = isDark ? MithicColors.cardAltDark : MithicColors.cardAlt;
    final bgStripe  = isDark ? MithicColors.cardDark    : MithicColors.card;
    final ink       = isDark ? MithicColors.inkDark      : MithicColors.ink;
    final ink3      = isDark ? MithicColors.ink3Dark     : MithicColors.ink3;
    final lineSoft  = isDark
        ? const Color(0x38F3EFE6)
        : MithicColors.lineSoft;

    return Container(
      width: compact ? 52 : 160,
      decoration: BoxDecoration(
        color: bg,
        border: Border(right: BorderSide(color: lineSoft, width: 1.25)),
      ),
      child: compact
          ? _buildCompact(context, ref, location, ink, ink3, lineSoft)
          : _buildFull(context, ref, location, bgStripe, ink, ink3, lineSoft),
    );
  }

  Widget _buildFull(
    BuildContext context,
    WidgetRef ref,
    String location,
    Color bgStripe,
    Color ink,
    Color ink3,
    Color lineSoft,
  ) {
    return Stack(
      children: [
        // Left stripe (the "spine" gutter)
        Positioned(
          left: 0, top: 0, bottom: 0,
          width: 24,
          child: Container(color: bgStripe),
        ),
        Positioned(
          left: 24, top: 0, bottom: 0,
          width: 1.25,
          child: Container(color: lineSoft),
        ),
        // Main content
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _Logo(ink: ink, compact: false),
            Container(height: 1.25, color: ink),
            const SizedBox(height: 8),
            ..._NavItem.values.map((item) => _FullNavRow(
              item: item,
              active: _isActive(item, location),
              ink: ink,
              ink3: ink3,
              lineSoft: lineSoft,
              onTap: () => _navigate(context, item),
            )),
            const Spacer(),
            _ComposeButton(compact: false),
            const SizedBox(height: 8),
            _AccountFooter(ink: ink, ink3: ink3, lineSoft: lineSoft, compact: false),
          ],
        ),
      ],
    );
  }

  Widget _buildCompact(
    BuildContext context,
    WidgetRef ref,
    String location,
    Color ink,
    Color ink3,
    Color lineSoft,
  ) {
    return Column(
      children: [
        _Logo(ink: ink, compact: true),
        const SizedBox(height: 4),
        ..._NavItem.values.map((item) => _IconNavButton(
          item: item,
          active: _isActive(item, location),
          ink: ink,
          onTap: () => _navigate(context, item),
        )),
        const Spacer(),
        _ComposeButton(compact: true),
        const SizedBox(height: 8),
        _AccountFooter(ink: ink, ink3: ink3, lineSoft: lineSoft, compact: true),
      ],
    );
  }

  bool _isActive(_NavItem item, String location) {
    switch (item) {
      case _NavItem.home:          return location == '/' || location.startsWith('/timeline');
      case _NavItem.search:        return location.startsWith('/search');
      case _NavItem.notifications: return location.startsWith('/notifications');
      case _NavItem.messages:      return location.startsWith('/messages');
      case _NavItem.profile:       return location.startsWith('/profile');
      case _NavItem.settings:      return location.startsWith('/settings');
    }
  }

  void _navigate(BuildContext context, _NavItem item) {
    switch (item) {
      case _NavItem.home:          context.go('/');
      case _NavItem.search:        context.go('/search');
      case _NavItem.notifications: context.go('/notifications');
      case _NavItem.messages:      context.go('/messages');
      case _NavItem.profile:       context.go('/profile');
      case _NavItem.settings:      context.go('/settings');
    }
  }
}

// ── Logo ──────────────────────────────────────────────────────────────────────
class _Logo extends StatelessWidget {
  final Color ink;
  final bool compact;

  const _Logo({required this.ink, required this.compact});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(10, 12, 10, 8),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.baseline,
        textBaseline: TextBaseline.alphabetic,
        children: [
          Text('[', style: GoogleFonts.jetBrainsMono(fontSize: 20, color: ink, fontWeight: FontWeight.w400)),
          Text('m', style: GoogleFonts.jetBrainsMono(fontSize: 20, color: MithicColors.accent, fontWeight: FontWeight.w700, fontStyle: FontStyle.italic)),
          Text(']', style: GoogleFonts.jetBrainsMono(fontSize: 20, color: ink, fontWeight: FontWeight.w400)),
          if (!compact) ...[
            const SizedBox(width: 6),
            Text('mithic', style: GoogleFonts.patrickHand(fontSize: 20, color: ink, height: 1)),
          ],
        ],
      ),
    );
  }
}

// ── Full-width nav row ────────────────────────────────────────────────────────
class _FullNavRow extends StatelessWidget {
  final _NavItem item;
  final bool active;
  final Color ink;
  final Color ink3;
  final Color lineSoft;
  final VoidCallback onTap;

  const _FullNavRow({
    required this.item,
    required this.active,
    required this.ink,
    required this.ink3,
    required this.lineSoft,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final num  = _navNumber(item);
    final label = _navLabel(item);
    final icon  = _navIcon(item);

    return InkWell(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.fromLTRB(0, 6, 4, 6),
        decoration: BoxDecoration(
          color: active ? MithicColors.accent2.withValues(alpha: 0.35) : Colors.transparent,
          border: Border(
            left: BorderSide(
              color: active ? MithicColors.accent : Colors.transparent,
              width: 4,
            ),
            bottom: BorderSide(color: active ? lineSoft : Colors.transparent, width: 1),
          ),
        ),
        child: Row(
          children: [
            SizedBox(
              width: 18,
              child: Text(
                num,
                textAlign: TextAlign.right,
                style: GoogleFonts.jetBrainsMono(
                  fontSize: 9,
                  color: active ? ink : ink3,
                ),
              ),
            ),
            const SizedBox(width: 8),
            Icon(icon, size: 16, color: active ? ink : ink3),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                label,
                style: GoogleFonts.jetBrainsMono(
                  fontSize: 11,
                  fontWeight: active ? FontWeight.w600 : FontWeight.w400,
                  letterSpacing: 0.12,
                  color: active ? ink : ink3,
                ),
              ),
            ),
            if (active)
              Text('●', style: TextStyle(color: MithicColors.accent, fontSize: 8)),
          ],
        ),
      ),
    );
  }
}

// ── Icon-only nav button (compact) ────────────────────────────────────────────
class _IconNavButton extends StatelessWidget {
  final _NavItem item;
  final bool active;
  final Color ink;
  final VoidCallback onTap;

  const _IconNavButton({
    required this.item,
    required this.active,
    required this.ink,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final icon = _navIcon(item);
    return Tooltip(
      message: _navLabel(item),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(6),
        child: Container(
          width: 36,
          height: 36,
          margin: const EdgeInsets.symmetric(vertical: 2, horizontal: 8),
          decoration: BoxDecoration(
            color: active ? ink : Colors.transparent,
            borderRadius: BorderRadius.circular(6),
          ),
          child: Icon(
            icon,
            size: 16,
            color: active
                ? (Theme.of(context).brightness == Brightness.dark
                    ? MithicColors.paperDark
                    : MithicColors.paper)
                : ink.withValues(alpha: 0.6),
          ),
        ),
      ),
    );
  }
}

// ── Compose button ────────────────────────────────────────────────────────────
class _ComposeButton extends StatelessWidget {
  final bool compact;
  const _ComposeButton({required this.compact});

  @override
  Widget build(BuildContext context) {
    if (compact) {
      return Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        child: InkWell(
          onTap: () => context.push('/compose'),
          borderRadius: BorderRadius.circular(6),
          child: Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: MithicColors.accent,
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: MithicColors.ink, width: 1.25),
              boxShadow: const [BoxShadow(color: MithicColors.ink, offset: Offset(2, 2))],
            ),
            child: const Icon(LucideIcons.plus, color: Colors.white, size: 18),
          ),
        ),
      );
    }

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      child: InkWell(
        onTap: () => context.push('/compose'),
        child: Container(
          padding: const EdgeInsets.fromLTRB(12, 10, 12, 10),
          decoration: BoxDecoration(
            color: MithicColors.accent,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: MithicColors.ink, width: 1.25),
            boxShadow: const [BoxShadow(color: MithicColors.ink, offset: Offset(3, 3))],
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'NEW',
                style: GoogleFonts.jetBrainsMono(
                  fontSize: 9,
                  color: Colors.white,
                  letterSpacing: 0.18,
                ),
              ),
              const SizedBox(height: 2),
              Text(
                '+ 投稿',
                style: GoogleFonts.patrickHand(fontSize: 18, color: Colors.white, height: 1),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ── Account footer ────────────────────────────────────────────────────────────
class _AccountFooter extends ConsumerWidget {
  final Color ink;
  final Color ink3;
  final Color lineSoft;
  final bool compact;

  const _AccountFooter({
    required this.ink,
    required this.ink3,
    required this.lineSoft,
    required this.compact,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);
    final handle = authState == AuthStatus.authenticated ? '@you' : '---';

    if (compact) {
      return Padding(
        padding: const EdgeInsets.only(bottom: 10),
        child: Container(
          width: 24,
          height: 24,
          margin: const EdgeInsets.symmetric(horizontal: 14),
          decoration: BoxDecoration(
            color: MithicColors.accent,
            shape: BoxShape.circle,
            border: Border.all(color: MithicColors.ink, width: 1.25),
          ),
          child: const Icon(LucideIcons.user, size: 12, color: Colors.white),
        ),
      );
    }

    return Container(
      padding: const EdgeInsets.fromLTRB(10, 10, 10, 12),
      decoration: BoxDecoration(
        border: Border(top: BorderSide(color: lineSoft, width: 1.25, style: BorderStyle.solid)),
      ),
      child: Row(
        children: [
          Container(
            width: 24,
            height: 24,
            decoration: BoxDecoration(
              color: MithicColors.accent,
              shape: BoxShape.circle,
              border: Border.all(color: MithicColors.ink, width: 1.25),
            ),
            child: const Icon(LucideIcons.user, size: 12, color: Colors.white),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(handle,
                    style: GoogleFonts.jetBrainsMono(fontSize: 10, color: ink, height: 1.1),
                    overflow: TextOverflow.ellipsis),
                Text('SIG · ok',
                    style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3)),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────
String _navNumber(_NavItem item) {
  const nums = {
    _NavItem.home:          '01',
    _NavItem.search:        '02',
    _NavItem.notifications: '03',
    _NavItem.messages:      '04',
    _NavItem.profile:       '05',
    _NavItem.settings:      '06',
  };
  return nums[item]!;
}

String _navLabel(_NavItem item) {
  const labels = {
    _NavItem.home:          'ホーム',
    _NavItem.search:        '検索',
    _NavItem.notifications: '通知',
    _NavItem.messages:      'メッセージ',
    _NavItem.profile:       'プロフィール',
    _NavItem.settings:      '設定',
  };
  return labels[item]!;
}

IconData _navIcon(_NavItem item) {
  switch (item) {
    case _NavItem.home:          return LucideIcons.home;
    case _NavItem.search:        return LucideIcons.search;
    case _NavItem.notifications: return LucideIcons.bell;
    case _NavItem.messages:      return LucideIcons.mail;
    case _NavItem.profile:       return LucideIcons.user;
    case _NavItem.settings:      return LucideIcons.settings;
  }
}
