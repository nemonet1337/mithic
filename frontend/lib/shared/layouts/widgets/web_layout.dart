import 'package:flutter/material.dart';

class WebLayout extends StatelessWidget {
  final Widget leftColumn;
  final Widget centerColumn;
  final Widget rightColumn;

  const WebLayout({
    super.key,
    required this.leftColumn,
    required this.centerColumn,
    required this.rightColumn,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        // Left column (Navigation, Lists, Antennas)
        SizedBox(
          width: 280,
          child: leftColumn,
        ),
        const VerticalDivider(width: 1),
        // Center column (Timeline, Note detail)
        Expanded(
          child: centerColumn,
        ),
        const VerticalDivider(width: 1),
        // Right column (Trends, Widgets)
        SizedBox(
          width: 320,
          child: rightColumn,
        ),
      ],
    );
  }
}
