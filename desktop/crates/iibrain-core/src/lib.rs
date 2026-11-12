//! The core of IIbrain: everything the phone app and the desktop app do
//! alike (docs/adr/0008).

/// The version of the apps, the same as the pubspec's.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
