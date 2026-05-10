/// Mithic デザインシステム共通ウィジェット
/// ワイヤーフレームの wf-* クラス群に対応する Flutter 実装
library mithic_widgets;

import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mithic/core/theme/tokens/colors.dart';

// ── ヘルパー ──────────────────────────────────────────────────────────────────
Color _ink(BuildContext ctx) =>
    ctx.isDark ? MithicColors.inkDark  : MithicColors.ink;
Color _ink3(BuildContext ctx) =>
    ctx.isDark ? MithicColors.ink3Dark : MithicColors.ink3;
Color _paper(BuildContext ctx) =>
    ctx.isDark ? MithicColors.paperDark : MithicColors.paper;
Color _card(BuildContext ctx) =>
    ctx.isDark ? MithicColors.cardDark  : MithicColors.card;
Color _cardAlt(BuildContext ctx) =>
    ctx.isDark ? MithicColors.cardAltDark : MithicColors.cardAlt;
Color _line(BuildContext ctx) =>
    ctx.isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;
Color _accent(BuildContext ctx) =>
    ctx.isDark ? MithicColors.accentDark : MithicColors.accent;

extension _BrightnessX on BuildContext {
  bool get isDark => Theme.of(this).brightness == Brightness.dark;
}

// ── MithicTopBar ──────────────────────────────────────────────────────────────
/// ページ上部の `[ folio ] タイトル` バー
class MithicTopBar extends StatelessWidget implements PreferredSizeWidget {
  final String title;
  final String? folio;
  final List<Widget> actions;
  final bool dense;

  const MithicTopBar({
    super.key,
    required this.title,
    this.folio,
    this.actions = const [],
    this.dense = false,
  });

  @override
  Size get preferredSize => Size.fromHeight(dense ? 50 : 58);

  @override
  Widget build(BuildContext context) {
    return Container(
      height: preferredSize.height,
      padding: EdgeInsets.symmetric(
        horizontal: 18,
        vertical: dense ? 10 : 14,
      ),
      decoration: BoxDecoration(
        color: _paper(context),
        border: Border(bottom: BorderSide(color: _line(context), width: 1.25)),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.baseline,
        textBaseline: TextBaseline.alphabetic,
        children: [
          Text(
            '[ ${folio ?? '—'} ]',
            style: GoogleFonts.jetBrainsMono(
              fontSize: 10,
              color: _ink3(context),
              letterSpacing: 0.14,
            ),
          ),
          const SizedBox(width: 10),
          Text(
            title,
            style: GoogleFonts.patrickHand(
              fontSize: 24,
              color: _ink(context),
              height: 1,
            ),
          ),
          const Spacer(),
          ...actions,
        ],
      ),
    );
  }
}

// ── MithicCard ────────────────────────────────────────────────────────────────
/// ワイヤーフレームの `.wf-card` / `.wf-card.raised`
class MithicCard extends StatelessWidget {
  final Widget child;
  final bool raised;
  final bool dashed;
  final bool outline;
  final EdgeInsetsGeometry? padding;
  final Color? color;
  final Color? borderColor;
  final BorderRadius? borderRadius;
  final VoidCallback? onTap;

  const MithicCard({
    super.key,
    required this.child,
    this.raised = false,
    this.dashed = false,
    this.outline = false,
    this.padding,
    this.color,
    this.borderColor,
    this.borderRadius,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final bc = borderColor ?? _ink(context);
    final bg = outline
        ? Colors.transparent
        : (color ?? _card(context));
    final br = borderRadius ?? BorderRadius.circular(10);

    Widget card = Container(
      decoration: BoxDecoration(
        color: bg,
        borderRadius: br,
        border: Border.all(
          color: bc.withValues(alpha: dashed ? 0.35 : 1.0),
          width: 1.25,
          strokeAlign: BorderSide.strokeAlignInside,
        ),
        boxShadow: raised
            ? [BoxShadow(color: bc, offset: const Offset(3, 3))]
            : null,
      ),
      child: padding != null
          ? Padding(padding: padding!, child: child)
          : child,
    );

    if (onTap != null) {
      card = InkWell(
        onTap: onTap,
        borderRadius: br,
        child: card,
      );
    }
    return card;
  }
}

// ── MithicLabel ───────────────────────────────────────────────────────────────
/// ワイヤーフレームの `.wf-label` — 小さいモノスペース大文字ラベル
class MithicLabel extends StatelessWidget {
  final String text;
  final Color? color;

