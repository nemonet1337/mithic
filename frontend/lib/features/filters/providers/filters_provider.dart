import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/filters.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/filter.dart';

final filtersProvider = FutureProvider.family<List<Filter>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final filtersEndpoints = FiltersEndpoints(apiClient);
  return await filtersEndpoints.getFilters();
});

final filtersActionsProvider = Provider<FiltersActions>((ref) {
  return FiltersActions(ref);
});

class FiltersActions {
  final Ref ref;

  FiltersActions(this.ref);

  Future<Filter> createFilter({
    required String phrase,
    required FilterContext context,
    bool? irreversible,
    bool? wholeWord,
    int? expiresIn,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final filtersEndpoints = FiltersEndpoints(apiClient);
    final filter = await filtersEndpoints.createFilter(
      phrase: phrase,
      context: context,
      irreversible: irreversible,
      wholeWord: wholeWord,
      expiresIn: expiresIn,
    );
    ref.invalidate(filtersProvider);
    return filter;
  }

  Future<Filter> updateFilter(
    String id, {
    String? phrase,
    FilterContext? context,
    bool? irreversible,
    bool? wholeWord,
    int? expiresIn,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final filtersEndpoints = FiltersEndpoints(apiClient);
    final filter = await filtersEndpoints.updateFilter(
      id,
      phrase: phrase,
      context: context,
      irreversible: irreversible,
      wholeWord: wholeWord,
      expiresIn: expiresIn,
    );
    ref.invalidate(filtersProvider);
    return filter;
  }

  Future<void> deleteFilter(String id) async {
    final apiClient = ref.read(apiClientProvider);
    final filtersEndpoints = FiltersEndpoints(apiClient);
    await filtersEndpoints.deleteFilter(id);
    ref.invalidate(filtersProvider);
  }
}
