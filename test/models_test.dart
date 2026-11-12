// Models, Private marks, Proposals, asking, Saved Answers and Conflicts
// on the phone (#26).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:iibrain/src/core/core.dart';

import 'support.dart';

Map<String, Object?> _model(String name, {required bool local}) => {
  'name': name,
  'address': 'http://10.0.0.2:11434/v1',
  'key': null,
  'model': 'llama3',
  'local': local,
};

void main() {
  group('Models', () {
    testWidgets('a Model is tested, added and shown as local or cloud', (
      tester,
    ) async {
      final core = FakeCore(brain: '/b');
      await pumpApp(tester, core);
      await tapText(tester, 'Settings');
      expect(find.textContaining('No Model is connected.'), findsOneWidget);

      await tapText(tester, 'Models');
      await tapText(tester, 'Connect a Model');
      expect(
        find.textContaining('stay on this device and never enter'),
        findsOneWidget,
      );
      await fill(tester, 'Name', 'home');
      await fill(tester, 'Address', 'http://10.0.0.2:11434/v1');
      await fill(tester, 'Key', 'secret');
      await fill(tester, 'Model at the endpoint', 'llama3');
      await tapText(tester, 'It runs under my own control (Local Model)');
      await tapText(tester, 'Test and add');

      expect(core.calls('test_model'), hasLength(1));
      expect(core.models.single, {
        'name': 'home',
        'address': 'http://10.0.0.2:11434/v1',
        'key': 'secret',
        'model': 'llama3',
        'local': true,
      });
      expect(find.text('home'), findsOneWidget);
      expect(find.text('Local Model'), findsOneWidget);
    });

    testWidgets('a Model that does not answer is told and can be added '
        'untested', (tester) async {
      final core = FakeCore(brain: '/b')
        ..answers['test_model'] = (_) =>
            throw const CoreError('model', 'the key was refused (401)');
      await pumpApp(tester, core);
      await tapText(tester, 'Settings');
      await tapText(tester, 'Models');
      await tapText(tester, 'Connect a Model');
      await fill(tester, 'Name', 'big');
      await fill(tester, 'Address', 'https://api.example.org/v1');
      await tapText(tester, 'Test and add');
      expect(
        find.textContaining('The Model did not answer: the key was refused'),
        findsOneWidget,
      );
      expect(core.models, isEmpty);

      await tapText(tester, 'Add without testing');
      expect(core.models.single['local'], false);
      expect(find.text('Cloud Model'), findsOneWidget);
    });

    testWidgets('one of several Models is chosen, and one removed', (
      tester,
    ) async {
      final core = FakeCore(brain: '/b')
        ..models.addAll([
          _model('home', local: true),
          _model('big', local: false),
        ])
        ..chosen = 'home';
      await pumpApp(tester, core, locale: 'de');
      await tapText(tester, 'Einstellungen');
      await tapText(tester, 'Modelle');
      expect(find.text('Lokales Modell'), findsOneWidget);
      expect(find.text('Cloud-Modell'), findsOneWidget);

      await tapText(tester, 'big');
      expect(core.chosen, 'big');

      await tester.tap(find.byTooltip('Entfernen').first);
      await tester.pumpAndSettle();
      expect(core.models.single['name'], 'big');
    });
  });

  group('Private', () {
    testWidgets('a Note is marked Private and says why it is', (tester) async {
      final core = FakeCore(brain: '/b')..add('Health/Blood.md', text: 'x');
      await pumpApp(tester, core);
      await tapText(tester, 'Health');
      await tapText(tester, 'Blood');
      expect(
        find.text(
          'What is Private reaches only a Local Model, never a Cloud Model.',
        ),
        findsOneWidget,
      );

      await tapText(tester, 'Private');
      expect(core.notes['Health/Blood.md']!['private'], true);

      await tester.pageBack();
      await tester.pumpAndSettle();
      expect(find.text('Blood · Private'), findsOneWidget);
    });

    testWidgets('a Folder is marked Private and its Notes say so', (
      tester,
    ) async {
      final core = FakeCore(brain: '/b')..add('Health/Blood.md', text: 'x');
      await pumpApp(tester, core);
      await tester.longPress(find.text('Health'));
      await tester.pumpAndSettle();
      await tapText(tester, 'Mark Private');
      expect(core.privateFolders, {'Health'});
      expect(find.text('Health · Private'), findsOneWidget);

      await tapText(tester, 'Health · Private');
      await tapText(tester, 'Blood · Private');
      expect(find.text('Private because its Folder is.'), findsOneWidget);
    });

    testWidgets('Unfiled Notes are Private until the setting opens them', (
      tester,
    ) async {
      final core = FakeCore(brain: '/b')
        ..add('Inbox/n.md', text: 'fresh thought');
      await pumpApp(tester, core, locale: 'de');
      await tapText(tester, 'Eingang');
      expect(find.text('fresh thought · privat'), findsOneWidget);

      await tapText(tester, 'Nicht abgelegte Notizen gelten als privat');
      expect(core.unfiledPrivate, false);
      expect(find.text('fresh thought'), findsOneWidget);
    });
  });

  group('Proposals', () {
    FakeCore brain() => FakeCore(brain: '/b')
      ..unfiledPrivate = false
      ..add('Home/keep.md')
      ..add('Inbox/n.md', text: 'The boiler leaks.');

    Future<void> openFiling(WidgetTester tester) async {
      await tapText(tester, 'Inbox');
      await tapText(tester, 'The boiler leaks.');
      await tapTooltip(tester, 'File…');
    }

    testWidgets('a Proposal fills the fields and is saved as it is', (
      tester,
    ) async {
      final core = brain()
        ..answers['propose'] = (_) => {
          'title': 'Plumber for the boiler',
          'tags': ['house', 'repair'],
          'folder': 'Home',
          'new_folder': false,
          'model': 'big',
        };
      await pumpApp(tester, core);
      await openFiling(tester);
      await tapText(tester, 'Propose');
      expect(
        find.text('Proposal by big. Correct it or save it as it is.'),
        findsOneWidget,
      );
      expect(core.notes.keys, contains('Inbox/n.md'), reason: 'not filed yet');

      await tapText(tester, 'Save');
      expect(core.notes['Home/Plumber for the boiler.md']!['tags'], [
        'house',
        'repair',
      ]);
    });

    testWidgets('a Proposal is corrected, and a new Folder shows as new', (
      tester,
    ) async {
      final core = brain()
        ..answers['propose'] = (_) => {
          'title': 'Boiler',
          'tags': <Object?>[],
          'folder': 'Heating',
          'new_folder': true,
          'model': 'big',
        };
      await pumpApp(tester, core);
      await openFiling(tester);
      await tapText(tester, 'Propose');
      expect(find.text('Heating (new Folder)'), findsOneWidget);

      await fill(tester, 'Title', 'Boiler leak');
      await tapText(tester, 'Save');
      expect(core.notes.keys, contains('Heating/Boiler leak.md'));
    });

    testWidgets('a Private Note gets no Proposal from a Cloud Model and '
        'says why', (tester) async {
      final core = brain()
        ..answers['propose'] = (_) => throw const CoreError('private', '');
      await pumpApp(tester, core);
      await openFiling(tester);
      await tapText(tester, 'Propose');
      expect(
        find.textContaining('This is Private and your Model is a Cloud Model'),
        findsOneWidget,
      );

      // Filing by hand goes on.
      await fill(tester, 'Title', 'Boiler');
      await tapText(tester, 'Save');
      expect(core.notes.keys, contains('Boiler.md'));
    });
  });

  group('asking', () {
    Map<String, Object?> answer({int leftOut = 0}) => {
      'answer': {
        'question': 'When does the plumber come?',
        'text': 'Mr Brandt comes on Monday [1].',
        'sources': [
          {'id': 'Home/Plumber.md', 'title': 'Plumber', 'folder': 'Home'},
        ],
        'model': 'big',
        'left_out': leftOut,
        'private': false,
      },
    };

    Future<void> ask(WidgetTester tester, String question) async {
      await tester.tap(find.widgetWithText(NavigationDestination, 'Ask'));
      await tester.pumpAndSettle();
      await fill(tester, 'Question', question);
      await tester.tap(find.widgetWithText(FilledButton, 'Ask'));
      await tester.pumpAndSettle();
    }

    testWidgets('an Answer shows with its Sources, and a Source opens', (
      tester,
    ) async {
      final core = FakeCore(brain: '/b')
        ..add('Home/Plumber.md', text: 'The boiler leaks.')
        ..answers['ask'] = (_) => answer(leftOut: 1);
      await pumpApp(tester, core);
      await ask(tester, 'When does the plumber come?');

      expect(
        core.calls('ask').single['question'],
        'When does the plumber come?',
      );
      expect(
        find.text('Answer by big, from the Sources below alone:'),
        findsOneWidget,
      );
      expect(find.text('Mr Brandt comes on Monday [1].'), findsOneWidget);
      expect(
        find.textContaining('1 Private Note was left out'),
        findsOneWidget,
      );

      await tapText(tester, 'Plumber');
      expect(find.text('The boiler leaks.'), findsOneWidget);
    });

    testWidgets('nothing found is said', (tester) async {
      final core = FakeCore(brain: '/b')
        ..answers['ask'] = (_) => {'nothing': 0};
      await pumpApp(tester, core);
      await ask(tester, 'Peru?');
      expect(
        find.text(
          'Nothing in your Brain fits this Question. No Model was asked.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('without a Model the screen leads to the Models', (
      tester,
    ) async {
      final core = FakeCore(brain: '/b')
        ..answers['ask'] = (_) => throw const CoreError('no_model', '');
      await pumpApp(tester, core);
      await ask(tester, 'boiler?');
      expect(find.textContaining('No Model is connected.'), findsOneWidget);

      await tapText(tester, 'Models…');
      expect(find.text('Connect a Model'), findsOneWidget);
    });

    testWidgets('a failing Model is told', (tester) async {
      final core = FakeCore(brain: '/b')
        ..answers['ask'] = (_) =>
            throw const CoreError('model', 'not reached: timeout');
      await pumpApp(tester, core);
      await ask(tester, 'boiler?');
      expect(
        find.textContaining('The Model did not answer: not reached'),
        findsOneWidget,
      );
    });

    testWidgets('an Answer is kept as a Note that shows its mark', (
      tester,
    ) async {
      final core = FakeCore(brain: '/b')
        ..add('Home/Plumber.md', text: 'The boiler leaks.')
        ..answers['ask'] = (_) => answer();
      core.answers['save_answer'] = (request) {
        core.add(
          'Inbox/answer.md',
          title: 'When does the plumber come?',
          text: 'Mr Brandt comes on Monday [1].',
          more: {
            'model': 'big',
            'sources': ['Home/Plumber.md', 'Home/Gone.md'],
          },
        );
        return 'Inbox/answer.md';
      };
      await pumpApp(tester, core);
      await ask(tester, 'When does the plumber come?');
      await tapText(tester, 'Keep as a Note');

      final kept = core.calls('save_answer').single['answer']! as Map;
      expect(kept['text'], 'Mr Brandt comes on Monday [1].');
      expect(
        find.text('Written by the Model big, from these Sources:'),
        findsOneWidget,
      );
      expect(find.text('Home/Gone.md (no longer there)'), findsOneWidget);
    });
  });

  group('Conflicts', () {
    FakeCore brain() {
      final core = FakeCore(brain: '/b')
        ..add(
          'Home/Plumber.md',
          text: 'Monday.',
          more: {
            'conflicts': ['Home/Plumber.sync-conflict-1.md'],
          },
        );
      core.answers['versions'] = (_) => [
        {'id': 'Home/Plumber.md', 'text': 'Monday.'},
        {'id': 'Home/Plumber.sync-conflict-1.md', 'text': 'Tuesday.'},
      ];
      void end(String text) {
        core.notes['Home/Plumber.md']!
          ..['text'] = text
          ..['conflicts'] = <Object?>[];
      }

      core.answers['keep_version'] = (request) {
        end(request['version'] == 'Home/Plumber.md' ? 'Monday.' : 'Tuesday.');
        return null;
      };
      core.answers['resolve'] = (request) {
        end(request['text']! as String);
        return null;
      };
      return core;
    }

    testWidgets('a Note in Conflict is marked and one version is kept', (
      tester,
    ) async {
      final core = brain();
      await pumpApp(tester, core);
      await tapText(tester, 'Home');
      await tapText(tester, 'Plumber · Conflict');
      expect(
        find.textContaining('2 versions of this Note lie side by side'),
        findsOneWidget,
      );

      await tapText(tester, 'Resolve…');
      expect(find.text('Version 1'), findsOneWidget);
      expect(find.text('Tuesday.'), findsOneWidget);
      await tapText(tester, 'Keep version 2');

      expect(
        core.calls('keep_version').single['version'],
        'Home/Plumber.sync-conflict-1.md',
      );
      expect(find.text('Tuesday.'), findsOneWidget);
      expect(find.textContaining('versions of this Note'), findsNothing);
    });

    testWidgets('a merged text ends the Conflict, in German', (tester) async {
      final core = brain();
      await pumpApp(tester, core, locale: 'de');
      await tapText(tester, 'Home');
      await tapText(tester, 'Plumber · Konflikt');
      await tapText(tester, 'Auflösen…');
      await fill(tester, 'Zusammengeführter Text', 'Monday or Tuesday.');
      await tester.ensureVisible(find.text('Zusammengeführten Text behalten'));
      await tapText(tester, 'Zusammengeführten Text behalten');

      expect(core.calls('resolve').single['text'], 'Monday or Tuesday.');
      expect(find.text('Monday or Tuesday.'), findsOneWidget);
    });
  });
}
