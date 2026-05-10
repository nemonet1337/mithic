import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/user.dart';

class FollowRequestsEndpoints {
  final ApiClient _client;

  FollowRequestsEndpoints(this._client);

  Future<List<User>> getFollowRequests() async {
    final response = await _client.get('/api/v1/follow_requests');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => User.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<User> acceptFollowRequest(String id) async {
    final response = await _client.post('/api/v1/follow_requests/$id/authorize');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<User> rejectFollowRequest(String id) async {
    final response = await _client.post('/api/v1/follow_requests/$id/reject');
    return User.fromJson(response.data as Map<String, dynamic>);
  }
}
