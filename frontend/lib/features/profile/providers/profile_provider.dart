import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/accounts.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/user.dart';
import 'package:mithic/models/relationship.dart';
import 'package:mithic/models/note.dart';

final profileProvider = FutureProvider.family<User, String>((ref, userId) async {
  final apiClient = ref.watch(apiClientProvider);
  final accountsEndpoints = AccountsEndpoints(apiClient);
  return await accountsEndpoints.getAccount(userId);
});

final profileRelationshipProvider = FutureProvider.family<Relationship, String>((ref, userId) async {
  if (AppConfig.isMockMode) {
    return Relationship(
      id: 'mock',
      following: false,
      followedBy: false,
      blocking: false,
      blockedBy: false,
      muting: false,
      mutedBy: false,
      requested: false,
    );
  }

  final apiClient = ref.watch(apiClientProvider);
  final accountsEndpoints = AccountsEndpoints(apiClient);
  final relations = await accountsEndpoints.getRelations(ids: [userId]);
  return relations.isNotEmpty ? relations.first : Relationship(
    id: userId,
    following: false,
    followedBy: false,
    blocking: false,
    blockedBy: false,
    muting: false,
    mutedBy: false,
    requested: false,
  );
});

final profileFollowersProvider = FutureProvider.family<List<User>, String>((ref, userId) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final accountsEndpoints = AccountsEndpoints(apiClient);
  return await accountsEndpoints.getFollowers(userId);
});

final profileFollowingProvider = FutureProvider.family<List<User>, String>((ref, userId) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final accountsEndpoints = AccountsEndpoints(apiClient);
  return await accountsEndpoints.getFollowing(userId);
});

enum ProfileTab {
  posts,
  replies,
  media,
}

final profileTabProvider = StateProvider<ProfileTab>((ref) => ProfileTab.posts);

final profilePostsProvider = FutureProvider.family<List<Note>, String>((ref, userId) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final accountsEndpoints = AccountsEndpoints(apiClient);
  return await accountsEndpoints.getUserStatuses(userId);
});

class ProfileActions {
  final Ref ref;

  ProfileActions(this.ref);

  Future<User> follow(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final accountsEndpoints = AccountsEndpoints(apiClient);
    final user = await accountsEndpoints.followAccount(userId);
    ref.invalidate(profileRelationshipProvider(userId));
    ref.invalidate(profileProvider(userId));
    return user;
  }

  Future<User> unfollow(String userId) async {
    final apiClient = ref.read(apiClientProvider);
    final accountsEndpoints = AccountsEndpoints(apiClient);
    final user = await accountsEndpoints.unfollowAccount(userId);
    ref.invalidate(profileRelationshipProvider(userId));
    ref.invalidate(profileProvider(userId));
    return user;
  }
}

final profileActionsProvider = Provider<ProfileActions>((ref) {
  return ProfileActions(ref);
});
