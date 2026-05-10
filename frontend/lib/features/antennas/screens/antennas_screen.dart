import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/antennas/providers/antennas_provider.dart';
import 'package:mithic/models/antenna.dart';

class AntennasScreen extends ConsumerWidget {
  const AntennasScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final antennasAsync = ref.watch(antennasProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('アンテナ'),
        actions: [
          IconButton(
            onPressed: () {
              _showAntennaDialog(context, ref);
            },
            icon: const Icon(Icons.add),
          ),
        ],
      ),
      body: antennasAsync.when(
        data: (antennas) {
          if (antennas.isEmpty) {
            return const Center(
              child: Text('アンテナがありません'),
            );
          }
          return ListView.builder(
            itemCount: antennas.length,
            itemBuilder: (context, index) {
              final antenna = antennas[index];
              return ListTile(
                title: Text(antenna.name),
                subtitle: Text(
                  'キーワード: ${antenna.keywords.join(", ")}',
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      onPressed: () {
                        _showAntennaDialog(context, ref, antenna: antenna);
                      },
                      icon: const Icon(Icons.edit),
                    ),
                    IconButton(
                      onPressed: () async {
                        await ref.read(antennasActionsProvider).deleteAntenna(antenna.id);
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

  void _showAntennaDialog(BuildContext context, WidgetRef ref, {Antenna? antenna}) {
    final nameController = TextEditingController(text: antenna?.name ?? '');
    final keywordsController = TextEditingController(
      text: antenna?.keywords.join(', ') ?? '',
    );
    final usersController = TextEditingController(
      text: antenna?.users.join(', ') ?? '',
    );
    final instancesController = TextEditingController(
      text: antenna?.instances.join(', ') ?? '',
    );
    bool caseSensitive = antenna?.caseSensitive ?? false;
    bool withReplies = antenna?.withReplies ?? false;
    bool withFile = antenna?.withFile ?? false;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) {
          return AlertDialog(
            title: Text(antenna == null ? 'アンテナを作成' : 'アンテナを編集'),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  TextField(
                    controller: nameController,
                    decoration: const InputDecoration(
                      labelText: '名前',
                      hintText: 'アンテナ名',
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: keywordsController,
                    decoration: const InputDecoration(
                      labelText: 'キーワード',
                      hintText: 'カンマ区切り',
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: usersController,
                    decoration: const InputDecoration(
                      labelText: 'ユーザー',
                      hintText: 'カンマ区切り',
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: instancesController,
                    decoration: const InputDecoration(
                      labelText: 'インスタンス',
                      hintText: 'カンマ区切り',
                    ),
                  ),
                  const SizedBox(height: 16),
                  CheckboxListTile(
                    title: const Text('大文字小文字を区別'),
                    value: caseSensitive,
                    onChanged: (value) {
                      setState(() {
                        caseSensitive = value ?? false;
                      });
                    },
                  ),
                  CheckboxListTile(
                    title: const Text('返信を含める'),
                    value: withReplies,
                    onChanged: (value) {
                      setState(() {
                        withReplies = value ?? false;
                      });
                    },
                  ),
                  CheckboxListTile(
                    title: const Text('ファイルを含める'),
                    value: withFile,
                    onChanged: (value) {
                      setState(() {
                        withFile = value ?? false;
                      });
                    },
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
                  final keywords = keywordsController.text
                      .split(',')
                      .map((e) => e.trim())
                      .where((e) => e.isNotEmpty)
                      .toList();
                  final users = usersController.text
                      .split(',')
                      .map((e) => e.trim())
                      .where((e) => e.isNotEmpty)
                      .toList();
                  final instances = instancesController.text
                      .split(',')
                      .map((e) => e.trim())
                      .where((e) => e.isNotEmpty)
                      .toList();

                  if (name.isEmpty) return;

                  if (antenna == null) {
                    await ref.read(antennasActionsProvider).createAntenna(
                          name: name,
                          keywords: keywords,
                          users: users,
                          instances: instances,
                          caseSensitive: caseSensitive,
                          withReplies: withReplies,
                          withFile: withFile,
                        );
                  } else {
                    await ref.read(antennasActionsProvider).updateAntenna(
                          antenna.id,
                          name: name,
                          keywords: keywords,
                          users: users,
                          instances: instances,
                          caseSensitive: caseSensitive,
                          withReplies: withReplies,
                          withFile: withFile,
                        );
                  }

                  if (context.mounted) {
                    Navigator.of(context).pop();
                  }
                },
                child: const Text('保存'),
              ),
            ],
          );
        },
      ),
    );
  }
}
