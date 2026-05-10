import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/note.dart';

class BookmarksEndpoints {
  final ApiClient _client;

  BookmarksEndpoints(this._client);

  Future<List<Note>> getBookmarks({
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/bookmarks',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<Note> bookmarkNote(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/bookmark');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> unbookmarkNote(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/unbookmark');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }
}
