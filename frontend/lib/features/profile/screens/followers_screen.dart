import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:mithic/features/profile/providers/profile_provider.dart';
import 'package:mithic/models/user.dart';

class FollowersScreen extends ConsumerWidget {
  final String userId;
  const FollowersScreen({super.key, required this.userId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final followersAsync = ref.watch(profileFollowersProvider(userId));

    return Scaffold(
      appBar: AppBar(
        title: const Text('フォロワー'),
      ),
      body: followersAsync.when(
        data: (followers) {
          if (followers.isEmpty) {
            return const Center(child: Text('フォロワーがいません'));
          }
          return ListView.builder(
            itemCount: followers.length,
            itemBuilder: (context, index) {
              final user = followers[index];
              return _buildUserTile(context, user);
            },
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Text('エラー: $error'),
        ),
      ),
    );
  }

  Widget _buildUserTile(BuildContext context, User user) {
    return ListTile(
      leading: user.avatarUrl != null
          ? CircleAvatar(
              backgroundImage: CachedNetworkImageProvider(user.avatarUrl!),
            )
          : CircleAvatar(
              child: Text(user.username[0].toUpperCase()),
            ),
      title: Text(user.name ?? user.username),
      subtitle: Text('@${user.username}'),
      onTap: () {
        // TODO: Navigate to user profile
      },
    );
  }
}
