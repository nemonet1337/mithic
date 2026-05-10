import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:lucide_icons/lucide_icons.dart';
import 'package:mithic/core/theme/tokens/colors.dart';

class RightRailWidget extends ConsumerWidget {
  const RightRailWidget({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final paper   = isDark ? MithicColors.paperDark  : MithicColors.paper;
    final card    = isDark ? MithicColors.cardDark    : MithicColors.card;
    final ink     = isDark ? MithicColors.inkDark     : MithicColors.ink;
    final ink3    = isDark ? MithicColors.ink3Dark    : MithicColors.ink3;
    final lineSoft = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    return Container(
      width: 240,
      decoration: BoxDecoration(
        color: paper,
        border: Border(left: BorderSide(color: lineSoft, width: 1.25)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const SizedBox(height: 14),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14),
            child: _SearchBox(ink: ink, ink3: ink3, lineSoft: lineSoft, card: card),
          ),
          const SizedBox(height: 14),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14),
            child: _TrendingSection(ink: ink, ink3: ink3, lineSoft: lineSoft, card: card),
          ),
          const SizedBox(height: 14),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14),
            child: _SuggestedSection(ink: ink, ink3: ink3, card: card),
          ),
          const Spacer(),
          Padding(
            padding: const EdgeInsets.fromLTRB(14, 0, 14, 14),
            child: Text(
              '— mithic · signal not noise —',
              style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3),
              textAlign: TextAlign.right,
            ),
          ),
        ],
      ),
    );
  }
}

// ── Search box ────────────────────────────────────────────────────────────────
class _SearchBox extends StatelessWidget {
  final Color ink;
  final Color ink3;
  final Color lineSoft;
  final Color card;

  const _SearchBox({
    required this.ink,
    required this.ink3,
    required this.lineSoft,
    required this.card,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => context.push('/search'),
      child: Container(
        height: 32,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: BoxDecoration(
          color: card,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: lineSoft, style: BorderStyle.solid, width: 1.25),
        ),
        child: Row(
          children: [
            Icon(LucideIcons.search, size: 13, color: ink3),
            const SizedBox(width: 6),
            Text('find…', style: GoogleFonts.dmSans(fontSize: 12, color: ink3)),
          ],
        ),
      ),
    );
  }
}

// ── Trending section ──────────────────────────────────────────────────────────
class _TrendingSection extends StatelessWidget {
  final Color ink;
  final Color ink3;
  final Color lineSoft;
  final Color card;

  const _TrendingSection({
    required this.ink,
    required this.ink3,
    required this.lineSoft,
    required this.card,
  });

  static const _tags = [
    '#design',
    '#wireframe',
    '#typography',
    '#余白',
    '#flutter',
    '#activitypub',
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '[ 急上昇 / TRENDING ]',
          style: GoogleFonts.jetBrainsMono(
            fontSize: 10,
            color: ink3,
            letterSpacing: 0.12,
          ),
        ),
        const SizedBox(height: 6),
        ..._tags.asMap().entries.map((e) => _TrendRow(
          index: e.key + 1,
          tag: e.value,
          count: '${(e.key + 1) * 1.2 + 0.7}k'.replaceAll(RegExp(r'\.0'), ''),
          ink: ink,
          ink3: ink3,
          lineSoft: lineSoft,
          onTap: () => context.push('/hashtags/${e.value.replaceFirst('#', '')}'),
        )),
      ],
    );
  }
}

class _TrendRow extends StatelessWidget {
  final int index;
  final String tag;
  final String count;
  final Color ink;
  final Color ink3;
  final Color lineSoft;
  final VoidCallback onTap;

  const _TrendRow({
    required this.index,
    required this.tag,
    required this.count,
    required this.ink,
    required this.ink3,
    required this.lineSoft,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 6),
        decoration: BoxDecoration(
          border: Border(bottom: BorderSide(color: lineSoft, width: 1, style: BorderStyle.solid)),
        ),
        child: Row(
          children: [
            Text(
              '${index.toString().padLeft(2, '0')}.',
              style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3),
            ),
            const SizedBox(width: 6),
            Expanded(
              child: Text(
                tag,
                style: GoogleFonts.jetBrainsMono(
                  fontSize: 11,
                  color: MithicColors.accent,
                ),
              ),
            ),
            Text(count, style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3)),
          ],
        ),
      ),
    );
  }
}

// ── Suggested users section ───────────────────────────────────────────────────
class _SuggestedSection extends StatelessWidget {
  final Color ink;
  final Color ink3;
  final Color card;

  const _SuggestedSection({
    required this.ink,
    required this.ink3,
    required this.card,
  });

  static const _users = [
    ('Hana K.', '@hana'),
    ('Riku M.', '@riku'),
    ('Aya T.',  '@aya'),
  ];

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          '[ おすすめ / SUGGESTED ]',
          style: GoogleFonts.jetBrainsMono(
            fontSize: 10,
            color: ink3,
            letterSpacing: 0.12,
          ),
        ),
        const SizedBox(height: 8),
        ..._users.map((u) => Padding(
          padding: const EdgeInsets.only(bottom: 8),
          child: Row(
            children: [
              Container(
                width: 28,
                height: 28,
                decoration: BoxDecoration(
                  color: card,
                  shape: BoxShape.circle,
                  border: Border.all(color: MithicColors.ink, width: 1.25),
                ),
                child: Icon(LucideIcons.user, size: 14, color: ink3),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(u.$1,
                        style: GoogleFonts.patrickHand(fontSize: 14, color: ink, height: 1)),
                    Text(u.$2,
                        style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3)),
                  ],
                ),
              ),
              _FollowButton(ink: ink),
            ],
          ),
        )),
      ],
    );
  }
}

class _FollowButton extends StatelessWidget {
  final Color ink;
  const _FollowButton({required this.ink});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: Colors.transparent,
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: ink, width: 1.25),
      ),
      child: Text(
        '+ 追う',
        style: GoogleFonts.dmSans(fontSize: 11, color: ink, fontWeight: FontWeight.w500),
      ),
    );
  }
}
