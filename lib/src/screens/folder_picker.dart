import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../l10n.dart';
import 'name_dialog.dart';

/// The folders of the phone's storage, to walk through and choose one
/// as the Brain.
class FolderPicker extends StatefulWidget {
  const FolderPicker({
    super.key,
    required this.title,
    this.explanation,
    required this.onChosen,
  });

  final String title;
  final String? explanation;

  /// Called with the path of the folder the person chose.
  final ValueChanged<String> onChosen;

  @override
  State<FolderPicker> createState() => _FolderPickerState();
}

class _FolderPickerState extends State<FolderPicker> {
  String? _root;
  String? _path;
  List<String> _folders = const [];
  bool _started = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!_started) {
      _started = true;
      unawaited(_start());
    }
  }

  Future<void> _start() async {
    final root = await AppScope.of(context).phone.storageRoot();
    if (!mounted) return;
    _root = root;
    await _go(root);
  }

  Future<void> _go(String path) async {
    final folders = await AppScope.of(context).phone.folders(path);
    if (!mounted) return;
    setState(() {
      _path = path;
      _folders = folders;
    });
  }

  Future<void> _newFolder() async {
    final t = context.t;
    final phone = AppScope.of(context).phone;
    final path = _path;
    if (path == null) return;
    final name = await askName(
      context,
      title: t.newFolderTitle,
      confirm: t.create,
    );
    if (name == null || !mounted) return;
    try {
      await phone.createFolder('$path/$name');
      await _go('$path/$name');
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final path = _path;
    final explanation = widget.explanation;
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.title),
        actions: [
          IconButton(
            tooltip: t.newFolder,
            onPressed: path == null ? null : () => unawaited(_newFolder()),
            icon: const Icon(Icons.create_new_folder_outlined),
          ),
        ],
      ),
      body: path == null
          ? const Center(child: CircularProgressIndicator())
          : Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (explanation != null)
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
                    child: Text(explanation),
                  ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Text(
                    path,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ),
                Expanded(
                  child: ListView(
                    children: [
                      if (path != _root)
                        ListTile(
                          leading: const Icon(Icons.arrow_upward),
                          title: Text(t.folderUp),
                          onTap: () => unawaited(
                            _go(path.substring(0, path.lastIndexOf('/'))),
                          ),
                        ),
                      if (_folders.isEmpty)
                        ListTile(title: Text(t.noFoldersHere)),
                      for (final name in _folders)
                        ListTile(
                          leading: const Icon(Icons.folder_outlined),
                          title: Text(name),
                          onTap: () => unawaited(_go('$path/$name')),
                        ),
                    ],
                  ),
                ),
                SafeArea(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: FilledButton(
                      onPressed: () => widget.onChosen(path),
                      child: Text(t.useThisFolder),
                    ),
                  ),
                ),
              ],
            ),
    );
  }
}
