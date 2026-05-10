import 'package:mithic/models/user.dart';
import 'package:mithic/models/note.dart';

/// モックデータ生成クラス
class MockData {
  static final User mockCurrentUser = User(
    id: 'mock_user_1',
    username: 'mockuser',
    name: 'モックユーザー',
    avatarUrl: null,
    bio: 'これはモックユーザーです',
    isBot: false,
    isCat: false,
    createdAt: DateTime.now().subtract(const Duration(days: 365)),
    updatedAt: DateTime.now(),
  );

  static final List<User> mockUsers = [
    mockCurrentUser,
    User(
      id: 'mock_user_2',
      username: 'alice',
      name: 'アリス',
      avatarUrl: null,
      bio: 'こんにちは！',
      isBot: false,
      isCat: false,
      createdAt: DateTime.now().subtract(const Duration(days: 180)),
      updatedAt: DateTime.now(),
    ),
    User(
      id: 'mock_user_3',
      username: 'bob',
      name: 'ボブ',
      avatarUrl: null,
      bio: 'エンジニア',
      isBot: false,
      isCat: true,
      createdAt: DateTime.now().subtract(const Duration(days: 90)),
      updatedAt: DateTime.now(),
    ),
  ];

  static final List<Note> mockNotes = [
    Note(
      id: 'note_1',
      createdAt: DateTime.now().subtract(const Duration(minutes: 5)),
      text: 'こんにちは、これは最初のモックノートです！',
      user: mockUsers[1],
      repliesCount: 3,
      renoteCount: 2,
      reactions: [
        Reaction(
          emoji: '👍',
          count: 5,
          isMyReaction: true,
        ),
      ],
    ),
    Note(
      id: 'note_2',
      createdAt: DateTime.now().subtract(const Duration(hours: 1)),
      text: '今日は良い天気ですね',
      user: mockUsers[2],
      repliesCount: 1,
      renoteCount: 0,
      reactions: [
        Reaction(
          emoji: '❤️',
          count: 3,
          isMyReaction: false,
        ),
        Reaction(
          emoji: '😊',
          count: 2,
          isMyReaction: true,
        ),
      ],
    ),
    Note(
      id: 'note_3',
      createdAt: DateTime.now().subtract(const Duration(hours: 2)),
      text: 'Flutterでアプリ開発をしています',
      user: mockCurrentUser,
      repliesCount: 0,
      renoteCount: 1,
      reactions: [],
    ),
    Note(
      id: 'note_4',
      createdAt: DateTime.now().subtract(const Duration(hours: 3)),
      text: '新しい機能をリリースしました！',
      user: mockUsers[1],
      repliesCount: 5,
      renoteCount: 10,
      reactions: [
        Reaction(
          emoji: '🎉',
          count: 20,
          isMyReaction: true,
        ),
      ],
    ),
    Note(
      id: 'note_5',
      createdAt: DateTime.now().subtract(const Duration(hours: 5)),
      text: 'リノート',
      user: mockUsers[2],
      repliesCount: 0,
      renoteCount: 0,
      reactions: [],
      renote: Note(
        id: 'note_6',
        createdAt: DateTime.now().subtract(const Duration(hours: 6)),
        text: 'これはリノートされた元のノートです',
        user: mockUsers[1],
        repliesCount: 2,
        renoteCount: 1,
        reactions: [],
      ),
    ),
    Note(
      id: 'note_7',
      createdAt: DateTime.now().subtract(const Duration(hours: 8)),
      text: 'CW付きのノート',
      user: mockUsers[0],
      repliesCount: 1,
      renoteCount: 0,
      reactions: [],
      cw: 'ネタバレ',
    ),
    Note(
      id: 'note_8',
      createdAt: DateTime.now().subtract(const Duration(days: 1)),
      text: '昨日の出来事について',
      user: mockUsers[1],
      repliesCount: 0,
      renoteCount: 0,
      reactions: [],
    ),
    Note(
      id: 'note_9',
      createdAt: DateTime.now().subtract(const Duration(days: 2)),
      text: '週末は何をしますか？',
      user: mockUsers[2],
      repliesCount: 4,
      renoteCount: 0,
      reactions: [
        Reaction(
          emoji: '🤔',
          count: 1,
          isMyReaction: false,
        ),
      ],
    ),
    Note(
      id: 'note_10',
      createdAt: DateTime.now().subtract(const Duration(days: 3)),
      text: 'プログラミングは楽しい！',
      user: mockCurrentUser,
      repliesCount: 2,
      renoteCount: 3,
      reactions: [
        Reaction(
          emoji: '💻',
          count: 7,
          isMyReaction: true,
        ),
        Reaction(
          emoji: '🚀',
          count: 4,
          isMyReaction: false,
        ),
      ],
    ),
  ];

  static Note generateRandomNote() {
    final randomUser = mockUsers[DateTime.now().millisecond % mockUsers.length];
    final texts = [
      'こんにちは！',
      '今日は良い一日です',
      '何か面白いことあった？',
      '新しいことを学んでいます',
      'コードを書いています',
    ];
    
    return Note(
      id: 'note_${DateTime.now().millisecondsSinceEpoch}',
      createdAt: DateTime.now(),
      text: texts[DateTime.now().millisecond % texts.length],
      user: randomUser,
      repliesCount: DateTime.now().millisecond % 5,
      renoteCount: DateTime.now().millisecond % 3,
      reactions: [],
    );
  }
}
