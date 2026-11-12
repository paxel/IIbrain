# One Rust core for phone and desktop, in a fresh codebase

IIbrain is a new codebase in Old Brain Fix's repo layout, with one Rust core that holds everything both apps do: the folder, the index, search, text extraction and Model access. The Flutter phone app calls the core; the Rust desktop app uses it directly. Phone and desktop are equal; only what a phone alone has stays phone-only (Print Capture, camera, Live Lookups).

Old Brain Fix has two cores, a Dart one on the phone and a Rust one on the desktop, with every rule written twice and held equal by shared fixtures. That was the price of reusing cat(a)log's sync. IIbrain has no such inheritance, and the Rust engines for text recognition and Whisper already run on the phone there, so one core halves the cost of every later feature.

## Considered Options

- Forking Old Brain Fix (rejected: its domain is a memory aid with Routines, Reminders and an Agenda; most of it would be deleted first).
- Two cores as in Old Brain Fix (rejected: double work with nothing gained).
- Flutter for phone and desktop with one Dart core (rejected: the engines are Rust, and the desktop tooling and packaging of Old Brain Fix are Rust).

## Consequences

- Lifted from Old Brain Fix as needed: text recognition, Whisper, packaging and release tooling. Not lifted: its sync, its log format, its Dart core.
- The phone app needs a bridge from Flutter into the core for everything, not only for the engines.
