import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';
import 'folder_picker.dart';
import 'models_page.dart';

/// The settings of this Device.
class SettingsTab extends StatefulWidget {
  const SettingsTab({super.key, required this.onBrainChanged});

  /// Called when the person pointed the app at another folder.
  final VoidCallback onBrainChanged;

  @override
  State<SettingsTab> createState() => _SettingsTabState();
}

class _SettingsTabState extends State<SettingsTab> {
  BrainState? _state;
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
      final state = await AppScope.of(context).core.state();
      if (!mounted) return;
      setState(() => _state = state);
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _lookup(String kind, bool allowed) async {
    try {
      final state = await AppScope.of(context).core
          .setLookupToCloud(kind, allowed: allowed);
      if (!mounted) return;
      setState(() => _state = state);
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _models() async {
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (context) => const ModelsPage()));
    await _reload();
  }

  Future<void> _changeFolder() async {
    final t = context.t;
    final core = AppScope.of(context).core;
    final folder = await Navigator.of(context).push(
      MaterialPageRoute<String>(
        builder: (context) => FolderPicker(
          title: t.changeFolder,
          onChosen: (folder) => Navigator.of(context).pop(folder),
        ),
      ),
    );
    if (folder == null) return;
    try {
      await core.chooseBrain(folder);
      widget.onBrainChanged();
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
    await _reload();
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final state = _state;
    if (state == null) return const Center(child: CircularProgressIndicator());
    return ListView(
      children: [
        ListTile(
          leading: const Icon(Icons.folder_outlined),
          title: Text(t.brainFolder),
          subtitle: Text(state.brain ?? ''),
          trailing: TextButton(
            onPressed: () => unawaited(_changeFolder()),
            child: Text(t.changeFolder),
          ),
        ),
        ListTile(
          leading: const Icon(Icons.print_outlined),
          title: Text(t.printCapture),
          subtitle: Text(t.printCaptureBody),
          isThreeLine: true,
          trailing: IconButton(
            tooltip: t.openPrintSettings,
            onPressed: () =>
                unawaited(AppScope.of(context).phone.openPrintSettings()),
            icon: const Icon(Icons.open_in_new),
          ),
        ),
        SwitchListTile(
          secondary: const Icon(Icons.event_outlined),
          value: state.calendarToCloud,
          title: Text(t.calendarToCloud),
          subtitle: Text(t.lookupLocalOnly),
          onChanged: (allowed) => unawaited(_lookup('calendar', allowed)),
        ),
        SwitchListTile(
          secondary: const Icon(Icons.contacts_outlined),
          value: state.contactsToCloud,
          title: Text(t.contactsToCloud),
          subtitle: Text(t.lookupLocalOnly),
          onChanged: (allowed) => unawaited(_lookup('contacts', allowed)),
        ),
        ListTile(
          leading: const Icon(Icons.hub_outlined),
          title: Text(t.modelsTitle),
          subtitle: Text(
            state.models.isEmpty
                ? t.noModels
                : state.models.map((model) => model.name).join(', '),
          ),
          onTap: () => unawaited(_models()),
        ),
        ListTile(
          leading: const Icon(Icons.info_outline),
          title: Text(t.appTitle),
          subtitle: Text(t.version(state.version)),
        ),
      ],
    );
  }
}
