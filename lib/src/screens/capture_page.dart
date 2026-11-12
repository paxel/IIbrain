import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../l10n.dart';

/// A typed Capture. A link on its own is kept as a link, with the text
/// of its page.
class CapturePage extends StatefulWidget {
  const CapturePage({super.key});

  @override
  State<CapturePage> createState() => _CapturePageState();
}

/// Whether [text] is nothing but one web link, as the core judges it.
bool isLink(String text) {
  final trimmed = text.trim();
  return (trimmed.startsWith('http://') || trimmed.startsWith('https://')) &&
      !trimmed.contains(RegExp(r'\s')) &&
      trimmed.length > 'https://'.length;
}

class _CapturePageState extends State<CapturePage> {
  final _text = TextEditingController();
  bool _saving = false;

  @override
  void dispose() {
    _text.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final core = AppScope.of(context).core;
    final navigator = Navigator.of(context);
    final text = _text.text;
    setState(() => _saving = true);
    try {
      if (isLink(text)) {
        await core.capture(via: 'link', link: text.trim());
      } else {
        await core.capture(text: text, via: 'typed');
      }
      navigator.pop();
    } catch (e) {
      if (!mounted) return;
      setState(() => _saving = false);
      showError(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    return Scaffold(
      appBar: AppBar(
        title: Text(t.newNote),
        actions: [
          TextButton(
            onPressed: _saving || _text.text.trim().isEmpty
                ? null
                : () => unawaited(_save()),
            child: Text(t.save),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(
              child: TextField(
                controller: _text,
                autofocus: true,
                maxLines: null,
                expands: true,
                textAlignVertical: TextAlignVertical.top,
                decoration: InputDecoration(
                  labelText: t.noteText,
                  alignLabelWithHint: true,
                  border: const OutlineInputBorder(),
                ),
                onChanged: (_) => setState(() {}),
              ),
            ),
            const SizedBox(height: 8),
            Text(t.linkHint, style: Theme.of(context).textTheme.bodySmall),
          ],
        ),
      ),
    );
  }
}
