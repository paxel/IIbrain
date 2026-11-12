//! What belongs to one Device and never enters the Brain's folder: which
//! folder is the Brain, and later the Models with their keys (ADR 0003).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::Model;

const FILE: &str = "device.json";

/// The settings of a Device, kept as one file in the app's own
/// directory on that Device.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The folder that is the Brain, once the person chose one.
    pub brain: Option<PathBuf>,
    /// The Models this Device knows, with their addresses and keys.
    pub models: Vec<Model>,
    /// The name of the Model that proposes and answers.
    pub chosen: Option<String>,
    /// Whether what a Live Lookup read in the calendar may go to a
    /// Cloud Model; at the start it goes to Local Models only.
    pub calendar_to_cloud: bool,
    /// The same for the contacts.
    pub contacts_to_cloud: bool,
}

/// One installation of IIbrain.
#[derive(Debug, Clone)]
pub struct Device {
    dir: PathBuf,
    pub settings: Settings,
}

impl Device {
    /// The Device whose own files lie in `dir`; the directory is made
    /// when it is missing.
    pub fn load(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir)?;
        let settings = match fs::read_to_string(dir.join(FILE)) {
            Ok(text) => {
                serde_json::from_str(&text).map_err(|e| Error::Unreadable(e.to_string()))?
            }
            Err(_) => Settings::default(),
        };
        Ok(Self {
            dir: dir.to_path_buf(),
            settings,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn save(&self) -> Result<()> {
        let text = serde_json::to_string_pretty(&self.settings)
            .map_err(|e| Error::Unreadable(e.to_string()))?;
        fs::write(self.dir.join(FILE), text)?;
        Ok(())
    }

    /// The Model that proposes and answers: the chosen one, or the
    /// only one.
    pub fn chosen_model(&self) -> Option<&Model> {
        let models = &self.settings.models;
        models
            .iter()
            .find(|m| Some(&m.name) == self.settings.chosen.as_ref())
            .or_else(|| models.first())
    }
}
