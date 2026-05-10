import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/statuses.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/note.dart';

final favoritesProvider = FutureProvider.family<List<Note>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final statusesEndpoints = StatusesEndpoints(apiClient);
  return await statusesEndpoints.getFavorites();
});

final favoritesActionsProvider = Provider<FavoritesActions>((ref) {
  return FavoritesActions(ref);
});

class FavoritesActions {
  final Ref ref;

  FavoritesActions(this.ref);

  Future<Note> favourite(String noteId) async {
    final apiClient = ref.read(apiClientProvider);
    final statusesEndpoints = StatusesEndpoints(apiClient);
    final note = await statusesEndpoints.favouriteStatus(noteId);
    ref.invalidate(favoritesProvider);
    return note;
  }

  Future<Note> unfavourite(String noteId) async {
    final apiClient = ref.read(apiClientProvider);
    final statusesEndpoints = StatusesEndpoints(apiClient);
    final note = await statusesEndpoints.unfavouriteStatus(noteId);
    ref.invalidate(favoritesProvider);
    return note;
  }
}
