import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/timeline.dart';
import 'package:mithic/core/config/app_config.dart';
import 'package:mithic/core/mock/mock_data.dart';
import 'package:mithic/models/note.dart';

final hashtagTimelineProvider = FutureProvider.family<List<Note>, String>((ref, tag) async {
  if (AppConfig.isMockMode) {
    return MockData.mockNotes;
  }

  final apiClient = ref.watch(apiClientProvider);
  final timelineEndpoints = TimelineEndpoints(apiClient);
  return await timelineEndpoints.hashtagTimeline(tag);
});