  const MithicLabel(this.text, {super.key, this.color});

  @override
  Widget build(BuildContext context) {
    return Text(
      text,
      style: GoogleFonts.jetBrainsMono(
        fontSize: 10,
        color: color ?? _ink3(context),
        letterSpacing: 0.12,
      ),
    );
  }
}

// ── MithicPill ────────────────────────────────────────────────────────────────
/// ワイヤーフレームの `.wf-pill`
class MithicPill extends StatelessWidget {
  final String text;
  final bool accent;
  final bool accent2;
  final bool ink;
  final VoidCallback? onTap;

  const MithicPill(
    this.text, {
    super.key,
    this.accent = false,
    this.accent2 = false,
    this.ink = false,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = context.isDark;
    Color bg;
    Color fg;
    Color border;

    if (accent) {
      bg = isDark ? MithicColors.accentDark : MithicColors.accent;
      fg = Colors.white;
      border = bg;
    } else if (accent2) {
      bg = isDark ? MithicColors.accent2Dark : MithicColors.accent2;
      fg = isDark ? Colors.white : MithicColors.ink;
      border = bg;
    } else if (ink) {
      bg = _ink(context);
      fg = _paper(context);
      border = bg;
    } else {
      bg = _card(context);
      fg = _ink(context);
      border = _line(context);
    }

    Widget pill = Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: border, width: 1),
      ),
      child: Text(
        text,
        style: GoogleFonts.jetBrainsMono(fontSize: 11, color: fg),
      ),
    );

    if (onTap != null) {
      return GestureDetector(onTap: onTap, child: pill);
    }
    return pill;
  }
}

// ── MithicButton ──────────────────────────────────────────────────────────────
/// ワイヤーフレームの `.wf-btn` 系
class MithicButton extends StatelessWidget {
  final String label;
  final VoidCallback? onPressed;
  final bool primary;
  final bool accent;
  final bool ghost;
  final bool small;
  final bool large;
  final IconData? icon;

