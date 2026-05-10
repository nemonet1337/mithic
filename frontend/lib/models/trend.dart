class Trend {
  final String tag;
  final int count;
  final List<String>? history;

  Trend({
    required this.tag,
    required this.count,
    this.history,
  });

  factory Trend.fromJson(Map<String, dynamic> json) {
    return Trend(
      tag: json['tag'] as String,
      count: json['count'] as int,
      history: json['history'] != null
          ? (json['history'] as List).map((e) => e as String).toList()
          : null,
    );
  }
}
