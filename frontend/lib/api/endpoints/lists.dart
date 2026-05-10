import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/user_list.dart';

class ListsEndpoints {
  final ApiClient _client;

  ListsEndpoints(this._client);

  Future<List<UserList>> getLists() async {
    final response = await _client.get('/api/v1/lists');
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => UserList.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<UserList> createList({
    required String title,
  }) async {
    final response = await _client.post(
      '/api/v1/lists',
      data: {
        'title': title,
      },
    );
    return UserList.fromJson(response.data as Map<String, dynamic>);
  }

  Future<UserList> updateList(
    String id, {
    String? title,
  }) async {
    final response = await _client.put(
      '/api/v1/lists/$id',
      data: {
        if (title != null) 'title': title,
      },
    );
    return UserList.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteList(String id) async {
    await _client.delete('/api/v1/lists/$id');
  }

  Future<void> addAccountToList(String listId, String accountId) async {
    await _client.post('/api/v1/lists/$listId/accounts', data: {
      'account_ids': [accountId],
    });
  }

  Future<void> removeAccountFromList(String listId, String accountId) async {
    await _client.delete('/api/v1/lists/$listId/accounts/$accountId');
  }
}
