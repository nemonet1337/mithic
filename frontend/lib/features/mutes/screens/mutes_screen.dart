import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/mutes/providers/mutes_provider.dart';
import 'package:cached_network_image/cached_network_image.dart';

class MutesScreen extends ConsumerWidget {
  const MutesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final mutesAsync = ref.watch(mutesProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('ミュート中のユーザー'),
      ),
      body: mutesAsync.when(
        data: (users) {
          if (users.isEmpty) {
            return const Center(
              child: Text('ミュート中のユーザーがいません'),
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
                    await ref.read(mutesActionsProvider).unmute(user.id);
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
