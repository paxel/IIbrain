import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../l10n.dart';
import 'folder_picker.dart';
import 'home_page.dart';

/// Where the app begins: it opens the Device, asks for access to the
/// phone's storage when that is missing, has the person choose the
/// Brain's folder on the first start, and then shows the Brain.
class StartPage extends StatefulWidget {
  const StartPage({super.key});

  @override
  State<StartPage> createState() => _StartPageState();
}

enum _Step { opening, access, folder, brain }

class _StartPageState extends State<StartPage> with WidgetsBindingObserver {
  _Step _step = _Step.opening;
  Object? _error;
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
      unawaited(_open());
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    // Back from the system's permission page.
    if (state == AppLifecycleState.resumed && _step == _Step.access) {
      unawaited(_open());
    }
  }

  Future<void> _open() async {
    final scope = AppScope.of(context);
    try {
      final granted = await scope.phone.storageGranted();
      final state = await scope.core.open(await scope.phone.deviceDir());
      if (!mounted) return;
      setState(() {
        _error = null;
        _step = state.brain != null
            ? _Step.brain
            : granted
            ? _Step.folder
            : _Step.access;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = e);
    }
  }

  Future<void> _choose(String folder) async {
    final core = AppScope.of(context).core;
    try {
      await core.chooseBrain(folder);
      if (!mounted) return;
      setState(() {
        _error = null;
        _step = _Step.brain;
      });
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final error = _error;
    if (error != null) {
      return Scaffold(
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Text(errorText(t, error)),
          ),
        ),
      );
    }
    return switch (_step) {
      _Step.opening => const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      ),
      _Step.access => Scaffold(
        body: SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  t.allowFilesTitle,
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(height: 16),
                Text(t.allowFilesBody),
                const SizedBox(height: 24),
                FilledButton(
                  onPressed: () =>
                      unawaited(AppScope.of(context).phone.requestStorage()),
                  child: Text(t.allowFilesButton),
                ),
              ],
            ),
          ),
        ),
      ),
      _Step.folder => FolderPicker(
        title: t.chooseBrainTitle,
        explanation: t.chooseBrainBody,
        onChosen: (folder) => unawaited(_choose(folder)),
      ),
      _Step.brain => HomePage(
        onBrainChanged: () => setState(() => _step = _Step.brain),
      ),
    };
  }
}
