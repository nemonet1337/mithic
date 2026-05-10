import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/lists.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/user_list.dart';

final listsProvider = FutureProvider.family<List<UserList>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final listsEndpoints = ListsEndpoints(apiClient);
  return await listsEndpoints.getLists();
});

final listsActionsProvider = Provider<ListsActions>((ref) {
  return ListsActions(ref);
});

class ListsActions {
  final Ref ref;

  ListsActions(this.ref);

  Future<UserList> createList({
    required String title,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final listsEndpoints = ListsEndpoints(apiClient);
    final list = await listsEndpoints.createList(title: title);
    ref.invalidate(listsProvider);
    return list;
  }

  Future<UserList> updateList(
    String id, {
    String? title,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final listsEndpoints = ListsEndpoints(apiClient);
    final list = await listsEndpoints.updateList(id, title: title);
    ref.invalidate(listsProvider);
    return list;
  }

  Future<void> deleteList(String id) async {
    final apiClient = ref.read(apiClientProvider);
    final listsEndpoints = ListsEndpoints(apiClient);
    await listsEndpoints.deleteList(id);
    ref.invalidate(listsProvider);
  }

  Future<void> addAccountToList(String listId, String accountId) async {
    final apiClient = ref.read(apiClientProvider);
    final listsEndpoints = ListsEndpoints(apiClient);
    await listsEndpoints.addAccountToList(listId, accountId);
  }

  Future<void> removeAccountFromList(String listId, String accountId) async {
    final apiClient = ref.read(apiClientProvider);
    final listsEndpoints = ListsEndpoints(apiClient);
    await listsEndpoints.removeAccountFromList(listId, accountId);
  }
}
