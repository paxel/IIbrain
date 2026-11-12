import 'dart:io';

import 'package:flutter/services.dart';

/// What only the phone itself can answer: its storage and the access to
/// it. The widget tests stand in for it.
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
}

/// The phone, through Android.
class AndroidPhone implements Phone {
  const AndroidPhone();

  static const _storage = MethodChannel('iibrain/storage');

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
}
