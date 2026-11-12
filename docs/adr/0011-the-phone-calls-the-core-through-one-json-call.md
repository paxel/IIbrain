# The phone app calls the core through one JSON call over dart:ffi

The Flutter phone app reaches the Rust core (ADR 0008) through a single C function: it hands in one request as JSON text and gets one answer as JSON text. The core is built as a shared library per Android ABI with cargo-ndk, as Old Brain Fix builds its text recognition, and loaded with `dart:ffi`. The requests are the same commands the desktop app calls directly in Rust, so there is one list of what the core can do.

One call keeps the unsafe surface to two functions (call, and free the answer), needs no code generator in the build, and gives the phone's widget tests their seam: a stand-in that answers the same requests.

## Considered Options

- A generated bridge (flutter_rust_bridge) (rejected: a code generator and its runtime in every build, for an interface that is a handful of commands).
- A platform channel through Kotlin and JNI, as Old Brain Fix's text recognition uses (rejected: every command would be written three times, in Dart, Kotlin and Rust).

## Consequences

- Photos, PDFs and recordings are not passed through the call; the phone hands the core a path to a file.
- A call blocks until the core answers, so the phone makes it off the UI isolate.
