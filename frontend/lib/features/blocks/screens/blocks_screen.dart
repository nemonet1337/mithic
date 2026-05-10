import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/blocks/providers/blocks_provider.dart';
import 'package:cached_network_image/cached_network_image.dart';

class BlocksScreen extends ConsumerWidget {
  const BlocksScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final blocksAsync = ref.watch(blocksProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('ブロック中のユーザー'),
      ),
      body: blocksAsync.when(
        data: (users) {
          if (users.isEmpty) {
            return const Center(
              child: Text('ブロック中のユーザーがいません'),
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
                trailing: TextButton(
                  onPressed: () async {
                    await ref.read(blocksActionsProvider).unblock(user.id);
                  },
                  child: const Text('解除'),
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
