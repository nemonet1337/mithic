import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/models/emoji.dart';

class MfmText extends ConsumerWidget {
  final String text;
  final List<Emoji>? emojis;

  const MfmText({
    super.key,
    required this.text,
    this.emojis,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return _parseMfm(text);
  }

  Widget _parseMfm(String text) {
    final spans = <InlineSpan>[];
    int index = 0;

    while (index < text.length) {
      // Check for bold **text**
      if (text.substring(index).startsWith('**')) {
        final endBold = text.indexOf('**', index + 2);
        if (endBold != -1) {
          spans.add(
            TextSpan(
              text: text.substring(index + 2, endBold),
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          );
          index = endBold + 2;
          continue;
        }
      }

      // Check for italic *text*
      if (text.substring(index).startsWith('*') && !text.substring(index).startsWith('**')) {
        final endItalic = text.indexOf('*', index + 1);
        if (endItalic != -1) {
          spans.add(
            TextSpan(
              text: text.substring(index + 1, endItalic),
              style: const TextStyle(fontStyle: FontStyle.italic),
            ),
          );
          index = endItalic + 1;
          continue;
        }
      }

      // Check for code `text`
      if (text.substring(index).startsWith('`')) {
        final endCode = text.indexOf('`', index + 1);
        if (endCode != -1) {
          spans.add(
            TextSpan(
              text: text.substring(index + 1, endCode),
              style: TextStyle(
                fontFamily: 'monospace',
                backgroundColor: Colors.grey[300],
              ),
            ),
          );
          index = endCode + 1;
          continue;
        }
      }

      // Check for strike ~~text~~
      if (text.substring(index).startsWith('~~')) {
        final endStrike = text.indexOf('~~', index + 2);
        if (endStrike != -1) {
          spans.add(
            TextSpan(
              text: text.substring(index + 2, endStrike),
              style: const TextStyle(decoration: TextDecoration.lineThrough),
            ),
          );
          index = endStrike + 2;
          continue;
        }
      }

      // Check for custom emoji :emoji:
      if (text.substring(index).startsWith(':')) {
        final endEmoji = text.indexOf(':', index + 1);
        if (endEmoji != -1) {
          final emojiName = text.substring(index + 1, endEmoji);
          final emoji = emojis?.firstWhere(
            (e) => e.name == emojiName,
            orElse: () => Emoji(id: emojiName, name: emojiName, url: ''),
          );
          final emojiUrl = emoji.url;
          if (emojiUrl != null && emojiUrl.isNotEmpty) {
            spans.add(
              WidgetSpan(
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 2),
                  child: Image.network(
                    emojiUrl,
                    width: 20,
                    height: 20,
                    errorBuilder: (context, error, stackTrace) {
                      return Text(':$emojiName:');
                    },
                  ),
                ),
              ),
            );
          } else {
            spans.add(TextSpan(text: ':$emojiName:'));
          }
          index = endEmoji + 1;
          continue;
        }
      }

      // Check for mention @username
      if (text.substring(index).startsWith('@')) {
        final spaceIndex = text.indexOf(' ', index);
        if (spaceIndex != -1) {
          final mention = text.substring(index, spaceIndex);
          spans.add(
            TextSpan(
              text: mention,
              style: const TextStyle(
                color: Colors.blue,
                decoration: TextDecoration.underline,
              ),
            ),
          );
          index = spaceIndex;
          continue;
        }
      }

      // Check for hashtag #tag
      if (text.substring(index).startsWith('#')) {
        final spaceIndex = text.indexOf(' ', index);
        if (spaceIndex != -1) {
          final tag = text.substring(index, spaceIndex);
          spans.add(
            TextSpan(
              text: tag,
              style: const TextStyle(
                color: Colors.blue,
                decoration: TextDecoration.underline,
              ),
            ),
          );
          index = spaceIndex;
          continue;
        }
      }

      // Regular text
      spans.add(TextSpan(text: text[index]));
      index++;
    }

    return RichText(
      text: TextSpan(
        style: const TextStyle(color: Colors.black),
        children: spans,
      ),
    );
  }
}
