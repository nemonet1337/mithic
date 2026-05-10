import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/clips.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/clip.dart';

final clipsProvider = FutureProvider.family<List<Clip>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final clipsEndpoints = ClipsEndpoints(apiClient);
  return await clipsEndpoints.getClips();
});

final clipsActionsProvider = Provider<ClipsActions>((ref) {
  return ClipsActions(ref);
});

class ClipsActions {
  final Ref ref;

  ClipsActions(this.ref);

  Future<Clip> createClip({
    required String name,
    String? description,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final clipsEndpoints = ClipsEndpoints(apiClient);
    final clip = await clipsEndpoints.createClip(
      name: name,
      description: description,
    );
    ref.invalidate(clipsProvider);
    return clip;
  }

  Future<Clip> updateClip(
    String id, {
    String? name,
    String? description,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final clipsEndpoints = ClipsEndpoints(apiClient);
    final clip = await clipsEndpoints.updateClip(
      id,
      name: name,
      description: description,
    );
    ref.invalidate(clipsProvider);
    return clip;
  }

  Future<void> deleteClip(String id) async {
    final apiClient = ref.read(apiClientProvider);
    final clipsEndpoints = ClipsEndpoints(apiClient);
    await clipsEndpoints.deleteClip(id);
    ref.invalidate(clipsProvider);
  }

  Future<void> addNoteToClip(String clipId, String noteId) async {
    final apiClient = ref.read(apiClientProvider);
    final clipsEndpoints = ClipsEndpoints(apiClient);
    await clipsEndpoints.addNoteToClip(clipId, noteId);
  }

  Future<void> removeNoteFromClip(String clipId, String noteId) async {
    final apiClient = ref.read(apiClientProvider);
    final clipsEndpoints = ClipsEndpoints(apiClient);
    await clipsEndpoints.removeNoteFromClip(clipId, noteId);
  }
}
