import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/antennas.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/antenna.dart';

final antennasProvider = FutureProvider.family<List<Antenna>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final antennasEndpoints = AntennasEndpoints(apiClient);
  return await antennasEndpoints.getAntennas();
});

final antennasActionsProvider = Provider<AntennasActions>((ref) {
  return AntennasActions(ref);
});

class AntennasActions {
  final Ref ref;

  AntennasActions(this.ref);

  Future<Antenna> createAntenna({
    required String name,
    required List<String> keywords,
    required List<String> users,
    required List<String> instances,
    bool? caseSensitive,
    bool? withReplies,
    bool? withFile,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final antennasEndpoints = AntennasEndpoints(apiClient);
    final antenna = await antennasEndpoints.createAntenna(
      name: name,
      keywords: keywords,
      users: users,
      instances: instances,
      caseSensitive: caseSensitive,
      withReplies: withReplies,
      withFile: withFile,
    );
    ref.invalidate(antennasProvider);
    return antenna;
  }

  Future<Antenna> updateAntenna(
    String id, {
    String? name,
    List<String>? keywords,
    List<String>? users,
    List<String>? instances,
    bool? caseSensitive,
    bool? withReplies,
    bool? withFile,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final antennasEndpoints = AntennasEndpoints(apiClient);
    final antenna = await antennasEndpoints.updateAntenna(
      id,
      name: name,
      keywords: keywords,
      users: users,
      instances: instances,
      caseSensitive: caseSensitive,
      withReplies: withReplies,
      withFile: withFile,
    );
    ref.invalidate(antennasProvider);
    return antenna;
  }

  Future<void> deleteAntenna(String id) async {
    final apiClient = ref.read(apiClientProvider);
    final antennasEndpoints = AntennasEndpoints(apiClient);
    await antennasEndpoints.deleteAntenna(id);
    ref.invalidate(antennasProvider);
  }
}
