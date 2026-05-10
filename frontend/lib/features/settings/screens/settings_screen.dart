import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mithic/core/theme.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/auth/providers/auth_provider.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode  = ref.watch(themeModeProvider);
    final dataSaving = ref.watch(dataSavingProvider);
    final locale     = ref.watch(localeProvider);
    final isDark     = Theme.of(context).brightness == Brightness.dark;
    final ink        = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3       = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final accent     = isDark ? MithicColors.accentDark : MithicColors.accent;
    final line       = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    return Scaffold(
      appBar: MithicTopBar(folio: '06', title: '設定'),
      body: ListView(
        padding: const EdgeInsets.only(bottom: 40),
        children: [
          _Section('外観', ink3: ink3, line: line, children: [
            // Theme mode
            _Row(
              icon: Icons.brightness_6,
              label: 'テーマ',
              ink: ink, ink3: ink3, line: line,
              trailing: Row(
                mainAxisSize: MainAxisSize.min,
                children: ThemeMode.values.map((m) {
                  final on = themeMode == m;
                  final lbl = switch (m) {
                    ThemeMode.light  => 'ライト',
                    ThemeMode.dark   => 'ダーク',
                    ThemeMode.system => '自動',
                  };
                  return GestureDetector(
                    onTap: () => ref.read(themeModeProvider.notifier).state = m,
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                      decoration: BoxDecoration(
                        color: on ? ink : Colors.transparent,
                        border: Border.all(color: on ? ink : line, width: 1),
                        borderRadius: BorderRadius.circular(6),
                      ),
                      child: Text(
                        lbl,
                        style: GoogleFonts.dmSans(
                          fontSize: 12,
                          color: on ? (isDark ? MithicColors.paperDark : MithicColors.paper) : ink3,
                        ),
                      ),
                    ),
                  );
                }).toList(),
              ),
            ),
            // Language
            _Row(
              icon: Icons.language,
              label: '言語',
              ink: ink, ink3: ink3, line: line,
              trailing: DropdownButton<Locale>(
                value: locale,
                underline: const SizedBox.shrink(),
                style: GoogleFonts.dmSans(fontSize: 13, color: ink),
                items: const [
                  DropdownMenuItem(value: Locale('ja'), child: Text('日本語')),
                  DropdownMenuItem(value: Locale('en'), child: Text('English')),
                ],
                onChanged: (v) { if (v != null) ref.read(localeProvider.notifier).state = v; },
              ),
            ),
            // Data saving
            _Row(
              icon: Icons.data_saver_on,
              label: 'データ節約',
              sublabel: 'メディアの自動再生を無効化',
              ink: ink, ink3: ink3, line: line,
              trailing: Switch(
                value: dataSaving,
                activeColor: accent,
                onChanged: (v) => ref.read(dataSavingProvider.notifier).state = v,
              ),
            ),
          ]),
          _Section('アカウント', ink3: ink3, line: line, children: [
            _NavRow(icon: Icons.favorite_outline,  label: 'お気に入り',          route: '/favorites',       ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.bookmark_outline,  label: 'ブックマーク',         route: '/bookmarks',       ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.block,             label: 'ブロック',             route: '/blocks',          ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.volume_off,        label: 'ミュート',             route: '/mutes',           ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.person_add_alt,    label: 'フォローリクエスト',    route: '/follow_requests', ink: ink, ink3: ink3, line: line, ctx: context),
            _ActionRow(
              icon: Icons.logout,
              label: 'ログアウト',
              ink: ink, ink3: ink3, line: line,
              destructive: true,
              onTap: () async => ref.read(authProvider.notifier).logout(),
            ),
          ]),
          _Section('整理', ink3: ink3, line: line, children: [
            _NavRow(icon: Icons.list_alt,            label: 'リスト',    route: '/lists',    ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.radar,               label: 'アンテナ',  route: '/antennas', ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.collections_bookmark,label: 'クリップ',  route: '/clips',    ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.filter_list,         label: 'フィルター', route: '/filters',  ink: ink, ink3: ink3, line: line, ctx: context),
          ]),
          _Section('高度な設定', ink3: ink3, line: line, children: [
            _NavRow(icon: Icons.public,            label: 'フェデレーション',  route: '/federation', ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.security,          label: '二要素認証',        route: '/2fa',        ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.apps,              label: 'OAuthアプリ',       route: '/oauth/apps', ink: ink, ink3: ink3, line: line, ctx: context),
            _NavRow(icon: Icons.admin_panel_settings, label: '管理者',         route: '/admin',      ink: ink, ink3: ink3, line: line, ctx: context),
          ]),
          _Section('アプリ情報', ink3: ink3, line: line, children: [
            _Row(
              icon: Icons.info_outline,
              label: 'バージョン',
              ink: ink, ink3: ink3, line: line,
              trailing: Text('0.1.0', style: GoogleFonts.jetBrainsMono(fontSize: 11, color: ink3)),
            ),
          ]),
        ],
      ),
    );
  }
}

