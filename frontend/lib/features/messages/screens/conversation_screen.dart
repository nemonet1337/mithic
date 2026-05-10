import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:lucide_icons/lucide_icons.dart';
import 'package:mithic/api/endpoints/messages.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/messages/providers/messages_provider.dart';
import 'package:intl/intl.dart';

class ConversationScreen extends ConsumerStatefulWidget {
  final String conversationId;
  final DirectConversation? conversation;

  const ConversationScreen({
    super.key,
    required this.conversationId,
    this.conversation,
  });

  @override
  ConsumerState<ConversationScreen> createState() => _ConversationScreenState();
}

class _ConversationScreenState extends ConsumerState<ConversationScreen> {
  final _inputController = TextEditingController();
  final _scrollController = ScrollController();
  String _toAcct = '';

  @override
  void initState() {
    super.initState();
    final other = widget.conversation?.accounts.firstOrNull;
    _toAcct = other?.acct ?? other?.username ?? '';
  }

  @override
  void dispose() {
    _inputController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isDark  = Theme.of(context).brightness == Brightness.dark;
    final paper   = isDark ? MithicColors.paperDark  : MithicColors.paper;
    final ink     = isDark ? MithicColors.inkDark     : MithicColors.ink;
    final ink3    = isDark ? MithicColors.ink3Dark    : MithicColors.ink3;
    final lineSoft = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    final other   = widget.conversation?.accounts.firstOrNull;
    final msgsAsync = ref.watch(messagesProvider(widget.conversationId));

    return Scaffold(
      backgroundColor: paper,
      body: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            decoration: BoxDecoration(
              color: paper,
              border: Border(bottom: BorderSide(color: lineSoft, width: 1.25)),
            ),
            child: Row(
              children: [
                IconButton(
                  icon: Icon(LucideIcons.arrowLeft, size: 18, color: ink),
                  onPressed: () => context.go('/messages'),
                ),
                const SizedBox(width: 4),
                _AvatarSmall(account: other),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        other?.name ?? 'Unknown',
                        style: GoogleFonts.patrickHand(
                            fontSize: 16, color: ink, height: 1),
                      ),
                      Text(
                        '@${other?.acct ?? other?.username ?? '...'}',
                        style: GoogleFonts.jetBrainsMono(
                            fontSize: 9, color: ink3),
                      ),
                    ],
                  ),
                ),
                IconButton(
                  icon: Icon(LucideIcons.moreHorizontal, size: 18, color: ink3),
                  onPressed: () {},
                ),
              ],
            ),
          ),

          // Messages
          Expanded(
            child: msgsAsync.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Center(
                child: Text('エラー: $e',
                    style: TextStyle(color: ink3)),
              ),
              data: (msgs) => msgs.isEmpty
                  ? _EmptyConversation(ink3: ink3)
                  : _MessageList(
                      messages: msgs,
                      scrollController: _scrollController,
                      ink: ink,
                      ink3: ink3,
                    ),
            ),
          ),

          // Input
          _InputBar(
            controller: _inputController,
            ink: ink,
            ink3: ink3,
            lineSoft: lineSoft,
            isDark: isDark,
            onSend: _sendMessage,
          ),
        ],
      ),
    );
  }

  void _sendMessage() {
    final content = _inputController.text.trim();
    if (content.isEmpty || _toAcct.isEmpty) return;
    ref.read(messagesProvider(widget.conversationId).notifier).sendMessage(
          toAcct: _toAcct,
          content: content,
        );
    _inputController.clear();
  }
}

// ── Messages list ─────────────────────────────────────────────────────────────
class _MessageList extends StatelessWidget {
  final List<DirectMessage> messages;
  final ScrollController scrollController;
  final Color ink;
  final Color ink3;

  const _MessageList({
    required this.messages,
    required this.scrollController,
    required this.ink,
    required this.ink3,
  });

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      controller: scrollController,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      itemCount: messages.length,
      itemBuilder: (ctx, i) {
        final msg = messages[i];
        final showDate = i == 0 ||
            !_sameDay(messages[i - 1].createdAt, msg.createdAt);
        return Column(
          children: [
            if (showDate) _DateSeparator(date: msg.createdAt, ink3: ink3),
            _MessageBubble(message: msg, ink: ink, ink3: ink3),
          ],
        );
      },
    );
  }

  bool _sameDay(DateTime a, DateTime b) =>
      a.year == b.year && a.month == b.month && a.day == b.day;
}

class _DateSeparator extends StatelessWidget {
  final DateTime date;
  final Color ink3;
  const _DateSeparator({required this.date, required this.ink3});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 10),
      child: Center(
        child: Text(
          '— ${DateFormat('yyyy年MM月dd日').format(date)} —',
          style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3),
        ),
      ),
    );
  }
}

class _MessageBubble extends StatelessWidget {
  final DirectMessage message;
  final Color ink;
  final Color ink3;

