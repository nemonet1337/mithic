import 'package:flutter/material.dart';
import 'radii.dart';

class AppElevation {
  // Neumorphic shadow offsets
  static const Offset lightShadowOffset = Offset(-4, -4);
  static const Offset darkShadowOffset = Offset(4, 4);

  // Shadow blur radii
  static const double shadowBlur = 12.0;
  static const double shadowBlurSmall = 8.0;

  // Shadow colors (will be adjusted dynamically based on theme)
  static Color lightShadowColor(Color surface) {
    return surface.withOpacity(0.7);
  }

  static Color darkShadowColor(Color surface) {
    return surface.withOpacity(0.5);
  }

  // Inset shadow for pressed state
  // Note: Flutter BoxShadow doesn't support inset directly.
  // Use PhysicalModel or custom painting for inset shadows.
  static BoxShadow neumorphicShadow(Color surface) {
    return BoxShadow(
      color: darkShadowColor(surface),
      offset: darkShadowOffset,
      blurRadius: shadowBlur,
      spreadRadius: 0,
    );
  }

  static BoxShadow neumorphicHighlight(Color surface) {
    return BoxShadow(
      color: lightShadowColor(surface),
      offset: lightShadowOffset,
      blurRadius: shadowBlur,
      spreadRadius: 0,
    );
  }

  static List<BoxShadow> neumorphic(Color surface) {
    return [
      neumorphicHighlight(surface),
      neumorphicShadow(surface),
    ];
  }

  // For inset shadows, use a BoxDecoration with custom gradient or
  // PhysicalModel with elevation. This is a placeholder for the concept.
  // Actual implementation will use a gradient-based approach in widgets.
  static BoxDecoration neumorphicInsetDecoration(Color surface) {
    return BoxDecoration(
      color: surface,
      borderRadius: AppRadii.mdRadius,
      boxShadow: [
        BoxShadow(
          color: darkShadowColor(surface),
          offset: darkShadowOffset,
          blurRadius: shadowBlur,
          spreadRadius: 0,
        ),
        BoxShadow(
          color: lightShadowColor(surface),
          offset: lightShadowOffset,
          blurRadius: shadowBlur,
          spreadRadius: 0,
        ),
      ],
    );
  }
}
