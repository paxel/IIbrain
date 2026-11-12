import 'package:flutter/widgets.dart';

import '../l10n/app_localizations.dart';
import 'core/core.dart';

export '../l10n/app_localizations.dart';

extension AppStrings on BuildContext {
  /// The app's strings in the person's language.
  AppLocalizations get t => AppLocalizations.of(this);
}

/// A Folder's name as the person reads it.
String folderName(AppLocalizations t, String folder) => switch (folder) {
  '' => t.brainRoot,
  'Inbox' => t.inbox,
  _ => folder,
};

/// An error of the core in the person's language.
String errorText(AppLocalizations t, Object error) {
  if (error is! CoreError) return t.errorOther('$error');
  return switch (error.kind) {
    'file' => t.errorFile(error.message),
    'index' => t.errorIndex(error.message),
    'not_found' => t.errorNotFound(error.message),
    'invalid' => t.errorNotPossible(error.message),
    'model' => t.errorModel(error.message),
    'no_model' => t.errorNoModel,
    'private' => t.errorPrivate,
    _ => t.errorOther(error.message),
  };
}

/// A capture time as the person reads it: the day and the minute.
String when(String captured) {
  final short = captured.length > 16 ? captured.substring(0, 16) : captured;
  return short.replaceFirst('T', ' ');
}
