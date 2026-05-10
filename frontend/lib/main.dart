import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/core/config/app_config.dart';
import 'app.dart';

void main() {
  // モックモードを有効化（本番モードにする場合はこの行を削除またはfalseに設定）
  AppConfig.isMockMode = true;
  
  runApp(
    const ProviderScope(
      child: MithicApp(),
    ),
  );
}
