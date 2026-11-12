import 'package:flutter/material.dart';

import 'src/app.dart';
import 'src/core/ffi_calls.dart';
import 'src/phone.dart';

void main() {
  runApp(const IIbrainApp(calls: FfiCalls(), phone: AndroidPhone()));
}
