import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';
import 'conflict_page.dart';
import 'file_page.dart';
import 'name_dialog.dart';
import 'text_page.dart';

/// One Note: its text, what is known about it, its Attachments.
class NotePage extends StatefulWidget {
  const NotePage({super.key, required this.id});

  final String id;

  @override
  State<NotePage> createState() => _NotePageState();
}

class _NotePageState extends State<NotePage> {
  /// The Note's id, which Filing changes.
  late String _id = widget.id;
  Note? _note;

  /// The Sources of a Saved Answer that are still there, by id.
  Map<String, Note> _sources = const {};
  bool _started = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!_started) {
      _started = true;
      unawaited(_reload());
    }
  }

  Future<void> _reload() async {
    final core = AppScope.of(context).core;
    try {
      final note = await core.note(_id);
      final sources = <String, Note>{};
      for (final source in note.sources) {
        try {
          sources[source] = await core.note(source);
        } on CoreError {
          // Moved or deleted since; the page says so.
        }
      }
      if (!mounted) return;
      setState(() {
        _note = note;
        _sources = sources;
      });
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _act(Future<void> Function(Core core) action) async {
    try {
      await action(AppScope.of(context).core);
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
    await _reload();
  }

  Future<void> _edit(Note note) async {
    final text = await Navigator.of(context).push(
      MaterialPageRoute<String>(
        builder: (context) =>
            TextPage(title: context.t.edit, initial: note.text),
      ),
    );
    if (text == null) return;
    await _act((core) => core.setText(note.id, text));
  }

  Future<void> _file(Note note) async {
    final id = await Navigator.of(context).push(
      MaterialPageRoute<String>(builder: (context) => FilePage(note: note)),
    );
    if (id == null) return;
    _id = id;
    await _reload();
  }

  Future<void> _delete(Note note) async {
    final core = AppScope.of(context).core;
    final navigator = Navigator.of(context);
    if (!await confirmDelete(context, context.t.deleteNoteQuestion)) return;
    try {
      await core.delete(note.id);
      navigator.pop();
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  /// Hands the Note to another app: its title and text, and its
  /// Attachments.
  Future<void> _share(Note note) async {
    final scope = AppScope.of(context);
    try {
      final files = [
        for (final attachment in note.attachments)
          await scope.core.attachmentPath(note.id, attachment.name),
      ];
      final text = note.text.trim();
      await scope.phone.share(
        text.startsWith(note.listTitle) ? text : '${note.listTitle}\n\n$text',
        files,
      );
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _push(Widget page) async {
    await Navigator.of(context)
        .push(MaterialPageRoute<void>(builder: (context) => page));
    await _reload();
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final note = _note;
    final text = Theme.of(context).textTheme;
    return Scaffold(
      appBar: AppBar(
        actions: [
          if (note != null) ...[
            IconButton(
              tooltip: t.edit,
              onPressed: () => unawaited(_edit(note)),
              icon: const Icon(Icons.edit_outlined),
            ),
            IconButton(
              tooltip: note.unfiled ? t.fileNote : t.noteDetails,
              onPressed: () => unawaited(_file(note)),
              icon: const Icon(Icons.drive_file_move_outlined),
            ),
            IconButton(
              tooltip: t.share,
              onPressed: () => unawaited(_share(note)),
              icon: const Icon(Icons.share_outlined),
            ),
            IconButton(
              tooltip: t.delete,
              onPressed: () => unawaited(_delete(note)),
              icon: const Icon(Icons.delete_outline),
            ),
          ],
        ],
      ),
      body: note == null
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                if (note.conflicts.isNotEmpty)
                  Card(
                    color: Theme.of(context).colorScheme.errorContainer,
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(t.conflictBanner(note.conflicts.length + 1)),
                          TextButton(
                            onPressed: () =>
                                unawaited(_push(ConflictPage(id: note.id))),
                            child: Text(t.resolveConflict),
                          ),
                        ],
                      ),
                    ),
                  ),
                Text(note.listTitle, style: text.headlineSmall),
                if (note.tags.isNotEmpty)
                  Text(
                    note.tags.map((tag) => '#$tag').join(' '),
                    style: text.bodySmall,
                  ),
                if (note.captured case final captured?)
                  Text(t.capturedOn(when(captured)), style: text.bodySmall),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  value: note.private,
                  title: Text(t.privateMark),
                  // Why it is Private without a mark of its own, or what
                  // the mark does.
                  subtitle: Text(
                    note.isPrivate && !note.private
                        ? (note.unfiled ? t.privateUnfiled : t.privateByFolder)
                        : t.privateExplain,
                  ),
                  onChanged: (private) => unawaited(
                    _act((core) => core.setPrivate(note.id, private: private)),
                  ),
                ),
                const Divider(),
                SelectableText(note.text),
                if (note.model case final model?) ...[
                  const Divider(),
                  Text(t.writtenByModel(model), style: text.bodySmall),
                  for (final source in note.sources)
                    switch (_sources[source]) {
                      final known? => ListTile(
                        contentPadding: EdgeInsets.zero,
                        leading: const Icon(Icons.description_outlined),
                        title: Text(known.title),
                        onTap: () => unawaited(_push(NotePage(id: known.id))),
                      ),
                      null => Text(t.sourceGone(source), style: text.bodySmall),
                    },
                ],
                if (note.link case final link?) ...[
                  const Divider(),
                  SelectableText(link),
                  switch (note.pageText) {
                    null => Text(t.pageFetching, style: text.bodySmall),
                    '' => Text(t.pageTextMissing, style: text.bodySmall),
                    final page => Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(t.pageText, style: text.bodySmall),
                        SelectableText(page),
                      ],
                    ),
                  },
                ],
                if (note.attachments.isNotEmpty) ...[
                  const Divider(),
                  Text(t.attachments, style: text.titleSmall),
                  for (final attachment in note.attachments) ...[
                    Text(attachment.name),
                    switch (attachment.extracted) {
                      null => const SizedBox.shrink(),
                      '' => Text(t.noTextFound, style: text.bodySmall),
                      final extracted => Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(t.extractedText, style: text.bodySmall),
                          SelectableText(extracted),
                        ],
                      ),
                    },
                  ],
                ],
              ],
            ),
    );
  }
}
