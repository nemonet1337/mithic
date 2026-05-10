import 'package:freezed_annotation/freezed_annotation.dart';

part 'drive_file.freezed.dart';
part 'drive_file.g.dart';

@freezed
class DriveFile with _$DriveFile {
  const factory DriveFile({
    required String id,
    required String name,
    required String url,
    String? thumbnailUrl,
    String? type,
    @Default(0) int size,
    String? md5,
    DateTime? createdAt,
    DateTime? uploadedAt,
    String? userId,
    @Default(false) bool isSensitive,
  }) = _DriveFile;

  factory DriveFile.fromJson(Map<String, dynamic> json) =>
      _$DriveFileFromJson(json);
}
