import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:lucide_icons/lucide_icons.dart';
import 'package:mithic/api/endpoints/messages.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/messages/providers/messages_provider.dart';
import 'package:intl/intl.dart';

class MessagesScreen extends ConsumerWidget {
  const MessagesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final paper  = isDark ? MithicColors.paperDark : MithicColors.paper;
    final ink    = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;

    return Scaffold(
      backgroundColor: paper,
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 20, 20, 0),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '[ 04 ]',
                        style: GoogleFonts.jetBrainsMono(
                          fontSize: 10,
                          color: ink3,
                          letterSpacing: 0.14,
                        ),
                      ),
                      Text(
                        '受信箱',
                        style: GoogleFonts.patrickHand(fontSize: 30, color: ink, height: 1),
                      ),
                    ],
                  ),
                ),
                _UnreadBadge(ink3: ink3),
                const SizedBox(width: 8),
                _NewMessageButton(ink: ink),
              ],
            ),
          ),
          const SizedBox(height: 12),
          // Tabs
          _TabRow(ink: ink, ink3: ink3),
          // List
          Expanded(
            child: _ConversationList(ink: ink, ink3: ink3),
          ),
        ],
      ),
    );
  }
}

// ── Header widgets ────────────────────────────────────────────────────────────
class _UnreadBadge extends ConsumerWidget {
  final Color ink3;
  const _UnreadBadge({required this.ink3});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final convsAsync = ref.watch(conversationsProvider);
    final unreadCount = convsAsync.maybeWhen(
      data: (list) => list.where((c) => c.unread).length,
      orElse: () => 0,
    );
    if (unreadCount == 0) return const SizedBox.shrink();
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: MithicColors.accent,
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: MithicColors.ink, width: 1),
      ),
      child: Text(
        '未読 $unreadCount',
        style: GoogleFonts.jetBrainsMono(fontSize: 10, color: Colors.white),
      ),
    );
  }
}

class _NewMessageButton extends StatelessWidget {
  final Color ink;
  const _NewMessageButton({required this.ink});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: () => _showNewMessageDialog(context),
      borderRadius: BorderRadius.circular(8),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: ink, width: 1.25),
        ),
        child: Row(
          children: [
            Icon(LucideIcons.plus, size: 14, color: ink),
            const SizedBox(width: 4),
            Text('新規', style: GoogleFonts.dmSans(fontSize: 12, color: ink)),
          ],
        ),
      ),
    );
  }

  void _showNewMessageDialog(BuildContext context) {
    showDialog<void>(
      context: context,
      builder: (_) => const _NewMessageDialog(),
    );
  }
}

// ── Tabs ──────────────────────────────────────────────────────────────────────
class _TabRow extends StatefulWidget {
  final Color ink;
  final Color ink3;
  const _TabRow({required this.ink, required this.ink3});

  @override
  State<_TabRow> createState() => _TabRowState();
}

class _TabRowState extends State<_TabRow> {
  int _selected = 0;

  @override
  Widget build(BuildContext context) {
    const labels = ['すべて', '未読', 'グループ', 'リクエスト'];
    return Container(
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: MithicColors.lineSoft, width: 1.25),
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 20),
        child: Row(
          children: labels.asMap().entries.map((e) {
            final on = e.key == _selected;
            return InkWell(
              onTap: () => setState(() => _selected = e.key),
              child: Container(
                padding: const EdgeInsets.fromLTRB(0, 8, 14, 8),
                decoration: BoxDecoration(
                  border: Border(
                    bottom: BorderSide(
                      color: on ? MithicColors.accent : Colors.transparent,
                      width: 2,
                    ),
                  ),
                ),
                child: Text(
                  e.value,
                  style: GoogleFonts.dmSans(
                    fontSize: 12,
                    color: on ? widget.ink : widget.ink3,
                    fontWeight: on ? FontWeight.w600 : FontWeight.w400,
                  ),
                ),
              ),
            );
          }).toList(),
        ),
      ),
    );
  }
}

// ── Conversation list ─────────────────────────────────────────────────────────
class _ConversationList extends ConsumerWidget {
  final Color ink;
  final Color ink3;

