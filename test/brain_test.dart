// The phone app on the core: opening the Brain, capturing text, filing
// by hand, searching (#22).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support.dart';

void main() {
  testWidgets('the first start asks for access, then for the folder', (
    tester,
  ) async {
    final core = FakeCore();
    final phone = FakePhone(granted: false);
    await pumpApp(tester, core, phone: phone);
    expect(find.text('IIbrain needs access to your files'), findsOneWidget);

    await tapText(tester, 'Open the permission');
    expect(phone.requests, 1);

    // Back from the system's page, with the access granted.
    phone.granted = true;
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pumpAndSettle();
    expect(find.text('Where is your Brain?'), findsOneWidget);

    await tapText(tester, 'Sync');
    await tapText(tester, 'Brain');
    expect(find.text('No folders in here.'), findsOneWidget);
    await tapText(tester, 'Use this folder');

    expect(core.brain, '/storage/Sync/Brain');
    expect(find.text('Inbox'), findsOneWidget);
    expect(find.text('Nothing waits Unfiled.'), findsOneWidget);
  });

  testWidgets('a folder is made in the picker and can be left again', (
    tester,
  ) async {
    final core = FakeCore();
    final phone = FakePhone();
    await pumpApp(tester, core, phone: phone);
    await tapText(tester, 'Documents');
    await tapTooltip(tester, 'New Folder…');
    await fill(tester, 'Name', 'Second Brain');
    await tapText(tester, 'Create');
    expect(phone.storage, contains('/storage/Documents/Second Brain'));
    expect(find.text('/storage/Documents/Second Brain'), findsOneWidget);

    await tapText(tester, 'Up');
    expect(find.text('Second Brain'), findsOneWidget);
  });

  testWidgets('the Brain is remembered at the next start', (tester) async {
    final core = FakeCore(brain: '/storage/Sync/Brain')
      ..add('Home/Plumber.md', text: 'Monday');
    await pumpApp(tester, core);
    expect(find.text('Where is your Brain?'), findsNothing);
    expect(find.text('Home'), findsOneWidget);
    expect(find.text('1 Note'), findsOneWidget);
    expect(core.calls('open').single['device_dir'], '/data/iibrain');
  });

  testWidgets('a typed Note lands in the Inbox and is counted', (tester) async {
    final core = FakeCore(brain: '/b');
    await pumpApp(tester, core);
    await tapTooltip(tester, 'New Note');
    await fill(tester, 'Text', 'Call the plumber on Monday.');
    await tapText(tester, 'Save');

    expect(core.calls('capture').single['capture'], {
      'text': 'Call the plumber on Monday.',
      'via': 'typed',
      'link': null,
      'private': false,
      'files': <Object?>[],
    });
    expect(find.text('1 Note waits Unfiled.'), findsOneWidget);
  });

  testWidgets('a link on its own is captured as a link', (tester) async {
    final core = FakeCore(brain: '/b');
    await pumpApp(tester, core);
    await tapTooltip(tester, 'New Note');
    await fill(tester, 'Text', 'https://example.org/apple-tree');
    await tapText(tester, 'Save');

    final capture = core.calls('capture').single['capture']! as Map;
    expect(capture['via'], 'link');
    expect(capture['link'], 'https://example.org/apple-tree');
    expect(capture['text'], '');
  });

  testWidgets('an Unfiled Note is filed by hand', (tester) async {
    final core = FakeCore(brain: '/b')
      ..unfiledPrivate = false
      ..add('Home/keep.md')
      ..add('Inbox/n.md', text: 'Call the plumber.');
    await pumpApp(tester, core);
    await tapText(tester, 'Inbox');
    await tapText(tester, 'Call the plumber.');

    await tapTooltip(tester, 'File…');
    expect(find.text('File this Note'), findsOneWidget);
    await fill(tester, 'Title', 'Plumber');
    await fill(tester, 'Tags', 'house, todo');
    await tapText(tester, 'Brain');
    await tapText(tester, 'Home');
    await tapText(tester, 'Save');

    expect(core.notes.keys, contains('Home/Plumber.md'));
    expect(core.notes['Home/Plumber.md']!['tags'], ['house', 'todo']);
    expect(find.text('Plumber'), findsOneWidget);
    expect(find.text('#house #todo'), findsOneWidget);
  });

  testWidgets('a Note is edited and deleted', (tester) async {
    final core = FakeCore(brain: '/b')..add('Home/Plumber.md', text: 'old');
    await pumpApp(tester, core);
    await tapText(tester, 'Home');
    await tapText(tester, 'Plumber');

    await tapTooltip(tester, 'Edit');
    await fill(tester, 'Text', 'new text');
    await tapText(tester, 'Save');
    expect(core.notes['Home/Plumber.md']!['text'], 'new text');
    expect(find.text('new text'), findsOneWidget);

    await tapTooltip(tester, 'Delete');
    expect(find.text('Delete this Note with its Attachments?'), findsOneWidget);
    await tapText(tester, 'Delete');
    expect(core.notes, isEmpty);
    expect(find.text('No Notes here.'), findsOneWidget);
  });

  testWidgets('a search lists the Notes that hold the words', (tester) async {
    final core = FakeCore(brain: '/b')
      ..add('Home/Plumber.md', text: 'The boiler leaks.')
      ..add('Money/Tax.md', text: 'Before May.');
    await pumpApp(tester, core);
    await tapText(tester, 'Search');
    expect(find.text('Type words to search your Brain.'), findsOneWidget);

    await fill(tester, 'Search', 'boiler');
    expect(find.text('1 Note found.'), findsOneWidget);
    expect(find.text('The boiler leaks.'), findsOneWidget);
    await tapText(tester, 'Plumber');
    expect(core.calls('note').last['id'], 'Home/Plumber.md');
  });

  testWidgets('Folders are made, renamed and removed; the Inbox is fixed', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b');
    await pumpApp(tester, core);
    await tapTooltip(tester, 'New Folder…');
    await fill(tester, 'Name', 'Home');
    await tapText(tester, 'Create');
    expect(core.folders, contains('Home'));

    await tester.longPress(find.text('Home'));
    await tester.pumpAndSettle();
    await tapText(tester, 'Rename…');
    await fill(tester, 'Name', 'House');
    await tapText(tester, 'Rename');
    expect(core.folders, {'Inbox', 'House'});

    await tester.longPress(find.text('House'));
    await tester.pumpAndSettle();
    await tapText(tester, 'Remove');
    expect(core.folders, {'Inbox'});

    await tester.longPress(find.text('Inbox'));
    await tester.pumpAndSettle();
    expect(find.text('Rename…'), findsNothing);
  });

  testWidgets('a Folder that holds Notes is not removed, said in German', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b')..add('Home/keep.md');
    await pumpApp(tester, core, locale: 'de');
    expect(find.text('Eingang'), findsOneWidget);
    expect(find.text('Nichts wartet auf Ablage.'), findsOneWidget);

    await tester.longPress(find.text('Home'));
    await tester.pumpAndSettle();
    await tapText(tester, 'Entfernen');
    expect(core.folders, contains('Home'));
    expect(find.textContaining('Das geht nicht'), findsOneWidget);
  });

  testWidgets('the settings show the Brain and point the app elsewhere', (
    tester,
  ) async {
    final core = FakeCore(brain: '/storage/Sync/Brain');
    await pumpApp(tester, core);
    await tapText(tester, 'Settings');
    expect(find.text('/storage/Sync/Brain'), findsOneWidget);
    expect(find.text('Version 0.1.0'), findsOneWidget);

    await tapText(tester, 'Change folder…');
    await tapText(tester, 'Documents');
    await tapText(tester, 'Use this folder');
    expect(core.brain, '/storage/Documents');
  });
}
