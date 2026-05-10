import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/follow_requests.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/user.dart';

final followRequestsProvider = FutureProvider.family<List<User>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final followRequestsEndpoints = FollowRequestsEndpoints(apiClient);
  return await followRequestsEndpoints.getFollowRequests();
});

final followRequestsActionsProvider = Provider<FollowRequestsActions>((ref) {
  return FollowRequestsActions(ref);
});

class FollowRequestsActions {
  final Ref ref;

  FollowRequestsActions(this.ref);

  Future<User> accept(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final followRequestsEndpoints = FollowRequestsEndpoints(apiClient);
    final user = await followRequestsEndpoints.acceptFollowRequest(userId);
    ref.invalidate(followRequestsProvider);
    return user;
  }

  Future<User> reject(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final followRequestsEndpoints = FollowRequestsEndpoints(apiClient);
    final user = await followRequestsEndpoints.rejectFollowRequest(userId);
    ref.invalidate(followRequestsProvider);
    return user;
  }
}
