// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'relationship.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$RelationshipImpl _$$RelationshipImplFromJson(Map<String, dynamic> json) =>
    _$RelationshipImpl(
      id: json['id'] as String,
      following: json['following'] as bool? ?? false,
      followedBy: json['followedBy'] as bool? ?? false,
      blocking: json['blocking'] as bool? ?? false,
      blockedBy: json['blockedBy'] as bool? ?? false,
      muting: json['muting'] as bool? ?? false,
      mutedBy: json['mutedBy'] as bool? ?? false,
      requested: json['requested'] as bool? ?? false,
      requestedBy: json['requestedBy'] as bool? ?? false,
      followedAt: json['followedAt'] == null
          ? null
          : DateTime.parse(json['followedAt'] as String),
      blockedAt: json['blockedAt'] == null
          ? null
          : DateTime.parse(json['blockedAt'] as String),
      mutedAt: json['mutedAt'] == null
          ? null
          : DateTime.parse(json['mutedAt'] as String),
    );

Map<String, dynamic> _$$RelationshipImplToJson(_$RelationshipImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'following': instance.following,
      'followedBy': instance.followedBy,
      'blocking': instance.blocking,
      'blockedBy': instance.blockedBy,
      'muting': instance.muting,
      'mutedBy': instance.mutedBy,
      'requested': instance.requested,
      'requestedBy': instance.requestedBy,
      'followedAt': instance.followedAt?.toIso8601String(),
      'blockedAt': instance.blockedAt?.toIso8601String(),
      'mutedAt': instance.mutedAt?.toIso8601String(),
    };
