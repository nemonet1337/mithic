// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'emoji.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$EmojiImpl _$$EmojiImplFromJson(Map<String, dynamic> json) => _$EmojiImpl(
      id: json['id'] as String,
      name: json['name'] as String,
      url: json['url'] as String?,
      category: json['category'] as String?,
      isLocal: json['isLocal'] as bool? ?? false,
    );

Map<String, dynamic> _$$EmojiImplToJson(_$EmojiImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'url': instance.url,
      'category': instance.category,
      'isLocal': instance.isLocal,
    };
