import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';

/// Filing an Unfiled Note, by hand or from a Model's Proposal, or
/// title, tags and Folder of a filed one. It pops with the Note's id
/// from then on.
class FilePage extends StatefulWidget {
  const FilePage({super.key, required this.note});

  final Note note;

  @override
  State<FilePage> createState() => _FilePageState();
}

class _FilePageState extends State<FilePage> {
  late final _title = TextEditingController(
    text: widget.note.unfiled ? '' : widget.note.title,
  );
  late final _tags = TextEditingController(text: widget.note.tags.join(', '));

  /// The chosen Folder; a Note is filed out of the Inbox, so the Inbox
  /// is no choice.
  late String _folder = widget.note.unfiled ? '' : widget.note.folder;
  List<String> _folders = const [''];
  bool _started = false;

  /// Whether the Model is asked for a Proposal right now.
  bool _proposing = false;

  /// The Model whose Proposal fills the fields.
  String? _proposedBy;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!_started) {
      _started = true;
      unawaited(_load());
    }
  }

  @override
  void dispose() {
    _title.dispose();
    _tags.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    try {
      final folders = await AppScope.of(context).core.folders();
      if (!mounted) return;
      setState(() {
        _folders = [
          '',
          for (final folder in folders)
            if (folder.name != 'Inbox') folder.name,
        ];
      });
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  /// Asks the Model for a Proposal and fills the fields with it. It
  /// changes nothing until the person saves.
  Future<void> _propose() async {
    final core = AppScope.of(context).core;
    setState(() => _proposing = true);
    try {
      final proposal = await core.propose(widget.note.id);
      if (!mounted) return;
      setState(() {
        _title.text = proposal.title;
        _tags.text = proposal.tags.join(', ');
        _folder = proposal.folder;
        _proposedBy = proposal.model;
      });
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    } finally {
      if (mounted) setState(() => _proposing = false);
    }
  }

  Future<void> _save() async {
    final core = AppScope.of(context).core;
    final navigator = Navigator.of(context);
    final tags = [
      for (final tag in _tags.text.split(','))
        if (tag.trim().isNotEmpty) tag.trim(),
    ];
    try {
      final id = await core.file(
        widget.note.id,
        _title.text.trim(),
        tags,
        _folder,
      );
      navigator.pop(id);
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    // A Folder that is no choice yet, as a Proposal may name one.
    final choices = [..._folders, if (!_folders.contains(_folder)) _folder];
    final proposedBy = _proposedBy;
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.note.unfiled ? t.fileNoteTitle : t.noteDetailsTitle),
        actions: [
          TextButton(
            onPressed: _title.text.trim().isEmpty
                ? null
                : () => unawaited(_save()),
            child: Text(t.save),
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          if (widget.note.unfiled) ...[
            if (_proposing)
              Row(
                children: [
                  const SizedBox.square(
                    dimension: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
                  const SizedBox(width: 12),
                  Text(t.modelWorking),
                ],
              )
            else
              Align(
                alignment: Alignment.centerLeft,
                child: OutlinedButton(
                  onPressed: () => unawaited(_propose()),
                  child: Text(t.propose),
                ),
              ),
            if (proposedBy != null)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(
                  t.proposalBy(proposedBy),
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ),
            const SizedBox(height: 12),
          ],
          TextField(
            controller: _title,
            decoration: InputDecoration(labelText: t.titleLabel),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _tags,
            decoration: InputDecoration(
              labelText: t.tagsLabel,
              hintText: t.tagsHint,
            ),
          ),
          const SizedBox(height: 12),
          DropdownButtonFormField<String>(
            // A new field when a Proposal changes the Folder.
            key: ValueKey(_folder),
            initialValue: _folder,
            decoration: InputDecoration(labelText: t.folderLabel),
            items: [
              for (final folder in choices)
                DropdownMenuItem(
                  value: folder,
                  child: Text(
                    _folders.contains(folder)
                        ? folderName(t, folder)
                        : t.newFolderSuffix(folder),
                  ),
                ),
            ],
            onChanged: (folder) => setState(() => _folder = folder ?? ''),
          ),
        ],
      ),
    );
  }
}
