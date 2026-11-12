import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';

/// The versions of a Note in Conflict, one under the other as a phone
/// shows side by side, and the text the person merges them into. It
/// pops once the Conflict is ended.
class ConflictPage extends StatefulWidget {
  const ConflictPage({super.key, required this.id});

  final String id;

  @override
  State<ConflictPage> createState() => _ConflictPageState();
}

class _ConflictPageState extends State<ConflictPage> {
  List<Note>? _versions;
  final _merged = TextEditingController();
  bool _started = false;

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
    _merged.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    try {
      final versions = await AppScope.of(context).core.versions(widget.id);
      if (!mounted) return;
      setState(() {
        _versions = versions;
        _merged.text = versions.isEmpty ? '' : versions.first.text;
      });
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _end(Future<void> Function(Core core) action) async {
    final core = AppScope.of(context).core;
    final navigator = Navigator.of(context);
    try {
      await action(core);
      navigator.pop();
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final versions = _versions;
    return Scaffold(
      appBar: AppBar(title: Text(t.conflictTitle)),
      body: versions == null
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                for (final (index, version) in versions.indexed)
                  Card(
                    child: Padding(
                      padding: const EdgeInsets.all(12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            t.versionOf(index + 1),
                            style: Theme.of(context).textTheme.titleSmall,
                          ),
                          SelectableText(version.text),
                          OutlinedButton(
                            onPressed: () => unawaited(
                              _end(
                                (core) =>
                                    core.keepVersion(widget.id, version.id),
                              ),
                            ),
                            child: Text(t.keepVersion(index + 1)),
                          ),
                        ],
                      ),
                    ),
                  ),
                const SizedBox(height: 12),
                TextField(
                  controller: _merged,
                  minLines: 4,
                  maxLines: null,
                  decoration: InputDecoration(
                    labelText: t.mergedText,
                    alignLabelWithHint: true,
                    border: const OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 12),
                FilledButton(
                  onPressed: () => unawaited(
                    _end((core) => core.resolve(widget.id, _merged.text)),
                  ),
                  child: Text(t.keepMerged),
                ),
              ],
            ),
    );
  }
}
