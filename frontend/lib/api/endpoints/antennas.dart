import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/antenna.dart';

class AntennasEndpoints {
  final ApiClient _client;

  AntennasEndpoints(this._client);

  Future<List<Antenna>> getAntennas() async {
    final response = await _client.get('/api/v1/antennas');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Antenna.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<Antenna> createAntenna({
    required String name,
    required List<String> keywords,
    required List<String> users,
    required List<String> instances,
    bool? caseSensitive,
    bool? withReplies,
    bool? withFile,
  }) async {
    final response = await _client.post(
      '/api/v1/antennas',
      data: {
        'name': name,
        'keywords': keywords,
        'users': users,
        'instances': instances,
        if (caseSensitive != null) 'case_sensitive': caseSensitive,
        if (withReplies != null) 'with_replies': withReplies,
        if (withFile != null) 'with_file': withFile,
      },
    );
    return Antenna.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Antenna> updateAntenna(
    String id, {
    String? name,
    List<String>? keywords,
    List<String>? users,
    List<String>? instances,
    bool? caseSensitive,
    bool? withReplies,
    bool? withFile,
  }) async {
    final response = await _client.put(
      '/api/v1/antennas/$id',
      data: {
        if (name != null) 'name': name,
        if (keywords != null) 'keywords': keywords,
        if (users != null) 'users': users,
        if (instances != null) 'instances': instances,
        if (caseSensitive != null) 'case_sensitive': caseSensitive,
        if (withReplies != null) 'with_replies': withReplies,
        if (withFile != null) 'with_file': withFile,
      },
    );
    return Antenna.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteAntenna(String id) async {
    await _client.delete('/api/v1/antennas/$id');
  }
}
