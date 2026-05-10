import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/user.dart';

class AuthEndpoints {
  final ApiClient _client;

  AuthEndpoints(this._client);

  Future<Map<String, dynamic>> signin({
    required String username,
    required String password,
  }) async {
    final response = await _client.post(
      '/api/v1/signin',
      data: {
        'username': username,
        'password': password,
      },
    );
    return response.data as Map<String, dynamic>;
  }

  Future<Map<String, dynamic>> signup({
    required String username,
    required String password,
    String? email,
  }) async {
    final response = await _client.post(
      '/api/v1/signup',
      data: {
        'username': username,
        'password': password,
        if (email != null) 'email': email,
      },
    );
    return response.data as Map<String, dynamic>;
  }

  Future<User> verifyCredentials() async {
    final response = await _client.get('/api/v1/accounts/verify_credentials');
    return User.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Map<String, dynamic>> getI() async {
    final response = await _client.post('/api/v1/i');
    return response.data as Map<String, dynamic>;
  }
}
