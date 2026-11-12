import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../core/core.dart';
import '../l10n.dart';
import 'models_page.dart';
import 'note_page.dart';

/// The ask screen: a Question in the person's own words, and the
/// Answer with its Sources.
class AskTab extends StatefulWidget {
  const AskTab({super.key});

  @override
  State<AskTab> createState() => _AskTabState();
}

class _AskTabState extends State<AskTab> {
  final _question = TextEditingController();
  bool _asking = false;
  Asked? _asked;
  bool _noModel = false;

  @override
  void dispose() {
    _question.dispose();
    super.dispose();
  }

  Future<void> _ask() async {
    final core = AppScope.of(context).core;
    setState(() {
      _asking = true;
      _asked = null;
      _noModel = false;
    });
    try {
      final asked = await core.ask(_question.text.trim());
      if (!mounted) return;
      setState(() => _asked = asked);
    } on CoreError catch (e) {
      if (!mounted) return;
      if (e.kind == 'no_model') {
        setState(() => _noModel = true);
      } else {
        showError(context, e);
      }
    } finally {
      if (mounted) setState(() => _asking = false);
    }
  }

  Future<void> _keep(Answer answer) async {
    final core = AppScope.of(context).core;
    final navigator = Navigator.of(context);
    try {
      final id = await core.saveAnswer(answer);
      await navigator.push(
        MaterialPageRoute<void>(builder: (context) => NotePage(id: id)),
      );
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _open(Widget page) async {
    await Navigator.of(context)
        .push(MaterialPageRoute<void>(builder: (context) => page));
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    final text = Theme.of(context).textTheme;
    final asked = _asked;
    final answer = asked?.answer;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(t.askTitle, style: text.headlineSmall),
        const SizedBox(height: 12),
        TextField(
          controller: _question,
          minLines: 2,
          maxLines: 5,
          decoration: InputDecoration(
            labelText: t.questionLabel,
            border: const OutlineInputBorder(),
          ),
          onChanged: (_) => setState(() {}),
        ),
        const SizedBox(height: 12),
        if (_asking)
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
            child: FilledButton(
              onPressed: _question.text.trim().isEmpty
                  ? null
                  : () => unawaited(_ask()),
              child: Text(t.ask),
            ),
          ),
        if (_noModel) ...[
          const Divider(),
          Text(t.errorNoModel),
          Align(
            alignment: Alignment.centerLeft,
            child: TextButton(
              onPressed: () => unawaited(_open(const ModelsPage())),
              child: Text(t.models),
            ),
          ),
        ],
        if (asked != null && answer == null) ...[
          const Divider(),
          Text(t.nothingFits),
        ],
        if (answer != null) ...[
          const Divider(),
          Text(t.answerBy(answer.model), style: text.bodySmall),
          SelectableText(answer.text),
        ],
        if (asked != null && asked.leftOut > 0)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(t.privateLeftOut(asked.leftOut), style: text.bodySmall),
          ),
        if (answer != null) ...[
          Align(
            alignment: Alignment.centerLeft,
            child: OutlinedButton(
              onPressed: () => unawaited(_keep(answer)),
              child: Text(t.keepAnswer),
            ),
          ),
          const Divider(),
          Text(t.sources, style: text.titleSmall),
          for (final source in answer.sources)
            ListTile(
              contentPadding: EdgeInsets.zero,
              leading: const Icon(Icons.description_outlined),
              title: Text(source.title),
              subtitle: Text(folderName(t, source.folder)),
              onTap: () => unawaited(_open(NotePage(id: source.id))),
            ),
        ],
      ],
    );
  }
}
