import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/two_factor.dart';
import 'package:mithic/core/config/app_config.dart';

final twoFactorStatusProvider = FutureProvider.family<Map<String, dynamic>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return {'enabled': false};
  }

  final apiClient = ref.watch(apiClientProvider);
  final twoFactorEndpoints = TwoFactorEndpoints(apiClient);
  return await twoFactorEndpoints.getTwoFactorStatus();
});

final twoFactorActionsProvider = Provider<TwoFactorActions>((ref) {
  return TwoFactorActions(ref);
});

class TwoFactorActions {
  final Ref ref;

  TwoFactorActions(this.ref);

  Future<Map<String, dynamic>> registerTwoFactor() async {
    final apiClient = ref.read(apiClientProvider);
    final twoFactorEndpoints = TwoFactorEndpoints(apiClient);
    return await twoFactorEndpoints.registerTwoFactor();
  }

  Future<void> enableTwoFactor(String token) async {
    final apiClient = ref.read(apiClientProvider);
    final twoFactorEndpoints = TwoFactorEndpoints(apiClient);
    await twoFactorEndpoints.enableTwoFactor(token);
    ref.invalidate(twoFactorStatusProvider);
  }

  Future<void> disableTwoFactor(String password) async {
    final apiClient = ref.read(apiClientProvider);
    final twoFactorEndpoints = TwoFactorEndpoints(apiClient);
    await twoFactorEndpoints.disableTwoFactor(password);
    ref.invalidate(twoFactorStatusProvider);
  }
}
