import 'dart:convert';

/// How a request reaches the core: one request as JSON text in, one
/// answer as JSON text out (docs/adr/0011). The app's one way into the
/// core, and the seam its widget tests stand in at.
abstract interface class CoreCalls {
  Future<String> send(String request);
}

/// What the core refused or failed at.
class CoreError implements Exception {
  const CoreError(this.kind, this.message);

  /// `file`, `index`, `not_found`, `invalid`, `unreadable`, `model`,
  /// `no_model` or `private`.
  final String kind;

  /// The detail after the kind, which the app wraps in its own words.
  final String message;

  @override
  String toString() => 'CoreError($kind, $message)';
}

/// Sends the call [name] with [arguments] and gives what stands under
/// `ok`; an `error` is thrown as a [CoreError].
Future<Object?> callCore(
  CoreCalls calls,
  String name, [
  Map<String, Object?> arguments = const {},
]) async {
  final answer = await calls.send(jsonEncode({'call': name, ...arguments}));
  final Object? decoded = jsonDecode(answer);
  if (decoded is! Map<String, Object?>) {
    throw const CoreError('invalid', 'the core gave no answer');
  }
  final error = decoded['error'];
  if (error is Map<String, Object?>) {
    throw CoreError(
      error['kind'] as String? ?? 'invalid',
      error['message'] as String? ?? '',
    );
  }
  return decoded['ok'];
}
