import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/favorites/providers/favorites_provider.dart';
import 'package:mithic/shared/widgets/note_card.dart';

class FavoritesScreen extends ConsumerWidget {
  const FavoritesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final favoritesAsync = ref.watch(favoritesProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('お気に入り'),
      ),
      body: favoritesAsync.when(
        data: (notes) {
          if (notes.isEmpty) {
            return const Center(
              child: Text('お気に入りがありません'),
            );
          }
          return ListView.builder(
            itemCount: notes.length,
            itemBuilder: (context, index) {
              return NoteCard(note: notes[index]);
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
