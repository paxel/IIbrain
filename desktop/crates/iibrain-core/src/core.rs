//! The core's interface: what the phone app and the desktop app do
//! alike, over one Brain on one Device (ADR 0008).

use std::path::Path;

use chrono::Local;

use crate::brain::{Brain, Capture, Folder, INBOX, Note};
use crate::device::Device;
use crate::error::{Error, Result};
use crate::index::{Hit, Index};

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

    /// The Notes that hold every word of `query`, in their title, tags,
    /// text or Extracted Text, best match first.
    pub fn search(&self, query: &str) -> Result<Vec<Hit>> {
        self.index.search(&[query.to_string()], false, 200)
    }

    // ------------------------------------------------------- writing

    /// Keeps a Capture as an Unfiled Note and says its id.
    pub fn capture(&mut self, capture: &Capture) -> Result<String> {
        let id = self.brain.capture(capture, Local::now())?;
        self.refresh()?;
        Ok(id)
    }

    /// Gives a Note its title, tags and Folder and says its id from
    /// then on: Filing for an Unfiled Note; moving, retitling and
    /// retagging for a filed one.
    pub fn file(&mut self, id: &str, title: &str, tags: &[String], folder: &str) -> Result<String> {
        let id = self.brain.file(id, title, tags, folder)?;
        self.refresh()?;
        Ok(id)
    }

    /// Replaces a Note's text.
    pub fn set_text(&mut self, id: &str, text: &str) -> Result<()> {
        self.brain.set_text(id, text)?;
        self.refresh()?;
        Ok(())
    }

    /// Removes a Note with its Attachments.
    pub fn delete(&mut self, id: &str) -> Result<()> {
        self.brain.delete(id)?;
        self.refresh()?;
        Ok(())
    }

    pub fn create_folder(&mut self, name: &str) -> Result<()> {
        self.brain.create_folder(name)
    }

    /// Renames a Folder with everything in it; the Inbox keeps its
    /// name.
    pub fn rename_folder(&mut self, name: &str, new_name: &str) -> Result<()> {
        self.brain.rename_folder(name, new_name)?;
        self.refresh()?;
        Ok(())
    }

    /// Removes a Folder that holds nothing; the Inbox stays.
    pub fn remove_folder(&mut self, name: &str) -> Result<()> {
        self.brain.remove_folder(name)
    }
}
