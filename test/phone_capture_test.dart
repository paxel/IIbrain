// What only the phone captures: the camera, files, what other apps
// share, and Print Capture; and sharing a Note out (#23, #24).

import 'package:flutter_test/flutter_test.dart';
import 'package:iibrain/src/phone.dart';

import 'support.dart';

Map<String, Object?> _capture(FakeCore core, [int index = 0]) =>
    core.calls('capture')[index]['capture']! as Map<String, Object?>;

void main() {
  testWidgets('a photo from the camera becomes a Capture', (tester) async {
    final core = FakeCore(brain: '/b');
    final phone = FakePhone()..photo = '/cache/captures/photo-1.jpg';
    await pumpApp(tester, core, phone: phone);
    await tapTooltip(tester, 'New Note');
    await tapText(tester, 'Take a photo');
    expect(find.text('photo-1.jpg'), findsOneWidget);
    await tapText(tester, 'Save');

    expect(_capture(core)['via'], 'photo');
    expect(_capture(core)['files'], ['/cache/captures/photo-1.jpg']);
    expect(phone.discarded, ['/cache/captures/photo-1.jpg']);
    expect(find.text('1 Note waits Unfiled.'), findsOneWidget);
  });

  testWidgets('picked files become a Capture, and one can be taken out', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b');
    final phone = FakePhone()
      ..picked = ['/cache/captures/scan.pdf', '/cache/captures/list.txt'];
    await pumpApp(tester, core, phone: phone);
    await tapTooltip(tester, 'New Note');
    await tapText(tester, 'Add files…');
    await tapTooltip(tester, 'Remove list.txt');
    await fill(tester, 'Text', 'the contract');
    await tapText(tester, 'Save');

    expect(_capture(core)['via'], 'file');
    expect(_capture(core)['text'], 'the contract');
    expect(_capture(core)['files'], ['/cache/captures/scan.pdf']);
  });

  testWidgets('backing out of the camera adds nothing', (tester) async {
    final core = FakeCore(brain: '/b');
    await pumpApp(tester, core, phone: FakePhone());
    await tapTooltip(tester, 'New Note');
    await tapText(tester, 'Take a photo');
    expect(find.byTooltip('Remove photo-1.jpg'), findsNothing);
  });

  testWidgets('what another app shared before the start is kept', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b');
    final phone = FakePhone()
      ..shared = const Shared(
        text: 'look at this',
        files: ['/cache/captures/receipt.jpg'],
      );
    await pumpApp(tester, core, phone: phone);

    expect(_capture(core)['via'], 'share');
    expect(_capture(core)['text'], 'look at this');
    expect(_capture(core)['files'], ['/cache/captures/receipt.jpg']);
    expect(phone.discarded, ['/cache/captures/receipt.jpg']);
    expect(find.text('1 Capture was kept in your Inbox.'), findsOneWidget);
    expect(find.text('1 Note waits Unfiled.'), findsOneWidget);
  });

  testWidgets('a link shared while the app runs is kept as a link', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b');
    final phone = FakePhone();
    await pumpApp(tester, core, phone: phone);
    expect(core.calls('capture'), isEmpty);

    phone.shareIn(const Shared(text: 'https://example.org/apple-tree'));
    await tester.pumpAndSettle();
    expect(_capture(core)['via'], 'link');
    expect(_capture(core)['link'], 'https://example.org/apple-tree');
  });

  testWidgets('what was printed to IIbrain is taken into the Inbox', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b');
    final phone = FakePhone()
      ..prints.addAll([
        '/data/prints/Statement.pdf',
        '/data/prints/Ticket.pdf',
      ]);
    await pumpApp(tester, core, phone: phone, locale: 'de');

    expect(core.calls('capture'), hasLength(2));
    expect(_capture(core)['via'], 'print');
    expect(_capture(core)['files'], ['/data/prints/Statement.pdf']);
    expect(_capture(core, 1)['files'], ['/data/prints/Ticket.pdf']);
    expect(phone.prints, isEmpty, reason: 'taken once');
    expect(
      find.text('2 Erfassungen liegen in deinem Eingang.'),
      findsOneWidget,
    );
  });

  testWidgets('the settings explain Print Capture and open its switch', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b');
    final phone = FakePhone();
    await pumpApp(tester, core, phone: phone);
    await tapText(tester, 'Settings');
    expect(find.text('Print Capture'), findsOneWidget);
    expect(
      find.textContaining('choose IIbrain as the printer in any app'),
      findsOneWidget,
    );
    await tapTooltip(tester, 'Open the print settings');
    expect(phone.printSettingsOpened, 1);
  });

  testWidgets('a Note is shared out with its text and its Attachments', (
    tester,
  ) async {
    final core = FakeCore(brain: '/b')
      ..add(
        'Home/Plumber.md',
        text: 'He comes on Monday.',
        more: {
          'attachments': [
            {'name': 'card.jpg'},
          ],
        },
      );
    final phone = FakePhone();
    await pumpApp(tester, core, phone: phone);
    await tapText(tester, 'Home');
    await tapText(tester, 'Plumber');
    await tapTooltip(tester, 'Share');

    final (text, files) = phone.sharedOut.single;
    expect(text, 'Plumber\n\nHe comes on Monday.');
    expect(files, ['/b/Home/Plumber.md.files/card.jpg']);
  });
}
