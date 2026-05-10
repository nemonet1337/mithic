import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:google_fonts/google_fonts.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mithic/api/client/dio_client.dart';
import 'package:mithic/api/endpoints/drive.dart';
import 'package:mithic/api/endpoints/statuses.dart';
import 'package:mithic/core/theme/tokens/colors.dart';
import 'package:mithic/models/drive_file.dart';
import 'package:mithic/shared/widgets/mithic_widgets.dart';

enum NoteVisibility { public, home, followers, specified }

class ComposeScreen extends ConsumerStatefulWidget {
  const ComposeScreen({super.key});

  @override
  ConsumerState<ComposeScreen> createState() => _ComposeScreenState();
}

class _ComposeScreenState extends ConsumerState<ComposeScreen> {
  final _textCtrl = TextEditingController();
  final _cwCtrl   = TextEditingController();
  bool _cwEnabled    = false;
  bool _submitting   = false;
  bool _localOnly    = false;
  bool _pollEnabled  = false;
  bool _pollMultiple = false;
  NoteVisibility _visibility = NoteVisibility.public;
  final List<DriveFile> _files = [];
  final List<TextEditingController> _pollCtrls = [
    TextEditingController(),
    TextEditingController(),
  ];
  final ImagePicker _picker = ImagePicker();

  @override
  void dispose() {
    _textCtrl.dispose();
    _cwCtrl.dispose();
    for (final c in _pollCtrls) { c.dispose(); }
    super.dispose();
  }

