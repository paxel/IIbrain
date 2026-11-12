//! The core of IIbrain: everything the phone app and the desktop app do
//! alike (docs/adr/0008).

pub mod api;
pub mod asking;
pub mod brain;
pub mod core;
pub mod device;
pub mod error;
pub mod extract;
pub mod filing;
pub mod index;
pub mod model;
pub mod privacy;
pub mod web;

pub use crate::asking::{Answer, Asking, Found, Lookup, Source};
pub use crate::brain::{Attachment, Brain, Capture, Folder, INBOX, Note};
pub use crate::core::Core;
pub use crate::device::Device;
pub use crate::error::{Error, Result};
pub use crate::filing::{Proposal, ProposalRequest};
pub use crate::index::Hit;
pub use crate::model::{Message, Model};

/// The version of the apps, the same as the pubspec's.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
