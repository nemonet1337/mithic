import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/user.dart';
import 'package:mithic/models/note.dart';

class AdminEndpoints {
  final ApiClient _client;

  AdminEndpoints(this._client);

  Future<Map<String, dynamic>> getStats() async {
    final response = await _client.get('/api/v1/admin/stats');
    return response.data as Map<String, dynamic>;
  }

  Future<List<User>> getUsers({
    int limit = 10,
    int offset = 0,
    String? sort,
    String? state,
  }) async {
    final response = await _client.get(
      '/api/v1/admin/users',
      queryParameters: {
        'limit': limit,
        'offset': offset,
        if (sort != null) 'sort': sort,
        if (state != null) 'state': state,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => User.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<User> getUser(String userId) async {
    final response = await _client.get('/api/v1/admin/users/$userId');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> suspendUser(String userId, String reason) async {
    await _client.post(
      '/api/v1/admin/users/$userId/suspend',
      data: {'reason': reason},
    );
  }

  Future<void> unsuspendUser(String userId) async {
    await _client.post('/api/v1/admin/users/$userId/unsuspend');
  }

  Future<void> deleteUser(String userId) async {
    await _client.post('/api/v1/admin/users/$userId/delete');
  }

  Future<void> deleteNote(String noteId) async {
    await _client.post('/api/v1/admin/notes/$noteId/delete');
  }

  Future<List<Note>> getAbuseReports() async {
    final response = await _client.get('/api/v1/admin/reports');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }
}
