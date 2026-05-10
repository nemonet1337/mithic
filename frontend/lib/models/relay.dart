class Relay {
  final String id;
  final String inbox;
  final String? status;
  final DateTime? acceptedAt;
  final DateTime? updatedAt;

  Relay({
    required this.id,
    required this.inbox,
    this.status,
    this.acceptedAt,
    this.updatedAt,
  });

  factory Relay.fromJson(Map<String, dynamic> json) {
    return Relay(
      id: json['id'] as String,
      inbox: json['inbox'] as String,
      status: json['status'] as String?,
      acceptedAt: json['acceptedAt'] != null
          ? DateTime.parse(json['acceptedAt'] as String)
          : null,
      updatedAt: json['updatedAt'] != null
          ? DateTime.parse(json['updatedAt'] as String)
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'inbox': inbox,
      if (status != null) 'status': status,
      if (acceptedAt != null) 'acceptedAt': acceptedAt!.toIso8601String(),
      if (updatedAt != null) 'updatedAt': updatedAt!.toIso8601String(),
    };
  }
}
