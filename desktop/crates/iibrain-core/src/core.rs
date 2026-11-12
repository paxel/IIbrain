//! The core's interface: what the phone app and the desktop app do
//! alike, over one Brain on one Device (ADR 0008).

use std::path::Path;

use crate::brain::{Brain, Folder, INBOX, Note};
use crate::device::Device;
use crate::error::{Error, Result};
use crate::index::Index;

const INDEX: &str = "index.sqlite";

pub struct Core {
    pub(crate) brain: Brain,
    pub(crate) index: Index,
    pub(crate) device: Device,
}

impl Core {
    /// Opens the Brain the Device's settings name and brings the index
    /// in step with its folder.
    pub fn open(device: Device) -> Result<Self> {
        let folder = device
            .settings
            .brain
            .clone()
            .ok_or_else(|| Error::NotFound("no Brain chosen".to_string()))?;
        let brain = Brain::open(&folder)?;
        let index = Index::open(&device.dir().join(INDEX), brain.root())?;
        let mut core = Self {
            brain,
            index,
            device,
        };
        core.refresh()?;
        Ok(core)
    }

    /// Makes `folder` the Brain of the Device in `device_dir` and opens
    /// it.
    pub fn open_folder(device_dir: &Path, folder: &Path) -> Result<Self> {
        let mut device = Device::load(device_dir)?;
        Brain::open(folder)?;
        device.settings.brain = Some(folder.to_path_buf());
        device.save()?;
        Self::open(device)
    }

    /// Takes in what changed in the Brain's folder, by this app or any
    /// other tool, and says whether anything had.
    pub fn refresh(&mut self) -> Result<bool> {
        self.index.refresh(&self.brain)
    }

    pub fn brain(&self) -> &Brain {
        &self.brain
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Every Folder, the Inbox first.
    pub fn folders(&self) -> Result<Vec<Folder>> {
        self.brain.folders()
    }

    /// The Notes lying directly in `folder`, newest Capture first.
    pub fn notes(&self, folder: &str) -> Result<Vec<Note>> {
        Ok(self
            .index
            .notes()?
            .into_iter()
            .filter(|n| n.folder == folder)
            .collect())
    }

    /// Every Note of the Brain.
    pub fn all_notes(&self) -> Result<Vec<Note>> {
        self.index.notes()
    }

    pub fn note(&self, id: &str) -> Result<Note> {
        self.index.note(id)
    }

    /// How many Notes wait Unfiled.
    pub fn unfiled(&self) -> Result<usize> {
        Ok(self.notes(INBOX)?.len())
    }
}
