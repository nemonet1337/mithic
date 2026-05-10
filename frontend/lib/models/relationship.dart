import 'package:freezed_annotation/freezed_annotation.dart';

part 'relationship.freezed.dart';
part 'relationship.g.dart';

@freezed
class Relationship with _$Relationship {
  const factory Relationship({
    required String id,
    @Default(false) bool following,
    @Default(false) bool followedBy,
    @Default(false) bool blocking,
    @Default(false) bool blockedBy,
    @Default(false) bool muting,
    @Default(false) bool mutedBy,
    @Default(false) bool requested,
    @Default(false) bool requestedBy,
    DateTime? followedAt,
    DateTime? blockedAt,
    DateTime? mutedAt,
  }) = _Relationship;

  factory Relationship.fromJson(Map<String, dynamic> json) =>
      _$RelationshipFromJson(json);
}
