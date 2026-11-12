import 'dart:async';

import 'package:flutter/material.dart';

import '../app.dart';
import '../l10n.dart';

/// A Capture on the phone: typed text, a photo from the camera, files,
/// or any of them together. A link on its own is kept as a link, with
/// the text of its page.
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

/// The file name at the end of [path].
String fileName(String path) => path.substring(path.lastIndexOf('/') + 1);

class _CapturePageState extends State<CapturePage> {
  final _text = TextEditingController();

  /// The files to keep as Attachments, as paths.
  final List<String> _files = [];

  /// Those of them the camera took.
  final Set<String> _photos = {};
  bool _saving = false;

  @override
  void dispose() {
    _text.dispose();
    super.dispose();
  }

  Future<void> _takePhoto() async {
    try {
      final photo = await AppScope.of(context).phone.takePhoto();
      if (photo == null || !mounted) return;
      setState(() {
        _files.add(photo);
        _photos.add(photo);
      });
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _addFiles() async {
    try {
      final files = await AppScope.of(context).phone.pickFiles();
      if (!mounted) return;
      setState(() => _files.addAll(files));
    } catch (e) {
      if (!mounted) return;
      showError(context, e);
    }
  }

  Future<void> _save() async {
    final scope = AppScope.of(context);
    final navigator = Navigator.of(context);
    final text = _text.text;
    setState(() => _saving = true);
    try {
      if (_files.isEmpty && isLink(text)) {
        await scope.core.capture(via: 'link', link: text.trim());
      } else {
        await scope.core.capture(
          text: text,
          via: _files.isEmpty
              ? 'typed'
              : _files.every(_photos.contains)
              ? 'photo'
              : 'file',
          files: _files,
        );
        // The core copied them into the Brain.
        await scope.phone.discard(_files);
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
    final something = _text.text.trim().isNotEmpty || _files.isNotEmpty;
    return Scaffold(
      appBar: AppBar(
        title: Text(t.newNote),
        actions: [
          TextButton(
            onPressed: _saving || !something ? null : () => unawaited(_save()),
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
            for (final file in _files)
              ListTile(
                dense: true,
                contentPadding: EdgeInsets.zero,
                leading: Icon(
                  _photos.contains(file)
                      ? Icons.photo_camera_outlined
                      : Icons.attach_file,
                ),
                title: Text(fileName(file)),
                trailing: IconButton(
                  tooltip: t.removeFile(fileName(file)),
                  onPressed: () => setState(() {
                    _files.remove(file);
                    _photos.remove(file);
                  }),
                  icon: const Icon(Icons.close),
                ),
              ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 8,
              children: [
                OutlinedButton.icon(
                  onPressed: () => unawaited(_takePhoto()),
                  icon: const Icon(Icons.photo_camera_outlined),
                  label: Text(t.takePhoto),
                ),
                OutlinedButton.icon(
                  onPressed: () => unawaited(_addFiles()),
                  icon: const Icon(Icons.attach_file),
                  label: Text(t.addFiles),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(t.linkHint, style: Theme.of(context).textTheme.bodySmall),
          ],
        ),
      ),
    );
  }
}
