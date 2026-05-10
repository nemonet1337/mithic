import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/notifications.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/models/notification.dart';

final notificationsProvider = FutureProvider<List<Notification>>((ref) async {
  if (AppConfig.isMockMode) {
    return [];
  }

  final apiClient = ref.watch(apiClientProvider);
  final notificationsEndpoints = NotificationsEndpoints(apiClient);
  return await notificationsEndpoints.getNotifications();
});

final unreadCountProvider = FutureProvider<int>((ref) async {
  if (AppConfig.isMockMode) {
    return 0;
  }

  final apiClient = ref.watch(apiClientProvider);
  final notificationsEndpoints = NotificationsEndpoints(apiClient);
  return await notificationsEndpoints.getUnreadCount();
});
