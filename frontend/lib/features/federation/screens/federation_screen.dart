import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/federation/providers/federation_provider.dart';

class FederationScreen extends ConsumerWidget {
  const FederationScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final instancesAsync = ref.watch(federationInstancesProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('フェデレーション'),
      ),
      body: instancesAsync.when(
        data: (instances) {
          if (instances.isEmpty) {
            return const Center(
              child: Text('インスタンスがありません'),
            );
          }
          return ListView.builder(
            itemCount: instances.length,
            itemBuilder: (context, index) {
              final instance = instances[index];
              return ListTile(
                leading: instance.iconUrl != null
                    ? CircleAvatar(
                        backgroundImage: NetworkImage(instance.iconUrl!),
                      )
                    : const CircleAvatar(
                        child: Icon(Icons.public),
                      ),
                title: Text(instance.name ?? instance.host),
                subtitle: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(instance.host),
                    if (instance.softwareName != null)
                      Text('ソフトウェア: ${instance.softwareName}'),
                    if (instance.usersCount != null)
                      Text('ユーザー数: ${instance.usersCount}'),
                  ],
                ),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    if (instance.isSuspended == true)
                      const Icon(Icons.block, color: Colors.red),
                    if (instance.isSilenced == true)
                      const Icon(Icons.volume_off, color: Colors.orange),
                    IconButton(
                      onPressed: () {
                        _showInstanceActionsDialog(context, ref, instance);
                      },
                      icon: const Icon(Icons.more_vert),
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

  void _showInstanceActionsDialog(BuildContext context, WidgetRef ref, instance) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(instance.name ?? instance.host),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.block),
              title: Text(instance.isSuspended == true ? 'ブロック解除' : 'ブロック'),
              onTap: () async {
                if (instance.isSuspended == true) {
                  await ref.read(federationActionsProvider).unblockInstance(instance.host);
                } else {
                  await ref.read(federationActionsProvider).blockInstance(instance.host);
                }
                if (context.mounted) {
                  Navigator.of(context).pop();
                }
              },
            ),
            ListTile(
              leading: const Icon(Icons.volume_off),
              title: Text(instance.isSilenced == true ? 'ミュート解除' : 'ミュート'),
              onTap: () async {
                if (instance.isSilenced == true) {
                  await ref.read(federationActionsProvider).unmuteInstance(instance.host);
                } else {
                  await ref.read(federationActionsProvider).muteInstance(instance.host);
                }
                if (context.mounted) {
                  Navigator.of(context).pop();
                }
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
}
