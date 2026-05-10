import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/lists/providers/lists_provider.dart';
import 'package:mithic/models/user_list.dart';

class ListsScreen extends ConsumerWidget {
  const ListsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final listsAsync = ref.watch(listsProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('リスト'),
        actions: [
          IconButton(
            onPressed: () {
              _showListDialog(context, ref);
            },
            icon: const Icon(Icons.add),
          ),
        ],
      ),
      body: listsAsync.when(
        data: (lists) {
          if (lists.isEmpty) {
            return const Center(
              child: Text('リストがありません'),
            );
          }
          return ListView.builder(
            itemCount: lists.length,
            itemBuilder: (context, index) {
              final list = lists[index];
              return ListTile(
                title: Text(list.title),
                subtitle: list.createdAt != null
                    ? Text('${list.createdAt}')
                    : null,
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      onPressed: () {
                        _showListDialog(context, ref, list: list);
                      },
                      icon: const Icon(Icons.edit),
                    ),
                    IconButton(
                      onPressed: () async {
                        await ref.read(listsActionsProvider).deleteList(list.id);
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

  void _showListDialog(BuildContext context, WidgetRef ref, {UserList? list}) {
    final titleController = TextEditingController(text: list?.title ?? '');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(list == null ? 'リストを作成' : 'リストを編集'),
        content: TextField(
          controller: titleController,
          decoration: const InputDecoration(
            labelText: 'タイトル',
            hintText: 'リスト名',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          TextButton(
            onPressed: () async {
              final title = titleController.text.trim();
              if (title.isEmpty) return;

              if (list == null) {
                await ref.read(listsActionsProvider).createList(title: title);
              } else {
                await ref.read(listsActionsProvider).updateList(list.id, title: title);
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