// ── Section ──────────────────────────────────────────────────────────────────
class _Section extends StatelessWidget {
  final String title;
  final List<Widget> children;
  final Color ink3;
  final Color line;

  const _Section(this.title, {required this.children, required this.ink3, required this.line});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(14, 24, 14, 8),
          child: MithicLabel(title.toUpperCase()),
        ),
        ...children,
        Container(height: 1.25, color: line),
      ],
    );
  }
}

// ── Generic row with trailing ─────────────────────────────────────────────────
class _Row extends StatelessWidget {
  final IconData icon;
  final String label;
  final String? sublabel;
  final Widget? trailing;
  final Color ink;
  final Color ink3;
  final Color line;
  final VoidCallback? onTap;

  const _Row({
    required this.icon,
    required this.label,
    this.sublabel,
    this.trailing,
    required this.ink,
    required this.ink3,
    required this.line,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    Widget row = Container(
      padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: line, width: 1)),
      ),
      child: Row(
        children: [
          Icon(icon, size: 18, color: ink3),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(label, style: GoogleFonts.dmSans(fontSize: 14, color: ink)),
                if (sublabel != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(sublabel!, style: GoogleFonts.dmSans(fontSize: 11, color: ink3)),
                  ),
              ],
            ),
          ),
          if (trailing != null) trailing!,
        ],
      ),
    );
    if (onTap != null) return GestureDetector(onTap: onTap, child: row);
    return row;
  }
}

// ── Navigation row (→ chevron) ───────────────────────────────────────────────
class _NavRow extends StatelessWidget {
  final IconData icon;
  final String label;
  final String route;
  final Color ink;
  final Color ink3;
  final Color line;
  final BuildContext ctx;

  const _NavRow({
    required this.icon,
    required this.label,
    required this.route,
    required this.ink,
    required this.ink3,
    required this.line,
    required this.ctx,
  });

  @override
  Widget build(BuildContext context) {
    return _Row(
      icon: icon,
      label: label,
      ink: ink, ink3: ink3, line: line,
      trailing: Icon(Icons.chevron_right, size: 18, color: ink3),
      onTap: () => ctx.push(route),
    );
  }
}

// ── Action row (destructive or plain) ────────────────────────────────────────
class _ActionRow extends StatelessWidget {
  final IconData icon;
  final String label;
  final Color ink;
  final Color ink3;
  final Color line;
  final bool destructive;
  final VoidCallback onTap;

  const _ActionRow({
    required this.icon,
    required this.label,
    required this.ink,
    required this.ink3,
    required this.line,
    required this.onTap,
    this.destructive = false,
  });

  @override
  Widget build(BuildContext context) {
    final color = destructive ? const Color(0xFFE03030) : ink;
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
        decoration: BoxDecoration(
          border: Border(bottom: BorderSide(color: line, width: 1)),
        ),
        child: Row(
          children: [
            Icon(icon, size: 18, color: color),
            const SizedBox(width: 14),
            Text(label, style: GoogleFonts.dmSans(fontSize: 14, color: color)),
          ],
        ),
      ),
    );
  }
}
