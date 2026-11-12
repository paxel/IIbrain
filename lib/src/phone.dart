import 'dart:io';

import 'package:flutter/services.dart';

/// What another app shared into IIbrain.
class Shared {
  const Shared({this.text = '', this.files = const []});

  final String text;

  /// The paths of the shared files, copied where the app may read them.
  final List<String> files;
}

/// What only the phone itself can do: its storage and the access to it,
/// the camera, the files and what other apps share, the prints that
/// wait, and the Live Lookups. The widget tests stand in for it.
abstract interface class Phone {
  /// Whether the app may read and write shared storage by file paths
  /// (docs/adr/0002).
  Future<bool> storageGranted();

  /// Opens the system's page or dialog that grants it.
  Future<void> requestStorage();

  /// Where shared storage begins.
  Future<String> storageRoot();

  /// Where the Device keeps its own files.
  Future<String> deviceDir();

  /// The names of the folders in [path], sorted, hidden ones left out.
  Future<List<String>> folders(String path);

  Future<void> createFolder(String path);

  /// Has a camera app take a photo; its path, or null without one.
  Future<String?> takePhoto();

  /// Has the person pick files; their paths, copied where the app may
  /// read them.
  Future<List<String>> pickFiles();

  /// What another app shared and the app has not taken yet.
  Future<Shared?> takeShared();

  /// Has [listener] called when another app shares while this one runs.
  void onShared(void Function() listener);

  /// Hands [text] and the files at [files] to another app.
  Future<void> share(String text, List<String> files);

  /// The PDFs printed to IIbrain that wait to be taken into the Brain.
  Future<List<String>> waitingPrints();

  /// Opens the system's list of print services.
  Future<void> openPrintSettings();

  /// The events of the next fourteen days as text; null when the
  /// person does not let the app read the calendar.
  Future<String?> calendar();

  /// The contacts whose names share a word with [words], as text; null
  /// when the person does not let the app read them.
  Future<String?> contacts(List<String> words);

  /// Drops files the core has taken into the Brain.
  Future<void> discard(List<String> paths);
}

/// The phone, through Android.
class AndroidPhone implements Phone {
  const AndroidPhone();

  static const _storage = MethodChannel('iibrain/storage');
  static const _capture = MethodChannel('iibrain/capture');

  @override
  Future<bool> storageGranted() async =>
      await _storage.invokeMethod<bool>('granted') ?? false;

  @override
  Future<void> requestStorage() => _storage.invokeMethod<void>('request');

  @override
  Future<String> storageRoot() async =>
      await _storage.invokeMethod<String>('root') ?? '/storage/emulated/0';

  @override
  Future<String> deviceDir() async =>
      await _storage.invokeMethod<String>('deviceDir') ?? '';

  @override
  Future<List<String>> folders(String path) async {
    final names = <String>[];
    try {
      await for (final entry in Directory(path).list(followLinks: false)) {
        final name = entry.uri.pathSegments.lastWhere(
          (s) => s.isNotEmpty,
          orElse: () => '',
        );
        if (entry is Directory && name.isNotEmpty && !name.startsWith('.')) {
          names.add(name);
        }
      }
    } on FileSystemException {
      // A folder out of reach lists as empty.
    }
    names.sort((a, b) => a.toLowerCase().compareTo(b.toLowerCase()));
    return names;
  }

  @override
  Future<void> createFolder(String path) async {
    await Directory(path).create(recursive: true);
  }

  @override
  Future<String?> takePhoto() => _capture.invokeMethod<String>('takePhoto');

  @override
  Future<List<String>> pickFiles() async =>
      await _capture.invokeListMethod<String>('pickFiles') ?? const [];

  @override
  Future<Shared?> takeShared() async {
    final shared = await _capture.invokeMapMethod<String, Object?>(
      'takeShared',
    );
    if (shared == null) return null;
    return Shared(
      text: shared['text'] as String? ?? '',
      files: [
        if (shared['files'] case final List<Object?> files)
          for (final file in files)
            if (file is String) file,
      ],
    );
  }

  @override
  void onShared(void Function() listener) {
    _capture.setMethodCallHandler((call) async {
      if (call.method == 'shared') listener();
    });
  }

  @override
  Future<void> share(String text, List<String> files) =>
      _capture.invokeMethod<void>('share', {'text': text, 'files': files});

  @override
  Future<List<String>> waitingPrints() async =>
      await _capture.invokeListMethod<String>('waitingPrints') ?? const [];

  @override
  Future<void> openPrintSettings() =>
      _capture.invokeMethod<void>('openPrintSettings');

  @override
  Future<String?> calendar() => _capture.invokeMethod<String>('calendar');

  @override
  Future<String?> contacts(List<String> words) =>
      _capture.invokeMethod<String>('contacts', {'words': words});

  @override
  Future<void> discard(List<String> paths) async {
    for (final path in paths) {
      try {
        await File(path).delete();
      } on FileSystemException {
        // Gone already.
      }
    }
  }
}
