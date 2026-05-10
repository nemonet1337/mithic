import 'package:mithic/api/client/dio_client.dart';

class TwoFactorEndpoints {
  final ApiClient _client;

  TwoFactorEndpoints(this._client);

  Future<Map<String, dynamic>> registerTwoFactor() async {
    final response = await _client.post('/api/v1/2fa/register');
    return response.data as Map<String, dynamic>;
  }

  Future<void> enableTwoFactor(String token) async {
    await _client.post('/api/v1/2fa/enable', data: {'token': token});
  }

  Future<void> disableTwoFactor(String password) async {
    await _client.post('/api/v1/2fa/disable', data: {'password': password});
  }

  Future<Map<String, dynamic>> getTwoFactorStatus() async {
    final response = await _client.get('/api/v1/2fa/status');
    return response.data as Map<String, dynamic>;
  }
}
