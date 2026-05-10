import 'package:flutter/material.dart';

/// Theme presets for user selection
enum AppThemePreset {
  aurora,      // Cyan × Lavender (default, brand color)
  sakura,      // Cherry pink × Cream
  mint,        // Mint × Ivory
  sunset,      // Peach × Lavender
  lavender,    // Light purple × Off-white
  mono,        // Monochrome × Single accent
  mithicWarm,  // Mithic brand: warm paper × hot pink
}

/// Mithic wireframe design tokens (light)
class MithicColors {
  static const Color paper     = Color(0xFFF4F1EA);
  static const Color card      = Color(0xFFFBF9F3);
  static const Color cardAlt   = Color(0xFFECE8DE);
  static const Color ink       = Color(0xFF1A1714);
  static const Color ink2      = Color(0xFF4A4640);
  static const Color ink3      = Color(0xFF8A847A);
  static const Color accent    = Color(0xFFFF3D8B);
  static const Color accent2   = Color(0xFFFFC6DD);
  static const Color accentSoft= Color(0xFFFFE1EC);
  static const Color warn      = Color(0xFFFFB24A);
  static const Color lineSoft  = Color(0x2E1A1714); // rgba(26,23,20,0.18)

  // Dark variants
  static const Color paperDark    = Color(0xFF16140F);
  static const Color cardDark     = Color(0xFF1F1C16);
  static const Color cardAltDark  = Color(0xFF2A261E);
  static const Color inkDark      = Color(0xFFF3EFE6);
  static const Color ink2Dark     = Color(0xFFC9C3B6);
  static const Color ink3Dark     = Color(0xFF7A746A);
  static const Color accentDark   = Color(0xFFFF5FA0);
  static const Color accent2Dark  = Color(0xFFC25080);
}

enum ThemeModeOption {
  light,
  dark,
  system,
}

class AppColors {
  // Base surface colors for Neumorphism
  static const Color lightSurface = Color(0xFFE0E5EC);
  static const Color darkSurface = Color(0xFF1A1D24);

  // Preset seed colors
  static const Map<AppThemePreset, Color> seedColors = {
    AppThemePreset.aurora:      Color(0xFF00B4D8),
    AppThemePreset.sakura:      Color(0xFFFFB7C5),
    AppThemePreset.mint:        Color(0xFF98FF98),
    AppThemePreset.sunset:      Color(0xFFFFB347),
    AppThemePreset.lavender:    Color(0xFFE6E6FA),
    AppThemePreset.mono:        Color(0xFF6B7280),
    AppThemePreset.mithicWarm:  MithicColors.accent,
  };

  static Color getSeedColor(AppThemePreset preset) {
    return seedColors[preset] ?? seedColors[AppThemePreset.aurora]!;
  }

  // Pastel Night colors for dark mode
  static const Color pastelNightSurface = Color(0xFF1A1D2E);
  static const Color pastelNightCard = Color(0xFF252936);
}
