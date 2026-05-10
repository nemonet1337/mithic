import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cached_network_image/cached_network_image.dart';
import 'package:mithic/features/profile/providers/profile_provider.dart';
import 'package:mithic/models/user.dart';

class FollowingScreen extends ConsumerWidget {
  final String userId;
  const FollowingScreen({super.key, required this.userId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final followingAsync = ref.watch(profileFollowingProvider(userId));

    return Scaffold(
      appBar: AppBar(
        title: const Text('フォロー中'),
      ),
      body: followingAsync.when(
        data: (following) {
          if (following.isEmpty) {
            return const Center(child: Text('フォローしていません'));
          }
          return ListView.builder(
            itemCount: following.length,
            itemBuilder: (context, index) {
              final user = following[index];
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
