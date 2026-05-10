// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'relationship.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

Relationship _$RelationshipFromJson(Map<String, dynamic> json) {
  return _Relationship.fromJson(json);
}

/// @nodoc
mixin _$Relationship {
  String get id => throw _privateConstructorUsedError;
  bool get following => throw _privateConstructorUsedError;
  bool get followedBy => throw _privateConstructorUsedError;
  bool get blocking => throw _privateConstructorUsedError;
  bool get blockedBy => throw _privateConstructorUsedError;
  bool get muting => throw _privateConstructorUsedError;
  bool get mutedBy => throw _privateConstructorUsedError;
  bool get requested => throw _privateConstructorUsedError;
  bool get requestedBy => throw _privateConstructorUsedError;
  DateTime? get followedAt => throw _privateConstructorUsedError;
  DateTime? get blockedAt => throw _privateConstructorUsedError;
  DateTime? get mutedAt => throw _privateConstructorUsedError;

  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
  @JsonKey(ignore: true)
  $RelationshipCopyWith<Relationship> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $RelationshipCopyWith<$Res> {
  factory $RelationshipCopyWith(
          Relationship value, $Res Function(Relationship) then) =
      _$RelationshipCopyWithImpl<$Res, Relationship>;
  @useResult
  $Res call(
      {String id,
      bool following,
      bool followedBy,
      bool blocking,
      bool blockedBy,
      bool muting,
      bool mutedBy,
      bool requested,
      bool requestedBy,
      DateTime? followedAt,
      DateTime? blockedAt,
      DateTime? mutedAt});
}

/// @nodoc
class _$RelationshipCopyWithImpl<$Res, $Val extends Relationship>
    implements $RelationshipCopyWith<$Res> {
  _$RelationshipCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? following = null,
    Object? followedBy = null,
    Object? blocking = null,
    Object? blockedBy = null,
    Object? muting = null,
    Object? mutedBy = null,
    Object? requested = null,
    Object? requestedBy = null,
    Object? followedAt = freezed,
    Object? blockedAt = freezed,
    Object? mutedAt = freezed,
  }) {
    return _then(_value.copyWith(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      following: null == following
          ? _value.following
          : following // ignore: cast_nullable_to_non_nullable
              as bool,
      followedBy: null == followedBy
          ? _value.followedBy
          : followedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      blocking: null == blocking
          ? _value.blocking
          : blocking // ignore: cast_nullable_to_non_nullable
              as bool,
      blockedBy: null == blockedBy
          ? _value.blockedBy
          : blockedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      muting: null == muting
          ? _value.muting
          : muting // ignore: cast_nullable_to_non_nullable
              as bool,
      mutedBy: null == mutedBy
          ? _value.mutedBy
          : mutedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      requested: null == requested
          ? _value.requested
          : requested // ignore: cast_nullable_to_non_nullable
              as bool,
      requestedBy: null == requestedBy
          ? _value.requestedBy
          : requestedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      followedAt: freezed == followedAt
          ? _value.followedAt
          : followedAt // ignore: cast_nullable_to_non_nullable
              as DateTime?,
      blockedAt: freezed == blockedAt
          ? _value.blockedAt
          : blockedAt // ignore: cast_nullable_to_non_nullable
              as DateTime?,
      mutedAt: freezed == mutedAt
          ? _value.mutedAt
          : mutedAt // ignore: cast_nullable_to_non_nullable
              as DateTime?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$RelationshipImplCopyWith<$Res>
    implements $RelationshipCopyWith<$Res> {
  factory _$$RelationshipImplCopyWith(
          _$RelationshipImpl value, $Res Function(_$RelationshipImpl) then) =
      __$$RelationshipImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String id,
      bool following,
      bool followedBy,
      bool blocking,
      bool blockedBy,
      bool muting,
      bool mutedBy,
      bool requested,
      bool requestedBy,
      DateTime? followedAt,
      DateTime? blockedAt,
      DateTime? mutedAt});
}

/// @nodoc
class __$$RelationshipImplCopyWithImpl<$Res>
    extends _$RelationshipCopyWithImpl<$Res, _$RelationshipImpl>
    implements _$$RelationshipImplCopyWith<$Res> {
  __$$RelationshipImplCopyWithImpl(
      _$RelationshipImpl _value, $Res Function(_$RelationshipImpl) _then)
      : super(_value, _then);

  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? id = null,
    Object? following = null,
    Object? followedBy = null,
    Object? blocking = null,
    Object? blockedBy = null,
    Object? muting = null,
    Object? mutedBy = null,
    Object? requested = null,
    Object? requestedBy = null,
    Object? followedAt = freezed,
    Object? blockedAt = freezed,
    Object? mutedAt = freezed,
  }) {
    return _then(_$RelationshipImpl(
      id: null == id
          ? _value.id
          : id // ignore: cast_nullable_to_non_nullable
              as String,
      following: null == following
          ? _value.following
          : following // ignore: cast_nullable_to_non_nullable
              as bool,
      followedBy: null == followedBy
          ? _value.followedBy
          : followedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      blocking: null == blocking
          ? _value.blocking
          : blocking // ignore: cast_nullable_to_non_nullable
              as bool,
      blockedBy: null == blockedBy
          ? _value.blockedBy
          : blockedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      muting: null == muting
          ? _value.muting
          : muting // ignore: cast_nullable_to_non_nullable
              as bool,
      mutedBy: null == mutedBy
          ? _value.mutedBy
          : mutedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      requested: null == requested
          ? _value.requested
          : requested // ignore: cast_nullable_to_non_nullable
              as bool,
      requestedBy: null == requestedBy
          ? _value.requestedBy
          : requestedBy // ignore: cast_nullable_to_non_nullable
              as bool,
      followedAt: freezed == followedAt
          ? _value.followedAt
          : followedAt // ignore: cast_nullable_to_non_nullable
              as DateTime?,
      blockedAt: freezed == blockedAt
          ? _value.blockedAt
          : blockedAt // ignore: cast_nullable_to_non_nullable
              as DateTime?,
      mutedAt: freezed == mutedAt
          ? _value.mutedAt
          : mutedAt // ignore: cast_nullable_to_non_nullable
              as DateTime?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$RelationshipImpl implements _Relationship {
  const _$RelationshipImpl(
      {required this.id,
      this.following = false,
      this.followedBy = false,
      this.blocking = false,
      this.blockedBy = false,
      this.muting = false,
      this.mutedBy = false,
      this.requested = false,
      this.requestedBy = false,
      this.followedAt,
      this.blockedAt,
      this.mutedAt});

  factory _$RelationshipImpl.fromJson(Map<String, dynamic> json) =>
      _$$RelationshipImplFromJson(json);

  @override
  final String id;
  @override
  @JsonKey()
  final bool following;
  @override
  @JsonKey()
  final bool followedBy;
  @override
  @JsonKey()
  final bool blocking;
  @override
  @JsonKey()
  final bool blockedBy;
  @override
  @JsonKey()
  final bool muting;
  @override
  @JsonKey()
  final bool mutedBy;
  @override
  @JsonKey()
  final bool requested;
  @override
  @JsonKey()
  final bool requestedBy;
  @override
  final DateTime? followedAt;
  @override
  final DateTime? blockedAt;
  @override
  final DateTime? mutedAt;

  @override
  String toString() {
    return 'Relationship(id: $id, following: $following, followedBy: $followedBy, blocking: $blocking, blockedBy: $blockedBy, muting: $muting, mutedBy: $mutedBy, requested: $requested, requestedBy: $requestedBy, followedAt: $followedAt, blockedAt: $blockedAt, mutedAt: $mutedAt)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RelationshipImpl &&
            (identical(other.id, id) || other.id == id) &&
            (identical(other.following, following) ||
                other.following == following) &&
            (identical(other.followedBy, followedBy) ||
                other.followedBy == followedBy) &&
            (identical(other.blocking, blocking) ||
                other.blocking == blocking) &&
            (identical(other.blockedBy, blockedBy) ||
                other.blockedBy == blockedBy) &&
            (identical(other.muting, muting) || other.muting == muting) &&
            (identical(other.mutedBy, mutedBy) || other.mutedBy == mutedBy) &&
            (identical(other.requested, requested) ||
                other.requested == requested) &&
            (identical(other.requestedBy, requestedBy) ||
                other.requestedBy == requestedBy) &&
            (identical(other.followedAt, followedAt) ||
                other.followedAt == followedAt) &&
            (identical(other.blockedAt, blockedAt) ||
                other.blockedAt == blockedAt) &&
            (identical(other.mutedAt, mutedAt) || other.mutedAt == mutedAt));
  }

  @JsonKey(ignore: true)
  @override
  int get hashCode => Object.hash(
      runtimeType,
      id,
      following,
      followedBy,
      blocking,
      blockedBy,
      muting,
      mutedBy,
      requested,
      requestedBy,
      followedAt,
      blockedAt,
      mutedAt);

  @JsonKey(ignore: true)
  @override
  @pragma('vm:prefer-inline')
  _$$RelationshipImplCopyWith<_$RelationshipImpl> get copyWith =>
      __$$RelationshipImplCopyWithImpl<_$RelationshipImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$RelationshipImplToJson(
      this,
    );
  }
}

abstract class _Relationship implements Relationship {
  const factory _Relationship(
      {required final String id,
      final bool following,
      final bool followedBy,
      final bool blocking,
      final bool blockedBy,
      final bool muting,
      final bool mutedBy,
      final bool requested,
      final bool requestedBy,
      final DateTime? followedAt,
      final DateTime? blockedAt,
      final DateTime? mutedAt}) = _$RelationshipImpl;

  factory _Relationship.fromJson(Map<String, dynamic> json) =
      _$RelationshipImpl.fromJson;

  @override
  String get id;
  @override
  bool get following;
  @override
  bool get followedBy;
  @override
  bool get blocking;
  @override
  bool get blockedBy;
  @override
  bool get muting;
  @override
  bool get mutedBy;
  @override
  bool get requested;
  @override
  bool get requestedBy;
  @override
  DateTime? get followedAt;
  @override
  DateTime? get blockedAt;
  @override
  DateTime? get mutedAt;
  @override
  @JsonKey(ignore: true)
  _$$RelationshipImplCopyWith<_$RelationshipImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
