import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:iibrain/main.dart';

void main() {
  testWidgets('the app starts and is called IIbrain', (tester) async {
    await tester.pumpWidget(const IIbrainApp());

    final app = tester.widget<MaterialApp>(find.byType(MaterialApp));
    expect(app.title, 'IIbrain');
    expect(find.byType(Scaffold), findsOneWidget);
  });
}
