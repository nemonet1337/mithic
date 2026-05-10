import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/clip.dart';

class ClipsEndpoints {
  final ApiClient _client;

  ClipsEndpoints(this._client);

  Future<List<Clip>> getClips() async {
    final response = await _client.get('/api/v1/clips');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Clip.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<Clip> createClip({
    required String name,
    String? description,
  }) async {
    final response = await _client.post(
      '/api/v1/clips',
      data: {
        'name': name,
        if (description != null) 'description': description,
      },
    );
    return Clip.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Clip> updateClip(
    String id, {
    String? name,
    String? description,
  }) async {
    final response = await _client.put(
      '/api/v1/clips/$id',
      data: {
        if (name != null) 'name': name,
        if (description != null) 'description': description,
      },
    );
    return Clip.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteClip(String id) async {
    await _client.delete('/api/v1/clips/$id');
  }

  Future<void> addNoteToClip(String clipId, String noteId) async {
    await _client.post('/api/v1/clips/$clipId/notes', data: {
      'note_id': noteId,
    });
  }

  Future<void> removeNoteFromClip(String clipId, String noteId) async {
    await _client.delete('/api/v1/clips/$clipId/notes/$noteId');
  }
}
