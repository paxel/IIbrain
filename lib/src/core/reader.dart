import 'dart:async';

import 'core.dart';

/// Whether a reading is under way.
bool _reading = false;

/// Reads the Extracted Text of every Attachment that waits for it and
/// fetches every page that does, one after the other, beside the
/// screens. A second call while one runs does nothing.
void readWaitingTexts(Core core, {void Function()? onRead}) {
  if (_reading) return;
  _reading = true;
  unawaited(() async {
    try {
      while (await core.extractOne()) {
        onRead?.call();
      }
    } on CoreError {
      // What could not be read now waits for the next start.
    } finally {
      _reading = false;
    }
  }());
}
