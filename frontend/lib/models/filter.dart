enum FilterContext {
  home,
  notifications,
  public,
  thread,
}

extension FilterContextExtension on FilterContext {
  String get name {
    switch (this) {
      case FilterContext.home:
        return 'home';
      case FilterContext.notifications:
        return 'notifications';
      case FilterContext.public:
        return 'public';
      case FilterContext.thread:
        return 'thread';
    }
  }

  static FilterContext fromName(String name) {
    switch (name) {
      case 'home':
        return FilterContext.home;
      case 'notifications':
        return FilterContext.notifications;
      case 'public':
        return FilterContext.public;
      case 'thread':
        return FilterContext.thread;
      default:
        return FilterContext.home;
    }
  }
}

class Filter {
  final String id;
  final String phrase;
  final FilterContext context;
  final bool irreversible;
  final bool wholeWord;
  final DateTime? expiresAt;
  final DateTime? createdAt;
  final DateTime? updatedAt;

  Filter({
    required this.id,
    required this.phrase,
    required this.context,
    this.irreversible = false,
    this.wholeWord = false,
    this.expiresAt,
    this.createdAt,
    this.updatedAt,
  });

  factory Filter.fromJson(Map<String, dynamic> json) {
    return Filter(
      id: json['id'] as String,
      phrase: json['phrase'] as String,
      context: FilterContextExtension.fromName(json['context'] as String),
      irreversible: json['irreversible'] as bool? ?? false,
      wholeWord: json['whole_word'] as bool? ?? false,
      expiresAt: json['expires_at'] != null
          ? DateTime.parse(json['expires_at'] as String)
          : null,
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
      'phrase': phrase,
      'context': context.name,
      'irreversible': irreversible,
      'whole_word': wholeWord,
      if (expiresAt != null) 'expires_at': expiresAt!.toIso8601String(),
      if (createdAt != null) 'created_at': createdAt!.toIso8601String(),
      if (updatedAt != null) 'updated_at': updatedAt!.toIso8601String(),
    };
  }
}

