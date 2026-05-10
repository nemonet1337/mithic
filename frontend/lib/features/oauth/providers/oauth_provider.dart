import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/oauth.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/oauth_app.dart';

final oauthAppsProvider = FutureProvider.family<List<OAuthApp>, void>((ref, _) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final oauthEndpoints = OAuthEndpoints(apiClient);
  return await oauthEndpoints.getApps();
});

final oauthActionsProvider = Provider<OAuthActions>((ref) {
  return OAuthActions(ref);
});

class OAuthActions {
  final Ref ref;

  OAuthActions(this.ref);

  Future<OAuthApp> createApp({
    required String name,
    required String callbackUrl,
    String? description,
    List<String>? permissions,
  }) async {
    final apiClient = ref.read(apiClientProvider);
    final oauthEndpoints = OAuthEndpoints(apiClient);
    final app = await oauthEndpoints.createApp(
      name: name,
      callbackUrl: callbackUrl,
      description: description,
      permissions: permissions,
    );
    ref.invalidate(oauthAppsProvider);
    return app;
  }

  Future<void> deleteApp(String appId) async {
    final apiClient = ref.read(apiClientProvider);
    final oauthEndpoints = OAuthEndpoints(apiClient);
    await oauthEndpoints.deleteApp(appId);
    ref.invalidate(oauthAppsProvider);
  }

  Future<String> generateAuthCode(String appId) async {
    final apiClient = ref.read(apiClientProvider);
    final oauthEndpoints = OAuthEndpoints(apiClient);
    return await oauthEndpoints.generateAuthCode(appId);
  }
}
