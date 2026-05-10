import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/admin/providers/admin_provider.dart';
import 'package:mithic/models/user.dart';

class AdminUsersScreen extends ConsumerWidget {
  const AdminUsersScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final usersAsync = ref.watch(adminUsersProvider(AdminUsersParams()));

    return Scaffold(
      appBar: AppBar(
        title: const Text('ユーザー管理'),
      ),
      body: usersAsync.when(
        data: (users) {
          if (users.isEmpty) {
            return const Center(
              child: Text('ユーザーがいません'),
            );
          }
          return ListView.builder(
            itemCount: users.length,
            itemBuilder: (context, index) {
              final user = users[index];
              return ListTile(
                leading: user.avatarUrl != null
                    ? CircleAvatar(
                        backgroundImage: NetworkImage(user.avatarUrl!),
                      )
                    : const CircleAvatar(
                        child: Icon(Icons.person),
                      ),
                title: Text(user.name ?? user.username),
                subtitle: Text('@${user.username}'),
                trailing: IconButton(
                  onPressed: () {
                    _showUserActionsDialog(context, ref, user);
                  },
                  icon: const Icon(Icons.more_vert),
                ),
              );
            },
          );
        },
        loading: () => const Center(
          child: CircularProgressIndicator(),
        ),
        error: (error, stack) => Center(
          child: Text('エラー: $error'),
        ),
      ),
    );
  }

  void _showUserActionsDialog(BuildContext context, WidgetRef ref, User user) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(user.name ?? user.username),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.block),
              title: const Text('ユーザーを停止'),
              onTap: () async {
                await _showSuspendDialog(context, ref, user.id);
              },
            ),
            ListTile(
              leading: const Icon(Icons.delete),
              title: const Text('ユーザーを削除'),
              onTap: () async {
                await _showDeleteUserDialog(context, ref, user.id);
              },
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('閉じる'),
          ),
        ],
      ),
    );
  }

  Future<void> _showSuspendDialog(BuildContext context, WidgetRef ref, String userId) async {
    final reasonController = TextEditingController();

    return showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('ユーザーを停止'),
        content: TextField(
          controller: reasonController,
          decoration: const InputDecoration(
            labelText: '理由',
            hintText: '停止理由を入力',
          ),
          maxLines: 3,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          TextButton(
            onPressed: () async {
              final reason = reasonController.text.trim();
              if (reason.isEmpty) return;

              await ref.read(adminActionsProvider).suspendUser(userId, reason);

              if (context.mounted) {
                Navigator.of(context).pop();
                Navigator.of(context).pop();
              }
            },
            child: const Text('停止'),
          ),
        ],
      ),
    );
  }

  Future<void> _showDeleteUserDialog(BuildContext context, WidgetRef ref, String userId) async {
    return showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('ユーザーを削除'),
        content: const Text('このユーザーを削除してもよろしいですか？この操作は取り消せません。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          TextButton(
            onPressed: () async {
              await ref.read(adminActionsProvider).deleteUser(userId);

              if (context.mounted) {
                Navigator.of(context).pop();
                Navigator.of(context).pop();
              }
            },
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('削除'),
          ),
        ],
      ),
    );
  }
}
