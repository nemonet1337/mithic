import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/note.dart';

class StatusesEndpoints {
  final ApiClient _client;

  StatusesEndpoints(this._client);

  Future<Note> createStatus({
    required String text,
    String? inReplyToId,
    String? cw,
    bool localOnly = false,
    String? visibility,
    List<String>? fileIds,
    PollData? poll,
  }) async {
    final response = await _client.post(
      '/api/v1/statuses',
      data: {
        'text': text,
        if (inReplyToId != null) 'in_reply_to_id': inReplyToId,
        if (cw != null) 'cw': cw,
        'local_only': localOnly,
        if (visibility != null) 'visibility': visibility,
        if (fileIds != null) 'file_ids': fileIds,
        if (poll != null) 'poll': poll.toJson(),
      },
    );
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> getStatus(String id) async {
    final response = await _client.get('/api/v1/statuses/$id');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteStatus(String id) async {
    await _client.delete('/api/v1/statuses/$id');
  }

  Future<Note> favouriteStatus(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/favourite');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> unfavouriteStatus(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/unfavourite');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> reblogStatus(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/reblog');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> unreblogStatus(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/unreblog');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> reactNote(String id, String emoji) async {
    final response = await _client.post(
      '/api/v1/statuses/$id/react',
      data: {'emoji': emoji},
    );
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> unreactNote(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/unreact');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<List<Map<String, dynamic>>> getReactions(String id) async {
    final response = await _client.get('/api/v1/statuses/$id/reactions');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => json as Map<String, dynamic>).toList();
  }

  Future<Map<String, dynamic>> getNoteState(String id) async {
    final response = await _client.get('/api/v1/statuses/$id/state');
    return response.data as Map<String, dynamic>;
  }

  Future<List<Note>> getFavorites({
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/favorites',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> getConversation(String id) async {
    final response = await _client.get('/api/v1/notes/$id/conversation');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> getReplies(String id) async {
    final response = await _client.get('/api/v1/notes/$id/replies');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> getRenotes(String id) async {
    final response = await _client.get('/api/v1/notes/$id/renotes');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> getMentions({
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/notes/mentions',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> getChildren(String id) async {
    final response = await _client.get('/api/v1/notes/$id/children');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<Note> pinNote(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/pin');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Note> unpinNote(String id) async {
    final response = await _client.post('/api/v1/statuses/$id/unpin');
    return Note.fromJson(response.data as Map<String, dynamic>);
  }
}

class PollData {
  final List<String> choices;
  final bool multiple;
  final int? expiresAfter;

  PollData({
    required this.choices,
    this.multiple = false,
    this.expiresAfter,
  });

  Map<String, dynamic> toJson() {
    return {
      'choices': choices,
      'multiple': multiple,
      if (expiresAfter != null) 'expires_after': expiresAfter,
    };
  }
}
