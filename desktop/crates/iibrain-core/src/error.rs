//! What can go wrong in the core.

/// An error of the core. The apps show its kind in their own words.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reading or writing the Brain's folder or the Device's own files.
    #[error("file: {0}")]
    Io(#[from] std::io::Error),
    /// The index.
    #[error("index: {0}")]
    Index(#[from] rusqlite::Error),
    /// A Note, Folder, Attachment or Model that is not there.
    #[error("not found: {0}")]
    NotFound(String),
    /// A request the core cannot follow: a name that is taken, a Folder
    /// that still holds Notes, a path that leaves the Brain.
    #[error("not possible: {0}")]
    Invalid(String),
    /// A file whose content the core cannot read as what it should be.
    #[error("unreadable: {0}")]
    Unreadable(String),
    /// The Model did not answer, or not in a form the core understands.
    #[error("model: {0}")]
    Model(String),
    /// No Model is connected, or none is chosen.
    #[error("no model")]
    NoModel,
    /// The gate refused: this would send something Private to a Cloud
    /// Model (ADR 0004).
    #[error("private")]
    Private,
}

/// The core's result.
pub type Result<T> = std::result::Result<T, Error>;
