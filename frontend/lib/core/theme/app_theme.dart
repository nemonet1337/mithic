import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:google_fonts/google_fonts.dart';
import 'tokens/colors.dart';
import 'tokens/radii.dart';
import 'tokens/spacing.dart';

class AppTheme {
  static ThemeData lightTheme({Color? seedColor}) {
    const bg   = MithicColors.paper;
    const card = MithicColors.card;
    const ink  = MithicColors.ink;

    final colorScheme = ColorScheme.fromSeed(
      seedColor: MithicColors.accent,
      brightness: Brightness.light,
      surface: bg,
      primary: MithicColors.accent,
      onPrimary: Colors.white,
      secondary: MithicColors.accent2,
      onSecondary: ink,
      onSurface: ink,
      outline: MithicColors.lineSoft,
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: colorScheme,
      scaffoldBackgroundColor: bg,
      textTheme: _textTheme(ink),
      cardTheme: CardThemeData(
        elevation: 0,
        color: card,
        shape: RoundedRectangleBorder(
          borderRadius: AppRadii.mdRadius,
          side: const BorderSide(color: ink, width: 1.25),
        ),
      ),
      appBarTheme: AppBarTheme(
        elevation: 0,
        scrolledUnderElevation: 0,
        backgroundColor: bg,
        foregroundColor: ink,
        centerTitle: false,
        titleTextStyle: GoogleFonts.patrickHand(
          fontSize: 24,
          color: ink,
          height: 1,
        ),
        systemOverlayStyle: SystemUiOverlayStyle.dark,
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: MithicColors.accent,
          foregroundColor: Colors.white,
          elevation: 0,
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.s16,
            vertical: AppSpacing.s8,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
            side: const BorderSide(color: ink, width: 1.25),
          ),
          textStyle: GoogleFonts.dmSans(fontWeight: FontWeight.w500),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: ink,
          elevation: 0,
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.s16,
            vertical: AppSpacing.s8,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
          side: const BorderSide(color: ink, width: 1.25),
          textStyle: GoogleFonts.dmSans(fontWeight: FontWeight.w500),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          foregroundColor: ink,
          textStyle: GoogleFonts.dmSans(fontWeight: FontWeight.w500),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: card,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: BorderSide(color: MithicColors.lineSoft, width: 1.25),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: BorderSide(color: MithicColors.lineSoft, width: 1.25),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: MithicColors.accent, width: 1.5),
        ),
        hintStyle: GoogleFonts.dmSans(color: MithicColors.ink3),
        labelStyle: GoogleFonts.jetBrainsMono(
          fontSize: 10,
          letterSpacing: 0.12,
          color: MithicColors.ink3,
        ),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.s12,
          vertical: AppSpacing.s8,
        ),
      ),
      dividerTheme: const DividerThemeData(
        color: MithicColors.lineSoft,
        thickness: 1.25,
      ),
      tabBarTheme: TabBarThemeData(
        labelColor: ink,
        unselectedLabelColor: MithicColors.ink3,
        labelStyle: GoogleFonts.dmSans(fontSize: 12, fontWeight: FontWeight.w600),
        unselectedLabelStyle: GoogleFonts.dmSans(fontSize: 12),
        indicator: const UnderlineTabIndicator(
          borderSide: BorderSide(color: MithicColors.accent, width: 2),
        ),
        indicatorSize: TabBarIndicatorSize.tab,
      ),
      bottomNavigationBarTheme: const BottomNavigationBarThemeData(
        backgroundColor: bg,
        selectedItemColor: MithicColors.accent,
        unselectedItemColor: MithicColors.ink3,
        type: BottomNavigationBarType.fixed,
        elevation: 0,
      ),
      chipTheme: ChipThemeData(
        backgroundColor: card,
        side: const BorderSide(color: MithicColors.lineSoft, width: 1),
        labelStyle: GoogleFonts.jetBrainsMono(fontSize: 11),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
      ),
      pageTransitionsTheme: const PageTransitionsTheme(
        builders: {
          TargetPlatform.android: CupertinoPageTransitionsBuilder(),
          TargetPlatform.iOS: CupertinoPageTransitionsBuilder(),
          TargetPlatform.windows: CupertinoPageTransitionsBuilder(),
          TargetPlatform.macOS: CupertinoPageTransitionsBuilder(),
          TargetPlatform.linux: CupertinoPageTransitionsBuilder(),
        },
      ),
    );
  }

  static ThemeData darkTheme({Color? seedColor}) {
    const bg   = MithicColors.paperDark;
    const card = MithicColors.cardDark;
    const ink  = MithicColors.inkDark;

    final colorScheme = ColorScheme.fromSeed(
      seedColor: MithicColors.accentDark,
      brightness: Brightness.dark,
      surface: bg,
      primary: MithicColors.accentDark,
      onPrimary: Colors.white,
      secondary: MithicColors.accent2Dark,
      onSecondary: Colors.white,
      onSurface: ink,
      outline: const Color(0x38F3EFE6),
    );

    return ThemeData(
      useMaterial3: true,
      colorScheme: colorScheme,
      scaffoldBackgroundColor: bg,
      textTheme: _textTheme(ink),
      cardTheme: CardThemeData(
        elevation: 0,
        color: card,
        shape: RoundedRectangleBorder(
          borderRadius: AppRadii.mdRadius,
          side: const BorderSide(color: Color(0x38F3EFE6), width: 1.25),
        ),
      ),
      appBarTheme: AppBarTheme(
        elevation: 0,
        scrolledUnderElevation: 0,
        backgroundColor: bg,
        foregroundColor: ink,
        centerTitle: false,
        titleTextStyle: GoogleFonts.patrickHand(
          fontSize: 24,
          color: ink,
          height: 1,
        ),
        systemOverlayStyle: SystemUiOverlayStyle.light,
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: MithicColors.accentDark,
          foregroundColor: Colors.white,
          elevation: 0,
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.s16,
            vertical: AppSpacing.s8,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
            side: const BorderSide(color: Color(0x38F3EFE6), width: 1.25),
          ),
          textStyle: GoogleFonts.dmSans(fontWeight: FontWeight.w500),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: ink,
          elevation: 0,
          padding: const EdgeInsets.symmetric(
            horizontal: AppSpacing.s16,
            vertical: AppSpacing.s8,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
          side: const BorderSide(color: Color(0x38F3EFE6), width: 1.25),
          textStyle: GoogleFonts.dmSans(fontWeight: FontWeight.w500),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          foregroundColor: ink,
          textStyle: GoogleFonts.dmSans(fontWeight: FontWeight.w500),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: card,
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0x38F3EFE6), width: 1.25),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0x38F3EFE6), width: 1.25),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: MithicColors.accentDark, width: 1.5),
        ),
        hintStyle: GoogleFonts.dmSans(color: MithicColors.ink3Dark),
        contentPadding: const EdgeInsets.symmetric(
          horizontal: AppSpacing.s12,
          vertical: AppSpacing.s8,
        ),
      ),
      dividerTheme: const DividerThemeData(
        color: Color(0x38F3EFE6),
        thickness: 1.25,
      ),
      tabBarTheme: TabBarThemeData(
        labelColor: ink,
        unselectedLabelColor: MithicColors.ink3Dark,
        labelStyle: GoogleFonts.dmSans(fontSize: 12, fontWeight: FontWeight.w600),
        unselectedLabelStyle: GoogleFonts.dmSans(fontSize: 12),
        indicator: const UnderlineTabIndicator(
          borderSide: BorderSide(color: MithicColors.accentDark, width: 2),
        ),
        indicatorSize: TabBarIndicatorSize.tab,
      ),
      bottomNavigationBarTheme: const BottomNavigationBarThemeData(
        backgroundColor: bg,
        selectedItemColor: MithicColors.accentDark,
        unselectedItemColor: MithicColors.ink3Dark,
        type: BottomNavigationBarType.fixed,
        elevation: 0,
      ),
      chipTheme: ChipThemeData(
        backgroundColor: card,
        side: const BorderSide(color: Color(0x38F3EFE6), width: 1),
        labelStyle: GoogleFonts.jetBrainsMono(fontSize: 11),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(999)),
      ),
      pageTransitionsTheme: const PageTransitionsTheme(
        builders: {
          TargetPlatform.android: CupertinoPageTransitionsBuilder(),
          TargetPlatform.iOS: CupertinoPageTransitionsBuilder(),
          TargetPlatform.windows: CupertinoPageTransitionsBuilder(),
          TargetPlatform.macOS: CupertinoPageTransitionsBuilder(),
          TargetPlatform.linux: CupertinoPageTransitionsBuilder(),
        },
      ),
    );
  }

  static TextTheme _textTheme(Color ink) => TextTheme(
    displayLarge:  GoogleFonts.patrickHand(fontSize: 48, color: ink, height: 1),
    displayMedium: GoogleFonts.patrickHand(fontSize: 36, color: ink, height: 1),
    displaySmall:  GoogleFonts.patrickHand(fontSize: 28, color: ink, height: 1),
    headlineLarge: GoogleFonts.patrickHand(fontSize: 24, color: ink, height: 1),
    headlineMedium:GoogleFonts.patrickHand(fontSize: 20, color: ink, height: 1),
    headlineSmall: GoogleFonts.patrickHand(fontSize: 18, color: ink, height: 1),
    titleLarge:    GoogleFonts.dmSans(fontSize: 17, fontWeight: FontWeight.w600, color: ink),
    titleMedium:   GoogleFonts.dmSans(fontSize: 15, fontWeight: FontWeight.w600, color: ink),
    titleSmall:    GoogleFonts.dmSans(fontSize: 13, fontWeight: FontWeight.w600, color: ink),
    bodyLarge:     GoogleFonts.dmSans(fontSize: 15, color: ink),
    bodyMedium:    GoogleFonts.dmSans(fontSize: 13.5, height: 1.55, color: ink),
    bodySmall:     GoogleFonts.dmSans(fontSize: 12, color: ink),
    labelLarge:    GoogleFonts.jetBrainsMono(fontSize: 11, letterSpacing: 0.12, color: ink),
    labelMedium:   GoogleFonts.jetBrainsMono(fontSize: 10, letterSpacing: 0.12, color: ink),
    labelSmall:    GoogleFonts.jetBrainsMono(fontSize: 9, letterSpacing: 0.08, color: ink),
  );
}
