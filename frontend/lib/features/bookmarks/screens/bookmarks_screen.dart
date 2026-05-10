import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/bookmarks/providers/bookmarks_provider.dart';
import 'package:mithic/shared/widgets/note_card.dart';

class BookmarksScreen extends ConsumerWidget {
  const BookmarksScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final bookmarksAsync = ref.watch(bookmarksProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('ブックマーク'),
      ),
      body: bookmarksAsync.when(
        data: (notes) {
          if (notes.isEmpty) {
            return const Center(
              child: Text('ブックマークがありません'),
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
