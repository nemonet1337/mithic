import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/note.dart';
import 'package:mithic/models/user.dart';
import 'package:mithic/models/trend.dart';

class SearchEndpoints {
  final ApiClient _client;

  SearchEndpoints(this._client);

  Future<Map<String, dynamic>> search(String query, {
    String? type,
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/search',
      queryParameters: {
        'q': query,
        if (type != null) 'type': type,
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    return response.data as Map<String, dynamic>;
  }

  Future<List<Note>> searchNotes(String query, {
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/search',
      queryParameters: {
        'q': query,
        'type': 'statuses',
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data['statuses'] as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<User>> searchUsers(String query, {
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/search',
      queryParameters: {
        'q': query,
        'type': 'accounts',
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data['accounts'] as List<dynamic>;
    return data.map((json) => User.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<String>> searchHashtags(String query, {
    int? limit,
  }) async {
    final response = await _client.get(
      '/api/v1/search',
      queryParameters: {
        'q': query,
        'type': 'hashtags',
        if (limit != null) 'limit': limit,
      },
    );
    final List<dynamic> data = response.data['hashtags'] as List<dynamic>;
    return data.map((json) => json as String).toList();
  }

  Future<List<Trend>> getTrends() async {
    final response = await _client.get('/api/v1/trends');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Trend.fromJson(json as Map<String, dynamic>)).toList();
  }
}