  const MithicButton(
    this.label, {
    super.key,
    this.onPressed,
    this.primary = false,
    this.accent = false,
    this.ghost = false,
    this.small = false,
    this.large = false,
    this.icon,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = context.isDark;
    Color bg;
    Color fg;
    Color border;

    if (accent) {
      bg = isDark ? MithicColors.accentDark : MithicColors.accent;
      fg = Colors.white;
      border = _ink(context);
    } else if (primary) {
      bg = _ink(context);
      fg = _paper(context);
      border = _ink(context);
    } else if (ghost) {
      bg = Colors.transparent;
      fg = _ink(context);
      border = _line(context);
    } else {
      bg = _card(context);
      fg = _ink(context);
      border = _ink(context);
    }

    final vpad = small ? 3.0 : large ? 10.0 : 6.0;
    final hpad = small ? 8.0 : large ? 16.0 : 12.0;
    final fs   = small ? 11.0 : large ? 14.0 : 12.0;

    return InkWell(
      onTap: onPressed,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        padding: EdgeInsets.symmetric(vertical: vpad, horizontal: hpad),
        decoration: BoxDecoration(
          color: bg,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: border, width: 1.25),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (icon != null) ...[
              Icon(icon, size: fs + 2, color: fg),
              const SizedBox(width: 6),
            ],
            Text(
              label,
              style: GoogleFonts.dmSans(
                fontSize: fs,
                color: fg,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ── MithicTabBar ──────────────────────────────────────────────────────────────
/// ワイヤーフレームの `.wf-tabs` スタイルのタブバー
class MithicTabBar extends StatelessWidget {
  final List<String> tabs;
  final int selected;
  final ValueChanged<int> onSelect;

  const MithicTabBar({
    super.key,
    required this.tabs,
    required this.selected,
    required this.onSelect,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: _line(context), width: 1.25),
        ),
      ),
      child: Row(
        children: tabs.asMap().entries.map((e) {
          final on = e.key == selected;
          return InkWell(
            onTap: () => onSelect(e.key),
            child: Container(
              padding: const EdgeInsets.fromLTRB(0, 8, 16, 8),
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(
                    color: on ? _accent(context) : Colors.transparent,
                    width: 2,
                  ),
                ),
              ),
              child: Text(
                e.value,
                style: GoogleFonts.dmSans(
                  fontSize: 12,
                  fontWeight: on ? FontWeight.w600 : FontWeight.w400,
                  color: on ? _ink(context) : _ink3(context),
                ),
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}

// ── MithicSegmentedControl ────────────────────────────────────────────────────
/// ワイヤーフレームの `.wf-seg-tl`（ホーム/ローカル/グローバル切り替え）
class MithicSegmentedControl extends StatelessWidget {
  final List<String> labels;
  final int selected;
  final ValueChanged<int> onSelect;

  const MithicSegmentedControl({
    super.key,
    required this.labels,
    required this.selected,
    required this.onSelect,
  });

  @override
  Widget build(BuildContext context) {
    final ink = _ink(context);
    return Container(
      decoration: BoxDecoration(
        color: _card(context),
        border: Border.all(color: ink, width: 1.25),
        borderRadius: BorderRadius.circular(8),
        boxShadow: [BoxShadow(color: ink.withValues(alpha: 0.6), offset: const Offset(2, 2))],
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: labels.asMap().entries.map((e) {
          final on = e.key == selected;
          final isLast = e.key == labels.length - 1;
          return InkWell(
            onTap: () => onSelect(e.key),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
              decoration: BoxDecoration(
                color: on ? ink : Colors.transparent,
                border: Border(
                  right: isLast
                      ? BorderSide.none
                      : BorderSide(color: _line(context), width: 1.25),
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    '${(e.key + 1).toString().padLeft(2, '0')}',
                    style: GoogleFonts.jetBrainsMono(
                      fontSize: 9,
                      letterSpacing: 0.14,
                      color: on ? _paper(context) : _ink3(context),
                    ),
                  ),
                  const SizedBox(width: 6),
                  Text(
                    e.value,
                    style: GoogleFonts.dmSans(
                      fontSize: 12,
                      color: on ? _paper(context) : _ink(context),
                    ),
                  ),
                ],
              ),
            ),
          );
        }).toList(),
      ),
    );
  }
}

// ── MithicAvatar ──────────────────────────────────────────────────────────────
/// ユーザーアバター（円形、フォールバックはイニシャル）
class MithicAvatar extends StatelessWidget {
  final String? url;
  final String fallbackName;
  final double size;
  final bool accentBorder;

  const MithicAvatar({
    super.key,
    this.url,
    required this.fallbackName,
    this.size = 40,
    this.accentBorder = false,
  });

  @override
  Widget build(BuildContext context) {
    final borderColor = accentBorder ? MithicColors.accent : _ink(context);

    Widget inner;
    if (url != null && url!.isNotEmpty) {
      inner = Image.network(
        url!,
        width: size,
        height: size,
        fit: BoxFit.cover,
        errorBuilder: (_, __, ___) => _initial(context),
      );
    } else {
      inner = _initial(context);
    }

    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        border: Border.all(color: borderColor, width: 1.25),
      ),
      child: ClipOval(child: inner),
    );
  }

  Widget _initial(BuildContext context) {
    final initial = fallbackName.isNotEmpty
        ? fallbackName[0].toUpperCase()
        : '?';
    return Container(
      width: size,
      height: size,
      color: _cardAlt(context),
      child: Center(
        child: Text(
          initial,
          style: GoogleFonts.patrickHand(
            fontSize: size * 0.4,
            color: _ink(context),
          ),
        ),
      ),
    );
  }
}

// ── MithicStamp ───────────────────────────────────────────────────────────────
/// ワイヤーフレームの postmark stamp（投稿左側の円形スタンプ）
class MithicStamp extends StatelessWidget {
  final String date;
  final String time;
  final bool accent;
  final bool accent2;
  final String? avatarUrl;
  final String fallbackName;

  const MithicStamp({
    super.key,
    required this.date,
    required this.time,
    this.accent = false,
    this.accent2 = false,
    this.avatarUrl,
    required this.fallbackName,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = context.isDark;
    Color bg;
    Color fg;
    Color border;

    if (accent) {
      bg = isDark ? MithicColors.accentDark : MithicColors.accent;
      fg = Colors.white;
      border = _ink(context);
    } else if (accent2) {
      bg = isDark ? MithicColors.accent2Dark : MithicColors.accent2;
      fg = _ink(context);
      border = _ink(context);
    } else {
      bg = _card(context);
      fg = _ink(context);
      border = _ink(context);
    }

    return SizedBox(
      width: 56,
      height: 56,
      child: Stack(
        children: [
          // Outer circle
          Container(
            width: 56,
            height: 56,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: bg,
              border: Border.all(color: border, width: 1.5),
            ),
          ),
          // Inner dashed ring
          Positioned.fill(
            child: Padding(
              padding: const EdgeInsets.all(5),
              child: CustomPaint(painter: _DashedCirclePainter(border.withValues(alpha: 0.35))),
            ),
          ),
          // Avatar or date/time text
          if (avatarUrl != null || fallbackName.isNotEmpty)
            Positioned.fill(
              child: Padding(
                padding: const EdgeInsets.all(6),
                child: ClipOval(
                  child: avatarUrl != null && avatarUrl!.isNotEmpty
                      ? Image.network(
                          avatarUrl!,
                          fit: BoxFit.cover,
                          errorBuilder: (_, __, ___) => _dateFallback(fg),
                        )
                      : _dateFallback(fg),
                ),
              ),
            )
          else
            Positioned.fill(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Text(date, style: GoogleFonts.jetBrainsMono(fontSize: 9, fontWeight: FontWeight.w600, color: fg)),
                  Text(time, style: GoogleFonts.jetBrainsMono(fontSize: 8, color: fg.withValues(alpha: 0.7))),
                ],
              ),
            ),
        ],
      ),
    );
  }

  Widget _dateFallback(Color fg) {
    final initial = fallbackName.isNotEmpty ? fallbackName[0].toUpperCase() : '?';
    return Container(
      color: Colors.transparent,
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(initial, style: GoogleFonts.patrickHand(fontSize: 18, color: fg, height: 1)),
            Text(time, style: GoogleFonts.jetBrainsMono(fontSize: 7, color: fg.withValues(alpha: 0.7))),
          ],
        ),
      ),
    );
  }
}

class _DashedCirclePainter extends CustomPainter {
  final Color color;
  _DashedCirclePainter(this.color);

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    const dashCount = 20;
    final rect = Rect.fromLTWH(0, 0, size.width, size.height);
    const step = 3.14159 * 2 / dashCount;
    const dashLen = step * 0.5;

    for (var i = 0; i < dashCount; i++) {
      final start = i * step;
      canvas.drawArc(rect, start, dashLen, false, paint);
    }
  }

  @override
  bool shouldRepaint(_DashedCirclePainter old) => old.color != color;
}

// ── MithicEmptyState ──────────────────────────────────────────────────────────
class MithicEmptyState extends StatelessWidget {
  final IconData icon;
  final String title;
  final String? subtitle;

  const MithicEmptyState({
    super.key,
    required this.icon,
    required this.title,
    this.subtitle,
  });

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 48, color: _ink3(context)),
          const SizedBox(height: 16),
          Text(title,
              style: GoogleFonts.patrickHand(fontSize: 20, color: _ink3(context))),
          if (subtitle != null) ...[
            const SizedBox(height: 6),
            Text(subtitle!,
                style: GoogleFonts.dmSans(fontSize: 13, color: _ink3(context))),
          ],
        ],
      ),
    );
  }
}
