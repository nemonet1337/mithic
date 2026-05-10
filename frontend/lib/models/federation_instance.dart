class FederationInstance {
  final String id;
  final String host;
  final String? name;
  final String? description;
  final String? softwareName;
  final String? softwareVersion;
  final String? iconUrl;
  final int? usersCount;
  final int? notesCount;
  final int? followingCount;
  final int? followersCount;
  final bool? isNotResponding;
  final bool? isSilenced;
  final bool? isSuspended;
  final DateTime? firstRetrievedAt;
  final DateTime? latestRequestReceivedAt;

  FederationInstance({
    required this.id,
    required this.host,
    this.name,
    this.description,
    this.softwareName,
    this.softwareVersion,
    this.iconUrl,
    this.usersCount,
    this.notesCount,
    this.followingCount,
    this.followersCount,
    this.isNotResponding,
    this.isSilenced,
    this.isSuspended,
    this.firstRetrievedAt,
    this.latestRequestReceivedAt,
  });

  factory FederationInstance.fromJson(Map<String, dynamic> json) {
    return FederationInstance(
      id: json['id'] as String,
      host: json['host'] as String,
      name: json['name'] as String?,
      description: json['description'] as String?,
      softwareName: json['softwareName'] as String?,
      softwareVersion: json['softwareVersion'] as String?,
      iconUrl: json['iconUrl'] as String?,
      usersCount: json['usersCount'] as int?,
      notesCount: json['notesCount'] as int?,
      followingCount: json['followingCount'] as int?,
      followersCount: json['followersCount'] as int?,
      isNotResponding: json['isNotResponding'] as bool?,
      isSilenced: json['isSilenced'] as bool?,
      isSuspended: json['isSuspended'] as bool?,
      firstRetrievedAt: json['firstRetrievedAt'] != null
          ? DateTime.parse(json['firstRetrievedAt'] as String)
          : null,
      latestRequestReceivedAt: json['latestRequestReceivedAt'] != null
          ? DateTime.parse(json['latestRequestReceivedAt'] as String)
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'host': host,
      if (name != null) 'name': name,
      if (description != null) 'description': description,
      if (softwareName != null) 'softwareName': softwareName,
      if (softwareVersion != null) 'softwareVersion': softwareVersion,
      if (iconUrl != null) 'iconUrl': iconUrl,
      if (usersCount != null) 'usersCount': usersCount,
      if (notesCount != null) 'notesCount': notesCount,
      if (followingCount != null) 'followingCount': followingCount,
      if (followersCount != null) 'followersCount': followersCount,
      if (isNotResponding != null) 'isNotResponding': isNotResponding,
      if (isSilenced != null) 'isSilenced': isSilenced,
      if (isSuspended != null) 'isSuspended': isSuspended,
      if (firstRetrievedAt != null) 'firstRetrievedAt': firstRetrievedAt!.toIso8601String(),
      if (latestRequestReceivedAt != null) 'latestRequestReceivedAt': latestRequestReceivedAt!.toIso8601String(),
    };
  }
}
