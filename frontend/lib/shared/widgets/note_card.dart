import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/models/note.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';

class NoteCard extends StatefulWidget {
  final Note note;
  final VoidCallback? onTap;
  final VoidCallback? onReply;
  final VoidCallback? onRenote;
  final VoidCallback? onReact;
  /// flat=true: timeline style (bottom-border separator)
  /// flat=false: standalone card with raised border + shadow
  final bool flat;

  const NoteCard({
    super.key,
    required this.note,
    this.onTap,
    this.onReply,
    this.onRenote,
    this.onReact,
    this.flat = true,
  });

  @override
  State<NoteCard> createState() => _NoteCardState();
}

class _NoteCardState extends State<NoteCard> {
  bool _isCwExpanded = false;

  Note get _root => widget.note.isRenote ? widget.note.renote! : widget.note;

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark    : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark   : MithicColors.ink3;
    final paper  = isDark ? MithicColors.paperDark  : MithicColors.paper;
    final card   = isDark ? MithicColors.cardDark   : MithicColors.card;
    final accent = isDark ? MithicColors.accentDark : MithicColors.accent;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    final note = _root;
    final user = note.user;
    final stamp = _stampParts(note.createdAt);

    return GestureDetector(
      onTap: widget.onTap ?? () => context.push('/notes/${note.id}'),
      child: Container(
        padding: const EdgeInsets.fromLTRB(14, 14, 14, 10),
        decoration: widget.flat
            ? BoxDecoration(
                color: paper,
                border: Border(bottom: BorderSide(color: line, width: 1.25)),
              )
            : BoxDecoration(
                color: card,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: ink, width: 1.25),
                boxShadow: [BoxShadow(color: ink, offset: const Offset(3, 3))],
              ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (widget.note.isRenote) _renoteHeader(accent),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                GestureDetector(
                  onTap: () => context.push('/profile?userId=${user.id}'),
                  child: MithicStamp(
                    date: stamp[0],
                    time: stamp[1],
                    avatarUrl: user.avatarUrl,
                    fallbackName: user.name ?? user.username,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _userRow(context, user, note.createdAt, ink, ink3),
                      const SizedBox(height: 6),
                      _content(context, note, ink, ink3, accent, line),
                      const SizedBox(height: 8),
                      _actions(context, note, ink, ink3, accent),
                    ],
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _renoteHeader(Color accent) {
    final booster = widget.note.user;
    return Padding(
      padding: const EdgeInsets.only(bottom: 7, left: 68),
      child: Row(
        children: [
          Text('↻ ', style: GoogleFonts.jetBrainsMono(fontSize: 11, color: accent, fontWeight: FontWeight.w600)),
          Flexible(
            child: Text(
              '${booster.name ?? booster.username} がリノート',
              overflow: TextOverflow.ellipsis,
              style: GoogleFonts.dmSans(fontSize: 12, color: accent),
            ),
          ),
        ],
      ),
    );
  }

  Widget _userRow(BuildContext context, user, DateTime createdAt, Color ink, Color ink3) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.baseline,
      textBaseline: TextBaseline.alphabetic,
      children: [
        Expanded(
          child: GestureDetector(
            onTap: () => context.push('/profile?userId=${user.id}'),
            child: Wrap(
              crossAxisAlignment: WrapCrossAlignment.end,
              spacing: 5,
              children: [
                Text(
                  user.name ?? user.username,
                  style: GoogleFonts.patrickHand(fontSize: 17, color: ink, height: 1),
                ),
                Text(
                  '@${user.username}',
                  style: GoogleFonts.jetBrainsMono(fontSize: 10, color: ink3, letterSpacing: 0.1),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(width: 6),
        Text(
          _ageLabel(createdAt),
          style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3, letterSpacing: 0.1),
        ),
      ],
    );
  }

  Widget _content(BuildContext context, Note note, Color ink, Color ink3, Color accent, Color line) {
    if (note.cw != null) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          GestureDetector(
            onTap: () => setState(() => _isCwExpanded = !_isCwExpanded),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
              decoration: BoxDecoration(
                border: Border.all(color: line, width: 1),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(note.cw!, style: GoogleFonts.dmSans(fontSize: 12.5, color: ink3)),
                  const SizedBox(width: 8),
                  Text(
                    _isCwExpanded ? '▲' : '▼',
                    style: GoogleFonts.jetBrainsMono(fontSize: 10, color: ink3),
                  ),
                ],
              ),
            ),
          ),
          if (_isCwExpanded) ...[
            const SizedBox(height: 8),
            _noteText(note.text, ink),
          ],
        ],
      );
    }
    if (note.text.isEmpty) return const SizedBox.shrink();
    return _noteText(note.text, ink);
  }

