//! The IIbrain desktop app: the screens over the core.

pub mod app;
pub mod job;
pub mod l10n;

pub use app::App;

/// The app id: the window's, and the name of the launcher entry.
pub const APP_ID: &str = "io.github.paxel.iibrain";
