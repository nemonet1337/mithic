import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:intl/intl.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/features/notification/providers/notification_provider.dart';
import 'package:mithic/models/notification.dart' as m;
import 'package:mithic/models/user.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';

class NotificationScreen extends ConsumerWidget {
  const NotificationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final notifAsync = ref.watch(notificationsProvider);
    final unreadAsync = ref.watch(unreadCountProvider);
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink3 = isDark ? MithicColors.ink3Dark : MithicColors.ink3;

    return Scaffold(
      appBar: MithicTopBar(
        folio: '03',
        title: '通知',
        actions: [
          unreadAsync.when(
            data: (count) => count > 0
                ? GestureDetector(
                    onTap: () {}, // TODO: mark all read
                    child: Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 4),
                      child: Stack(
                        clipBehavior: Clip.none,
                        children: [
                          Text('✓✓', style: GoogleFonts.jetBrainsMono(fontSize: 13, color: ink3)),
                          Positioned(
                            top: -4,
                            right: -8,
                            child: Container(
                              width: 16,
                              height: 16,
                              decoration: const BoxDecoration(
                                color: MithicColors.accent,
                                shape: BoxShape.circle,
                              ),
                              child: Center(
                                child: Text(
                                  '$count',
                                  style: const TextStyle(fontSize: 9, color: Colors.white),
                                ),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  )
                : const SizedBox.shrink(),
            loading: () => const SizedBox.shrink(),
            error: (_, __) => const SizedBox.shrink(),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: notifAsync.when(
        data: (notifications) {
          if (notifications.isEmpty) {
            return const MithicEmptyState(
              icon: Icons.notifications_none,
              title: '通知がありません',
            );
          }
          return ListView.builder(
            itemCount: notifications.length,
            itemBuilder: (ctx, i) => _NotifRow(notification: notifications[i]),
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, _) => Center(
          child: MithicEmptyState(
            icon: Icons.error_outline,
            title: 'エラーが発生しました',
            subtitle: err.toString(),
          ),
        ),
      ),
    );
  }
}

class _NotifRow extends StatelessWidget {
  final m.Notification notification;

  const _NotifRow({required this.notification});

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final paper  = isDark ? MithicColors.paperDark : MithicColors.paper;
    final accent = isDark ? MithicColors.accentDark : MithicColors.accent;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    return GestureDetector(
      onTap: () {
        if (notification.note != null) {
          context.push('/notes/${notification.note!.id}');
        } else if (notification.user != null) {
          context.push('/profile?userId=${notification.user!.id}');
        }
      },
      child: Container(
        padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
        decoration: BoxDecoration(
          color: notification.isRead ? paper : accent.withValues(alpha: 0.04),
          border: Border(bottom: BorderSide(color: line, width: 1.25)),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Left: type icon + unread dot
            SizedBox(
              width: 36,
              child: Column(
                children: [
                  Container(
                    width: 32,
                    height: 32,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      border: Border.all(color: ink, width: 1.25),
                    ),
                    child: Center(
                      child: Icon(
                        _typeIcon(notification.type),
                        size: 14,
                        color: _typeColor(notification.type, accent, ink3),
                      ),
                    ),
                  ),
                  if (!notification.isRead) ...[
                    const SizedBox(height: 4),
                    Container(
                      width: 6,
                      height: 6,
                      decoration: BoxDecoration(color: accent, shape: BoxShape.circle),
                    ),
                  ],
                ],
              ),
            ),
            const SizedBox(width: 10),
            // Right: avatar + text
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      if (notification.user != null)
                        _Avatar(user: notification.user!, ink: ink),
                      const SizedBox(width: 8),
                      Expanded(
                        child: RichText(
                          text: TextSpan(
                            children: [
                              if (notification.user != null)
                                TextSpan(
                                  text: notification.user!.name ?? notification.user!.username,
                                  style: GoogleFonts.patrickHand(
                                    fontSize: 15,
                                    color: ink,
                                    height: 1,
                                  ),
                                ),
                              TextSpan(
                                text: ' ${_typeText(notification.type)}',
                                style: GoogleFonts.dmSans(fontSize: 13, color: ink3),
                              ),
                            ],
                          ),
                        ),
                      ),
                      Text(
                        _ageLabel(notification.createdAt),
                        style: GoogleFonts.jetBrainsMono(fontSize: 9, color: ink3),
                      ),
                    ],
                  ),
                  if (notification.note != null && notification.note!.text.isNotEmpty) ...[
                    const SizedBox(height: 6),
                    Container(
                      padding: const EdgeInsets.fromLTRB(10, 6, 10, 6),
                      decoration: BoxDecoration(
                        border: Border.all(color: line, width: 1),
                        borderRadius: BorderRadius.circular(6),
                      ),
                      child: Text(
                        notification.note!.text,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                        style: GoogleFonts.dmSans(fontSize: 12, color: ink3, height: 1.5),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  IconData _typeIcon(m.NotificationType type) => switch (type) {
    m.NotificationType.mention            => Icons.alternate_email,
    m.NotificationType.reply              => Icons.reply,
    m.NotificationType.renote             => Icons.repeat,
    m.NotificationType.quote              => Icons.format_quote,
    m.NotificationType.reaction           => Icons.favorite,
    m.NotificationType.follow             => Icons.person_add,
    m.NotificationType.followRequestAccepted => Icons.check_circle,
    m.NotificationType.followRequest      => Icons.person_add_disabled,
    m.NotificationType.pollVote           => Icons.how_to_vote,
    m.NotificationType.pollEnded          => Icons.poll,
  };

  Color _typeColor(m.NotificationType type, Color accent, Color ink3) => switch (type) {
    m.NotificationType.reaction ||
    m.NotificationType.follow   ||
    m.NotificationType.followRequestAccepted => accent,
    _ => ink3,
  };

  String _typeText(m.NotificationType type) => switch (type) {
    m.NotificationType.mention            => 'があなたをメンション',
    m.NotificationType.reply              => 'が返信',
    m.NotificationType.renote             => 'がリノート',
    m.NotificationType.quote              => 'が引用',
    m.NotificationType.reaction           => 'がリアクション',
    m.NotificationType.follow             => 'がフォロー',
    m.NotificationType.followRequestAccepted => 'のフォローが承認',
    m.NotificationType.followRequest      => 'がフォローリクエスト',
    m.NotificationType.pollVote           => 'がアンケートに投票',
    m.NotificationType.pollEnded          => 'のアンケートが終了',
  };

  String _ageLabel(DateTime dt) {
    final diff = DateTime.now().difference(dt);
    if (diff.inMinutes < 1)  return 'now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m';
    if (diff.inHours < 24)   return '${diff.inHours}h';
    if (diff.inDays < 7)     return '${diff.inDays}d';
    return DateFormat('MM/dd').format(dt);
  }
}

class _Avatar extends StatelessWidget {
  final User user;
  final Color ink;

  const _Avatar({required this.user, required this.ink});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 28,
      height: 28,
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        border: Border.all(color: ink, width: 1.25),
      ),
      child: ClipOval(
        child: user.avatarUrl != null
            ? Image.network(user.avatarUrl!, fit: BoxFit.cover,
                errorBuilder: (_, __, ___) => _initials())
            : _initials(),
      ),
    );
  }

  Widget _initials() {
    return Container(
      color: const Color(0xFFE8E4DC),
      child: Center(
        child: Text(
          user.username[0].toUpperCase(),
          style: GoogleFonts.patrickHand(fontSize: 13, color: const Color(0xFF1A1714)),
        ),
      ),
    );
  }
}
