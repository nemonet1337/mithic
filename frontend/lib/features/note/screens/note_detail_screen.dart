import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/models/note.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';
import 'package:mithic/shared/widgets/note_card.dart';

class NoteDetailScreen extends ConsumerStatefulWidget {
  final String noteId;

  const NoteDetailScreen({super.key, required this.noteId});

  @override
  ConsumerState<NoteDetailScreen> createState() => _NoteDetailScreenState();
}

class _NoteDetailScreenState extends ConsumerState<NoteDetailScreen> {
  Note? _note;
  List<Note> _replies = [];
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final note    = await _fetchNote();
      final replies = await _fetchReplies();
      if (mounted) setState(() { _note = note; _replies = replies; _loading = false; });
    } catch (e) {
      if (mounted) setState(() { _error = e.toString(); _loading = false; });
    }
  }

  Future<Note?> _fetchNote() async {
    // TODO: implement API call
    return null;
  }

  Future<List<Note>> _fetchReplies() async {
    // TODO: implement API call
    return [];
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    return Scaffold(
      appBar: MithicTopBar(
        folio: 'note',
        title: 'ノート詳細',
        actions: [
          GestureDetector(
            onTap: () => context.push('/compose'),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 6),
              child: Icon(Icons.reply, size: 18, color: ink3),
            ),
          ),
          const SizedBox(width: 4),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : _error != null
              ? Center(child: MithicEmptyState(
                  icon: Icons.error_outline, title: 'エラー', subtitle: _error))
              : _note == null
                  ? const Center(child: MithicEmptyState(
                      icon: Icons.article_outlined, title: 'ノートが見つかりません'))
                  : _DetailBody(
                      note: _note!,
                      replies: _replies,
                      line: line,
                    ),
    );
  }
}

class _DetailBody extends StatelessWidget {
  final Note note;
  final List<Note> replies;
  final Color line;

  const _DetailBody({required this.note, required this.replies, required this.line});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;

    return ListView(
      children: [
        // Parent note (if reply)
        if (note.reply != null) ...[
          Opacity(
            opacity: 0.65,
            child: NoteCard(note: note.reply!),
          ),
          Padding(
            padding: const EdgeInsets.only(left: 40),
            child: Container(
              width: 1.25,
              height: 16,
              color: line,
            ),
          ),
        ],
        // Main note — card style, raised
        Padding(
          padding: const EdgeInsets.fromLTRB(14, 14, 14, 0),
          child: NoteCard(note: note, flat: false),
        ),
        // Replies header
        Container(
          padding: const EdgeInsets.fromLTRB(14, 16, 14, 6),
          decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: line, width: 1.25)),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.baseline,
            textBaseline: TextBaseline.alphabetic,
            children: [
              Text(
                '↪',
                style: GoogleFonts.jetBrainsMono(fontSize: 12, color: ink3),
              ),
              const SizedBox(width: 8),
              Text(
                '返信 ${replies.length}',
                style: GoogleFonts.patrickHand(fontSize: 18,
                  color: isDark ? MithicColors.inkDark : MithicColors.ink, height: 1),
              ),
            ],
          ),
        ),
        // Replies list
        if (replies.isEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 24),
            child: MithicEmptyState(
              icon: Icons.chat_bubble_outline,
              title: '返信はまだありません',
            ),
          )
        else
          ...replies.map((r) => NoteCard(note: r)),
      ],
    );
  }
}
