import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iibrain/src/app.dart';
import 'package:iibrain/src/core/core.dart';
import 'package:iibrain/src/phone.dart';

/// A stand-in for the core: a Brain kept in memory that answers the
/// calls the screens make, and keeps what it was asked.
class FakeCore implements CoreCalls {
  FakeCore({this.brain});

  /// The Brain's folder, once chosen.
  String? brain;
  final Map<String, Map<String, Object?>> notes = {};
  final Set<String> folders = {'Inbox'};
  final Set<String> privateFolders = {};
  final List<Map<String, Object?>> models = [];
  String? chosen;
  bool unfiledPrivate = true;
  bool calendarToCloud = false;
  bool contactsToCloud = false;

  /// Every request, in order.
  final List<Map<String, Object?>> asked = [];

  /// Answers that replace the Brain's own, by call name; one may throw
  /// a [CoreError].
  final Map<String, Object? Function(Map<String, Object?> request)> answers =
      {};

  int _next = 1;

  /// Puts a Note into the Brain, as if another tool had written it.
  void add(
    String id, {
    String? title,
    String text = '',
    List<String> tags = const [],
    Map<String, Object?> more = const {},
  }) {
    final folder = id.contains('/') ? id.substring(0, id.lastIndexOf('/')) : '';
    if (folder.isNotEmpty) folders.add(folder);
    final name = id.substring(id.lastIndexOf('/') + 1).replaceAll('.md', '');
    notes[id] = {
      'id': id,
      'folder': folder,
      'title': title ?? name,
      'tags': tags,
      'text': text,
      'private': false,
      'attachments': <Object?>[],
      'sources': <Object?>[],
      'conflicts': <Object?>[],
      ...more,
    };
  }

  /// The requests of the call [name].
  List<Map<String, Object?>> calls(String name) => [
    for (final request in asked)
      if (request['call'] == name) request,
  ];

  @override
  Future<String> send(String request) async {
    final decoded = jsonDecode(request) as Map<String, Object?>;
    asked.add(decoded);
    try {
      final own = answers[decoded['call']];
      return jsonEncode({'ok': own != null ? own(decoded) : _answer(decoded)});
    } on CoreError catch (e) {
      return jsonEncode({
        'error': {'kind': e.kind, 'message': e.message},
      });
    }
  }

  Map<String, Object?> _shown(Map<String, Object?> note) {
    final folder = note['folder'] as String? ?? '';
    final unfiled = folder == 'Inbox';
    return {
      ...note,
      'unfiled': unfiled,
      'is_private':
          note['private'] == true ||
          privateFolders.contains(folder) ||
          (unfiled && unfiledPrivate),
    };
  }

  Map<String, Object?> _state() => {
    'brain': brain,
    if (brain != null) ...{
      'models': models,
      'chosen': chosen,
      'unfiled_private': unfiledPrivate,
      'calendar_to_cloud': calendarToCloud,
      'contacts_to_cloud': contactsToCloud,
      'version': '0.1.0',
    },
  };

  Map<String, Object?> _note(Object? id) {
    final note = notes[id];
    if (note == null) throw CoreError('not_found', '$id');
    return note;
  }

