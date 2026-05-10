import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/federation_instance.dart';

class FederationEndpoints {
  final ApiClient _client;

  FederationEndpoints(this._client);

  Future<List<FederationInstance>> getInstances() async {
    final response = await _client.get('/api/v1/federation/instances');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => FederationInstance.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<FederationInstance> getInstance(String host) async {
    final response = await _client.get('/api/v1/federation/instances/$host');
    return FederationInstance.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> blockInstance(String host) async {
    await _client.post('/api/v1/federation/instances/block', data: {'host': host});
  }

  Future<void> unblockInstance(String host) async {
    await _client.post('/api/v1/federation/instances/unblock', data: {'host': host});
  }

  Future<void> muteInstance(String host) async {
    await _client.post('/api/v1/federation/instances/mute', data: {'host': host});
  }

  Future<void> unmuteInstance(String host) async {
    await _client.post('/api/v1/federation/instances/unmute', data: {'host': host});
  }
}
