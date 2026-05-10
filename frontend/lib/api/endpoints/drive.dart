import 'package:dio/dio.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/drive_file.dart';

class DriveEndpoints {
  final ApiClient _client;

  DriveEndpoints(this._client);

  Future<DriveFile> uploadFile(String filePath, {
    String? folderId,
    bool? isSensitive,
    String? force,
  }) async {
    final formData = FormData.fromMap({
      'file': await MultipartFile.fromFile(filePath),
      if (folderId != null) 'folderId': folderId,
      if (isSensitive != null) 'isSensitive': isSensitive,
      if (force != null) 'force': force,
    });

    final response = await _client.post('/api/v1/drive/files/create', data: formData);
    return DriveFile.fromJson(response.data as Map<String, dynamic>);
  }

  Future<DriveFile> uploadFileFromBytes(List<int> bytes, String filename, {
    String? folderId,
    bool? isSensitive,
    String? force,
  }) async {
    final formData = FormData.fromMap({
      'file': MultipartFile.fromBytes(bytes, filename: filename),
      if (folderId != null) 'folderId': folderId,
      if (isSensitive != null) 'isSensitive': isSensitive,
      if (force != null) 'force': force,
    });

    final response = await _client.post('/api/v1/drive/files/create', data: formData);
    return DriveFile.fromJson(response.data as Map<String, dynamic>);
  }

  Future<void> deleteFile(String fileId) async {
    await _client.post('/api/v1/drive/files/delete', data: {
      'fileId': fileId,
    });
  }
}
