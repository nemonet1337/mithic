import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/models/notification.dart';

class NotificationsEndpoints {
  final ApiClient _client;

  NotificationsEndpoints(this._client);

  Future<List<Notification>> getNotifications({
    int? limit,
    String? sinceId,
    String? untilId,
    List<NotificationType>? types,
  }) async {
    final response = await _client.get(
      '/api/v1/notifications',
      queryParameters: {
        if (limit != null) 'limit': limit,
        if (sinceId != null) 'since_id': sinceId,
        if (untilId != null) 'until_id': untilId,
        if (types != null) 'types': types.map((t) => t.name).toList(),
      },
    );
    final List<dynamic> data = response.data as List<dynamic>;
    return data.map((json) => Notification.fromJson(json as Map<String, dynamic>)).toList();
  }

  Future<int> getUnreadCount() async {
    final response = await _client.get('/api/v1/notifications/unread_count');
    return response.data['count'] as int;
  }

  Future<void> markAsRead(String id) async {
    await _client.post('/api/v1/notifications/$id/mark_as_read');
  }

  Future<void> markAllAsRead() async {
    await _client.post('/api/v1/notifications/mark_all_as_read');
  }
}
