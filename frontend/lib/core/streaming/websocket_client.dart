import 'dart:async';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/core/storage/secure_storage.dart';

enum StreamingEventType {
  note,
  reaction,
  delete,
  notification,
}

class StreamingEvent {
  final StreamingEventType type;
  final Map<String, dynamic> data;

  StreamingEvent({required this.type, required this.data});
}

class WebSocketClient {
  WebSocketChannel? _channel;
  final StreamController<StreamingEvent> _eventController =
      StreamController<StreamingEvent>.broadcast();

  Stream<StreamingEvent> get events => _eventController.stream;

  Future<void> connect(String baseUrl, String token) async {
    if (_channel != null) {
      await disconnect();
    }

    final wsUrl = baseUrl.replaceFirst('http', 'ws') + '/streaming';
    _channel = WebSocketChannel.connect(
      Uri.parse(wsUrl),
      protocols: ['misskey-v1'],
    );

    _channel!.stream.listen(
      _handleMessage,
      onError: _handleError,
      onDone: _handleDone,
    );
  }

  void _handleMessage(dynamic message) {
    try {
      final data = message as Map<String, dynamic>;
      final type = data['type'] as String?;
      final body = data['body'] as Map<String, dynamic>?;

      if (type == null || body == null) return;

      StreamingEventType? eventType;
      switch (type) {
        case 'note':
          eventType = StreamingEventType.note;
          break;
        case 'reaction':
          eventType = StreamingEventType.reaction;
          break;
        case 'delete':
          eventType = StreamingEventType.delete;
          break;
        case 'notification':
          eventType = StreamingEventType.notification;
          break;
      }

      if (eventType != null) {
        _eventController.add(StreamingEvent(type: eventType, data: body));
      }
    } catch (e) {
      // エラーを無視して処理を続行
    }
  }

  void _handleError(dynamic error) {
    // エラーハンドリング
  }

  void _handleDone() {
    // 接続終了
  }

  Future<void> disconnect() async {
    await _channel?.sink.close();
    _channel = null;
  }

  void subscribe(String channel, Map<String, dynamic> params) {
    if (_channel == null) return;

    _channel!.sink.add({
      'type': 'connect',
      'body': {
        'channel': channel,
        'id': DateTime.now().millisecondsSinceEpoch.toString(),
        'params': params,
      },
    });
  }

  void unsubscribe(String channelId) {
    if (_channel == null) return;

    _channel!.sink.add({
      'type': 'disconnect',
      'body': {
        'id': channelId,
      },
    });
  }

  void dispose() {
    disconnect();
    _eventController.close();
  }
}

final webSocketClientProvider = Provider<WebSocketClient>((ref) {
  final client = WebSocketClient();
  ref.onDispose(() => client.dispose());
  return client;
});

final streamingConnectionProvider = FutureProvider<void>((ref) async {
  // モックモードの場合は接続をスキップ
  if (AppConfig.isMockMode) {
    return;
  }
  
  final client = ref.watch(webSocketClientProvider);
  final storage = await ref.read(appStorageProvider.future);
  final baseUrl = await storage.getBaseUrl();
  final token = await storage.getAccessToken();

  if (baseUrl != null && token != null) {
    await client.connect(baseUrl, token);
  }
});

final streamingChannelProvider = Provider.family<void, String>((ref, channel) {
  final client = ref.watch(webSocketClientProvider);
  // モックモードの場合はサブスクライブをスキップ
  if (AppConfig.isMockMode) {
    return;
  }
  
  client.subscribe(channel, {});
  
  ref.onDispose(() {
    client.unsubscribe(channel);
  });
});
