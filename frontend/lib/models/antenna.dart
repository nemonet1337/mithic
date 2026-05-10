class Antenna {
  final String id;
  final String name;
  final List<String> keywords;
  final List<String> users;
  final List<String> instances;
  final bool caseSensitive;
  final bool withReplies;
  final bool withFile;
  final DateTime? createdAt;
  final DateTime? updatedAt;

  Antenna({
    required this.id,
    required this.name,
    required this.keywords,
    required this.users,
    required this.instances,
    this.caseSensitive = false,
    this.withReplies = false,
    this.withFile = false,
    this.createdAt,
    this.updatedAt,
  });

  factory Antenna.fromJson(Map<String, dynamic> json) {
    return Antenna(
      id: json['id'] as String,
      name: json['name'] as String,
      keywords: (json['keywords'] as List<dynamic>).map((e) => e as String).toList(),
      users: (json['users'] as List<dynamic>).map((e) => e as String).toList(),
      instances: (json['instances'] as List<dynamic>).map((e) => e as String).toList(),
      caseSensitive: json['case_sensitive'] as bool? ?? false,
      withReplies: json['with_replies'] as bool? ?? false,
      withFile: json['with_file'] as bool? ?? false,
      createdAt: json['created_at'] != null
          ? DateTime.parse(json['created_at'] as String)
          : null,
      updatedAt: json['updated_at'] != null
          ? DateTime.parse(json['updated_at'] as String)
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'keywords': keywords,
      'users': users,
      'instances': instances,
      'case_sensitive': caseSensitive,
      'with_replies': withReplies,
      'with_file': withFile,
      if (createdAt != null) 'created_at': createdAt!.toIso8601String(),
      if (updatedAt != null) 'updated_at': updatedAt!.toIso8601String(),
    };
  }
}
