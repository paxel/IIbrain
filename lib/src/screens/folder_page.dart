import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';
import 'note_page.dart';

/// The Notes of one Folder.
class FolderPage extends StatefulWidget {
  const FolderPage({super.key, required this.folder});

  final String folder;

  @override
  State<FolderPage> createState() => _FolderPageState();
}

class _FolderPageState extends State<FolderPage> {
  List<Note>? _notes;

  /// Whether Unfiled Notes count as Private.
  bool _unfiledPrivate = true;
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
    try {
      final core = AppScope.of(context).core;
      final notes = await core.notes(widget.folder);
      final state = await core.state();
      if (!mounted) return;
      setState(() {
        _notes = notes;
        _unfiledPrivate = state.unfiledPrivate;
      });
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _setPrivate(bool private) async {
    try {
      await AppScope.of(context).core.setUnfiledPrivate(private: private);
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
    await _reload();
  }

  Future<void> _open(Note note) async {
    await Navigator.of(context).push(
      MaterialPageRoute<void>(builder: (context) => NotePage(id: note.id)),
    );
    await _reload();
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final notes = _notes;
    final inbox = widget.folder == 'Inbox';
    return Scaffold(
      appBar: AppBar(title: Text(folderName(t, widget.folder))),
      body: notes == null
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              children: [
                if (inbox) ...[
                  ListTile(title: Text(t.unfiledWaiting(notes.length))),
                  SwitchListTile(
                    value: _unfiledPrivate,
                    title: Text(t.unfiledPrivateSetting),
                    subtitle: Text(t.privateExplain),
                    onChanged: (private) => unawaited(_setPrivate(private)),
                  ),
                ] else if (notes.isEmpty)
                  ListTile(title: Text(t.noNotesHere)),
                for (final note in notes)
                  ListTile(
                    title: Text(noteTitle(t, note)),
                    subtitle: switch (note.captured) {
                      final captured? => Text(when(captured)),
                      null => null,
                    },
                    onTap: () => unawaited(_open(note)),
                  ),
              ],
            ),
    );
  }
}

/// A Note's title in a list, with what marks it: a Saved Answer, a
/// Conflict, Private.
String noteTitle(AppLocalizations t, Note note) {
  var title = note.listTitle;
  if (note.model != null) title = t.answerSuffix(title);
  if (note.conflicts.isNotEmpty) title = t.conflictSuffix(title);
  if (note.isPrivate) title = t.privateSuffix(title);
  return title;
}
