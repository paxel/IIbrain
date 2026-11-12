import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';

/// The Models this Device knows: which one proposes and answers, and
/// the way to connect another.
class ModelsPage extends StatefulWidget {
  const ModelsPage({super.key});

  @override
  State<ModelsPage> createState() => _ModelsPageState();
}

class _ModelsPageState extends State<ModelsPage> {
  BrainState? _state;
  bool _started = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!_started) {
      _started = true;
      unawaited(_do((core) => core.state()));
    }
  }

  Future<void> _do(Future<BrainState> Function(Core core) action) async {
    try {
      final state = await action(AppScope.of(context).core);
      if (!mounted) return;
      setState(() => _state = state);
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _connect() async {
    await Navigator.of(context).push(
      MaterialPageRoute<void>(builder: (context) => const ModelFormPage()),
    );
    await _do((core) => core.state());
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final state = _state;
    return Scaffold(
      appBar: AppBar(title: Text(t.modelsTitle)),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => unawaited(_connect()),
        icon: const Icon(Icons.add),
        label: Text(t.connectModel),
      ),
      body: state == null
          ? const Center(child: CircularProgressIndicator())
          : RadioGroup<String>(
              groupValue: state.chosen,
              onChanged: (name) {
                if (name != null) {
                  unawaited(_do((core) => core.chooseModel(name)));
                }
              },
              child: ListView(
                children: [
                  if (state.models.isEmpty)
                    Padding(
                      padding: const EdgeInsets.all(16),
                      child: Text(t.noModels),
                    ),
                  for (final model in state.models)
                    RadioListTile<String>(
                      value: model.name,
                      title: Text(model.name),
                      subtitle: Text(model.local ? t.localModel : t.cloudModel),
                      secondary: IconButton(
                        tooltip: t.remove,
                        onPressed: () => unawaited(
                          _do((core) => core.removeModel(model.name)),
                        ),
                        icon: const Icon(Icons.delete_outline),
                      ),
                    ),
                ],
              ),
            ),
    );
  }
}

/// The fields for a new Model.
class ModelFormPage extends StatefulWidget {
  const ModelFormPage({super.key});

  @override
  State<ModelFormPage> createState() => _ModelFormPageState();
}

class _ModelFormPageState extends State<ModelFormPage> {
  final _name = TextEditingController();
  final _address = TextEditingController();
  final _key = TextEditingController();
  final _model = TextEditingController();
  bool _local = false;
  bool _busy = false;

  @override
  void dispose() {
    _name.dispose();
    _address.dispose();
    _key.dispose();
    _model.dispose();
    super.dispose();
  }

  Future<void> _add({required bool test}) async {
    final core = AppScope.of(context).core;
    final navigator = Navigator.of(context);
    final model = Model(
      name: _name.text.trim(),
      address: _address.text.trim(),
      key: _key.text.trim().isEmpty ? null : _key.text.trim(),
      model: _model.text.trim(),
      local: _local,
    );
    setState(() => _busy = true);
    try {
      if (test) await core.testModel(model);
      await core.saveModel(model);
      navigator.pop();
    } catch (e) {
      if (!mounted) return;
      setState(() => _busy = false);
      showError(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final filled =
        _name.text.trim().isNotEmpty && _address.text.trim().isNotEmpty;
    return Scaffold(
      appBar: AppBar(title: Text(t.connectModel)),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          TextField(
            controller: _name,
            decoration: InputDecoration(labelText: t.modelName),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _address,
            keyboardType: TextInputType.url,
            decoration: InputDecoration(
              labelText: t.modelAddress,
              hintText: t.modelAddressHint,
            ),
            onChanged: (_) => setState(() {}),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _key,
            obscureText: true,
            decoration: InputDecoration(labelText: t.modelKey),
          ),
          const SizedBox(height: 12),
          TextField(
            controller: _model,
            decoration: InputDecoration(labelText: t.modelId),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            value: _local,
            title: Text(t.modelIsLocal),
            onChanged: (local) => setState(() => _local = local),
          ),
          Text(t.modelStaysHere, style: Theme.of(context).textTheme.bodySmall),
          const SizedBox(height: 16),
          if (_busy)
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
          else ...[
            FilledButton(
              onPressed: filled ? () => unawaited(_add(test: true)) : null,
              child: Text(t.testAndAdd),
            ),
            TextButton(
              onPressed: filled ? () => unawaited(_add(test: false)) : null,
              child: Text(t.addUntested),
            ),
          ],
        ],
      ),
    );
  }
}
