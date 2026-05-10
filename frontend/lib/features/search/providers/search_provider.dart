import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/search.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/note.dart';
import 'package:mithic/models/user.dart';
import 'package:mithic/models/trend.dart';

enum SearchType {
  notes,
  users,
  hashtags,
}

final searchTypeProvider = StateProvider<SearchType>((ref) => SearchType.notes);

final searchQueryProvider = StateProvider<String>((ref) => '');

final searchNotesProvider = FutureProvider.family<List<Note>, String>((ref, query) async {
  if (AppConfig.isMockMode || query.isEmpty) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final searchEndpoints = SearchEndpoints(apiClient);
  return await searchEndpoints.searchNotes(query);
});

final searchUsersProvider = FutureProvider.family<List<User>, String>((ref, query) async {
  if (AppConfig.isMockMode || query.isEmpty) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final searchEndpoints = SearchEndpoints(apiClient);
  return await searchEndpoints.searchUsers(query);
});

final searchHashtagsProvider = FutureProvider.family<List<String>, String>((ref, query) async {
  if (AppConfig.isMockMode || query.isEmpty) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final searchEndpoints = SearchEndpoints(apiClient);
  return await searchEndpoints.searchHashtags(query);
});

final trendsProvider = FutureProvider<List<Trend>>((ref) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final searchEndpoints = SearchEndpoints(apiClient);
  return await searchEndpoints.getTrends();
});
