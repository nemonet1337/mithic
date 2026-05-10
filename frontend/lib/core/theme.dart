import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'theme/app_theme.dart';
import 'theme/tokens/colors.dart';

// Theme mode provider
final themeModeProvider = StateProvider<ThemeMode>((ref) {
  return ThemeMode.system;
});

// Theme preset provider (user-selectable accent color)
final themePresetProvider = StateProvider<AppThemePreset>((ref) {
  return AppThemePreset.mithicWarm;
});

// Locale provider for i18n
final localeProvider = StateProvider<Locale>((ref) {
  return const Locale('ja');
});

// Data saving provider
final dataSavingProvider = StateProvider<bool>((ref) {
  return false;
});

// Light theme provider
final lightThemeProvider = Provider<ThemeData>((ref) {
  final preset = ref.watch(themePresetProvider);
  return AppTheme.lightTheme(seedColor: AppColors.getSeedColor(preset));
});

// Dark theme provider
final darkThemeProvider = Provider<ThemeData>((ref) {
  final preset = ref.watch(themePresetProvider);
  return AppTheme.darkTheme(seedColor: AppColors.getSeedColor(preset));
});
