import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/filters/providers/filters_provider.dart';
import 'package:mithic/models/filter.dart';

class FiltersScreen extends ConsumerWidget {
  const FiltersScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final filtersAsync = ref.watch(filtersProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('フィルター'),
        actions: [
          IconButton(
            onPressed: () {
              _showFilterDialog(context, ref);
            },
            icon: const Icon(Icons.add),
          ),
        ],
      ),
      body: filtersAsync.when(
        data: (filters) {
          if (filters.isEmpty) {
            return const Center(
              child: Text('フィルターがありません'),
            );
          }
          return ListView.builder(
            itemCount: filters.length,
            itemBuilder: (context, index) {
              final filter = filters[index];
              return ListTile(
                title: Text(filter.phrase),
                subtitle: Text('コンテキスト: ${_getContextLabel(filter.context)}'),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      onPressed: () {
                        _showFilterDialog(context, ref, filter: filter);
                      },
                      icon: const Icon(Icons.edit),
                    ),
                    IconButton(
                      onPressed: () async {
                        await ref.read(filtersActionsProvider).deleteFilter(filter.id);
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

  String _getContextLabel(FilterContext context) {
    switch (context) {
      case FilterContext.home:
        return 'ホーム';
      case FilterContext.notifications:
        return '通知';
      case FilterContext.public:
        return '公開';
      case FilterContext.thread:
        return 'スレッド';
    }
  }

  void _showFilterDialog(BuildContext context, WidgetRef ref, {Filter? filter}) {
    final phraseController = TextEditingController(text: filter?.phrase ?? '');
    FilterContext selectedContext = filter?.context ?? FilterContext.home;

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) {
          return AlertDialog(
            title: Text(filter == null ? 'フィルターを作成' : 'フィルターを編集'),
            content: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: phraseController,
                  decoration: const InputDecoration(
                    labelText: 'フレーズ',
                    hintText: 'フィルターする語句',
                  ),
                ),
                const SizedBox(height: 16),
                const Text('コンテキスト'),
                ...FilterContext.values.map((context) {
                  return RadioListTile<FilterContext>(
                    title: Text(_getContextLabel(context)),
                    value: context,
                    groupValue: selectedContext,
                    onChanged: (value) {
                      if (value != null) {
                        setState(() {
                          selectedContext = value;
                        });
                      }
                    },
                  );
                }),
              ],
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('キャンセル'),
              ),
              TextButton(
                onPressed: () async {
                  final phrase = phraseController.text.trim();
                  if (phrase.isEmpty) return;

                  if (filter == null) {
                    await ref.read(filtersActionsProvider).createFilter(
                          phrase: phrase,
                          context: selectedContext,
                        );
                  } else {
                    await ref.read(filtersActionsProvider).updateFilter(
                          filter.id,
                          phrase: phrase,
                          context: selectedContext,
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