  Future<void> _pickImage() async {
    final picked = await _picker.pickImage(source: ImageSource.gallery);
    if (picked == null || !mounted) return;
    try {
      final file = await DriveEndpoints(ref.read(apiClientProvider)).uploadFile(picked.path);
      setState(() => _files.add(file));
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('アップロード失敗: $e')));
    }
  }

  Future<void> _submit() async {
    final text = _textCtrl.text.trim();
    final cw   = _cwEnabled ? _cwCtrl.text.trim() : null;
    final fileIds = _files.map((f) => f.id).toList();

    PollData? poll;
    if (_pollEnabled) {
      final choices = _pollCtrls.map((c) => c.text.trim()).where((t) => t.isNotEmpty).toList();
      if (choices.length >= 2) poll = PollData(choices: choices, multiple: _pollMultiple);
    }

    if (text.isEmpty && fileIds.isEmpty && poll == null) return;

    setState(() => _submitting = true);
    try {
      await StatusesEndpoints(ref.read(apiClientProvider)).createStatus(
        text: text,
        cw: cw,
        localOnly: _localOnly,
        visibility: _visibility.name,
        fileIds: fileIds.isEmpty ? null : fileIds,
        poll: poll,
      );
      if (mounted) Navigator.of(context).pop();
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('投稿失敗: $e')));
    } finally {
      if (mounted) setState(() => _submitting = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final isDark  = Theme.of(context).brightness == Brightness.dark;
    final ink     = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3    = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final paper   = isDark ? MithicColors.paperDark : MithicColors.paper;
    final accent  = isDark ? MithicColors.accentDark : MithicColors.accent;
    final line    = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    final charCount = _textCtrl.text.length;
    final charLeft  = 500 - charCount;
    final nearLimit = charLeft < 50;

    return Scaffold(
      appBar: MithicTopBar(
        folio: 'new',
        title: 'ノートを作成',
        actions: [
          GestureDetector(
            onTap: _submitting ? null : _submit,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
              decoration: BoxDecoration(
                color: _submitting ? ink3 : accent,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: ink, width: 1.25),
                boxShadow: [BoxShadow(color: ink, offset: const Offset(2, 2))],
              ),
              child: _submitting
                  ? SizedBox(
                      width: 14, height: 14,
                      child: CircularProgressIndicator(strokeWidth: 2, color: paper),
                    )
                  : Text(
                      '投稿',
                      style: GoogleFonts.dmSans(
                        fontSize: 13, fontWeight: FontWeight.w600, color: Colors.white,
                      ),
                    ),
            ),
          ),
          const SizedBox(width: 4),
        ],
      ),
      body: Column(
        children: [
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(14),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // CW field
                  if (_cwEnabled) ...[
                    _FieldBox(
                      child: TextField(
                        controller: _cwCtrl,
                        style: GoogleFonts.dmSans(fontSize: 13.5, color: ink),
                        decoration: InputDecoration(
                          hintText: '警告文（CW）',
                          hintStyle: GoogleFonts.dmSans(fontSize: 13.5, color: ink3),
                          border: InputBorder.none,
                          isDense: true,
                          contentPadding: const EdgeInsets.all(12),
                        ),
                      ),
                      ink: ink, line: line,
                    ),
                    const SizedBox(height: 10),
                  ],
                  // Main text
                  _FieldBox(
                    child: TextField(
                      controller: _textCtrl,
                      maxLines: null,
                      minLines: 6,
                      style: GoogleFonts.dmSans(fontSize: 13.5, color: ink, height: 1.55),
                      onChanged: (_) => setState(() {}),
                      decoration: InputDecoration(
                        hintText: '今どうしてる？',
                        hintStyle: GoogleFonts.dmSans(fontSize: 13.5, color: ink3),
                        border: InputBorder.none,
                        isDense: true,
                        contentPadding: const EdgeInsets.all(12),
                      ),
                    ),
                    ink: ink, line: line,
                  ),
                  // Char counter
                  Align(
                    alignment: Alignment.centerRight,
                    child: Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: Text(
                        '$charLeft',
                        style: GoogleFonts.jetBrainsMono(
                          fontSize: 10,
                          color: nearLimit ? accent : ink3,
                        ),
                      ),
                    ),
                  ),
                  // Attachments
                  if (_files.isNotEmpty) ...[
                    const SizedBox(height: 10),
                    SizedBox(
                      height: 88,
                      child: ListView.builder(
                        scrollDirection: Axis.horizontal,
                        itemCount: _files.length,
                        itemBuilder: (_, i) => _Thumbnail(
                          file: _files[i],
                          onRemove: () => setState(() => _files.removeAt(i)),
                          ink: ink,
                        ),
                      ),
                    ),
                  ],
                  // Poll
                  if (_pollEnabled) ...[
                    const SizedBox(height: 10),
                    _PollSection(
                      controllers: _pollCtrls,
                      multiple: _pollMultiple,
                      onToggleMultiple: (v) => setState(() => _pollMultiple = v),
                      onAddOption: () => setState(() => _pollCtrls.add(TextEditingController())),
                      onRemoveOption: (i) => setState(() {
                        _pollCtrls[i].dispose();
                        _pollCtrls.removeAt(i);
                      }),
                      ink: ink, ink3: ink3, line: line, accent: accent,
                    ),
                  ],
                ],
              ),
            ),
          ),
          // Toolbar
          _Toolbar(
            cwEnabled: _cwEnabled,
            pollEnabled: _pollEnabled,
            localOnly: _localOnly,
            visibility: _visibility,
            onCw: () => setState(() => _cwEnabled = !_cwEnabled),
            onImage: _pickImage,
            onPoll: () => setState(() => _pollEnabled = !_pollEnabled),
            onLocalOnly: () => setState(() => _localOnly = !_localOnly),
            onVisibility: _showVisibilitySheet,
            ink: ink, ink3: ink3, line: line, accent: accent, paper: paper,
          ),
        ],
      ),
    );
  }

  void _showVisibilitySheet() {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    final ink    = isDark ? MithicColors.inkDark   : MithicColors.ink;
    final ink3   = isDark ? MithicColors.ink3Dark  : MithicColors.ink3;
    final paper  = isDark ? MithicColors.paperDark : MithicColors.paper;
    final line   = isDark ? const Color(0x38F3EFE6) : MithicColors.lineSoft;

    showModalBottomSheet(
      context: context,
      backgroundColor: paper,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(16)),
      ),
      builder: (_) => Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            MithicLabel('公開範囲'),
            const SizedBox(height: 12),
            ...NoteVisibility.values.map((v) {
              final lbl  = _visibilityLabel(v);
              final desc = _visibilityDesc(v);
              final on   = _visibility == v;
              return GestureDetector(
                onTap: () {
                  setState(() => _visibility = v);
                  Navigator.of(context).pop();
                },
                child: Container(
                  padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 4),
                  decoration: BoxDecoration(
                    border: Border(bottom: BorderSide(color: line, width: 1)),
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 14, height: 14,
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          color: on ? ink : Colors.transparent,
                          border: Border.all(color: ink, width: 1.25),
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(lbl, style: GoogleFonts.dmSans(fontSize: 14, color: ink)),
                            Text(desc, style: GoogleFonts.dmSans(fontSize: 11, color: ink3)),
                          ],
                        ),
                      ),
                    ],
                  ),
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  String _visibilityLabel(NoteVisibility v) => switch (v) {
    NoteVisibility.public    => '公開',
    NoteVisibility.home      => 'ホーム',
    NoteVisibility.followers => 'フォロワー',
    NoteVisibility.specified => '指定',
  };

  String _visibilityDesc(NoteVisibility v) => switch (v) {
    NoteVisibility.public    => '全員に公開',
    NoteVisibility.home      => 'ホームタイムラインのみ',
    NoteVisibility.followers => 'フォロワーのみ',
    NoteVisibility.specified => '指定ユーザーのみ',
  };
}

