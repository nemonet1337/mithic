import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/filter.dart';

class FiltersEndpoints {
  final ApiClient _client;

  FiltersEndpoints(this._client);

  Future<List<Filter>> getFilters() async {
    final response = await _client.get('/api/v1/filters');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Filter.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<Filter> createFilter({
    required String phrase,
    required FilterContext context,
    bool? irreversible,
    bool? wholeWord,
    int? expiresIn,
  }) async {
    final response = await _client.post(
      '/api/v1/filters',
      data: {
        'phrase': phrase,
        'context': context.name,
        if (irreversible != null) 'irreversible': irreversible,
        if (wholeWord != null) 'whole_word': wholeWord,
        if (expiresIn != null) 'expires_in': expiresIn,
      },
    );
    return Filter.fromJson(response.data as Map<String, dynamic>);
  }

  Future<Filter> updateFilter(
    String id, {
    String? phrase,
    FilterContext? context,
    bool? irreversible,
    bool? wholeWord,
    int? expiresIn,
  }) async {
    final response = await _client.put(
      '/api/v1/filters/$id',
      data: {
        if (phrase != null) 'phrase': phrase,
        if (context != null) 'context': context.name,
        if (irreversible != null) 'irreversible': irreversible,
        if (wholeWord != null) 'whole_word': wholeWord,
        if (expiresIn != null) 'expires_in': expiresIn,
      },
    );
    return Filter.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteFilter(String id) async {
    await _client.delete('/api/v1/filters/$id');
  }
}
