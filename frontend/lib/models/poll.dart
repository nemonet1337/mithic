import 'package:freezed_annotation/freezed_annotation.dart';

part 'poll.freezed.dart';
part 'poll.g.dart';

@freezed
class Poll with _$Poll {
  const factory Poll({
    required String id,
    required List<PollOption> choices,
    @Default(false) bool multiple,
    DateTime? expiresAt,
    @Default(0) int expiresAfter,
  }) = _Poll;

  factory Poll.fromJson(Map<String, dynamic> json) => _$PollFromJson(json);
}

@freezed
class PollOption with _$PollOption {
  const factory PollOption({
    required String text,
    @Default(0) int votes,
    @Default(0) int votesCount,
    @Default(false) bool isVoted,
  }) = _PollOption;

  factory PollOption.fromJson(Map<String, dynamic> json) =>
      _$PollOptionFromJson(json);
}
