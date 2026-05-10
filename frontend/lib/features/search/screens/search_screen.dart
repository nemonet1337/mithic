import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/search/providers/search_provider.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';
import 'package:mithic/shared/widgets/note_card.dart';

class SearchScreen extends ConsumerStatefulWidget {
  const SearchScreen({super.key});

  @override
  ConsumerState<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends ConsumerState<SearchScreen> {
  final _searchCtrl = TextEditingController();
  int _tab = 0;

  @override
  void dispose() {
    _searchCtrl.dispose();
    super.dispose();
  }

  void _submit(String q) {
    ref.read(searchQueryProvider.notifier).state = q.trim();
    ref.read(searchTypeProvider.notifier).state = SearchType.values[_tab];
  }

  void _clear() {
    _searchCtrl.clear();
    ref.read(searchQueryProvider.notifier).state = '';
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final paper  = isDark ? MithicColors.paperDark : MithicColors.paper;
    final card   = isDark ? MithicColors.cardDark  : MithicColors.card;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    final query = ref.watch(searchQueryProvider);

    return Scaffold(
      appBar: MithicTopBar(
        folio: '02',
        title: '検索',
      ),
      body: Column(
        children: [
          // Search input bar
          Container(
            padding: const EdgeInsets.fromLTRB(14, 10, 14, 10),
            decoration: BoxDecoration(
              border: Border(bottom: BorderSide(color: line, width: 1.25)),
            ),
            child: Row(
              children: [
                Expanded(
                  child: Container(
                    height: 38,
                    decoration: BoxDecoration(
                      color: card,
                      border: Border.all(color: ink, width: 1.25),
                      borderRadius: BorderRadius.circular(8),
                      boxShadow: [BoxShadow(color: ink, offset: const Offset(2, 2))],
                    ),
                    child: Row(
                      children: [
                        const SizedBox(width: 10),
                        Icon(Icons.search, size: 16, color: ink3),
                        const SizedBox(width: 8),
                        Expanded(
                          child: TextField(
                            controller: _searchCtrl,
                            style: GoogleFonts.dmSans(fontSize: 13.5, color: ink),
                            decoration: InputDecoration(
                              hintText: 'ノート、ユーザー、タグを検索',
                              hintStyle: GoogleFonts.dmSans(fontSize: 13.5, color: ink3),
                              border: InputBorder.none,
                              isDense: true,
                              contentPadding: EdgeInsets.zero,
                            ),
                            onSubmitted: _submit,
                            textInputAction: TextInputAction.search,
                          ),
                        ),
                        if (query.isNotEmpty)
                          GestureDetector(
                            onTap: _clear,
                            child: Padding(
                              padding: const EdgeInsets.symmetric(horizontal: 8),
                              child: Icon(Icons.close, size: 14, color: ink3),
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                GestureDetector(
                  onTap: () => _submit(_searchCtrl.text),
                  child: Container(
                    height: 38,
                    padding: const EdgeInsets.symmetric(horizontal: 12),
                    decoration: BoxDecoration(
                      color: ink,
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Center(
                      child: Text(
                        'GO',
                        style: GoogleFonts.jetBrainsMono(
                          fontSize: 11,
                          color: paper,
                          fontWeight: FontWeight.w600,
                          letterSpacing: 0.5,
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
          // Tabs when searching
          if (query.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(14, 0, 14, 0),
              child: MithicTabBar(
                tabs: const ['ノート', 'ユーザー', 'ハッシュタグ'],
                selected: _tab,
                onSelect: (i) {
                  setState(() => _tab = i);
                  ref.read(searchTypeProvider.notifier).state = SearchType.values[i];
                },
              ),
            ),
          // Body
          Expanded(
            child: query.isEmpty
                ? _TrendsSection()
                : _SearchResults(query: query, tab: _tab),
          ),
        ],
      ),
    );
  }
}

class _TrendsSection extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    final trendsAsync = ref.watch(trendsProvider);

    return ListView(
      padding: const EdgeInsets.all(14),
      children: [
        Padding(
          padding: const EdgeInsets.only(bottom: 10),
          child: MithicLabel('トレンド'),
        ),
        trendsAsync.when(
          data: (trends) {
            if (trends.isEmpty) {
              return const MithicEmptyState(
                icon: Icons.trending_up,
                title: 'トレンドがありません',
              );
            }
            return Column(
              children: trends.asMap().entries.map((e) {
                final trend = e.value;
                final num   = (e.key + 1).toString().padLeft(2, '0');
                return GestureDetector(
                  onTap: () => context.push('/hashtags/${trend.tag}'),
                  child: Container(
                    padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 2),
                    decoration: BoxDecoration(
                      border: Border(bottom: BorderSide(color: line, width: 1)),
                    ),
                    child: Row(
                      children: [
                        Text(
                          num,
                          style: GoogleFonts.jetBrainsMono(
                            fontSize: 11, color: ink3, letterSpacing: 0.1,
                          ),
                        ),
                        const SizedBox(width: 14),
                        Expanded(
                          child: Text(
                            '#${trend.tag}',
                            style: GoogleFonts.dmSans(
                              fontSize: 14, color: ink, fontWeight: FontWeight.w500,
                            ),
                          ),
                        ),
                        Text(
                          '${trend.count} 件',
                          style: GoogleFonts.jetBrainsMono(
                            fontSize: 10, color: ink3,
                          ),
                        ),
                      ],
                    ),
                  ),
                );
              }).toList(),
            );
          },
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (err, _) => MithicEmptyState(
            icon: Icons.error_outline,
            title: 'エラー',
            subtitle: err.toString(),
          ),
        ),
      ],
    );
  }
}

class _SearchResults extends ConsumerWidget {
  final String query;
  final int tab;

  const _SearchResults({required this.query, required this.tab});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return switch (tab) {
      0 => _NotesTab(query: query),
      1 => _UsersTab(query: query),
      _ => _HashtagsTab(query: query),
    };
  }
}

class _NotesTab extends ConsumerWidget {
  final String query;
  const _NotesTab({required this.query});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ref.watch(searchNotesProvider(query)).when(
      data: (notes) => notes.isEmpty
          ? const MithicEmptyState(icon: Icons.article_outlined, title: 'ノートが見つかりませんでした')
          : ListView.builder(
              itemCount: notes.length,
              itemBuilder: (_, i) => NoteCard(note: notes[i]),
            ),
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => MithicEmptyState(icon: Icons.error_outline, title: 'エラー', subtitle: e.toString()),
    );
  }
}

class _UsersTab extends ConsumerWidget {
  final String query;
  const _UsersTab({required this.query});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark  : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark : MithicColors.ink3;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    return ref.watch(searchUsersProvider(query)).when(
      data: (users) => users.isEmpty
          ? const MithicEmptyState(icon: Icons.people_outline, title: 'ユーザーが見つかりませんでした')
          : ListView.builder(
              itemCount: users.length,
              itemBuilder: (ctx, i) {
                final user = users[i];
                return GestureDetector(
                  onTap: () => ctx.push('/profile?userId=${user.id}'),
                  child: Container(
                    padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
                    decoration: BoxDecoration(
                      border: Border(bottom: BorderSide(color: line, width: 1.25)),
                    ),
                    child: Row(
                      children: [
                        MithicAvatar(url: user.avatarUrl, fallbackName: user.name ?? user.username, size: 42),
                        const SizedBox(width: 12),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                user.name ?? user.username,
                                style: GoogleFonts.patrickHand(fontSize: 16, color: ink, height: 1),
                              ),
                              const SizedBox(height: 2),
                              Text(
                                '@${user.username}',
                                style: GoogleFonts.jetBrainsMono(fontSize: 11, color: ink3),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => MithicEmptyState(icon: Icons.error_outline, title: 'エラー', subtitle: e.toString()),
    );
  }
}

class _HashtagsTab extends ConsumerWidget {
  final String query;
  const _HashtagsTab({required this.query});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark  : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark : MithicColors.ink3;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    return ref.watch(searchHashtagsProvider(query)).when(
      data: (tags) => tags.isEmpty
          ? const MithicEmptyState(icon: Icons.tag, title: 'ハッシュタグが見つかりませんでした')
          : ListView.builder(
              itemCount: tags.length,
              itemBuilder: (ctx, i) => GestureDetector(
                onTap: () => ctx.push('/hashtags/${tags[i].replaceFirst('#', '')}'),
                child: Container(
                  padding: const EdgeInsets.fromLTRB(14, 13, 14, 13),
                  decoration: BoxDecoration(
                    border: Border(bottom: BorderSide(color: line, width: 1.25)),
                  ),
                  child: Row(
                    children: [
                      Text('#', style: GoogleFonts.jetBrainsMono(fontSize: 14, color: ink3)),
                      const SizedBox(width: 4),
                      Text(
                        tags[i].replaceFirst('#', ''),
                        style: GoogleFonts.dmSans(fontSize: 14, color: ink, fontWeight: FontWeight.w500),
                      ),
                    ],
                  ),
                ),
              ),
            ),
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => MithicEmptyState(icon: Icons.error_outline, title: 'エラー', subtitle: e.toString()),
    );
  }
}
