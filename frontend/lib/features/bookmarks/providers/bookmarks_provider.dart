import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/bookmarks.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/note.dart';

final bookmarksProvider = FutureProvider.family<List<Note>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final bookmarksEndpoints = BookmarksEndpoints(apiClient);
  return await bookmarksEndpoints.getBookmarks();
});

final bookmarksActionsProvider = Provider<BookmarksActions>((ref) {
  return BookmarksActions(ref);
});

class BookmarksActions {
  final Ref ref;

  BookmarksActions(this.ref);

  Future<Note> bookmark(String noteId) async {
    final apiClient = ref.read(apiClientProvider);
    final bookmarksEndpoints = BookmarksEndpoints(apiClient);
    final note = await bookmarksEndpoints.bookmarkNote(noteId);
    ref.invalidate(bookmarksProvider);
    return note;
  }

  Future<Note> unbookmark(String noteId) async {
    final apiClient = ref.read(apiClientProvider);
    final bookmarksEndpoints = BookmarksEndpoints(apiClient);
    final note = await bookmarksEndpoints.unbookmarkNote(noteId);
    ref.invalidate(bookmarksProvider);
    return note;
  }
}
