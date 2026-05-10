import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/statuses.dart';
import 'package:mithic/models/note.dart';

final noteActionsProvider = Provider<NoteActions>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return NoteActions(apiClient);
});

class NoteActions {
  final ApiClient _client;

  NoteActions(this._client);

  Future<Note> reply(String noteId, String text) async {
    final endpoints = StatusesEndpoints(_client);
    return await endpoints.createStatus(
      text: text,
      inReplyToId: noteId,
    );
  }

  Future<Note> renote(String noteId) async {
    final endpoints = StatusesEndpoints(_client);
    return await endpoints.reblogStatus(noteId);
  }

  Future<Note> unrenote(String noteId) async {
    final endpoints = StatusesEndpoints(_client);
    return await endpoints.unreblogStatus(noteId);
  }

  Future<Note> favourite(String noteId) async {
    final endpoints = StatusesEndpoints(_client);
    return await endpoints.favouriteStatus(noteId);
  }

  Future<Note> unfavourite(String noteId) async {
    final endpoints = StatusesEndpoints(_client);
    return await endpoints.unfavouriteStatus(noteId);
  }

  Future<Note> react(String noteId, String emoji) async {
    final endpoints = StatusesEndpoints(_client);
    return await endpoints.reactNote(noteId, emoji);
  }

  Future<Note> unreact(String noteId) async {
    final endpoints = StatusesEndpoints(_client);
    return await endpoints.unreactNote(noteId);
  }
}
