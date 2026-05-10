import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/oauth_app.dart';

class OAuthEndpoints {
  final ApiClient _client;

  OAuthEndpoints(this._client);

  Future<List<OAuthApp>> getApps() async {
    final response = await _client.get('/api/v1/apps');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => OAuthApp.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<OAuthApp> createApp({
    required String name,
    required String callbackUrl,
    String? description,
    List<String>? permissions,
  }) async {
    final response = await _client.post(
      '/api/v1/apps/create',
      data: {
        'name': name,
        'callbackUrl': callbackUrl,
        if (description != null) 'description': description,
        if (permissions != null) 'permission': permissions.join(','),
      },
    );
    return OAuthApp.fromJson(response.data as Map<String, dynamic>);
  }

  Future<OAuthApp> getApp(String appId) async {
    final response = await _client.get('/api/v1/apps/$appId');
    return OAuthApp.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteApp(String appId) async {
    await _client.post('/api/v1/apps/$appId/delete');
  }

  Future<String> generateAuthCode(String appId) async {
    final response = await _client.post('/api/v1/apps/$appId/generate-auth-code');
    return response.data['code'] as String;
  }
}