  Widget _noteText(String text, Color ink) => Text(
    text,
    style: GoogleFonts.dmSans(fontSize: 13.5, color: ink, height: 1.55),
  );

  Widget _actions(BuildContext context, Note note, Color ink, Color ink3, Color accent) {
    return Row(
      children: [
        _reactButton(ink, ink3),
        ...note.reactions.take(4).map((r) => _reactionPill(r, ink, ink3, accent)),
        const Spacer(),
        _counter('↻', note.renoteCount, widget.onRenote, ink, ink3, accent, renote: true),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 7),
          child: Text('·', style: GoogleFonts.jetBrainsMono(fontSize: 10, color: ink3)),
        ),
        _counter('↪', note.repliesCount, widget.onReply, ink, ink3, accent, renote: false),
      ],
    );
  }

  Widget _reactButton(Color ink, Color ink3) {
    return GestureDetector(
      onTap: widget.onReact,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        decoration: BoxDecoration(
          border: Border.all(color: ink.withValues(alpha: 0.22), width: 1),
          borderRadius: BorderRadius.circular(999),
        ),
        child: Text(
          '+ REACT',
          style: GoogleFonts.jetBrainsMono(fontSize: 10, color: ink3, letterSpacing: 0.1),
        ),
      ),
    );
  }

  Widget _reactionPill(Reaction r, Color ink, Color ink3, Color accent) {
    final active = r.isMyReaction;
    return Padding(
      padding: const EdgeInsets.only(left: 5),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
        decoration: BoxDecoration(
          color: active ? accent.withValues(alpha: 0.1) : Colors.transparent,
          border: Border.all(
            color: active ? accent : ink.withValues(alpha: 0.22),
            width: 1,
          ),
          borderRadius: BorderRadius.circular(999),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(r.emoji, style: const TextStyle(fontSize: 13)),
            const SizedBox(width: 3),
            Text(
              '${r.count}',
              style: GoogleFonts.jetBrainsMono(fontSize: 10, color: active ? accent : ink3),
            ),
          ],
        ),
      ),
    );
  }

  Widget _counter(String glyph, int count, VoidCallback? onTap, Color ink, Color ink3, Color accent, {required bool renote}) {
    final active = count > 0;
    final color = active ? (renote ? accent : ink) : ink3;
    return GestureDetector(
      onTap: onTap,
      child: Text(
        '$glyph $count',
        style: GoogleFonts.jetBrainsMono(fontSize: 10, color: color, letterSpacing: 0.1),
      ),
    );
  }

  // [mmdd, hh:mm] for the stamp widget
  List<String> _stampParts(DateTime dt) {
    final m   = dt.month.toString().padLeft(2, '0');
    final d   = dt.day.toString().padLeft(2, '0');
    final h   = dt.hour.toString().padLeft(2, '0');
    final min = dt.minute.toString().padLeft(2, '0');
    return ['$m/$d', '$h:$min'];
  }

  String _ageLabel(DateTime dt) {
    final diff = DateTime.now().difference(dt);
    if (diff.inMinutes < 1)  return 'now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m';
    if (diff.inHours < 24)   return '${diff.inHours}h';
    if (diff.inDays < 7)     return '${diff.inDays}d';
    return '${dt.month}/${dt.day}';
  }
}
