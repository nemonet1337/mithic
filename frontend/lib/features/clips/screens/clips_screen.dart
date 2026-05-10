import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/clips/providers/clips_provider.dart';
import 'package:mithic/models/clip.dart';

class ClipsScreen extends ConsumerWidget {
  const ClipsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final clipsAsync = ref.watch(clipsProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('クリップ'),
        actions: [
          IconButton(
            onPressed: () {
              _showClipDialog(context, ref);
            },
            icon: const Icon(Icons.add),
          ),
        ],
      ),
      body: clipsAsync.when(
        data: (clips) {
          if (clips.isEmpty) {
            return const Center(
              child: Text('クリップがありません'),
            );
          }
          return ListView.builder(
            itemCount: clips.length,
            itemBuilder: (context, index) {
              final clip = clips[index];
              return ListTile(
                title: Text(clip.name),
                subtitle: clip.description != null
                    ? Text(clip.description!)
                    : null,
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      onPressed: () {
                        _showClipDialog(context, ref, clip: clip);
                      },
                      icon: const Icon(Icons.edit),
                    ),
                    IconButton(
                      onPressed: () async {
                        await ref.read(clipsActionsProvider).deleteClip(clip.id);
                      },
                      icon: const Icon(Icons.delete),
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

  void _showClipDialog(BuildContext context, WidgetRef ref, {Clip? clip}) {
    final nameController = TextEditingController(text: clip?.name ?? '');
    final descriptionController = TextEditingController(text: clip?.description ?? '');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(clip == null ? 'クリップを作成' : 'クリップを編集'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameController,
              decoration: const InputDecoration(
                labelText: '名前',
                hintText: 'クリップ名',
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
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          TextButton(
            onPressed: () async {
              final name = nameController.text.trim();
              final description = descriptionController.text.trim();

              if (name.isEmpty) return;

              if (clip == null) {
                await ref.read(clipsActionsProvider).createClip(
                      name: name,
                      description: description.isEmpty ? null : description,
                    );
              } else {
                await ref.read(clipsActionsProvider).updateClip(
                      clip.id,
                      name: name,
                      description: description.isEmpty ? null : description,
                    );
              }

              if (context.mounted) {
                Navigator.of(context).pop();
              }
            },
            child: const Text('保存'),
          ),
        ],
      ),
    );
  }
}
