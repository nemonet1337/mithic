import 'package:flutter/animation.dart';

class AppMotion {
  // Durations in milliseconds
  static const Duration fast = Duration(milliseconds: 150);
  static const Duration base = Duration(milliseconds: 240);
  static const Duration slow = Duration(milliseconds: 400);
  static const Duration page = Duration(milliseconds: 480);

  // Curves
  static const Curve emphasized = Cubic(0.2, 0.0, 0.0, 1.0);
  static const Curve standard = Cubic(0.4, 0.0, 0.2, 1.0);
  static const Curve spring = Cubic(0.175, 0.885, 0.32, 1.275);
  static const Curve easeIn = Cubic(0.4, 0.0, 1.0, 1.0);
  static const Curve easeOut = Cubic(0.0, 0.0, 0.2, 1.0);
  static const Curve easeInOut = Cubic(0.4, 0.0, 0.2, 1.0);
}
