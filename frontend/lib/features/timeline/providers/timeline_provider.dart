import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/timeline.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/core/mock/mock_data.dart';
import 'package:mithic/core/streaming/websocket_client.dart';
import 'package:mithic/models/note.dart';

part 'timeline_provider.g.dart';

enum TimelineType {
  home,
  local,
  global,
}

@riverpod
class TimelineNotes extends _$TimelineNotes {
  @override
  Future<List<Note>> build() async {
    // モックモードの場合はモックデータを返す
    if (AppConfig.isMockMode) {
      return MockData.mockNotes;
    }
    
    final apiClient = ref.read(apiClientProvider);
    final timelineEndpoints = TimelineEndpoints(apiClient);
    return await timelineEndpoints.homeTimeline();
  }

  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() => build());
  }

  void addNote(Note note) {
    state.whenData((notes) {
      state = AsyncValue.data([note, ...notes]);
    });
  }

  void removeNote(String noteId) {
    state.whenData((notes) {
      state = AsyncValue.data(
        notes.where((note) => note.id != noteId).toList(),
      );
    });
  }
}

@riverpod
class HomeTimeline extends _$HomeTimeline {
  @override
  Future<List<Note>> build() async {
    final notes = await ref.watch(timelineNotesProvider.future);
    
    // モックモードの場合はストリーミングをスキップ
    if (AppConfig.isMockMode) {
      return notes;
    }
    
    // Subscribe to streaming for real-time updates
    final wsClient = ref.read(webSocketClientProvider);
    final subscription = wsClient.events.listen((event) {
      if (event.type == StreamingEventType.note) {
        final note = Note.fromJson(event.data);
        ref.read(timelineNotesProvider.notifier).addNote(note);
      } else if (event.type == StreamingEventType.delete) {
        final noteId = event.data['id'] as String;
        ref.read(timelineNotesProvider.notifier).removeNote(noteId);
      }
    });

    ref.onDispose(() {
      subscription.cancel();
      wsClient.unsubscribe('homeTimeline');
    });

    wsClient.subscribe('homeTimeline', {});

    return notes;
  }

  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() => build());
  }
}

@riverpod
class LocalTimeline extends _$LocalTimeline {
  @override
  Future<List<Note>> build() async {
    // モックモードの場合はモックデータを返す
    if (AppConfig.isMockMode) {
      return MockData.mockNotes;
    }
    
    final apiClient = ref.read(apiClientProvider);
    final timelineEndpoints = TimelineEndpoints(apiClient);
    return await timelineEndpoints.localTimeline();
  }

  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() => build());
  }
}

@riverpod
class GlobalTimeline extends _$GlobalTimeline {
  @override
  Future<List<Note>> build() async {
    // モックモードの場合はモックデータを返す
    if (AppConfig.isMockMode) {
      return MockData.mockNotes;
    }
    
    final apiClient = ref.read(apiClientProvider);
    final timelineEndpoints = TimelineEndpoints(apiClient);
    return await timelineEndpoints.globalTimeline();
  }

  Future<void> refresh() async {
    state = const AsyncValue.loading();
    state = await AsyncValue.guard(() => build());
  }
}
