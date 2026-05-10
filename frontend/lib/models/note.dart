import 'package:freezed_annotation/freezed_annotation.dart';
import 'user.dart';

part 'note.freezed.dart';
part 'note.g.dart';

@freezed
class Reaction with _$Reaction {
  const factory Reaction({
    required String emoji,
    required int count,
    @Default(false) bool isMyReaction,
  }) = _Reaction;

  factory Reaction.fromJson(Map<String, dynamic> json) => _$ReactionFromJson(json);
}

@freezed
class Note with _$Note {
  const factory Note({
    required String id,
    required DateTime createdAt,
    required String text,
    required User user,
    @Default(0) int repliesCount,
    @Default(0) int renoteCount,
    @Default([]) List<Reaction> reactions,
    Note? reply,
    Note? renote,
    String? cw,
    @Default(false) bool localOnly,
    List<String>? fileIds,
    @Default(false) bool visibleUserIds,
    DateTime? updatedAt,
  }) = _Note;

  factory Note.fromJson(Map<String, dynamic> json) => _$NoteFromJson(json);
}

extension NoteExtension on Note {
  bool get isRenote => renote != null && text.isEmpty;
}
