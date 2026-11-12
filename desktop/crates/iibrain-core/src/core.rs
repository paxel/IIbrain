//! The core's interface: what the phone app and the desktop app do
//! alike, over one Brain on one Device (ADR 0008).

use std::path::Path;

use chrono::Local;

use crate::brain::{Brain, Capture, Folder, INBOX, Note};
use crate::device::Device;
use crate::error::{Error, Result};
use crate::extract::{self, Engines};
use crate::index::{Hit, Index};
use crate::model::Model;

const INDEX: &str = "index.sqlite";

pub struct Core {
    pub(crate) brain: Brain,
    pub(crate) index: Index,
    pub(crate) device: Device,
    pub(crate) engines: Engines,
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
            engines: Engines::default(),
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

    // ------------------------------------------------------- models

    /// The Models this Device knows.
    pub fn models(&self) -> &[Model] {
        &self.device.settings.models
    }

    /// The Model that proposes and answers.
    pub fn chosen_model(&self) -> Option<&Model> {
        self.device.chosen_model()
    }

    /// Adds a Model, or replaces the one of the same name. The first
    /// Model added is the chosen one.
    pub fn save_model(&mut self, model: Model) -> Result<()> {
        if model.name.trim().is_empty() || model.address.trim().is_empty() {
            return Err(Error::Invalid(
                "a Model needs a name and an address".to_string(),
            ));
        }
        let models = &mut self.device.settings.models;
        match models.iter_mut().find(|m| m.name == model.name) {
            Some(known) => *known = model,
            None => models.push(model),
        }
        if self.device.settings.chosen.is_none() {
            self.device.settings.chosen = self.device.chosen_model().map(|m| m.name.clone());
        }
        self.device.save()
    }

    /// Removes a Model; when it was the chosen one, the first of the
    /// others is.
    pub fn remove_model(&mut self, name: &str) -> Result<()> {
        let settings = &mut self.device.settings;
        let before = settings.models.len();
        settings.models.retain(|m| m.name != name);
        if settings.models.len() == before {
            return Err(Error::NotFound(name.to_string()));
        }
        if settings.chosen.as_deref() == Some(name) {
            settings.chosen = settings.models.first().map(|m| m.name.clone());
        }
        self.device.save()
    }

    /// Makes the Model `name` the one that proposes and answers.
    pub fn choose_model(&mut self, name: &str) -> Result<()> {
        if !self.models().iter().any(|m| m.name == name) {
            return Err(Error::NotFound(name.to_string()));
        }
        self.device.settings.chosen = Some(name.to_string());
        self.device.save()
    }

    // ------------------------------------------------------- conflicts

    /// The versions of a Note in Conflict: its own file first, then
    /// the ones a Sync Tool left beside it.
    pub fn versions(&self, id: &str) -> Result<Vec<Note>> {
        let note = self.brain.read(id)?;
        let mut versions = vec![self.brain.read_version(id)?];
        for version in &note.conflicts {
            versions.push(self.brain.read_version(version)?);
        }
        Ok(versions)
    }

    /// Ends a Conflict by keeping the version `version`; the files of
    /// the others are removed.
    pub fn keep_version(&mut self, id: &str, version: &str) -> Result<()> {
        self.brain.keep_version(id, version)?;
        self.refresh()?;
        Ok(())
    }

    /// Ends a Conflict with a text the person merged; the files of the
    /// other versions are removed.
    pub fn resolve(&mut self, id: &str, text: &str) -> Result<()> {
        self.brain.resolve(id, text)?;
        self.refresh()?;
        Ok(())
    }

    // ------------------------------------------------------- privacy

    /// Whether `note` is Private: by its own mark, by its Folder's, or
    /// as an Unfiled Note while those count as Private.
    pub fn is_private(&self, note: &Note) -> bool {
        self.brain.is_private(note)
    }

    /// Whether `note` may be sent to `model`: the gate every send to a
    /// Model passes.
    pub fn may_send(&self, note: &Note, model: &Model) -> bool {
        crate::privacy::may_send(self.is_private(note), model)
    }

    /// Sets or lifts a Note's own Private mark.
    pub fn set_private(&mut self, id: &str, private: bool) -> Result<()> {
        self.brain.set_private(id, private)?;
        self.refresh()?;
        Ok(())
    }

    /// Sets or lifts the Private mark of a Folder.
    pub fn set_folder_private(&mut self, folder: &str, private: bool) -> Result<()> {
        if folder.is_empty() {
            return Err(Error::Invalid("the Brain's root is no Folder".to_string()));
        }
        self.brain.set_folder_private(folder, private)
    }

    /// Whether Unfiled Notes count as Private.
    pub fn unfiled_private(&self) -> bool {
        self.brain.unfiled_private()
    }

    pub fn set_unfiled_private(&mut self, private: bool) -> Result<()> {
        self.brain.set_unfiled_private(private)
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

    /// Reads the Extracted Text of one Attachment that still waits for
    /// it; says whether there was one. The apps call this until it
    /// says no, or run [`extract::extract_one`] beside themselves.
    pub fn extract_one(&mut self) -> Result<bool> {
        let read = extract::extract_one(&self.brain, &mut self.engines)?;
        if read {
            self.refresh()?;
        }
        Ok(read)
    }

    /// Reads the Extracted Text of every Attachment that waits for it.
    pub fn extract_pending(&mut self) -> Result<usize> {
        let mut read = 0;
        while self.extract_one()? {
            read += 1;
        }
        Ok(read)
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
