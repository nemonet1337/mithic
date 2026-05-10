import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/timeline/providers/timeline_provider.dart';
import 'package:mithic/models/note.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';
import 'package:mithic/shared/widgets/note_card.dart';

class HomeTimelineScreen extends ConsumerStatefulWidget {
  const HomeTimelineScreen({super.key});

  @override
  ConsumerState<HomeTimelineScreen> createState() => _HomeTimelineScreenState();
}

class _HomeTimelineScreenState extends ConsumerState<HomeTimelineScreen> {
  int _tlIndex = 0;

  static const _tlLabels = ['ホーム', 'ローカル', 'グローバル'];

  void _refresh() {
    switch (_tlIndex) {
      case 0: ref.read(homeTimelineProvider.notifier).refresh();
      case 1: ref.read(localTimelineProvider.notifier).refresh();
      case 2: ref.read(globalTimelineProvider.notifier).refresh();
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink3 = isDark ? MithicColors.ink3Dark : MithicColors.ink3;

    final tlAsync = switch (_tlIndex) {
      0 => ref.watch(homeTimelineProvider),
      1 => ref.watch(localTimelineProvider),
      _ => ref.watch(globalTimelineProvider),
    };

    return Scaffold(
      appBar: MithicTopBar(
        folio: '01',
        title: 'タイムライン',
        actions: [
          GestureDetector(
            onTap: _refresh,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 4),
              child: Text(
                '↺',
                style: GoogleFonts.jetBrainsMono(fontSize: 16, color: ink3),
              ),
            ),
          ),
          const SizedBox(width: 4),
        ],
      ),
      body: Column(
        children: [
          _SegmentRow(
            selected: _tlIndex,
            labels: _tlLabels,
            onSelect: (i) => setState(() => _tlIndex = i),
          ),
          Expanded(child: _TimelineList(tlAsync: tlAsync, onRefresh: _refresh)),
        ],
      ),
      floatingActionButton: _ComposeFab(),
    );
  }
}

class _SegmentRow extends StatelessWidget {
  final int selected;
  final List<String> labels;
  final ValueChanged<int> onSelect;

  const _SegmentRow({
    required this.selected,
    required this.labels,
    required this.onSelect,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final line = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    return Container(
      padding: const EdgeInsets.fromLTRB(14, 10, 14, 10),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: line, width: 1.25)),
      ),
      child: Align(
        alignment: Alignment.centerLeft,
        child: MithicSegmentedControl(
          labels: labels,
          selected: selected,
          onSelect: onSelect,
        ),
      ),
    );
  }
}

class _TimelineList extends StatelessWidget {
  final AsyncValue<List<Note>> tlAsync;
  final VoidCallback onRefresh;

  const _TimelineList({required this.tlAsync, required this.onRefresh});

  @override
  Widget build(BuildContext context) {
    return tlAsync.when(
      data: (notes) {
        if (notes.isEmpty) {
          return const MithicEmptyState(
            icon: Icons.newspaper_outlined,
            title: 'まだノートがありません',
            subtitle: 'フォローするとここに表示されます',
          );
        }
        return RefreshIndicator(
          onRefresh: () async => onRefresh(),
          child: ListView.builder(
            itemCount: notes.length,
            itemBuilder: (ctx, i) => NoteCard(note: notes[i]),
          ),
        );
      },
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (err, _) => Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            MithicEmptyState(
              icon: Icons.error_outline,
              title: 'エラーが発生しました',
              subtitle: err.toString(),
            ),
            const SizedBox(height: 16),
            MithicButton('リトライ', onPressed: onRefresh),
          ],
        ),
      ),
    );
  }
}

class _ComposeFab extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink = isDark ? MithicColors.inkDark : MithicColors.ink;
    final paper = isDark ? MithicColors.paperDark : MithicColors.paper;
    final accent = isDark ? MithicColors.accentDark : MithicColors.accent;

    return GestureDetector(
      onTap: () => context.push('/compose'),
      child: Container(
        width: 52,
        height: 52,
        decoration: BoxDecoration(
          color: accent,
          shape: BoxShape.circle,
          border: Border.all(color: ink, width: 1.25),
          boxShadow: [BoxShadow(color: ink, offset: const Offset(3, 3))],
        ),
        child: Center(
          child: Text(
            '+',
            style: GoogleFonts.patrickHand(fontSize: 28, color: paper, height: 1),
          ),
        ),
      ),
    );
  }
}
