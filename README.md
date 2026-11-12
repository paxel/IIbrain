# IIbrain

A private second brain for Android, Linux, macOS and Windows. Everything worth
keeping goes in from the phone or the computer, gets structured, and can be
asked later, with answers drawn only from your own notes.

**Work in progress. Nothing is released yet, and the apps do nothing so far.**

## What it will be

- **Your files.** A Brain is a folder of Markdown files with their Attachments
  beside them. Any other tool can read it; the app's database is only an index.
- **Your sync.** The app reads and writes a local folder. What carries that
  folder between your Devices is a Sync Tool of your choice.
- **Your model.** IIbrain ships no language model. You connect one through an
  OpenAI-compatible endpoint: in the cloud, at home, or on your own machine.
- **Private stays local.** Notes and Folders marked Private reach only a Model
  you declared local.
- **Answers with Sources.** A Question is answered from the Notes found for it,
  and every Answer lists them.
- **Capture from anywhere.** Text, photos, recordings, links, files, and on the
  phone Print Capture: choose IIbrain as the printer in any app. Text in
  photos, PDFs and recordings is read on the Device.
- **No store, no account.** The apps will ship through a Homebrew tap, a Scoop
  bucket and an F-Droid repository.

The words with capitals are defined in [CONTEXT.md](CONTEXT.md). The decisions
behind all of this, with their reasons and open points, are in
[docs/adr](docs/adr).

## Layout

| Path                           | What                                                     |
|--------------------------------|----------------------------------------------------------|
| `lib/`, `test/`, `android/`    | The phone app (Flutter)                                  |
| `desktop/crates/iibrain-core`  | The Rust core, meant for both apps                       |
| `desktop/crates/iibrain-gui`   | The desktop app (Rust), binary `iibrain`                 |
| `CONTEXT.md`                   | The domain language                                      |
| `docs/adr/`                    | The decisions                                            |
| `docs/agents/`                 | Notes for AI coding agents working in this repository    |

The phone app does not call the core yet.

## Build

```sh
# Phone app
flutter analyze
flutter test

# Desktop app and core
cd desktop
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p iibrain-gui
```

## License

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at
your option.
