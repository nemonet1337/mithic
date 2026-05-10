import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/relay.dart';

class RelaysEndpoints {
  final ApiClient _client;

  RelaysEndpoints(this._client);

  Future<List<Relay>> getRelays() async {
    final response = await _client.get('/api/v1/admin/relays/list');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Relay.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<Relay> addRelay(String inboxUrl) async {
    final response = await _client.post(
      '/api/v1/admin/relays/add',
      data: {'inbox': inboxUrl},
    );
    return Relay.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> removeRelay(String relayId) async {
    await _client.post(
      '/api/v1/admin/relays/remove',
      data: {'relayId': relayId},
    );
  }

  Future<List<Relay>> getAcceptedRelays() async {
    final response = await _client.get('/api/v1/relays');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Relay.fromJson(json as Map<String, dynamic>)).toList();
  }
}