  const _MessageBubble({
    required this.message,
    required this.ink,
    required this.ink3,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final isMine = message.mine;

    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        mainAxisAlignment:
            isMine ? MainAxisAlignment.end : MainAxisAlignment.start,
        children: [
          if (!isMine) ...[
            const SizedBox(width: 4),
          ],
          ConstrainedBox(
            constraints: BoxConstraints(
              maxWidth: MediaQuery.of(context).size.width * 0.65,
            ),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: isMine
                    ? MithicColors.accent
                    : (isDark ? MithicColors.cardDark : MithicColors.card),
                borderRadius: BorderRadius.only(
                  topLeft: const Radius.circular(12),
                  topRight: const Radius.circular(12),
                  bottomLeft: isMine
                      ? const Radius.circular(12)
                      : const Radius.circular(4),
                  bottomRight: isMine
                      ? const Radius.circular(4)
                      : const Radius.circular(12),
                ),
                border: Border.all(
                  color: isMine
                      ? MithicColors.ink
                      : (isDark
                          ? const Color(0x38F3EFE6)
                          : MithicColors.lineSoft),
                  width: 1.25,
                ),
              ),
              child: Column(
                crossAxisAlignment: isMine
                    ? CrossAxisAlignment.end
                    : CrossAxisAlignment.start,
                children: [
                  Text(
                    message.content,
                    style: GoogleFonts.dmSans(
                      fontSize: 13,
                      color: isMine ? Colors.white : ink,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    DateFormat('HH:mm').format(message.createdAt),
                    style: GoogleFonts.jetBrainsMono(
                      fontSize: 9,
                      color: isMine
                          ? Colors.white.withValues(alpha: 0.7)
                          : ink3,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

// ── Input bar ─────────────────────────────────────────────────────────────────
class _InputBar extends StatelessWidget {
  final TextEditingController controller;
  final Color ink;
  final Color ink3;
  final Color lineSoft;
  final bool isDark;
  final VoidCallback onSend;

  const _InputBar({
    required this.controller,
    required this.ink,
    required this.ink3,
    required this.lineSoft,
    required this.isDark,
    required this.onSend,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: isDark ? MithicColors.paperDark : MithicColors.paper,
        border: Border(top: BorderSide(color: lineSoft, width: 1.25)),
      ),
      child: Row(
        children: [
          Expanded(
            child: TextField(
              controller: controller,
              decoration: InputDecoration(
                hintText: 'メッセージを入力…',
                hintStyle: GoogleFonts.dmSans(color: ink3, fontSize: 13),
                filled: true,
                fillColor: isDark ? MithicColors.cardDark : MithicColors.card,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(8),
                  borderSide: BorderSide(color: lineSoft, width: 1.25),
                ),
                enabledBorder: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(8),
                  borderSide: BorderSide(color: lineSoft, width: 1.25),
                ),
                focusedBorder: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(8),
                  borderSide: const BorderSide(color: MithicColors.accent, width: 1.5),
                ),
                contentPadding:
                    const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              ),
              style: GoogleFonts.dmSans(fontSize: 13, color: ink),
              onSubmitted: (_) => onSend(),
            ),
          ),
          const SizedBox(width: 8),
          IconButton(
            icon: const Icon(LucideIcons.paperclip),
            iconSize: 18,
            color: ink3,
            onPressed: () {},
          ),
          const SizedBox(width: 2),
          _SendButton(onSend: onSend),
        ],
      ),
    );
  }
}

class _SendButton extends StatelessWidget {
  final VoidCallback onSend;
  const _SendButton({required this.onSend});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onSend,
      borderRadius: BorderRadius.circular(8),
      child: Container(
        width: 36,
        height: 36,
        decoration: BoxDecoration(
          color: MithicColors.accent,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: MithicColors.ink, width: 1.25),
          boxShadow: const [BoxShadow(color: MithicColors.ink, offset: Offset(2, 2))],
        ),
        child: const Icon(LucideIcons.send, size: 16, color: Colors.white),
      ),
    );
  }
}

class _EmptyConversation extends StatelessWidget {
  final Color ink3;
  const _EmptyConversation({required this.ink3});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text(
        'まだメッセージはありません\n最初のメッセージを送りましょう',
        style: GoogleFonts.patrickHand(fontSize: 18, color: ink3, height: 1.5),
        textAlign: TextAlign.center,
      ),
    );
  }
}

class _AvatarSmall extends StatelessWidget {
  final ConversationAccount? account;
  const _AvatarSmall({this.account});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    if (account?.avatarUrl != null) {
      return CircleAvatar(
        radius: 16,
        backgroundImage: NetworkImage(account!.avatarUrl!),
      );
    }
    return Container(
      width: 32,
      height: 32,
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
            fontSize: 14,
            color: isDark ? MithicColors.inkDark : MithicColors.ink,
          ),
        ),
      ),
    );
  }
}
