import 'package:mithic/api/client/dio_client.dart';

/// DM会話の概要
class DirectConversation {
  final String id;
  final List<ConversationAccount> accounts;
  final bool unread;
  final ConversationLastMessage? lastMessage;

  const DirectConversation({
    required this.id,
    required this.accounts,
    required this.unread,
    this.lastMessage,
  });

  factory DirectConversation.fromJson(Map<String, dynamic> json) {
    return DirectConversation(
      id: json['id'] as String,
      accounts: (json['accounts'] as List<dynamic>)
          .map((a) => ConversationAccount.fromJson(a as Map<String, dynamic>))
          .toList(),
      unread: json['unread'] as bool? ?? false,
      lastMessage: json['last_status'] != null
          ? ConversationLastMessage.fromJson(
              json['last_status'] as Map<String, dynamic>)
          : null,
    );
  }
}

class ConversationAccount {
  final String id;
  final String username;
  final String? displayName;
  final String? avatarUrl;
  final String? acct;

  const ConversationAccount({
    required this.id,
    required this.username,
    this.displayName,
    this.avatarUrl,
    this.acct,
  });

  String get name => displayName?.isNotEmpty == true ? displayName! : username;

  factory ConversationAccount.fromJson(Map<String, dynamic> json) {
    return ConversationAccount(
      id: json['id'] as String,
      username: json['username'] as String,
      displayName: json['display_name'] as String?,
      avatarUrl: json['avatar'] as String?,
      acct: json['acct'] as String?,
    );
  }
}

class ConversationLastMessage {
  final String id;
  final String content;
  final DateTime createdAt;
  final String accountId;

  const ConversationLastMessage({
    required this.id,
    required this.content,
    required this.createdAt,
    required this.accountId,
  });

  factory ConversationLastMessage.fromJson(Map<String, dynamic> json) {
    return ConversationLastMessage(
      id: json['id'] as String,
      content: _stripHtml(json['content'] as String? ?? ''),
      createdAt: DateTime.parse(json['created_at'] as String),
      accountId: json['account']?['id'] as String? ?? '',
    );
  }

  static String _stripHtml(String html) => html
      .replaceAll(RegExp(r'<[^>]*>'), '')
      .replaceAll('&amp;', '&')
      .replaceAll('&lt;', '<')
      .replaceAll('&gt;', '>')
      .replaceAll('&quot;', '"')
      .trim();
}

/// 1件のDMメッセージ
class DirectMessage {
  final String id;
  final String content;
  final DateTime createdAt;
  final String accountId;
  final bool mine;

  const DirectMessage({
    required this.id,
    required this.content,
    required this.createdAt,
    required this.accountId,
    required this.mine,
  });

  factory DirectMessage.fromJson(Map<String, dynamic> json, String myId) {
    final accountId = json['account']?['id'] as String? ?? '';
    return DirectMessage(
      id: json['id'] as String,
      content: _stripHtml(json['content'] as String? ?? ''),
      createdAt: DateTime.parse(json['created_at'] as String),
      accountId: accountId,
      mine: accountId == myId,
    );
  }

  static String _stripHtml(String html) => html
      .replaceAll(RegExp(r'<[^>]*>'), '')
      .replaceAll('&amp;', '&')
      .replaceAll('&lt;', '<')
      .replaceAll('&gt;', '>')
      .replaceAll('&quot;', '"')
      .trim();
}

class MessagesEndpoints {
  final ApiClient _client;
  MessagesEndpoints(this._client);

  Future<List<DirectConversation>> getConversations({int limit = 20}) async {
    final resp = await _client.get(
      '/api/v1/conversations',
      queryParameters: {'limit': limit},
    );
    final data = resp.data as List<dynamic>? ?? [];
    return data
        .map((e) => DirectConversation.fromJson(e as Map<String, dynamic>))
        .toList();
  }

  Future<List<DirectMessage>> getMessages({
    String? myId,
    int limit = 40,
    String? maxId,
  }) async {
    final resp = await _client.get(
      '/api/v1/timelines/direct',
      queryParameters: {
        'limit': limit,
        if (maxId != null) 'max_id': maxId,
      },
    );
    final data = resp.data as List<dynamic>? ?? [];
    final id = myId ?? '';
    return data
        .map((e) => DirectMessage.fromJson(e as Map<String, dynamic>, id))
        .toList();
  }

  Future<void> sendMessage({
    required String toAcct,
    required String content,
  }) async {
    await _client.post(
      '/api/v1/statuses',
      data: {
        'status': '@$toAcct $content',
        'visibility': 'direct',
      },
    );
  }

  Future<void> deleteConversation(String id) async {
    await _client.delete('/api/v1/conversations/$id');
  }

  Future<void> markRead(String id) async {
    await _client.post('/api/v1/conversations/$id/read');
  }
}
