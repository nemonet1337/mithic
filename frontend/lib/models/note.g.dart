// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'note.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$ReactionImpl _$$ReactionImplFromJson(Map<String, dynamic> json) =>
    _$ReactionImpl(
      emoji: json['emoji'] as String,
      count: (json['count'] as num).toInt(),
      isMyReaction: json['isMyReaction'] as bool? ?? false,
    );

Map<String, dynamic> _$$ReactionImplToJson(_$ReactionImpl instance) =>
    <String, dynamic>{
      'emoji': instance.emoji,
      'count': instance.count,
      'isMyReaction': instance.isMyReaction,
    };

_$NoteImpl _$$NoteImplFromJson(Map<String, dynamic> json) => _$NoteImpl(
      id: json['id'] as String,
      createdAt: DateTime.parse(json['createdAt'] as String),
      text: json['text'] as String,
      user: User.fromJson(json['user'] as Map<String, dynamic>),
      repliesCount: (json['repliesCount'] as num?)?.toInt() ?? 0,
      renoteCount: (json['renoteCount'] as num?)?.toInt() ?? 0,
      reactions: (json['reactions'] as List<dynamic>?)
              ?.map((e) => Reaction.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      reply: json['reply'] == null
          ? null
          : Note.fromJson(json['reply'] as Map<String, dynamic>),
      renote: json['renote'] == null
          ? null
          : Note.fromJson(json['renote'] as Map<String, dynamic>),
      cw: json['cw'] as String?,
      localOnly: json['localOnly'] as bool? ?? false,
      fileIds:
          (json['fileIds'] as List<dynamic>?)?.map((e) => e as String).toList(),
      visibleUserIds: json['visibleUserIds'] as bool? ?? false,
      updatedAt: json['updatedAt'] == null
          ? null
          : DateTime.parse(json['updatedAt'] as String),
    );

Map<String, dynamic> _$$NoteImplToJson(_$NoteImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'createdAt': instance.createdAt.toIso8601String(),
      'text': instance.text,
      'user': instance.user,
      'repliesCount': instance.repliesCount,
      'renoteCount': instance.renoteCount,
      'reactions': instance.reactions,
      'reply': instance.reply,
      'renote': instance.renote,
      'cw': instance.cw,
      'localOnly': instance.localOnly,
      'fileIds': instance.fileIds,
      'visibleUserIds': instance.visibleUserIds,
      'updatedAt': instance.updatedAt?.toIso8601String(),
    };
