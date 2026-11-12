import 'package:flutter/material.dart';

import '../l10n.dart';

/// Asks for a name, as of a Folder; null when the person cancels.
Future<String?> askName(
  BuildContext context, {
  required String title,
  required String confirm,
  String initial = '',
}) => showDialog<String>(
  context: context,
  builder: (context) =>
      _NameDialog(title: title, confirm: confirm, initial: initial),
);

class _NameDialog extends StatefulWidget {
  const _NameDialog({
    required this.title,
    required this.confirm,
    required this.initial,
  });

  final String title;
  final String confirm;
  final String initial;

  @override
  State<_NameDialog> createState() => _NameDialogState();
}

class _NameDialogState extends State<_NameDialog> {
  late final TextEditingController _name = TextEditingController(
    text: widget.initial,
  );

  @override
  void dispose() {
    _name.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final t = context.t;
    return AlertDialog(
      title: Text(widget.title),
      content: TextField(
        controller: _name,
        autofocus: true,
        decoration: InputDecoration(labelText: t.folderName),
        onChanged: (_) => setState(() {}),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(t.cancel),
        ),
        FilledButton(
          onPressed: _name.text.trim().isEmpty
              ? null
              : () => Navigator.of(context).pop(_name.text.trim()),
          child: Text(widget.confirm),
        ),
      ],
    );
  }
}

/// Asks before something is deleted; true when the person confirms.
Future<bool> confirmDelete(BuildContext context, String question) async {
  final t = context.t;
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      content: Text(question),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(false),
          child: Text(t.cancel),
        ),
        FilledButton(
          onPressed: () => Navigator.of(context).pop(true),
          child: Text(t.delete),
        ),
      ],
    ),
  );
  return confirmed ?? false;
}