// ── Sub-widgets ───────────────────────────────────────────────────────────────

class _FieldBox extends StatelessWidget {
  final Widget child;
  final Color ink;
  final Color line;

  const _FieldBox({required this.child, required this.ink, required this.line});

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: ink, width: 1.25),
        borderRadius: BorderRadius.circular(8),
        boxShadow: [BoxShadow(color: ink, offset: const Offset(3, 3))],
      ),
      child: child,
    );
  }
}

class _Thumbnail extends StatelessWidget {
  final DriveFile file;
  final VoidCallback onRemove;
  final Color ink;

  const _Thumbnail({required this.file, required this.onRemove, required this.ink});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(right: 8),
      child: Stack(
        children: [
          ClipRRect(
            borderRadius: BorderRadius.circular(8),
            child: Image.network(
              file.url, width: 88, height: 88, fit: BoxFit.cover,
              errorBuilder: (_, __, ___) => Container(
                width: 88, height: 88, color: const Color(0xFFE8E4DC),
                child: const Icon(Icons.broken_image, color: Color(0xFF8A8074)),
              ),
            ),
          ),
          Positioned(
            top: 4, right: 4,
            child: GestureDetector(
              onTap: onRemove,
              child: Container(
                padding: const EdgeInsets.all(3),
                decoration: const BoxDecoration(
                  color: Color(0xFF1A1714),
                  shape: BoxShape.circle,
                ),
                child: const Icon(Icons.close, size: 12, color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _PollSection extends StatelessWidget {
  final List<TextEditingController> controllers;
  final bool multiple;
  final ValueChanged<bool> onToggleMultiple;
  final VoidCallback onAddOption;
  final ValueChanged<int> onRemoveOption;
  final Color ink, ink3, line, accent;

  const _PollSection({
    required this.controllers,
    required this.multiple,
    required this.onToggleMultiple,
    required this.onAddOption,
    required this.onRemoveOption,
    required this.ink,
    required this.ink3,
    required this.line,
    required this.accent,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        MithicLabel('アンケート'),
        const SizedBox(height: 8),
        ...controllers.asMap().entries.map((e) => Padding(
          padding: const EdgeInsets.only(bottom: 6),
          child: Row(
            children: [
              Expanded(
                child: Container(
                  decoration: BoxDecoration(
                    border: Border.all(color: ink, width: 1.25),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: TextField(
                    controller: e.value,
                    style: GoogleFonts.dmSans(fontSize: 13, color: ink),
                    decoration: InputDecoration(
                      hintText: '選択肢 ${e.key + 1}',
                      hintStyle: GoogleFonts.dmSans(fontSize: 13, color: ink3),
                      border: InputBorder.none,
                      isDense: true,
                      contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    ),
                  ),
                ),
              ),
              if (controllers.length > 2) ...[
                const SizedBox(width: 6),
                GestureDetector(
                  onTap: () => onRemoveOption(e.key),
                  child: Icon(Icons.close, size: 16, color: ink3),
                ),
              ],
            ],
          ),
        )),
        if (controllers.length < 10)
          GestureDetector(
            onTap: onAddOption,
            child: Padding(
              padding: const EdgeInsets.only(bottom: 8),
              child: Text(
                '+ 選択肢を追加',
                style: GoogleFonts.dmSans(fontSize: 13, color: accent),
              ),
            ),
          ),
        GestureDetector(
          onTap: () => onToggleMultiple(!multiple),
          child: Row(
            children: [
              Container(
                width: 14, height: 14,
                decoration: BoxDecoration(
                  border: Border.all(color: ink, width: 1.25),
                  borderRadius: BorderRadius.circular(3),
                  color: multiple ? ink : Colors.transparent,
                ),
              ),
              const SizedBox(width: 8),
              Text('複数回答可', style: GoogleFonts.dmSans(fontSize: 13, color: ink)),
            ],
          ),
        ),
      ],
    );
  }
}

class _Toolbar extends StatelessWidget {
  final bool cwEnabled;
  final bool pollEnabled;
  final bool localOnly;
  final NoteVisibility visibility;
  final VoidCallback onCw;
  final VoidCallback onImage;
  final VoidCallback onPoll;
  final VoidCallback onLocalOnly;
  final VoidCallback onVisibility;
  final Color ink, ink3, line, accent, paper;

  const _Toolbar({
    required this.cwEnabled,
    required this.pollEnabled,
    required this.localOnly,
    required this.visibility,
    required this.onCw,
    required this.onImage,
    required this.onPoll,
    required this.onLocalOnly,
    required this.onVisibility,
    required this.ink,
    required this.ink3,
    required this.line,
    required this.accent,
    required this.paper,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 16),
      decoration: BoxDecoration(
        color: paper,
        border: Border(top: BorderSide(color: line, width: 1.25)),
      ),
      child: Row(
        children: [
          _Btn(icon: cwEnabled ? Icons.visibility : Icons.visibility_off,
            active: cwEnabled, onTap: onCw, ink: ink, ink3: ink3, line: line, accent: accent),
          const SizedBox(width: 8),
          _Btn(icon: Icons.image_outlined, onTap: onImage, ink: ink, ink3: ink3, line: line, accent: accent),
          const SizedBox(width: 8),
          _Btn(icon: pollEnabled ? Icons.poll : Icons.poll_outlined,
            active: pollEnabled, onTap: onPoll, ink: ink, ink3: ink3, line: line, accent: accent),
          const SizedBox(width: 8),
          _Btn(icon: localOnly ? Icons.lock_outline : Icons.public_outlined,
            active: localOnly, onTap: onLocalOnly, ink: ink, ink3: ink3, line: line, accent: accent),
          const Spacer(),
          GestureDetector(
            onTap: onVisibility,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
              decoration: BoxDecoration(
                border: Border.all(color: ink.withValues(alpha: 0.3), width: 1),
                borderRadius: BorderRadius.circular(6),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    _visLabel(visibility),
                    style: GoogleFonts.jetBrainsMono(fontSize: 10, color: ink3, letterSpacing: 0.1),
                  ),
                  const SizedBox(width: 4),
                  Icon(Icons.expand_more, size: 14, color: ink3),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  String _visLabel(NoteVisibility v) => switch (v) {
    NoteVisibility.public    => '公開',
    NoteVisibility.home      => 'ホーム',
    NoteVisibility.followers => 'フォロワー',
    NoteVisibility.specified => '指定',
  };
}

class _Btn extends StatelessWidget {
  final IconData icon;
  final bool active;
  final VoidCallback onTap;
  final Color ink, ink3, line, accent;

  const _Btn({
    required this.icon,
    required this.onTap,
    required this.ink,
    required this.ink3,
    required this.line,
    required this.accent,
    this.active = false,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.all(7),
        decoration: BoxDecoration(
          color: active ? accent.withValues(alpha: 0.1) : Colors.transparent,
          border: Border.all(
            color: active ? accent : ink.withValues(alpha: 0.25),
            width: 1,
          ),
          borderRadius: BorderRadius.circular(7),
        ),
        child: Icon(icon, size: 18, color: active ? accent : ink3),
      ),
    );
  }
}
