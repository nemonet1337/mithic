import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

final secureStorageProvider = Provider<FlutterSecureStorage>((ref) {
  return const FlutterSecureStorage(
    aOptions: AndroidOptions(
      encryptedSharedPreferences: true,
    ),
  );
});

final sharedPreferencesProvider = FutureProvider<SharedPreferences>((ref) async {
  return await SharedPreferences.getInstance();
});

class AppStorage {
  final FlutterSecureStorage _secureStorage;
  final SharedPreferences _prefs;

  AppStorage(this._secureStorage, this._prefs);

  static const String _keyBaseUrl = 'instance_url';
  static const String _keyAccessToken = 'access_token';
  static const String _keyRefreshToken = 'refresh_token';
  static const String _keyUserId = 'user_id';
  static const String _keyThemePreset = 'theme_preset';
  static const String _keyThemeMode = 'theme_mode';

  // Instance URL (secure storage)
  Future<void> setBaseUrl(String url) async {
    await _secureStorage.write(key: _keyBaseUrl, value: url);
  }

  Future<String?> getBaseUrl() async {
    return await _secureStorage.read(key: _keyBaseUrl);
  }

  Future<void> clearBaseUrl() async {
    await _secureStorage.delete(key: _keyBaseUrl);
  }

  // Access Token (secure storage)
  Future<void> setAccessToken(String token) async {
    await _secureStorage.write(key: _keyAccessToken, value: token);
  }

  Future<String?> getAccessToken() async {
    return await _secureStorage.read(key: _keyAccessToken);
  }

  Future<void> clearAccessToken() async {
    await _secureStorage.delete(key: _keyAccessToken);
  }

  // Refresh Token (secure storage)
  Future<void> setRefreshToken(String token) async {
    await _secureStorage.write(key: _keyRefreshToken, value: token);
  }

  Future<String?> getRefreshToken() async {
    return await _secureStorage.read(key: _keyRefreshToken);
  }

  Future<void> clearRefreshToken() async {
    await _secureStorage.delete(key: _keyRefreshToken);
  }

  // User ID (secure storage)
  Future<void> setUserId(String userId) async {
    await _secureStorage.write(key: _keyUserId, value: userId);
  }

  Future<String?> getUserId() async {
    return await _secureStorage.read(key: _keyUserId);
  }

  Future<void> clearUserId() async {
    await _secureStorage.delete(key: _keyUserId);
  }

  // Theme Preset (shared preferences)
  Future<void> setThemePreset(String preset) async {
    await _prefs.setString(_keyThemePreset, preset);
  }

  Future<String?> getThemePreset() async {
    return _prefs.getString(_keyThemePreset);
  }

  // Theme Mode (shared preferences)
  Future<void> setThemeMode(String mode) async {
    await _prefs.setString(_keyThemeMode, mode);
  }

  Future<String?> getThemeMode() async {
    return _prefs.getString(_keyThemeMode);
  }

  // Clear all auth data
  Future<void> clearAuthData() async {
    await Future.wait([
      clearAccessToken(),
      clearRefreshToken(),
      clearUserId(),
    ]);
  }

  // Clear all data
  Future<void> clearAll() async {
    await Future.wait([
      clearBaseUrl(),
      clearAccessToken(),
      clearRefreshToken(),
      clearUserId(),
    ]);
  }
}

final appStorageProvider = FutureProvider<AppStorage>((ref) async {
  final secureStorage = ref.watch(secureStorageProvider);
  final prefs = await ref.watch(sharedPreferencesProvider.future);
  return AppStorage(secureStorage, prefs);
});
