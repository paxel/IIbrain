import 'package:flutter/material.dart';

void main() {
  runApp(const IIbrainApp());
}

/// The phone app. An empty shell so far.
class IIbrainApp extends StatelessWidget {
  const IIbrainApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(title: 'IIbrain', home: Scaffold());
  }
}
