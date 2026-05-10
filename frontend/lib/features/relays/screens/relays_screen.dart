import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/relays/providers/relays_provider.dart';

class RelaysScreen extends ConsumerWidget {
  const RelaysScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final relaysAsync = ref.watch(relaysProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('リレー'),
        actions: [
          IconButton(
            onPressed: () {
              _showAddRelayDialog(context, ref);
            },
            icon: const Icon(Icons.add),
          ),
        ],
      ),
      body: relaysAsync.when(
        data: (relays) {
          if (relays.isEmpty) {
            return const Center(
              child: Text('リレーがありません'),
            );
          }
          return ListView.builder(
            itemCount: relays.length,
            itemBuilder: (context, index) {
              final relay = relays[index];
              return ListTile(
                title: Text(relay.inbox),
                subtitle: relay.status != null
                    ? Text('状態: ${relay.status}')
                    : null,
                trailing: IconButton(
                  onPressed: () async {
                    await ref.read(relaysActionsProvider).removeRelay(relay.id);
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

  void _showAddRelayDialog(BuildContext context, WidgetRef ref) {
    final inboxController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('リレーを追加'),
        content: TextField(
          controller: inboxController,
          decoration: const InputDecoration(
            labelText: 'Inbox URL',
            hintText: 'https://example.com/inbox',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          TextButton(
            onPressed: () async {
              final inboxUrl = inboxController.text.trim();

              if (inboxUrl.isEmpty) return;

              await ref.read(relaysActionsProvider).addRelay(inboxUrl);

              if (context.mounted) {
                Navigator.of(context).pop();
              }
            },
            child: const Text('追加'),
          ),
        ],
      ),
    );
  }
}
