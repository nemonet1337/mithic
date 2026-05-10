import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/admin.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/user.dart';

final adminStatsProvider = FutureProvider.family<Map<String, dynamic>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return {};
  }

  final apiClient = ref.watch(apiClientProvider);
  final adminEndpoints = AdminEndpoints(apiClient);
  return await adminEndpoints.getStats();
});

final adminUsersProvider = FutureProvider.family<List<User>, AdminUsersParams>((ref, params) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final adminEndpoints = AdminEndpoints(apiClient);
  return await adminEndpoints.getUsers(
    limit: params.limit,
    offset: params.offset,
    sort: params.sort,
    state: params.state,
  );
});

final adminActionsProvider = Provider<AdminActions>((ref) {
  return AdminActions(ref);
});

class AdminActions {
  final Ref ref;

  AdminActions(this.ref);

  Future<void> suspendUser(String userId, String reason) async {
    final apiClient = ref.read(apiClientProvider);
    final adminEndpoints = AdminEndpoints(apiClient);
    await adminEndpoints.suspendUser(userId, reason);
    ref.invalidate(adminUsersProvider);
  }

  Future<void> unsuspendUser(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final adminEndpoints = AdminEndpoints(apiClient);
    await adminEndpoints.unsuspendUser(userId);
    ref.invalidate(adminUsersProvider);
  }

  Future<void> deleteUser(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final adminEndpoints = AdminEndpoints(apiClient);
    await adminEndpoints.deleteUser(userId);
    ref.invalidate(adminUsersProvider);
  }

  Future<void> deleteNote(String noteId) async {
    final apiClient = ref.read(apiClientProvider);
    final adminEndpoints = AdminEndpoints(apiClient);
    await adminEndpoints.deleteNote(noteId);
  }
}

class AdminUsersParams {
  final int limit;
  final int offset;
  final String? sort;
  final String? state;

  AdminUsersParams({
    this.limit = 10,
    this.offset = 0,
    this.sort,
    this.state,
  });
}
