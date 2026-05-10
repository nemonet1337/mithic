import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/messages.dart';

// ── Endpoints ─────────────────────────────────────────────────────────────────
final messagesEndpointsProvider = Provider<MessagesEndpoints>((ref) {
  final client = ref.watch(apiClientProvider);
  return MessagesEndpoints(client);
});

// ── Conversations list ────────────────────────────────────────────────────────
class ConversationsNotifier
    extends AsyncNotifier<List<DirectConversation>> {
  @override
  Future<List<DirectConversation>> build() async {
    return _fetch();
  }

  Future<List<DirectConversation>> _fetch() async {
    final ep = ref.read(messagesEndpointsProvider);
    return ep.getConversations();
  }

  Future<void> refresh() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(_fetch);
  }
}

final conversationsProvider =
    AsyncNotifierProvider<ConversationsNotifier, List<DirectConversation>>(
  ConversationsNotifier.new,
);

// ── Messages in a conversation ────────────────────────────────────────────────
class MessagesNotifier
    extends FamilyAsyncNotifier<List<DirectMessage>, String> {
  @override
  Future<List<DirectMessage>> build(String conversationId) async {
    return _fetch();
  }

  Future<List<DirectMessage>> _fetch({String? maxId}) async {
    final ep = ref.read(messagesEndpointsProvider);
    return ep.getMessages(maxId: maxId);
  }

  Future<void> refresh() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(_fetch);
  }

  Future<void> sendMessage({
    required String toAcct,
    required String content,
  }) async {
    final ep = ref.read(messagesEndpointsProvider);
    await ep.sendMessage(toAcct: toAcct, content: content);
    await refresh();
  }
}

final messagesProvider =
    AsyncNotifierProviderFamily<MessagesNotifier, List<DirectMessage>, String>(
  MessagesNotifier.new,
);
