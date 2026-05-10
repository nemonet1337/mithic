// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'poll.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$PollImpl _$$PollImplFromJson(Map<String, dynamic> json) => _$PollImpl(
      id: json['id'] as String,
      choices: (json['choices'] as List<dynamic>)
          .map((e) => PollOption.fromJson(e as Map<String, dynamic>))
          .toList(),
      multiple: json['multiple'] as bool? ?? false,
      expiresAt: json['expiresAt'] == null
          ? null
          : DateTime.parse(json['expiresAt'] as String),
      expiresAfter: (json['expiresAfter'] as num?)?.toInt() ?? 0,
    );

Map<String, dynamic> _$$PollImplToJson(_$PollImpl instance) =>
    <String, dynamic>{
      'id': instance.id,
      'choices': instance.choices,
      'multiple': instance.multiple,
      'expiresAt': instance.expiresAt?.toIso8601String(),
      'expiresAfter': instance.expiresAfter,
    };

_$PollOptionImpl _$$PollOptionImplFromJson(Map<String, dynamic> json) =>
    _$PollOptionImpl(
      text: json['text'] as String,
      votes: (json['votes'] as num?)?.toInt() ?? 0,
      votesCount: (json['votesCount'] as num?)?.toInt() ?? 0,
      isVoted: json['isVoted'] as bool? ?? false,
    );

Map<String, dynamic> _$$PollOptionImplToJson(_$PollOptionImpl instance) =>
    <String, dynamic>{
      'text': instance.text,
      'votes': instance.votes,
      'votesCount': instance.votesCount,
      'isVoted': instance.isVoted,
    };
