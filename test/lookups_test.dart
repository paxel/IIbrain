// Live Lookups of calendar and contacts on the phone (#27).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support.dart';

Future<void> _openAsk(WidgetTester tester) async {
  await tester.tap(find.widgetWithText(NavigationDestination, 'Ask'));
  await tester.pumpAndSettle();
}

Future<void> _send(WidgetTester tester, String question) async {
  await fill(tester, 'Question', question);
  await tester.tap(find.widgetWithText(FilledButton, 'Ask'));
  await tester.pumpAndSettle();
}

void main() {
  FakeCore brain() =>
      FakeCore(brain: '/b')..answers['ask'] = (_) => {'nothing': 0};

  testWidgets('without the switches nothing is read', (tester) async {
    final core = brain();
    final phone = FakePhone();
    await pumpApp(tester, core, phone: phone);
    await _openAsk(tester);
    await _send(tester, 'When is the dentist?');

    expect(phone.calendarReads, 0);
    expect(phone.contactWords, isEmpty);
    expect(core.calls('ask').single['lookups'], <Object?>[]);
  });

  testWidgets('the calendar is read for one Question and handed over', (
    tester,
  ) async {
    final core = brain();
    final phone = FakePhone();
    await pumpApp(tester, core, phone: phone);
    await _openAsk(tester);
    await tapText(tester, 'Calendar');
    expect(
      find.text(
        'Read now, for this Question only. Nothing of it is kept in your Brain.',
      ),
      findsOneWidget,
    );
    await _send(tester, 'When is the dentist?');

    expect(core.calls('ask').single['lookups'], [
      {'kind': 'calendar', 'text': '2026-11-16 14:00 to 15:00: Dentist'},
    ]);
    expect(core.calls('capture'), isEmpty, reason: 'nothing is kept');

    // The switch is off again for the next Question.
    await _send(tester, 'And after that?');
    expect(phone.calendarReads, 1);
    expect(core.calls('ask').last['lookups'], <Object?>[]);
  });

  testWidgets('the contacts are read by the words of the Question', (
    tester,
  ) async {
    final core = brain();
    final phone = FakePhone();
    await pumpApp(tester, core, phone: phone, locale: 'de');
    await _openAsk(tester);
    await tapText(tester, 'Kontakte');
    await fill(tester, 'Frage', 'Wie erreiche ich Max Brandt?');
    await tester.tap(find.widgetWithText(FilledButton, 'Fragen'));
    await tester.pumpAndSettle();

    expect(phone.contactWords.single, [
      'Wie',
      'erreiche',
      'ich',
      'Max',
      'Brandt',
    ]);
    expect(core.calls('ask').single['lookups'], [
      {'kind': 'contacts', 'text': 'Max Brandt: 0151 555'},
    ]);
  });

  testWidgets('a refused permission is said, and the Question still asked', (
    tester,
  ) async {
    final core = brain();
    final phone = FakePhone()..events = null;
    await pumpApp(tester, core, phone: phone);
    await _openAsk(tester);
    await tapText(tester, 'Calendar');
    await _send(tester, 'When is the dentist?');

    expect(find.textContaining('IIbrain may not read this.'), findsOneWidget);
    expect(core.calls('ask').single['lookups'], <Object?>[]);
  });

  testWidgets('both go to Local Models only until a switch is opened', (
    tester,
  ) async {
    final core = brain();
    await pumpApp(tester, core);
    await tapText(tester, 'Settings');
    expect(find.text('Off: it goes to Local Models only.'), findsNWidgets(2));
    expect(core.calendarToCloud, false);
    expect(core.contactsToCloud, false);

    await tapText(tester, 'Contacts may go to Cloud Models');
    expect(core.contactsToCloud, true);
    expect(core.calendarToCloud, false);
  });
}
