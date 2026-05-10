import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/accounts.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/user.dart';

final mutesProvider = FutureProvider.family<List<User>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  // Note: The Mastodon API doesn't have a direct endpoint to list muted users
  // This might need to be implemented differently or use a different API
  // For now, returning empty list
  return [];
});

final mutesActionsProvider = Provider<MutesActions>((ref) {
  return MutesActions(ref);
});

class MutesActions {
  final Ref ref;

  MutesActions(this.ref);

  Future<User> mute(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final accountsEndpoints = AccountsEndpoints(apiClient);
    final user = await accountsEndpoints.muteAccount(userId);
    ref.invalidate(mutesProvider);
    return user;
  }

  Future<User> unmute(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final accountsEndpoints = AccountsEndpoints(apiClient);
    final user = await accountsEndpoints.unmuteAccount(userId);
    ref.invalidate(mutesProvider);
    return user;
  }
}
