import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/follow_requests/providers/follow_requests_provider.dart';
import 'package:cached_network_image/cached_network_image.dart';

class FollowRequestsScreen extends ConsumerWidget {
  const FollowRequestsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final followRequestsAsync = ref.watch(followRequestsProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('フォローリクエスト'),
      ),
      body: followRequestsAsync.when(
        data: (users) {
          if (users.isEmpty) {
            return const Center(
              child: Text('フォローリクエストがありません'),
            );
          }
          return ListView.builder(
            itemCount: users.length,
            itemBuilder: (context, index) {
              final user = users[index];
              return ListTile(
                leading: CircleAvatar(
                  backgroundImage: user.avatarUrl != null
                      ? CachedNetworkImageProvider(user.avatarUrl!)
                      : null,
                ),
                title: Text(user.name ?? user.username),
                subtitle: Text('@${user.username}'),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextButton(
                      onPressed: () async {
                        await ref.read(followRequestsActionsProvider).accept(user.id);
                      },
                      child: const Text('承認'),
                    ),
                    TextButton(
                      onPressed: () async {
                        await ref.read(followRequestsActionsProvider).reject(user.id);
                      },
                      child: const Text('拒否'),
                    ),
                  ],
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
}
