import 'package:flutter/material.dart';
import 'tokens/radii.dart';
import 'tokens/elevation.dart';
import 'tokens/spacing.dart';

class NeumorphicContainer extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final double? width;
  final double? height;
  final bool isPressed;
  final BorderRadius? borderRadius;
  final Color? color;

  const NeumorphicContainer({
    super.key,
    required this.child,
    this.padding,
    this.margin,
    this.width,
    this.height,
    this.isPressed = false,
    this.borderRadius,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    final surfaceColor = color ?? Theme.of(context).colorScheme.surface;
    final effectiveBorderRadius = borderRadius ?? AppRadii.mdRadius;

    return Container(
      width: width,
      height: height,
      margin: margin,
      padding: padding ?? const EdgeInsets.all(AppSpacing.s16),
      decoration: BoxDecoration(
        color: surfaceColor,
        borderRadius: effectiveBorderRadius,
        boxShadow: isPressed ? [] : AppElevation.neumorphic(surfaceColor),
      ),
      child: child,
    );
  }
}

class NeumorphicButton extends StatefulWidget {
  final Widget child;
  final VoidCallback? onPressed;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final double? width;
  final double? height;
  final BorderRadius? borderRadius;
  final Color? color;

  const NeumorphicButton({
    super.key,
    required this.child,
    this.onPressed,
    this.padding,
    this.margin,
    this.width,
    this.height,
    this.borderRadius,
    this.color,
  });

  @override
  State<NeumorphicButton> createState() => _NeumorphicButtonState();
}

class _NeumorphicButtonState extends State<NeumorphicButton> {
  bool _isPressed = false;

  @override
  Widget build(BuildContext context) {
    final surfaceColor = widget.color ?? Theme.of(context).colorScheme.surface;
    final effectiveBorderRadius = widget.borderRadius ?? AppRadii.mdRadius;

    return GestureDetector(
      onTapDown: widget.onPressed != null ? (_) => setState(() => _isPressed = true) : null,
      onTapUp: widget.onPressed != null ? (_) {
        setState(() => _isPressed = false);
        widget.onPressed?.call();
      } : null,
      onTapCancel: widget.onPressed != null ? () => setState(() => _isPressed = false) : null,
      child: Container(
        width: widget.width,
        height: widget.height,
        margin: widget.margin,
        decoration: BoxDecoration(
          color: surfaceColor,
          borderRadius: effectiveBorderRadius,
          boxShadow: _isPressed ? [] : AppElevation.neumorphic(surfaceColor),
        ),
        child: Padding(
          padding: widget.padding ?? const EdgeInsets.all(AppSpacing.s16),
          child: Center(child: widget.child),
        ),
      ),
    );
  }
}

class NeumorphicCard extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;
  final EdgeInsetsGeometry? margin;
  final double? width;
  final double? height;
  final BorderRadius? borderRadius;
  final Color? color;

  const NeumorphicCard({
    super.key,
    required this.child,
    this.padding,
    this.margin,
    this.width,
    this.height,
    this.borderRadius,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    final surfaceColor = color ?? Theme.of(context).colorScheme.surface;
    final effectiveBorderRadius = borderRadius ?? AppRadii.lgRadius;

    return Container(
      width: width,
      height: height,
      margin: margin,
      padding: padding ?? const EdgeInsets.all(AppSpacing.s16),
      decoration: BoxDecoration(
        color: surfaceColor,
        borderRadius: effectiveBorderRadius,
        boxShadow: AppElevation.neumorphic(surfaceColor),
      ),
      child: child,
    );
  }
}
