import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:mithic/api/endpoints/messages.dart';
import 'package:mithic/features/admin/screens/admin_screen.dart';
import 'package:mithic/features/admin/screens/admin_users_screen.dart';
import 'package:mithic/features/auth/providers/auth_provider.dart';
import 'package:mithic/features/auth/screens/login_screen.dart';
import 'package:mithic/features/antennas/screens/antennas_screen.dart';
import 'package:mithic/features/bookmarks/screens/bookmarks_screen.dart';
import 'package:mithic/features/blocks/screens/blocks_screen.dart';
import 'package:mithic/features/clips/screens/clips_screen.dart';
import 'package:mithic/features/compose/screens/compose_screen.dart';
import 'package:mithic/features/favorites/screens/favorites_screen.dart';
import 'package:mithic/features/federation/screens/federation_screen.dart';
import 'package:mithic/features/filters/screens/filters_screen.dart';
import 'package:mithic/features/follow_requests/screens/follow_requests_screen.dart';
import 'package:mithic/features/hashtag/screens/hashtag_screen.dart';
import 'package:mithic/features/lists/screens/lists_screen.dart';
import 'package:mithic/features/messages/screens/conversation_screen.dart';
import 'package:mithic/features/messages/screens/messages_screen.dart';
import 'package:mithic/features/mutes/screens/mutes_screen.dart';
import 'package:mithic/features/note/screens/note_detail_screen.dart';
import 'package:mithic/features/notification/screens/notification_screen.dart';
import 'package:mithic/features/oauth/screens/oauth_apps_screen.dart';
import 'package:mithic/features/profile/screens/profile_screen.dart';
import 'package:mithic/features/profile/screens/followers_screen.dart';
import 'package:mithic/features/profile/screens/following_screen.dart';
import 'package:mithic/features/relays/screens/relays_screen.dart';
import 'package:mithic/features/search/screens/search_screen.dart';
import 'package:mithic/features/settings/screens/settings_screen.dart';
import 'package:mithic/features/timeline/screens/home_timeline_screen.dart';
import 'package:mithic/features/two_factor/screens/two_factor_screen.dart';
import 'package:mithic/shared/layouts/app_shell.dart';

final routerProvider = Provider<GoRouter>((ref) {
  final authStatus = ref.watch(authProvider);

  return GoRouter(
    initialLocation: '/',
    redirect: (context, state) {
      final isLoggedIn      = authStatus == AuthStatus.authenticated;
      final isUninitialized = authStatus == AuthStatus.uninitialized;
      final isLoginPage     = state.matchedLocation == '/login';

      if (isUninitialized) return null;
      if (!isLoggedIn && !isLoginPage) return '/login';
      if (isLoggedIn && isLoginPage)   return '/';
      return null;
    },
    routes: [
      // ── Public ──────────────────────────────────────────────────────────
      GoRoute(
        path: '/login',
        builder: (context, state) => const LoginScreen(),
      ),

      // ── Authenticated (wrapped in AppShell) ──────────────────────────────
      ShellRoute(
        builder: (context, state, child) => AppShell(child: child),
        routes: [
          GoRoute(
            path: '/',
            builder: (context, state) => const HomeTimelineScreen(),
          ),
          GoRoute(
            path: '/timeline',
            builder: (context, state) => const HomeTimelineScreen(),
          ),

          // Compose (opens as overlay/modal inside shell)
          GoRoute(
            path: '/compose',
            builder: (context, state) => const ComposeScreen(),
          ),

          // Profile
          GoRoute(
            path: '/profile',
            builder: (context, state) {
              final userId = state.uri.queryParameters['userId'];
              return ProfileScreen(userId: userId);
            },
          ),
          GoRoute(
            path: '/profile/:userId/followers',
            builder: (context, state) {
              final userId = state.pathParameters['userId']!;
              return FollowersScreen(userId: userId);
            },
          ),
          GoRoute(
            path: '/profile/:userId/following',
            builder: (context, state) {
              final userId = state.pathParameters['userId']!;
              return FollowingScreen(userId: userId);
            },
          ),

          // Notes
          GoRoute(
            path: '/notes/:noteId',
            builder: (context, state) {
              final noteId = state.pathParameters['noteId']!;
              return NoteDetailScreen(noteId: noteId);
            },
          ),

          // Hashtags
          GoRoute(
            path: '/hashtags/:tag',
            builder: (context, state) {
              final tag = state.pathParameters['tag']!;
              return HashtagScreen(tag: tag);
            },
          ),

          // Notifications
          GoRoute(
            path: '/notifications',
            builder: (context, state) => const NotificationScreen(),
          ),

          // Messages (DM)
          GoRoute(
            path: '/messages',
            builder: (context, state) => const MessagesScreen(),
          ),
          GoRoute(
            path: '/messages/:conversationId',
            builder: (context, state) {
              final id   = state.pathParameters['conversationId']!;
              final conv = state.extra as DirectConversation?;
              return ConversationScreen(
                conversationId: id,
                conversation: conv,
              );
            },
          ),

          // Search
          GoRoute(
            path: '/search',
            builder: (context, state) => const SearchScreen(),
          ),

          // Settings
          GoRoute(
            path: '/settings',
            builder: (context, state) => const SettingsScreen(),
          ),

          // Social actions
          GoRoute(
            path: '/favorites',
            builder: (context, state) => const FavoritesScreen(),
          ),
          GoRoute(
            path: '/bookmarks',
            builder: (context, state) => const BookmarksScreen(),
          ),
          GoRoute(
            path: '/blocks',
            builder: (context, state) => const BlocksScreen(),
          ),
          GoRoute(
            path: '/mutes',
            builder: (context, state) => const MutesScreen(),
          ),
          GoRoute(
            path: '/follow_requests',
            builder: (context, state) => const FollowRequestsScreen(),
          ),
          GoRoute(
            path: '/filters',
            builder: (context, state) => const FiltersScreen(),
          ),

          // Lists / Antennas / Clips
          GoRoute(
            path: '/lists',
            builder: (context, state) => const ListsScreen(),
          ),
          GoRoute(
            path: '/antennas',
            builder: (context, state) => const AntennasScreen(),
          ),
          GoRoute(
            path: '/clips',
            builder: (context, state) => const ClipsScreen(),
          ),

          // Federation
          GoRoute(
            path: '/relays',
            builder: (context, state) => const RelaysScreen(),
          ),
          GoRoute(
            path: '/federation',
            builder: (context, state) => const FederationScreen(),
          ),

          // Admin
          GoRoute(
            path: '/admin',
            builder: (context, state) => const AdminScreen(),
          ),
          GoRoute(
            path: '/admin/users',
            builder: (context, state) => const AdminUsersScreen(),
          ),

          // OAuth / 2FA
          GoRoute(
            path: '/oauth/apps',
            builder: (context, state) => const OAuthAppsScreen(),
          ),
          GoRoute(
            path: '/2fa',
            builder: (context, state) => const TwoFactorScreen(),
          ),
        ],
      ),
    ],
  );
});
