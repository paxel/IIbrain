import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../core/reader.dart';
import '../l10n.dart';
import 'ask_tab.dart';
import 'capture_page.dart';
import 'folder_page.dart';
import 'name_dialog.dart';
import 'search_tab.dart';
import 'settings_tab.dart';

/// The Brain on the phone: its Folders, the search and the settings.
class HomePage extends StatefulWidget {
  const HomePage({super.key, required this.onBrainChanged});

  /// Called when the person pointed the app at another folder.
  final VoidCallback onBrainChanged;

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> with WidgetsBindingObserver {
  int _tab = 0;
  List<Folder> _folders = const [];
  int _atRoot = 0;
  bool _loaded = false;
  bool _started = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!_started) {
      _started = true;
      unawaited(_reload());
      _readWaitingTexts();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    // What the Sync Tool brought while the app was away.
    if (state == AppLifecycleState.resumed) unawaited(_reload(refresh: true));
  }

  Future<void> _reload({bool refresh = false}) async {
    final core = AppScope.of(context).core;
    try {
      if (refresh) await core.refresh();
      final folders = await core.folders();
      final atRoot = (await core.notes('')).length;
      if (!mounted) return;
      setState(() {
        _folders = folders;
        _atRoot = atRoot;
        _loaded = true;
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

  Future<void> _open(String folder) async {
    await Navigator.of(context).push(
      MaterialPageRoute<void>(builder: (context) => FolderPage(folder: folder)),
    );
    await _reload();
  }

  /// Reads the texts of new Attachments and fetches the pages of new
  /// links, beside the screens.
  void _readWaitingTexts() => readWaitingTexts(AppScope.of(context).core);

  Future<void> _capture() async {
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (context) => const CapturePage()));
    if (!mounted) return;
    _readWaitingTexts();
    await _reload();
  }

  Future<void> _newFolder() async {
    final t = context.t;
    final name = await askName(
      context,
      title: t.newFolderTitle,
      confirm: t.create,
    );
    if (name == null) return;
    await _act((core) => core.createFolder(name));
  }

  Future<void> _folderMenu(Folder folder) async {
    final t = context.t;
    final choice = await showModalBottomSheet<String>(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.lock_outline),
              title: Text(
                folder.private ? t.folderLiftPrivate : t.folderMarkPrivate,
              ),
              subtitle: Text(t.privateExplain),
              onTap: () => Navigator.of(context).pop('private'),
            ),
            ListTile(
              leading: const Icon(Icons.edit_outlined),
              title: Text(t.renameFolder),
              onTap: () => Navigator.of(context).pop('rename'),
            ),
            ListTile(
              leading: const Icon(Icons.delete_outline),
              title: Text(t.removeFolder),
              onTap: () => Navigator.of(context).pop('remove'),
            ),
          ],
        ),
      ),
    );
    if (!mounted) return;
    switch (choice) {
      case 'rename':
        final name = await askName(
          context,
          title: t.renameFolderTitle,
          confirm: t.rename,
          initial: folder.name,
        );
        if (name == null) return;
        await _act((core) => core.renameFolder(folder.name, name));
      case 'private':
        await _act(
          (core) =>
              core.setFolderPrivate(folder.name, private: !folder.private),
        );
      case 'remove':
        await _act((core) => core.removeFolder(folder.name));
    }
  }

  Widget _brain(BuildContext context) {
    final t = context.t;
    if (!_loaded) return const Center(child: CircularProgressIndicator());
    return RefreshIndicator(
      onRefresh: () => _reload(refresh: true),
      child: ListView(
        children: [
          for (final folder in _folders)
            ListTile(
              leading: Icon(
                folder.name == 'Inbox'
                    ? Icons.inbox_outlined
                    : Icons.folder_outlined,
              ),
              title: Text(
                folder.private
                    ? t.privateSuffix(folderName(t, folder.name))
                    : folderName(t, folder.name),
              ),
              subtitle: Text(
                folder.name == 'Inbox'
                    ? t.unfiledWaiting(folder.notes)
                    : t.notesCount(folder.notes),
              ),
              onTap: () => unawaited(_open(folder.name)),
              // The Inbox is fixed.
              onLongPress: folder.name == 'Inbox'
                  ? null
                  : () => unawaited(_folderMenu(folder)),
            ),
          if (_atRoot > 0)
            ListTile(
              leading: const Icon(Icons.home_outlined),
              title: Text(t.brainRoot),
              subtitle: Text(t.notesCount(_atRoot)),
              onTap: () => unawaited(_open('')),
            ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    return Scaffold(
      appBar: AppBar(
        title: Text(t.appTitle),
        actions: [
          if (_tab == 0)
            IconButton(
              tooltip: t.newFolder,
              onPressed: () => unawaited(_newFolder()),
              icon: const Icon(Icons.create_new_folder_outlined),
            ),
        ],
      ),
      body: switch (_tab) {
        0 => _brain(context),
        1 => const SearchTab(),
        2 => const AskTab(),
        _ => SettingsTab(onBrainChanged: widget.onBrainChanged),
      },
      floatingActionButton: _tab == 0
          ? FloatingActionButton(
              tooltip: t.newNote,
              onPressed: () => unawaited(_capture()),
              child: const Icon(Icons.add),
            )
          : null,
      bottomNavigationBar: NavigationBar(
        selectedIndex: _tab,
        onDestinationSelected: (tab) {
          setState(() => _tab = tab);
          if (tab == 0) unawaited(_reload());
        },
        destinations: [
          NavigationDestination(
            icon: const Icon(Icons.psychology_outlined),
            label: t.tabBrain,
          ),
          NavigationDestination(
            icon: const Icon(Icons.search),
            label: t.tabSearch,
          ),
          NavigationDestination(
            icon: const Icon(Icons.question_answer_outlined),
            label: t.ask,
          ),
          NavigationDestination(
            icon: const Icon(Icons.settings_outlined),
            label: t.tabSettings,
          ),
        ],
      ),
    );
  }
}
