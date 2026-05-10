class OAuthApp {
  final String id;
  final String name;
  final String? description;
  final String callbackUrl;
  final List<String>? permissions;
  final String? secret;
  final DateTime? createdAt;

  OAuthApp({
    required this.id,
    required this.name,
    this.description,
    required this.callbackUrl,
    this.permissions,
    this.secret,
    this.createdAt,
  });

  factory OAuthApp.fromJson(Map<String, dynamic> json) {
    return OAuthApp(
      id: json['id'] as String,
      name: json['name'] as String,
      description: json['description'] as String?,
      callbackUrl: json['callbackUrl'] as String,
      permissions: json['permission'] != null
          ? (json['permission'] as String).split(',')
          : null,
      secret: json['secret'] as String?,
      createdAt: json['createdAt'] != null
          ? DateTime.parse(json['createdAt'] as String)
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      if (description != null) 'description': description,
      'callbackUrl': callbackUrl,
      if (permissions != null) 'permission': permissions!.join(','),
      if (secret != null) 'secret': secret,
      if (createdAt != null) 'createdAt': createdAt!.toIso8601String(),
    };
  }
}