  const _ConversationList({required this.ink, required this.ink3});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final convsAsync = ref.watch(conversationsProvider);

    return convsAsync.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (e, _) => Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('エラーが発生しました', style: TextStyle(color: ink)),
            const SizedBox(height: 8),
            TextButton(
              onPressed: () => ref.read(conversationsProvider.notifier).refresh(),
              child: const Text('リトライ'),
            ),
          ],
        ),
      ),
      data: (convs) {
        if (convs.isEmpty) {
          return _EmptyState(ink: ink, ink3: ink3);
        }
        return RefreshIndicator(
          onRefresh: () => ref.read(conversationsProvider.notifier).refresh(),
          child: ListView.builder(
            itemCount: convs.length,
            itemBuilder: (ctx, i) => _ConversationTile(
              conversation: convs[i],
              ink: ink,
              ink3: ink3,
            ),
          ),
        );
      },
    );
  }
}

class _ConversationTile extends StatelessWidget {
  final DirectConversation conversation;
  final Color ink;
  final Color ink3;

  const _ConversationTile({
    required this.conversation,
    required this.ink,
    required this.ink3,
  });

  @override
  Widget build(BuildContext context) {
    final other  = conversation.accounts.isNotEmpty
        ? conversation.accounts.first
        : null;
    final lastMsg = conversation.lastMessage;

    return InkWell(
      onTap: () {
        context.push(
          '/messages/${conversation.id}',
          extra: conversation,
        );
      },
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
        decoration: BoxDecoration(
          color: conversation.unread ? MithicColors.accentSoft.withValues(alpha: 0.3) : Colors.transparent,
          border: Border(
            left: BorderSide(
              color: conversation.unread ? MithicColors.accent : Colors.transparent,
              width: 4,
            ),
            bottom: BorderSide(color: MithicColors.lineSoft, width: 1),
          ),
        ),
        child: Row(
          children: [
            // Avatar
            _Avatar(account: other, size: 40),
            const SizedBox(width: 12),
            // Name + preview
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: Text(
                          other?.name ?? 'Unknown',
                          style: GoogleFonts.patrickHand(
                            fontSize: 16,
                            color: ink,
                            height: 1,
                          ),
                        ),
                      ),
                      if (lastMsg != null)
                        Text(
                          _relativeTime(lastMsg.createdAt),
                          style: GoogleFonts.jetBrainsMono(
                            fontSize: 9,
                            color: ink3,
                          ),
                        ),
                    ],
                  ),
                  const SizedBox(height: 2),
                  Text(
                    lastMsg?.content ?? '—',
                    style: GoogleFonts.dmSans(
                      fontSize: 12,
                      color: conversation.unread ? ink : ink3,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ],
              ),
            ),
            if (conversation.unread) ...[
              const SizedBox(width: 8),
              Container(
                width: 7,
                height: 7,
                decoration: const BoxDecoration(
                  color: MithicColors.accent,
                  shape: BoxShape.circle,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  String _relativeTime(DateTime dt) {
    final diff = DateTime.now().difference(dt);
    if (diff.inMinutes < 1)  return 'いま';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m';
    if (diff.inHours < 24)   return '${diff.inHours}h';
    return DateFormat('MM/dd').format(dt);
  }
}

class _EmptyState extends StatelessWidget {
  final Color ink;
  final Color ink3;
  const _EmptyState({required this.ink, required this.ink3});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(LucideIcons.mailOpen, size: 48, color: ink3),
          const SizedBox(height: 16),
          Text('メッセージはありません',
              style: GoogleFonts.patrickHand(fontSize: 20, color: ink3)),
          const SizedBox(height: 8),
          Text('新規ボタンから会話を始めましょう',
              style: GoogleFonts.dmSans(fontSize: 13, color: ink3)),
        ],
      ),
    );
  }
}

