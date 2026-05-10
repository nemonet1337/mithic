/// アプリケーション設定
class AppConfig {
  /// モックモードフラグ
  /// trueの場合、バックエンドAPIを使用せずにモックデータで動作する
  static bool isMockMode = false;

  /// モックモードを有効化する
  static void enableMockMode() {
    isMockMode = true;
  }

  /// モックモードを無効化する
  static void disableMockMode() {
    isMockMode = false;
  }
}