  Object? _answer(Map<String, Object?> request) {
    switch (request['call']) {
      case 'open' || 'state':
        return _state();
      case 'choose_brain':
        brain = request['folder'] as String?;
        return _state();
      case 'refresh':
        return false;
      case 'folders':
        final names = folders.toList()
          ..sort((a, b) {
            if (a == 'Inbox') return -1;
            if (b == 'Inbox') return 1;
            return a.compareTo(b);
          });
        return [
          for (final name in names)
            {
              'name': name,
              'private': privateFolders.contains(name),
              'notes': notes.values.where((n) => n['folder'] == name).length,
            },
        ];
      case 'notes':
        return [
          for (final note in notes.values)
            if (note['folder'] == request['folder']) _shown(note),
        ];
      case 'note':
        return _shown(_note(request['id']));
      case 'search':
        final query = (request['query'] as String? ?? '').toLowerCase();
        return [
          for (final note in notes.values)
            if ('${note['title']} ${note['text']}'.toLowerCase().contains(
              query,
            ))
              {
                'id': note['id'],
                'title': note['title'],
                'folder': note['folder'],
                'excerpt': note['text'],
              },
        ];
      case 'capture':
        final capture = request['capture'] as Map<String, Object?>;
        final id = 'Inbox/capture ${_next++}.md';
        add(
          id,
          text: capture['text'] as String? ?? '',
          more: {
            'via': capture['via'],
            'link': capture['link'],
            'private': capture['private'] ?? false,
            'captured': '2026-11-12T15:30:00+01:00',
          },
        );
        return id;
      case 'file':
        final note = notes.remove(request['id']);
        if (note == null) throw CoreError('not_found', '${request['id']}');
        final folder = request['folder'] as String? ?? '';
        final title = request['title'] as String? ?? '';
        final id = folder.isEmpty ? '$title.md' : '$folder/$title.md';
        if (folder.isNotEmpty) folders.add(folder);
        notes[id] = {
          ...note,
          'id': id,
          'folder': folder,
          'title': title,
          'tags': request['tags'],
        };
        return id;
      case 'set_text':
        _note(request['id'])['text'] = request['text'];
        return null;
      case 'delete':
        _note(request['id']);
        notes.remove(request['id']);
        return null;
      case 'create_folder':
        if (!folders.add(request['name'] as String? ?? '')) {
          throw CoreError('invalid', '${request['name']}');
        }
        return null;
      case 'rename_folder':
        final from = request['name'] as String? ?? '';
        final to = request['new_name'] as String? ?? '';
        folders
          ..remove(from)
          ..add(to);
        for (final note in notes.values.toList()) {
          if (note['folder'] == from) {
            notes.remove(note['id']);
            final id = '$to/${note['title']}.md';
            notes[id] = {...note, 'id': id, 'folder': to};
          }
        }
        return null;
      case 'remove_folder':
        final name = request['name'];
        if (notes.values.any((n) => n['folder'] == name)) {
          throw CoreError('invalid', '$name is not empty');
        }
        folders.remove(name);
        return null;
      case 'set_private':
        _note(request['id'])['private'] = request['private'];
        return null;
      case 'set_folder_private':
        final folder = request['folder'] as String? ?? '';
        if (request['private'] == true) {
          privateFolders.add(folder);
        } else {
          privateFolders.remove(folder);
        }
        return null;
      case 'set_unfiled_private':
        unfiledPrivate = request['private'] == true;
        return _state();
      case 'set_lookup_to_cloud':
        if (request['kind'] == 'calendar') {
          calendarToCloud = request['allowed'] == true;
        } else {
          contactsToCloud = request['allowed'] == true;
        }
        return _state();
      case 'save_model':
        final model = request['model'] as Map<String, Object?>;
        models
          ..removeWhere((m) => m['name'] == model['name'])
          ..add(model);
        chosen ??= model['name'] as String?;
        return _state();
      case 'test_model':
        return null;
      case 'remove_model':
        models.removeWhere((m) => m['name'] == request['name']);
        if (chosen == request['name']) {
          chosen = models.isEmpty ? null : models.first['name'] as String?;
        }
        return _state();
      case 'choose_model':
        chosen = request['name'] as String?;
        return _state();
      case 'extract_one':
        return false;
      default:
        throw CoreError('invalid', 'no call named ${request['call']}');
    }
  }
}

/// A stand-in for the phone: a storage of folders kept in memory.
class FakePhone implements Phone {
  FakePhone({this.granted = true});

  bool granted;
  int requests = 0;

  /// The folders of the storage, as paths.
  final Set<String> storage = {
    '/storage',
    '/storage/Documents',
    '/storage/Sync',
    '/storage/Sync/Brain',
  };

  @override
  Future<bool> storageGranted() async => granted;

  @override
  Future<void> requestStorage() async => requests++;

  @override
  Future<String> storageRoot() async => '/storage';

  @override
  Future<String> deviceDir() async => '/data/iibrain';

  @override
  Future<List<String>> folders(String path) async => [
    for (final folder in storage.toList()..sort())
      if (folder.startsWith('$path/') &&
          !folder.substring(path.length + 1).contains('/'))
        folder.substring(path.length + 1),
  ];

  @override
  Future<void> createFolder(String path) async => storage.add(path);
}

/// Starts the app over the stand-ins and lets it settle.
Future<void> pumpApp(
  WidgetTester tester,
  FakeCore core, {
  FakePhone? phone,
  String locale = 'en',
}) async {
  await tester.pumpWidget(
    IIbrainApp(
      calls: core,
      phone: phone ?? FakePhone(),
      locale: Locale(locale),
    ),
  );
  await tester.pumpAndSettle();
}

/// Taps the one thing that shows [text] and lets the app settle.
Future<void> tapText(WidgetTester tester, String text) async {
  await tester.tap(find.text(text));
  await tester.pumpAndSettle();
}

/// Taps the one button whose tooltip is [tooltip] and lets the app
/// settle.
Future<void> tapTooltip(WidgetTester tester, String tooltip) async {
  await tester.tap(find.byTooltip(tooltip));
  await tester.pumpAndSettle();
}

/// Types [text] into the field labelled [label].
Future<void> fill(WidgetTester tester, String label, String text) async {
  await tester.enterText(find.widgetWithText(TextField, label), text);
  await tester.pumpAndSettle();
}
