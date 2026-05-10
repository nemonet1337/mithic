import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/auth/providers/auth_provider.dart';
import 'package:mithic/features/profile/providers/profile_provider.dart';
import 'package:mithic/models/user.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';
import 'package:mithic/shared/widgets/note_card.dart';

class ProfileScreen extends ConsumerWidget {
  final String? userId;
  const ProfileScreen({super.key, this.userId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final currentUser = ref.watch(currentUserProvider);
    final targetId = userId ?? currentUser?.id;

    if (targetId == null) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }

    final userAsync = ref.watch(profileProvider(targetId));

    return Scaffold(
      appBar: MithicTopBar(
        folio: '05',
        title: userAsync.when(
          data: (u) => u.name ?? u.username,
          loading: () => '…',
          error: (_, __) => 'プロフィール',
        ),
        actions: [
          if (userId == null)
            GestureDetector(
              onTap: () => context.push('/settings'),
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 6),
                child: Icon(Icons.settings_outlined, size: 18,
                  color: Theme.of(context).brightness == Brightness.dark
                      ? MithicColors.ink3Dark : MithicColors.ink3),
              ),
            ),
          const SizedBox(width: 4),
        ],
      ),
      body: userAsync.when(
        data: (user) => _ProfileBody(user: user, targetId: targetId, isSelf: userId == null),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, _) => Center(
          child: MithicEmptyState(icon: Icons.error_outline, title: 'エラー', subtitle: err.toString()),
        ),
      ),
    );
  }
}

class _ProfileBody extends ConsumerWidget {
  final User user;
  final String targetId;
  final bool isSelf;

  const _ProfileBody({required this.user, required this.targetId, required this.isSelf});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final paper  = isDark ? MithicColors.paperDark : MithicColors.paper;
    final card   = isDark ? MithicColors.cardDark  : MithicColors.card;
    final accent = isDark ? MithicColors.accentDark : MithicColors.accent;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    final currentTab = ref.watch(profileTabProvider);
    final postsAsync = ref.watch(profilePostsProvider(targetId));

    return CustomScrollView(
      slivers: [
        // Banner + avatar header
        SliverToBoxAdapter(
          child: _Header(
            user: user,
            targetId: targetId,
            isSelf: isSelf,
            ink: ink, ink3: ink3, paper: paper, card: card, accent: accent, line: line,
          ),
        ),
        // Tab bar
        SliverPersistentHeader(
          pinned: true,
          delegate: _TabHeaderDelegate(
            tabs: const ['投稿', '返信', 'メディア'],
            selected: currentTab.index,
            onSelect: (i) => ref.read(profileTabProvider.notifier).state = ProfileTab.values[i],
            ink: ink, ink3: ink3, paper: paper, line: line, accent: accent,
          ),
        ),
        // Posts
        if (currentTab == ProfileTab.posts)
          postsAsync.when(
            data: (posts) => posts.isEmpty
                ? SliverFillRemaining(
                    child: MithicEmptyState(
                      icon: Icons.article_outlined,
                      title: 'まだノートがありません',
                    ),
                  )
                : SliverList(
                    delegate: SliverChildBuilderDelegate(
                      (ctx, i) => NoteCard(note: posts[i]),
                      childCount: posts.length,
                    ),
                  ),
            loading: () => const SliverFillRemaining(child: Center(child: CircularProgressIndicator())),
            error: (e, _) => SliverFillRemaining(
              child: MithicEmptyState(icon: Icons.error_outline, title: 'エラー', subtitle: e.toString()),
            ),
          )
        else
          const SliverFillRemaining(
            child: MithicEmptyState(icon: Icons.construction, title: '準備中'),
          ),
      ],
    );
  }
}

class _Header extends ConsumerWidget {
  final User user;
  final String targetId;
  final bool isSelf;
  final Color ink, ink3, paper, card, accent, line;

