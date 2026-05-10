import 'package:freezed_annotation/freezed_annotation.dart';
import 'note.dart';
import 'user.dart';

part 'notification.freezed.dart';
part 'notification.g.dart';

enum NotificationType {
  mention,
  reply,
  renote,
  quote,
  reaction,
  follow,
  followRequestAccepted,
  followRequest,
  pollVote,
  pollEnded,
}

@freezed
class Notification with _$Notification {
  const factory Notification({
    required String id,
    required NotificationType type,
    required DateTime createdAt,
    User? user,
    Note? note,
    String? reaction,
    @Default(false) bool isRead,
  }) = _Notification;

  factory Notification.fromJson(Map<String, dynamic> json) =>
      _$NotificationFromJson(json);
}
