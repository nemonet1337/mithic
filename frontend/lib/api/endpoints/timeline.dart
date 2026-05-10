import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/note.dart';

class TimelineEndpoints {
  final ApiClient _client;

  TimelineEndpoints(this._client);

  Future<List<Note>> homeTimeline({
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/timelines/home',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> localTimeline({
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/timelines/local',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> globalTimeline({
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/timelines/global',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> hybridTimeline({
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/timelines/hybrid',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> listTimeline(
    String listId, {
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/timelines/list/$listId',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> hashtagTimeline(
    String tag, {
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/timelines/tag/$tag',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Note.fromJson(json as Map<String, dynamic>)).toList();
  }
}