  const _Header({
    required this.user,
    required this.targetId,
    required this.isSelf,
    required this.ink,
    required this.ink3,
    required this.paper,
    required this.card,
    required this.accent,
    required this.line,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final relationshipAsync = isSelf ? null : ref.watch(profileRelationshipProvider(targetId));

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Banner
        Container(
          height: 100,
          decoration: BoxDecoration(
            color: card,
            border: Border(bottom: BorderSide(color: line, width: 1.25)),
            image: null,
          ),
        ),
        // Avatar row
        Padding(
          padding: const EdgeInsets.fromLTRB(14, 0, 14, 0),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              Transform.translate(
                offset: const Offset(0, -24),
                child: Container(
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    border: Border.all(color: paper, width: 3),
                  ),
                  child: MithicAvatar(
                    url: user.avatarUrl,
                    fallbackName: user.name ?? user.username,
                    size: 72,
                  ),
                ),
              ),
              const Spacer(),
              const SizedBox(height: 52), // align to bottom of banner cutout
              if (!isSelf)
                Padding(
                  padding: const EdgeInsets.only(bottom: 8),
                  child: relationshipAsync?.when(
                    data: (rel) => GestureDetector(
                      onTap: () async {
                        final actions = ref.read(profileActionsProvider);
                        if (rel.following) {
                          await actions.unfollow(targetId);
                        } else {
                          await actions.follow(targetId);
                        }
                      },
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 7),
                        decoration: BoxDecoration(
                          color: rel.following ? paper : accent,
                          border: Border.all(color: ink, width: 1.25),
                          borderRadius: BorderRadius.circular(8),
                          boxShadow: [BoxShadow(color: ink, offset: const Offset(2, 2))],
                        ),
                        child: Text(
                          rel.following ? 'フォロー中' : 'フォロー',
                          style: GoogleFonts.dmSans(
                            fontSize: 13,
                            fontWeight: FontWeight.w500,
                            color: rel.following ? ink : Colors.white,
                          ),
                        ),
                      ),
                    ),
                    loading: () => const SizedBox.shrink(),
                    error: (_, __) => const SizedBox.shrink(),
                  ),
                ),
            ],
          ),
        ),
        // Name + handle
        Padding(
          padding: const EdgeInsets.fromLTRB(14, 0, 14, 8),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                user.name ?? user.username,
                style: GoogleFonts.patrickHand(fontSize: 24, color: ink, height: 1),
              ),
              const SizedBox(height: 2),
              Text(
                '@${user.username}',
                style: GoogleFonts.jetBrainsMono(fontSize: 11, color: ink3, letterSpacing: 0.1),
              ),
            ],
          ),
        ),
        // Bio
        if (user.bio != null && user.bio!.isNotEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(14, 0, 14, 10),
            child: Text(
              user.bio!,
              style: GoogleFonts.dmSans(fontSize: 13.5, color: ink, height: 1.55),
            ),
          ),
        // Stats
        Padding(
          padding: const EdgeInsets.fromLTRB(14, 0, 14, 14),
          child: Row(
            children: [
              _Stat(value: user.followingCount?.toString() ?? '0', label: 'フォロー',
                ink: ink, ink3: ink3,
                onTap: () => context.push('/profile/$targetId/following')),
              const SizedBox(width: 24),
              _Stat(value: user.followersCount?.toString() ?? '0', label: 'フォロワー',
                ink: ink, ink3: ink3,
                onTap: () => context.push('/profile/$targetId/followers')),
              ],
          ),
        ),
        Container(height: 1.25, color: line),
      ],
    );
  }
}

class _Stat extends StatelessWidget {
  final String value;
  final String label;
  final Color ink;
  final Color ink3;
  final VoidCallback? onTap;

  const _Stat({
    required this.value,
    required this.label,
    required this.ink,
    required this.ink3,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.baseline,
        textBaseline: TextBaseline.alphabetic,
        children: [
          Text(
            value,
            style: GoogleFonts.patrickHand(fontSize: 20, color: ink, height: 1),
          ),
          const SizedBox(width: 4),
          Text(
            label,
            style: GoogleFonts.jetBrainsMono(fontSize: 10, color: ink3, letterSpacing: 0.1),
          ),
        ],
      ),
    );
  }
}

class _TabHeaderDelegate extends SliverPersistentHeaderDelegate {
  final List<String> tabs;
  final int selected;
  final ValueChanged<int> onSelect;
  final Color ink, ink3, paper, line, accent;

  const _TabHeaderDelegate({
    required this.tabs,
    required this.selected,
    required this.onSelect,
    required this.ink,
    required this.ink3,
    required this.paper,
    required this.line,
    required this.accent,
  });

  @override
  double get minExtent => 44;
  @override
  double get maxExtent => 44;

  @override
  Widget build(BuildContext context, double shrinkOffset, bool overlapsContent) {
    return Container(
      color: paper,
      padding: const EdgeInsets.symmetric(horizontal: 14),
      child: MithicTabBar(
        tabs: tabs,
        selected: selected,
        onSelect: onSelect,
      ),
    );
  }

  @override
  bool shouldRebuild(_TabHeaderDelegate old) =>
      old.selected != selected || old.tabs != tabs;
}
