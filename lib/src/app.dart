import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'core/core.dart';
import 'l10n.dart';
import 'phone.dart';
import 'screens/start_page.dart';

/// What every screen reaches for: the core and the phone.
class AppScope extends InheritedWidget {
  const AppScope({
    super.key,
    required this.core,
    required this.phone,
    required super.child,
  });

  final Core core;
  final Phone phone;

  static AppScope of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<AppScope>();
    if (scope == null) throw StateError('no AppScope above this widget');
    return scope;
  }

  @override
  bool updateShouldNotify(AppScope oldWidget) =>
      core != oldWidget.core || phone != oldWidget.phone;
}

/// The phone app.
class IIbrainApp extends StatelessWidget {
  const IIbrainApp({
    super.key,
    required this.calls,
    required this.phone,
    this.locale,
  });

  /// The way into the core.
  final CoreCalls calls;
  final Phone phone;

  /// The language, for a test; the system's otherwise.
  final Locale? locale;

  @override
  Widget build(BuildContext context) {
    return AppScope(
      core: Core(calls),
      phone: phone,
      child: MaterialApp(
        onGenerateTitle: (context) => context.t.appTitle,
        locale: locale,
        localizationsDelegates: const [
          AppLocalizations.delegate,
          GlobalMaterialLocalizations.delegate,
          GlobalWidgetsLocalizations.delegate,
          GlobalCupertinoLocalizations.delegate,
        ],
        supportedLocales: AppLocalizations.supportedLocales,
        theme: ThemeData(colorSchemeSeed: Colors.indigo),
        darkTheme: ThemeData(
          colorSchemeSeed: Colors.indigo,
          brightness: Brightness.dark,
        ),
        home: const StartPage(),
      ),
    );
  }
}

/// Shows what went wrong, in the person's language.
void showError(BuildContext context, Object error) {
  ScaffoldMessenger.of(context)
    ..hideCurrentSnackBar()
    ..showSnackBar(SnackBar(content: Text(errorText(context.t, error))));
}
