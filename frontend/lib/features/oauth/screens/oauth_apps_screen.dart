import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/oauth/providers/oauth_provider.dart';

class OAuthAppsScreen extends ConsumerWidget {
  const OAuthAppsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final appsAsync = ref.watch(oauthAppsProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('OAuthアプリ'),
        actions: [
          IconButton(
            onPressed: () {
              _showCreateAppDialog(context, ref);
            },
            icon: const Icon(Icons.add),
          ),
        ],
      ),
      body: appsAsync.when(
        data: (apps) {
          if (apps.isEmpty) {
            return const Center(
              child: Text('OAuthアプリがありません'),
            );
          }
          return ListView.builder(
            itemCount: apps.length,
            itemBuilder: (context, index) {
              final app = apps[index];
              return ListTile(
                title: Text(app.name),
                subtitle: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(app.description ?? ''),
                    Text('コールバック: ${app.callbackUrl}'),
                    if (app.permissions != null)
                      Text('権限: ${app.permissions!.join(', ')}'),
                  ],
                ),
                trailing: IconButton(
                  onPressed: () async {
                    await ref.read(oauthActionsProvider).deleteApp(app.id);
                  },
                  icon: const Icon(Icons.delete),
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

  void _showCreateAppDialog(BuildContext context, WidgetRef ref) {
    final nameController = TextEditingController();
    final callbackUrlController = TextEditingController();
    final descriptionController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('OAuthアプリを作成'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: nameController,
                decoration: const InputDecoration(
                  labelText: 'アプリ名',
                  hintText: 'アプリ名を入力',
                ),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: callbackUrlController,
                decoration: const InputDecoration(
                  labelText: 'コールバックURL',
                  hintText: 'https://example.com/callback',
                ),
              ),
              const SizedBox(height: 16),
              TextField(
                controller: descriptionController,
                decoration: const InputDecoration(
                  labelText: '説明',
                  hintText: '説明（省略可）',
                ),
                maxLines: 3,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          TextButton(
            onPressed: () async {
              final name = nameController.text.trim();
              final callbackUrl = callbackUrlController.text.trim();
              final description = descriptionController.text.trim();

              if (name.isEmpty || callbackUrl.isEmpty) return;

              await ref.read(oauthActionsProvider).createApp(
                    name: name,
                    callbackUrl: callbackUrl,
                    description: description.isEmpty ? null : description,
                  );

              if (context.mounted) {
                Navigator.of(context).pop();
              }
            },
            child: const Text('作成'),
          ),
        ],
      ),
    );
  }
}
