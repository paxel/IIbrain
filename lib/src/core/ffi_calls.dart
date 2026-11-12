import 'dart:ffi';
import 'dart:isolate';

import 'package:ffi/ffi.dart';

import 'calls.dart';

typedef _Call = Pointer<Utf8> Function(Pointer<Utf8> request);
typedef _FreeNative = Void Function(Pointer<Utf8> answer);
typedef _Free = void Function(Pointer<Utf8> answer);

/// The core's shared library, called through `dart:ffi`. Every call
/// runs off the UI isolate: the core answers when it is done, which for
/// a Model takes seconds. All isolates reach the one core of the
/// process.
class FfiCalls implements CoreCalls {
  const FfiCalls();

  @override
  Future<String> send(String request) => Isolate.run(() => _send(request));
}

String _send(String request) {
  final library = DynamicLibrary.open('libiibrain.so');
  final call = library.lookupFunction<_Call, _Call>('iibrain_call');
  final free = library.lookupFunction<_FreeNative, _Free>('iibrain_free');
  final asked = request.toNativeUtf8();
  try {
    final answer = call(asked);
    try {
      return answer.toDartString();
    } finally {
      free(answer);
    }
  } finally {
    malloc.free(asked);
  }
}
