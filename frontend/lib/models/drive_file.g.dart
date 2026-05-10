// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'drive_file.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$DriveFileImpl _$$DriveFileImplFromJson(Map<String, dynamic> json) =>
    _$DriveFileImpl(
      id: json['id'] as String,
      name: json['name'] as String,
      url: json['url'] as String,
      thumbnailUrl: json['thumbnailUrl'] as String?,
      type: json['type'] as String?,
      size: (json['size'] as num?)?.toInt() ?? 0,
      md5: json['md5'] as String?,
      createdAt: json['createdAt'] == null
          ? null
          : DateTime.parse(json['createdAt'] as String),
      uploadedAt: json['uploadedAt'] == null
          ? null
          : DateTime.parse(json['uploadedAt'] as String),
      userId: json['userId'] as String?,
      isSensitive: json['isSensitive'] as bool? ?? false,
    );

Map<String, dynamic> _$$DriveFileImplToJson(_$DriveFileImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'url': instance.url,
      'thumbnailUrl': instance.thumbnailUrl,
      'type': instance.type,
      'size': instance.size,
      'md5': instance.md5,
      'createdAt': instance.createdAt?.toIso8601String(),
      'uploadedAt': instance.uploadedAt?.toIso8601String(),
      'userId': instance.userId,
      'isSensitive': instance.isSensitive,
    };
