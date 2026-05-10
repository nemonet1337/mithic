import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/user.dart';
import 'package:mithic/models/relationship.dart';
import 'package:mithic/models/note.dart';

class AccountsEndpoints {
  final ApiClient _client;

  AccountsEndpoints(this._client);

  Future<User> getAccount(String id) async {
    final response = await _client.get('/api/v1/accounts/$id');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> followAccount(String id) async {
    final response = await _client.post('/api/v1/accounts/$id/follow');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> unfollowAccount(String id) async {
    final response = await _client.post('/api/v1/accounts/$id/unfollow');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> blockAccount(String id) async {
    final response = await _client.post('/api/v1/accounts/$id/block');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> unblockAccount(String id) async {
    final response = await _client.post('/api/v1/accounts/$id/unblock');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> muteAccount(String id) async {
    final response = await _client.post('/api/v1/accounts/$id/mute');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> unmuteAccount(String id) async {
    final response = await _client.post('/api/v1/accounts/$id/unmute');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> updateCredentials({
    String? displayName,
    String? bio,
    String? avatar,
  }) async {
    final response = await _client.put(
      '/api/v1/accounts/update_credentials',
      data: {
        if (displayName != null) 'display_name': displayName,
        if (bio != null) 'bio': bio,
        if (avatar != null) 'avatar': avatar,
      },
    );
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<List<Relationship>> getRelations({
    List<String>? ids,
  }) async {
    final response = await _client.get(
      '/api/v1/accounts/relations',
      queryParameters: {
        if (ids != null) 'ids': ids,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Relationship.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<User>> getFollowers(String id, {
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/accounts/$id/followers',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => User.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<User>> getFollowing(String id, {
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/accounts/$id/following',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => User.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<List<Note>> getUserStatuses(String id, {
    int? limit,
    String? sinceId,
    String? untilId,
  }) async {
    final response = await _client.get(
      '/api/v1/accounts/$id/statuses',
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
