import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/federation.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/federation_instance.dart';

final federationInstancesProvider = FutureProvider.family<List<FederationInstance>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final federationEndpoints = FederationEndpoints(apiClient);
  return await federationEndpoints.getInstances();
});

final federationActionsProvider = Provider<FederationActions>((ref) {
  return FederationActions(ref);
});

class FederationActions {
  final Ref ref;

  FederationActions(this.ref);

  Future<void> blockInstance(String host) async {
    final apiClient = ref.read(apiClientProvider);
    final federationEndpoints = FederationEndpoints(apiClient);
    await federationEndpoints.blockInstance(host);
    ref.invalidate(federationInstancesProvider);
  }

  Future<void> unblockInstance(String host) async {
    final apiClient = ref.read(apiClientProvider);
    final federationEndpoints = FederationEndpoints(apiClient);
    await federationEndpoints.unblockInstance(host);
    ref.invalidate(federationInstancesProvider);
  }

  Future<void> muteInstance(String host) async {
    final apiClient = ref.read(apiClientProvider);
    final federationEndpoints = FederationEndpoints(apiClient);
    await federationEndpoints.muteInstance(host);
    ref.invalidate(federationInstancesProvider);
  }

  Future<void> unmuteInstance(String host) async {
    final apiClient = ref.read(apiClientProvider);
    final federationEndpoints = FederationEndpoints(apiClient);
    await federationEndpoints.unmuteInstance(host);
    ref.invalidate(federationInstancesProvider);
  }
}
