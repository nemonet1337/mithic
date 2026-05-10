import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

class AppTypography {
  // Font families
  static const String fontFamilyLatin = 'Plus Jakarta Sans';
  static const String fontFamilyJapanese = 'Noto Sans JP';

  // Text styles using Google Fonts
  static TextStyle get headline1 => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w700,
        fontSize: 32,
      );

  static TextStyle get headline2 => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w600,
        fontSize: 28,
      );

  static TextStyle get headline3 => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w600,
        fontSize: 24,
      );

  static TextStyle get headline4 => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w600,
        fontSize: 20,
      );

  static TextStyle get headline5 => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w500,
        fontSize: 18,
      );

  static TextStyle get bodyLarge => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w400,
        fontSize: 16,
      );

  static TextStyle get bodyMedium => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w400,
        fontSize: 14,
      );

  static TextStyle get bodySmall => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w400,
        fontSize: 12,
      );

  static TextStyle get labelLarge => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w500,
        fontSize: 14,
      );

  static TextStyle get labelMedium => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w500,
        fontSize: 12,
      );

  static TextStyle get labelSmall => GoogleFonts.getFont(
        fontFamilyLatin,
        fontWeight: FontWeight.w500,
        fontSize: 10,
      );

  // Japanese text styles
  static TextStyle get jpBodyLarge => GoogleFonts.getFont(
        fontFamilyJapanese,
        fontWeight: FontWeight.w400,
        fontSize: 16,
      );

  static TextStyle get jpBodyMedium => GoogleFonts.getFont(
        fontFamilyJapanese,
        fontWeight: FontWeight.w400,
        fontSize: 14,
      );

  // Apply to Material Theme
  static TextTheme get lightTextTheme => TextTheme(
        displayLarge: headline1,
        displayMedium: headline2,
        displaySmall: headline3,
        headlineMedium: headline4,
        headlineSmall: headline5,
        bodyLarge: bodyLarge,
        bodyMedium: bodyMedium,
        bodySmall: bodySmall,
        labelLarge: labelLarge,
        labelMedium: labelMedium,
        labelSmall: labelSmall,
      );

  static TextTheme get darkTextTheme => lightTextTheme;
}
