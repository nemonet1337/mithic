import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/accounts.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/user.dart';

final blocksProvider = FutureProvider.family<List<User>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  // Note: The Mastodon API doesn't have a direct endpoint to list blocked users
  // This might need to be implemented differently or use a different API
  // For now, returning empty list
  return [];
});

final blocksActionsProvider = Provider<BlocksActions>((ref) {
  return BlocksActions(ref);
});

class BlocksActions {
  final Ref ref;

  BlocksActions(this.ref);

  Future<User> block(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final accountsEndpoints = AccountsEndpoints(apiClient);
    final user = await accountsEndpoints.blockAccount(userId);
    ref.invalidate(blocksProvider);
    return user;
  }

  Future<User> unblock(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final accountsEndpoints = AccountsEndpoints(apiClient);
    final user = await accountsEndpoints.unblockAccount(userId);
    ref.invalidate(blocksProvider);
    return user;
  }
}
