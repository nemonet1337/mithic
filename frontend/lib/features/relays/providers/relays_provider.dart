import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/relays.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/relay.dart';

final relaysProvider = FutureProvider.family<List<Relay>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final relaysEndpoints = RelaysEndpoints(apiClient);
  return await relaysEndpoints.getRelays();
});

final acceptedRelaysProvider = FutureProvider.family<List<Relay>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final relaysEndpoints = RelaysEndpoints(apiClient);
  return await relaysEndpoints.getAcceptedRelays();
});

final relaysActionsProvider = Provider<RelaysActions>((ref) {
  return RelaysActions(ref);
});

class RelaysActions {
  final Ref ref;

  RelaysActions(this.ref);

  Future<Relay> addRelay(String inboxUrl) async {
    final apiClient = ref.read(apiClientProvider);
    final relaysEndpoints = RelaysEndpoints(apiClient);
    final relay = await relaysEndpoints.addRelay(inboxUrl);
    ref.invalidate(relaysProvider);
    ref.invalidate(acceptedRelaysProvider);
    return relay;
  }

  Future<void> removeRelay(String relayId) async {
    final apiClient = ref.read(apiClientProvider);
    final relaysEndpoints = RelaysEndpoints(apiClient);
    await relaysEndpoints.removeRelay(relayId);
    ref.invalidate(relaysProvider);
    ref.invalidate(acceptedRelaysProvider);
  }
}
