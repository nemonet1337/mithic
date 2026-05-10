import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/auth.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/core/mock/mock_data.dart';
import 'package:mithic/core/storage/secure_storage.dart';
import 'package:mithic/models/user.dart';

part 'auth_provider.g.dart';

enum AuthStatus {
  uninitialized,
  authenticated,
  unauthenticated,
  loading,
}

@riverpod
class Auth extends _$Auth {
  @override
  AuthStatus build() {
    return AuthStatus.uninitialized;
  }

  Future<void> initialize() async {
    state = AuthStatus.loading;
    
    // モックモードの場合は自動認証
    if (AppConfig.isMockMode) {
      state = AuthStatus.authenticated;
      return;
    }
    
    final storage = await ref.read(appStorageProvider.future);
    final token = await storage.getAccessToken();
    final baseUrl = await storage.getBaseUrl();

    if (token != null && baseUrl != null) {
      final apiClient = ref.read(apiClientProvider);
      apiClient.setBaseUrl(baseUrl);
      apiClient.setAccessToken(token);
      state = AuthStatus.authenticated;
    } else {
      state = AuthStatus.unauthenticated;
    }
  }

  Future<void> login(String instanceUrl, String username, String password) async {
    state = AuthStatus.loading;
    try {
      // モックモードの場合は即座に認証
      if (AppConfig.isMockMode) {
        state = AuthStatus.authenticated;
        return;
      }
      
      final apiClient = ref.read(apiClientProvider);
      apiClient.setBaseUrl(instanceUrl);

      final authEndpoints = AuthEndpoints(apiClient);
      final response = await authEndpoints.signin(
        username: username,
        password: password,
      );

      final storage = await ref.read(appStorageProvider.future);
      await storage.setBaseUrl(instanceUrl);
      await storage.setAccessToken(response['access_token'] as String);
      final refreshToken = response['refresh_token'] as String?;
      if (refreshToken != null) {
        await storage.setRefreshToken(refreshToken);
      }
      await storage.setUserId(response['user_id'] as String);

      apiClient.setAccessToken(response['access_token'] as String);
      state = AuthStatus.authenticated;
    } catch (e) {
      state = AuthStatus.unauthenticated;
      rethrow;
    }
  }

  Future<void> signup(String instanceUrl, String username, String password, {String? email}) async {
    state = AuthStatus.loading;
    try {
      // モックモードの場合は即座に認証
      if (AppConfig.isMockMode) {
        state = AuthStatus.authenticated;
        return;
      }
      
      final apiClient = ref.read(apiClientProvider);
      apiClient.setBaseUrl(instanceUrl);

      final authEndpoints = AuthEndpoints(apiClient);
      final response = await authEndpoints.signup(
        username: username,
        password: password,
        email: email,
      );

      final storage = await ref.read(appStorageProvider.future);
      await storage.setBaseUrl(instanceUrl);
      await storage.setAccessToken(response['access_token'] as String);
      final refreshToken = response['refresh_token'] as String?;
      if (refreshToken != null) {
        await storage.setRefreshToken(refreshToken);
      }
      await storage.setUserId(response['user_id'] as String);

      apiClient.setAccessToken(response['access_token'] as String);
      state = AuthStatus.authenticated;
    } catch (e) {
      state = AuthStatus.unauthenticated;
      rethrow;
    }
  }

  Future<void> logout() async {
    state = AuthStatus.loading;
    try {
      // モックモードの場合はローカル状態のみクリア
      if (AppConfig.isMockMode) {
        state = AuthStatus.unauthenticated;
        return;
      }
      
      final apiClient = ref.read(apiClientProvider);
      final storage = await ref.read(appStorageProvider.future);

      await storage.clearAuthData();
      apiClient.clearAuth();

      state = AuthStatus.unauthenticated;
    } catch (e) {
      state = AuthStatus.authenticated;
      rethrow;
    }
  }
}

@riverpod
User? currentUser(CurrentUserRef ref) {
  final authStatus = ref.watch(authProvider);
  
  // モックモードの場合はモックユーザーを返す
  if (AppConfig.isMockMode && authStatus == AuthStatus.authenticated) {
    return MockData.mockCurrentUser;
  }
  
  if (authStatus != AuthStatus.authenticated) {
    return null;
  }

  // TODO: Fetch user data from API when authenticated
  // For now, return a mock user
  return User(
    id: 'mock_user_id',
    username: 'mock_user',
    name: 'Mock User',
  );
}