// ── Avatar helper ─────────────────────────────────────────────────────────────
class _Avatar extends StatelessWidget {
  final ConversationAccount? account;
  final double size;
  const _Avatar({this.account, required this.size});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    if (account?.avatarUrl != null) {
      return CircleAvatar(
        radius: size / 2,
        backgroundImage: NetworkImage(account!.avatarUrl!),
      );
    }
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: MithicColors.accent2,
        shape: BoxShape.circle,
        border: Border.all(
          color: isDark ? MithicColors.inkDark : MithicColors.ink,
          width: 1.25,
        ),
      ),
      child: Center(
        child: Text(
          account?.name.isNotEmpty == true
              ? account!.name[0].toUpperCase()
              : '?',
          style: GoogleFonts.patrickHand(
            fontSize: size * 0.45,
            color: isDark ? MithicColors.inkDark : MithicColors.ink,
          ),
        ),
      ),
    );
  }
}

// ── New message dialog ────────────────────────────────────────────────────────
class _NewMessageDialog extends StatefulWidget {
  const _NewMessageDialog();

  @override
  State<_NewMessageDialog> createState() => _NewMessageDialogState();
}

class _NewMessageDialogState extends State<_NewMessageDialog> {
  final _acctController = TextEditingController();
  final _msgController  = TextEditingController();

  @override
  void dispose() {
    _acctController.dispose();
    _msgController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final card   = isDark ? MithicColors.cardDark : MithicColors.card;
    final ink    = isDark ? MithicColors.inkDark  : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark : MithicColors.ink3;

    return Dialog(
      backgroundColor: card,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(10),
        side: BorderSide(color: ink, width: 1.25),
      ),
      child: Container(
        width: 400,
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  '[ COMPOSE ]',
                  style: GoogleFonts.jetBrainsMono(
                      fontSize: 10, color: ink3, letterSpacing: 0.14),
                ),
                const SizedBox(width: 8),
                Text('新しいDM',
                    style: GoogleFonts.patrickHand(fontSize: 22, color: ink)),
                const Spacer(),
                IconButton(
                  icon: const Icon(LucideIcons.x),
                  onPressed: () => Navigator.pop(context),
                  iconSize: 18,
                  color: ink3,
                ),
              ],
            ),
            const SizedBox(height: 14),
            Text('宛先',
                style: GoogleFonts.jetBrainsMono(
                    fontSize: 10, color: ink3, letterSpacing: 0.12)),
            const SizedBox(height: 4),
            TextField(
              controller: _acctController,
              decoration: InputDecoration(
                hintText: '@username',
                hintStyle: TextStyle(color: ink3),
                filled: true,
                fillColor: isDark ? MithicColors.cardAltDark : MithicColors.cardAlt,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(8),
                  borderSide: BorderSide(color: ink, width: 1.25),
                ),
              ),
              style: GoogleFonts.jetBrainsMono(fontSize: 12, color: ink),
            ),
            const SizedBox(height: 12),
            Text('メッセージ',
                style: GoogleFonts.jetBrainsMono(
                    fontSize: 10, color: ink3, letterSpacing: 0.12)),
            const SizedBox(height: 4),
            TextField(
              controller: _msgController,
              maxLines: 4,
              decoration: InputDecoration(
                hintText: 'メッセージを入力…',
                hintStyle: GoogleFonts.patrickHand(fontSize: 18, color: ink3),
                filled: true,
                fillColor: isDark ? MithicColors.cardAltDark : MithicColors.cardAlt,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(8),
                  borderSide: BorderSide(color: ink, width: 1.25),
                ),
              ),
              style: GoogleFonts.dmSans(fontSize: 13, color: ink),
            ),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton(
                  onPressed: () => Navigator.pop(context),
                  child: Text('キャンセル',
                      style: GoogleFonts.dmSans(color: ink3)),
                ),
                const SizedBox(width: 8),
                ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: MithicColors.accent,
                    foregroundColor: Colors.white,
                    elevation: 0,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(8),
                      side: const BorderSide(color: MithicColors.ink, width: 1.25),
                    ),
                  ),
                  onPressed: () {
                    // TODO: send via provider
                    Navigator.pop(context);
                  },
                  child: Text('送信', style: GoogleFonts.dmSans()),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
