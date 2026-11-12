//! The window: the Folders on the left, the Notes of the chosen Folder
//! beside them, and the opened Note.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use egui::{RichText, Ui};
use iibrain_core::{Core, Device, Folder, INBOX, Note};

use crate::l10n::L10n;

/// Asks the person for a folder; `None` when they cancel.
pub type PickFolder = Box<dyn FnMut() -> Option<PathBuf>>;

pub struct App {
    t: L10n,
    device_dir: PathBuf,
    core: Option<Core>,
    /// What went wrong last, in the person's language, until the next
    /// action.
    error: Option<String>,
    /// The Folder whose Notes are listed.
    folder: String,
    /// The Note that is open.
    open: Option<String>,
    refreshed: Instant,
    pick_folder: PickFolder,
}

impl App {
    /// The app of the Device whose own files lie in `device_dir`. It
    /// opens the Brain that Device remembers, if any.
    pub fn new(device_dir: &Path, locale: &str) -> Self {
        let core = Device::load(device_dir)
            .ok()
            .filter(|device| device.settings.brain.is_some())
            .and_then(|device| Core::open(device).ok());
        Self {
            t: L10n::new(locale),
            device_dir: device_dir.to_path_buf(),
            core,
            error: None,
            folder: INBOX.to_string(),
            open: None,
            refreshed: Instant::now(),
            pick_folder: Box::new(|| rfd::FileDialog::new().pick_folder()),
        }
    }

    /// Replaces the system's folder dialog, for a test.
    pub fn with_folder_picker(mut self, pick: PickFolder) -> Self {
        self.pick_folder = pick;
        self
    }

    pub fn t(&self) -> &L10n {
        &self.t
    }

    pub fn core(&self) -> Option<&Core> {
        self.core.as_ref()
    }

    /// Makes `folder` the Brain.
    pub fn choose_brain(&mut self, folder: &Path) {
        match Core::open_folder(&self.device_dir, folder) {
            Ok(core) => {
                self.core = Some(core);
                self.error = None;
                self.folder = INBOX.to_string();
                self.open = None;
            }
            Err(e) => self.error = Some(self.t.folder_not_opened(&e.to_string())),
        }
    }

    /// Takes in what other tools changed in the Brain's folder.
    pub fn refresh(&mut self) {
        self.refreshed = Instant::now();
        if let Some(core) = &mut self.core
            && let Err(e) = core.refresh()
        {
            self.error = Some(e.to_string());
        }
    }

    /// [`App::refresh`], when the last one is at least `every` ago.
    pub fn refresh_if_due(&mut self, every: Duration) {
        if self.refreshed.elapsed() >= every {
            self.refresh();
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if self.core.is_none() {
            self.show_first_start(ui);
            return;
        }
        egui::Panel::top("bar").show(ui, |ui| self.show_bar(ui));
        egui::Panel::left("folders")
            .resizable(true)
            .default_size(200.0)
            .show(ui, |ui| self.show_folders(ui));
        egui::Panel::left("notes")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| self.show_notes(ui));
        egui::CentralPanel::default().show(ui, |ui| self.show_note(ui));
    }

    fn pick_brain(&mut self) {
        if let Some(folder) = (self.pick_folder)() {
            self.choose_brain(&folder);
        }
    }

    fn show_first_start(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.set_max_width(520.0);
                ui.heading(self.t.choose_brain_title());
                ui.add_space(8.0);
                ui.label(self.t.choose_brain_body());
                ui.add_space(16.0);
                if ui.button(self.t.choose_folder()).clicked() {
                    self.pick_brain();
                }
                self.show_error(ui);
            });
        });
    }

    fn show_error(&self, ui: &mut Ui) {
        if let Some(error) = &self.error {
            ui.add_space(8.0);
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
    }

    fn show_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let root = self
                .core
                .as_ref()
                .map(|core| core.brain().root().display().to_string())
                .unwrap_or_default();
            ui.label(RichText::new(root).weak());
            if ui.button(self.t.change_folder()).clicked() {
                self.pick_brain();
            }
        });
        self.show_error(ui);
    }

    /// A Folder's name as the person reads it.
    fn folder_name(&self, folder: &str) -> String {
        match folder {
            "" => self.t.brain_root().to_string(),
            INBOX => self.t.inbox().to_string(),
            other => other.to_string(),
        }
    }

    fn show_folders(&mut self, ui: &mut Ui) {
        ui.strong(self.t.folders());
        let folders: Vec<Folder> = self
            .core
            .as_ref()
            .and_then(|core| core.folders().ok())
            .unwrap_or_default();
        let at_root = self
            .core
            .as_ref()
            .and_then(|core| core.notes("").ok())
            .map(|notes| notes.len())
            .unwrap_or(0);
        egui::ScrollArea::vertical()
            .id_salt("folders")
            .auto_shrink(false)
            .show(ui, |ui| {
                let mut rows: Vec<(String, usize)> =
                    folders.into_iter().map(|f| (f.name, f.notes)).collect();
                if at_root > 0 {
                    rows.push((String::new(), at_root));
                }
                for (name, notes) in rows {
                    let label = format!("{} ({notes})", self.folder_name(&name));
                    let chosen = self.folder == name;
                    if ui.add(egui::Button::selectable(chosen, label)).clicked() {
                        self.folder = name;
                        self.open = None;
                    }
                }
            });
    }

    fn show_notes(&mut self, ui: &mut Ui) {
        ui.strong(self.folder_name(&self.folder));
        let notes: Vec<Note> = self
            .core
            .as_ref()
            .and_then(|core| core.notes(&self.folder).ok())
            .unwrap_or_default();
        if notes.is_empty() {
            ui.weak(self.t.no_notes_here());
            return;
        }
        egui::ScrollArea::vertical()
            .id_salt("notes")
            .auto_shrink(false)
            .show(ui, |ui| {
                for note in notes {
                    let chosen = self.open.as_deref() == Some(note.id.as_str());
                    if ui
                        .add(egui::Button::selectable(chosen, note.title.as_str()))
                        .clicked()
                    {
                        self.open = Some(note.id);
                    }
                }
            });
    }

    fn show_note(&mut self, ui: &mut Ui) {
        let note = self
            .open
            .as_deref()
            .and_then(|id| self.core.as_ref()?.note(id).ok());
        let Some(note) = note else {
            // A Note another tool removed is no longer open.
            self.open = None;
            ui.weak(self.t.choose_note());
            return;
        };
        egui::ScrollArea::vertical()
            .id_salt("note")
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.heading(note.title.as_str());
                if !note.tags.is_empty() {
                    let tags: Vec<String> = note.tags.iter().map(|t| format!("#{t}")).collect();
                    ui.weak(tags.join(" "));
                }
                if let Some(captured) = &note.captured {
                    ui.weak(self.t.captured_on(&when(captured)));
                }
                ui.separator();
                ui.add(egui::Label::new(note.text.as_str()).selectable(true));
                if !note.attachments.is_empty() {
                    ui.separator();
                    ui.strong(self.t.attachments());
                    for attachment in &note.attachments {
                        ui.label(attachment.name.as_str());
                        if let Some(text) = attachment.extracted.as_deref()
                            && !text.is_empty()
                        {
                            ui.weak(self.t.extracted_text());
                            ui.add(egui::Label::new(text).selectable(true));
                        }
                    }
                }
            });
    }
}

/// A capture time as the person reads it: the day and the minute,
/// without seconds and offset.
fn when(captured: &str) -> String {
    let short: String = captured.chars().take(16).collect();
    short.replace('T', " ")
}
