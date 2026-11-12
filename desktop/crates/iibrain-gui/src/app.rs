//! The window: the Folders on the left, the Notes of the chosen Folder
//! beside them, the opened Note, and the dialogs over them.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use egui::{RichText, Ui};
use iibrain_core::{Answer, Capture, Core, Device, Error, Folder, INBOX, Model, Note, Proposal};

use crate::ask_view::Asked;
use crate::job::Job;
use crate::l10n::L10n;

/// Asks the person for a folder; `None` when they cancel.
pub type PickFolder = Box<dyn FnMut() -> Option<PathBuf>>;
/// Asks the person for files; none when they cancel.
pub type PickFiles = Box<dyn FnMut() -> Vec<PathBuf>>;

/// What lies over the window and waits for the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Dialog {
    /// A Capture: typed text, files, or both.
    Capture {
        text: String,
        files: Vec<PathBuf>,
    },
    /// A Note's text, being edited.
    EditText {
        id: String,
        text: String,
    },
    /// Filing an Unfiled Note, or title, tags and Folder of a filed one.
    File {
        id: String,
        title: String,
        tags: String,
        folder: String,
    },
    ConfirmDelete {
        id: String,
    },
    NewFolder {
        name: String,
    },
    RenameFolder {
        from: String,
        name: String,
    },
    /// The connected Models, and the fields for a new one.
    Models {
        name: String,
        address: String,
        key: String,
        model: String,
        local: bool,
    },
}

impl Dialog {
    /// The Models dialog with empty fields for a new Model.
    pub(crate) fn new_model() -> Self {
        Dialog::Models {
            name: String::new(),
            address: String::new(),
            key: String::new(),
            model: String::new(),
            local: false,
        }
    }
}

pub struct App {
    pub(crate) t: L10n,
    device_dir: PathBuf,
    pub(crate) core: Option<Core>,
    /// What went wrong last, in the person's language, until the next
    /// action.
    pub(crate) error: Option<String>,
    /// The Folder whose Notes are listed.
    pub(crate) folder: String,
    /// The Note that is open.
    pub(crate) open: Option<String>,
    pub(crate) dialog: Option<Dialog>,
    /// What the person searches for; empty shows the chosen Folder.
    pub(crate) search: String,
    refreshed: Instant,
    pick_folder: PickFolder,
    pick_files: PickFiles,
    /// Whether Extracted Text is read beside the window; a test reads
    /// it at once instead.
    pub(crate) beside: bool,
    /// Whether a reading beside the window is under way.
    reading: Arc<AtomicBool>,
    /// The Proposal the filing dialog waits for.
    proposing: Option<Job<Proposal>>,
    /// The Model whose Proposal fills the filing dialog.
    proposed_by: Option<String>,
    /// Whether the ask screen shows in place of the opened Note.
    pub(crate) ask_view: bool,
    pub(crate) question: String,
    /// The Answer the ask screen waits for.
    pub(crate) asking: Option<Job<Answer>>,
    pub(crate) asked: Option<Asked>,
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
            dialog: None,
            search: String::new(),
            refreshed: Instant::now(),
            pick_folder: Box::new(|| rfd::FileDialog::new().pick_folder()),
            pick_files: Box::new(|| rfd::FileDialog::new().pick_files().unwrap_or_default()),
            beside: true,
            reading: Arc::new(AtomicBool::new(false)),
            proposing: None,
            proposed_by: None,
            ask_view: false,
            question: String::new(),
            asking: None,
            asked: None,
        }
    }

    /// Replaces the system's folder dialog, for a test.
    pub fn with_folder_picker(mut self, pick: PickFolder) -> Self {
        self.pick_folder = pick;
        self
    }

    /// Replaces the system's file dialog, for a test.
    pub fn with_file_picker(mut self, pick: PickFiles) -> Self {
        self.pick_files = pick;
        self
    }

    /// Reads Extracted Text at once instead of beside the window, for a
    /// test.
    pub fn reading_at_once(mut self) -> Self {
        self.beside = false;
        self
    }

    /// Reads the Extracted Text of every Attachment that waits for it:
    /// beside the window, which takes the texts in as they come.
    pub fn read_waiting_texts(&mut self) {
        if !self.beside {
            self.act(|core| core.extract_pending());
            return;
        }
        let Some(brain) = self.core.as_ref().map(|core| core.brain().clone()) else {
            return;
        };
        if self.reading.swap(true, Ordering::SeqCst) {
            return;
        }
        let reading = Arc::clone(&self.reading);
        std::thread::spawn(move || {
            let mut engines = iibrain_core::extract::Engines::default();
            while let Ok(true) = iibrain_core::extract::extract_one(&brain, &mut engines) {}
            reading.store(false, Ordering::SeqCst);
        });
    }

    /// Adds `model` to this Device without trying its connection, and
    /// says whether that worked.
    pub fn connect(&mut self, model: Model) -> bool {
        self.act(|core| core.save_model(model)).is_some()
    }

    pub fn t(&self) -> &L10n {
        &self.t
    }

    pub fn core(&self) -> Option<&Core> {
        self.core.as_ref()
    }

    /// What the ask screen shows as the last Answer.
    pub fn answer(&self) -> Option<&Answer> {
        match &self.asked {
            Some(Asked::Answer(answer)) => Some(answer),
            _ => None,
        }
    }

    /// The id of the Note that is open.
    pub fn open_note(&self) -> Option<&str> {
        self.open.as_deref()
    }

    /// Confirms the dialog that asks before a Note is deleted, as its
    /// Delete button does.
    pub fn confirm_delete(&mut self) {
        if let Some(Dialog::ConfirmDelete { id }) = self.dialog.take()
            && self.act(|core| core.delete(&id)).is_some()
        {
            self.open = None;
        }
    }

    /// Makes `folder` the Brain.
    pub fn choose_brain(&mut self, folder: &Path) {
        match Core::open_folder(&self.device_dir, folder) {
            Ok(core) => {
                self.core = Some(core);
                self.error = None;
                self.folder = INBOX.to_string();
                self.open = None;
                self.dialog = None;
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
            self.error = Some(error_text(&self.t, &e));
        }
    }

    /// [`App::refresh`], when the last one is at least `every` ago.
    pub fn refresh_if_due(&mut self, every: Duration) {
        if self.refreshed.elapsed() >= every {
            self.refresh();
        }
    }

    /// Does `action` on the core and says what it gave; a failure is
    /// shown in the person's language.
    pub(crate) fn act<T>(
        &mut self,
        action: impl FnOnce(&mut Core) -> iibrain_core::Result<T>,
    ) -> Option<T> {
        let core = self.core.as_mut()?;
        match action(core) {
            Ok(value) => {
                self.error = None;
                Some(value)
            }
            Err(e) => {
                self.error = Some(error_text(&self.t, &e));
                None
            }
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
            .default_size(220.0)
            .show(ui, |ui| self.show_folders(ui));
        egui::Panel::left("notes")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| self.show_notes(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            if self.ask_view {
                self.show_ask(ui);
            } else {
                self.show_note(ui);
            }
        });
        self.show_dialog(ui);
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
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
    }

    fn show_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button(self.t.new_note()).clicked() {
                self.dialog = Some(Dialog::Capture {
                    text: String::new(),
                    files: Vec::new(),
                });
            }
            let toggle = if self.ask_view {
                self.t.back_to_notes()
            } else {
                self.t.ask()
            };
            if ui.button(toggle).clicked() {
                self.ask_view = !self.ask_view;
            }
            if ui.button(self.t.models()).clicked() {
                self.dialog = Some(Dialog::new_model());
            }
            ui.separator();
            let label = ui.label(self.t.search());
            ui.add(egui::TextEdit::singleline(&mut self.search).desired_width(240.0))
                .labelled_by(label.id);
            if !self.search.is_empty() && ui.button(self.t.clear_search()).clicked() {
                self.search.clear();
            }
            ui.separator();
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
    pub(crate) fn folder_name(&self, folder: &str) -> String {
        match folder {
            "" => self.t.brain_root().to_string(),
            INBOX => self.t.inbox().to_string(),
            other => other.to_string(),
        }
    }

    /// A Folder's name in the choice of a Note's Folder: one that
    /// does not exist yet says so.
    fn folder_choice(&self, folder: &str) -> String {
        let new = !folder.is_empty() && !self.folders().iter().any(|f| f.name == folder);
        if new {
            self.t.new_folder_suffix(folder)
        } else {
            self.folder_name(folder)
        }
    }

    pub(crate) fn folders(&self) -> Vec<Folder> {
        self.core
            .as_ref()
            .and_then(|core| core.folders().ok())
            .unwrap_or_default()
    }

    fn show_folders(&mut self, ui: &mut Ui) {
        ui.strong(self.t.folders());
        let folders = self.folders();
        let at_root = self
            .core
            .as_ref()
            .and_then(|core| core.notes("").ok())
            .map(|notes| notes.len())
            .unwrap_or(0);
        egui::Panel::bottom("folder_actions").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button(self.t.new_folder()).clicked() {
                    self.dialog = Some(Dialog::NewFolder {
                        name: String::new(),
                    });
                }
                // The Inbox is fixed, and the Brain's root is no Folder.
                if self.folder != INBOX && !self.folder.is_empty() {
                    if ui.button(self.t.rename_folder()).clicked() {
                        self.dialog = Some(Dialog::RenameFolder {
                            from: self.folder.clone(),
                            name: self.folder.clone(),
                        });
                    }
                    let private = self
                        .core
                        .as_ref()
                        .is_some_and(|core| core.brain().folder_private(&self.folder));
                    let toggle = if private {
                        self.t.folder_lift_private()
                    } else {
                        self.t.folder_mark_private()
                    };
                    if ui.button(toggle).clicked() {
                        let folder = self.folder.clone();
                        self.act(|core| core.set_folder_private(&folder, !private));
                    }
                    if ui.button(self.t.remove_folder()).clicked() {
                        let folder = self.folder.clone();
                        if self.act(|core| core.remove_folder(&folder)).is_some() {
                            self.folder = INBOX.to_string();
                            self.open = None;
                        }
                    }
                }
            });
        });
        egui::ScrollArea::vertical()
            .id_salt("folders")
            .auto_shrink(false)
            .show(ui, |ui| {
                let mut rows: Vec<(String, usize, bool)> = folders
                    .into_iter()
                    .map(|f| (f.name, f.notes, f.private))
                    .collect();
                if at_root > 0 {
                    rows.push((String::new(), at_root, false));
                }
                for (name, notes, private) in rows {
                    let label = format!("{} ({notes})", self.folder_name(&name));
                    let label = if private {
                        self.t.private_suffix(&label)
                    } else {
                        label
                    };
                    let chosen = self.folder == name;
                    if ui.add(egui::Button::selectable(chosen, label)).clicked() {
                        self.folder = name;
                        self.open = None;
                        self.ask_view = false;
                    }
                }
            });
    }

    fn show_hits(&mut self, ui: &mut Ui) {
        ui.strong(self.t.search());
        let hits = self
            .core
            .as_ref()
            .and_then(|core| core.search(&self.search).ok())
            .unwrap_or_default();
        ui.weak(
            self.t
                .search_results(i64::try_from(hits.len()).unwrap_or(i64::MAX)),
        );
        egui::ScrollArea::vertical()
            .id_salt("hits")
            .auto_shrink(false)
            .show(ui, |ui| {
                for hit in hits {
                    let chosen = self.open.as_deref() == Some(hit.id.as_str());
                    if ui
                        .add(egui::Button::selectable(chosen, hit.title.as_str()))
                        .clicked()
                    {
                        self.open = Some(hit.id);
                    }
                    ui.weak(hit.excerpt.as_str());
                    ui.add_space(4.0);
                }
            });
    }

    fn show_notes(&mut self, ui: &mut Ui) {
        if !self.search.trim().is_empty() {
            self.show_hits(ui);
            return;
        }
        ui.strong(self.folder_name(&self.folder));
        let notes: Vec<Note> = self
            .core
            .as_ref()
            .and_then(|core| core.notes(&self.folder).ok())
            .unwrap_or_default();
        if self.folder == INBOX {
            let count = i64::try_from(notes.len()).unwrap_or(i64::MAX);
            ui.weak(self.t.unfiled_waiting(count));
            let mut private = self
                .core
                .as_ref()
                .is_some_and(|core| core.unfiled_private());
            if ui
                .checkbox(&mut private, self.t.unfiled_private_setting())
                .changed()
            {
                self.act(|core| core.set_unfiled_private(private));
            }
        } else if notes.is_empty() {
            ui.weak(self.t.no_notes_here());
        }
        egui::ScrollArea::vertical()
            .id_salt("notes")
            .auto_shrink(false)
            .show(ui, |ui| {
                for note in notes {
                    let chosen = self.open.as_deref() == Some(note.id.as_str());
                    let private = self
                        .core
                        .as_ref()
                        .is_some_and(|core| core.is_private(&note));
                    let title = list_title(&note);
                    let title = if note.model.is_some() {
                        self.t.answer_suffix(&title)
                    } else {
                        title
                    };
                    let title = if private {
                        self.t.private_suffix(&title)
                    } else {
                        title
                    };
                    let clicked = ui.add(egui::Button::selectable(chosen, title)).clicked();
                    if clicked {
                        self.ask_view = false;
                    }
                    if clicked {
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
        ui.horizontal_wrapped(|ui| {
            if ui.button(self.t.edit()).clicked() {
                self.dialog = Some(Dialog::EditText {
                    id: note.id.clone(),
                    text: note.text.clone(),
                });
            }
            let file = if note.unfiled() {
                self.t.file_note()
            } else {
                self.t.note_details()
            };
            if ui.button(file).clicked() {
                self.proposing = None;
                self.proposed_by = None;
                self.dialog = Some(Dialog::File {
                    id: note.id.clone(),
                    title: if note.unfiled() {
                        String::new()
                    } else {
                        note.title.clone()
                    },
                    tags: note.tags.join(", "),
                    folder: if note.unfiled() {
                        String::new()
                    } else {
                        note.folder.clone()
                    },
                });
            }
            if ui.button(self.t.delete()).clicked() {
                self.dialog = Some(Dialog::ConfirmDelete {
                    id: note.id.clone(),
                });
            }
        });
        ui.separator();
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
                let mut private = note.private;
                if ui
                    .checkbox(&mut private, self.t.private_mark())
                    .on_hover_text(self.t.private_explain())
                    .changed()
                {
                    self.act(|core| core.set_private(&note.id, private));
                }
                let by_folder = self
                    .core
                    .as_ref()
                    .is_some_and(|core| core.brain().folder_private(&note.folder));
                let by_inbox = note.unfiled()
                    && self
                        .core
                        .as_ref()
                        .is_some_and(|core| core.unfiled_private());
                if by_folder {
                    ui.weak(self.t.private_by_folder());
                } else if by_inbox && !note.private {
                    ui.weak(self.t.private_unfiled());
                }
                ui.separator();
                ui.add(egui::Label::new(note.text.as_str()).selectable(true));
                if let Some(model) = &note.model {
                    ui.separator();
                    ui.weak(self.t.written_by_model(model));
                    for source in &note.sources {
                        let known = self.core.as_ref().and_then(|core| core.note(source).ok());
                        match known {
                            Some(known) => {
                                if ui.link(known.title.as_str()).clicked() {
                                    self.folder = known.folder;
                                    self.open = Some(known.id);
                                }
                            }
                            None => {
                                ui.weak(self.t.source_gone(source));
                            }
                        }
                    }
                }
                if let Some(link) = &note.link {
                    ui.separator();
                    ui.hyperlink(link);
                    match note.page_text.as_deref() {
                        Some("") => {
                            ui.weak(self.t.page_text_missing());
                        }
                        Some(text) => {
                            ui.weak(self.t.page_text());
                            ui.add(egui::Label::new(text).selectable(true));
                        }
                        None => {
                            ui.weak(self.t.page_fetching());
                        }
                    }
                }
                if !note.attachments.is_empty() {
                    ui.separator();
                    ui.strong(self.t.attachments());
                    for attachment in &note.attachments {
                        ui.label(attachment.name.as_str());
                        match attachment.extracted.as_deref() {
                            Some("") => {
                                ui.weak(self.t.no_text_found());
                            }
                            Some(text) => {
                                ui.weak(self.t.extracted_text());
                                ui.add(egui::Label::new(text).selectable(true));
                            }
                            None if iibrain_core::extract::kind_of(&attachment.name)
                                != iibrain_core::extract::Kind::Other =>
                            {
                                ui.weak(self.t.reading_text());
                            }
                            None => {}
                        }
                    }
                }
            });
    }

    // ------------------------------------------------------- dialogs

    fn show_dialog(&mut self, ui: &mut Ui) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        let title = match &dialog {
            Dialog::Capture { .. } => self.t.new_note(),
            Dialog::EditText { .. } => self.t.edit(),
            Dialog::File { id, .. } if id.starts_with(&format!("{INBOX}/")) => {
                self.t.file_note_title()
            }
            Dialog::File { .. } => self.t.note_details_title(),
            Dialog::ConfirmDelete { .. } => self.t.delete(),
            Dialog::NewFolder { .. } => self.t.new_folder_title(),
            Dialog::RenameFolder { .. } => self.t.rename_folder_title(),
            Dialog::Models { .. } => self.t.models_title(),
        };
        let mut keep = true;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                keep = self.dialog_body(ui, &mut dialog);
                self.show_error(ui);
            });
        // A dialog that opened another one keeps that one.
        if keep && self.dialog.is_none() {
            self.dialog = Some(dialog);
        }
    }

    /// Draws `dialog` and says whether it stays open.
    fn dialog_body(&mut self, ui: &mut Ui, dialog: &mut Dialog) -> bool {
        match dialog {
            Dialog::Capture { text, files } => {
                let label = ui.label(self.t.note_text());
                ui.add(
                    egui::TextEdit::multiline(text)
                        .desired_rows(8)
                        .desired_width(480.0),
                )
                .labelled_by(label.id);
                if ui.button(self.t.add_files()).clicked() {
                    files.extend((self.pick_files)());
                }
                for file in files.iter() {
                    let name = file.file_name().unwrap_or(file.as_os_str());
                    ui.label(name.to_string_lossy());
                }
                ui.weak(self.t.link_hint());
                ui.weak(self.t.phone_only());
                let something = !text.trim().is_empty() || !files.is_empty();
                match self.confirm_row(ui, self.t.save(), something) {
                    Some(true) => {
                        // A link on its own is a link Capture.
                        let link = iibrain_core::web::is_link(text) && files.is_empty();
                        let capture = Capture {
                            text: if link { String::new() } else { text.clone() },
                            via: if link {
                                "link"
                            } else if files.is_empty() {
                                "typed"
                            } else {
                                "file"
                            }
                            .to_string(),
                            link: link.then(|| text.trim().to_string()),
                            files: files.clone(),
                            ..Capture::default()
                        };
                        match self.act(|core| core.capture(&capture)) {
                            Some(id) => {
                                self.folder = INBOX.to_string();
                                self.open = Some(id);
                                self.read_waiting_texts();
                                false
                            }
                            None => true,
                        }
                    }
                    Some(false) => false,
                    None => true,
                }
            }
            Dialog::EditText { id, text } => {
                let label = ui.label(self.t.note_text());
                ui.add(
                    egui::TextEdit::multiline(text)
                        .desired_rows(16)
                        .desired_width(560.0),
                )
                .labelled_by(label.id);
                match self.confirm_row(ui, self.t.save(), true) {
                    Some(true) => self.act(|core| core.set_text(id, text)).is_none(),
                    Some(false) => false,
                    None => true,
                }
            }
            Dialog::File {
                id,
                title,
                tags,
                folder,
            } => {
                if id.starts_with(&format!("{INBOX}/")) {
                    let done = self.proposing.as_ref().and_then(Job::take);
                    match done {
                        Some(Ok(proposal)) => {
                            *title = proposal.title;
                            *tags = proposal.tags.join(", ");
                            *folder = proposal.folder;
                            self.proposed_by = Some(proposal.model);
                            self.proposing = None;
                            self.error = None;
                        }
                        Some(Err(e)) => {
                            self.error = Some(error_text(&self.t, &e));
                            self.proposing = None;
                        }
                        None => {}
                    }
                    ui.horizontal(|ui| {
                        let waiting = self.proposing.is_some();
                        if ui
                            .add_enabled(!waiting, egui::Button::new(self.t.propose()))
                            .clicked()
                            && let Some(request) = self.act(|core| core.proposal_request(id))
                        {
                            self.proposed_by = None;
                            self.proposing = Some(Job::start(self.beside, move || request.send()));
                        }
                        if waiting {
                            ui.spinner();
                            ui.weak(self.t.model_working());
                            ui.ctx().request_repaint_after(Duration::from_millis(200));
                        }
                    });
                    if let Some(model) = &self.proposed_by {
                        ui.weak(self.t.proposal_by(model));
                    }
                }
                egui::Grid::new("file").num_columns(2).show(ui, |ui| {
                    let label = ui.label(self.t.title_label());
                    ui.add(egui::TextEdit::singleline(title).desired_width(360.0))
                        .labelled_by(label.id);
                    ui.end_row();
                    let label = ui.label(self.t.tags_label());
                    ui.add(
                        egui::TextEdit::singleline(tags)
                            .hint_text(self.t.tags_hint())
                            .desired_width(360.0),
                    )
                    .labelled_by(label.id);
                    ui.end_row();
                    let label = ui.label(self.t.folder_label());
                    egui::ComboBox::from_id_salt("file_folder")
                        .selected_text(self.folder_choice(folder))
                        .show_ui(ui, |ui| {
                            // A Note is filed out of the Inbox, so the
                            // Inbox is no choice.
                            let mut names = vec![String::new()];
                            names.extend(
                                self.folders()
                                    .into_iter()
                                    .map(|f| f.name)
                                    .filter(|n| n != INBOX),
                            );
                            // A Folder a Proposal suggests is a choice
                            // too, before it exists.
                            if !names.contains(folder) {
                                names.push(folder.clone());
                            }
                            for name in names {
                                let shown = self.folder_choice(&name);
                                ui.selectable_value(folder, name, shown);
                            }
                        })
                        .response
                        .labelled_by(label.id);
                    ui.end_row();
                });
                match self.confirm_row(ui, self.t.save(), !title.trim().is_empty()) {
                    Some(true) => {
                        let tags: Vec<String> = tags
                            .split(',')
                            .map(|t| t.trim().to_string())
                            .filter(|t| !t.is_empty())
                            .collect();
                        match self.act(|core| core.file(id, title, &tags, folder)) {
                            Some(new_id) => {
                                self.folder = folder.clone();
                                self.open = Some(new_id);
                                false
                            }
                            None => true,
                        }
                    }
                    Some(false) => false,
                    None => true,
                }
            }
            Dialog::ConfirmDelete { id } => {
                ui.label(self.t.delete_note_question());
                match self.confirm_row(ui, self.t.delete(), true) {
                    Some(true) => {
                        if self.act(|core| core.delete(id)).is_some() {
                            self.open = None;
                        }
                        false
                    }
                    Some(false) => false,
                    None => true,
                }
            }
            Dialog::NewFolder { name } => {
                let label = ui.label(self.t.folder_name());
                ui.add(egui::TextEdit::singleline(name).desired_width(320.0))
                    .labelled_by(label.id);
                match self.confirm_row(ui, self.t.create(), !name.trim().is_empty()) {
                    Some(true) => {
                        let wanted = name.trim().to_string();
                        match self.act(|core| core.create_folder(&wanted)) {
                            Some(()) => {
                                self.folder = wanted;
                                self.open = None;
                                false
                            }
                            None => true,
                        }
                    }
                    Some(false) => false,
                    None => true,
                }
            }
            Dialog::Models {
                name,
                address,
                key,
                model,
                local,
            } => {
                let models: Vec<Model> = self
                    .core
                    .as_ref()
                    .map(|core| core.models().to_vec())
                    .unwrap_or_default();
                let chosen = self
                    .core
                    .as_ref()
                    .and_then(|core| core.chosen_model())
                    .map(|m| m.name.clone());
                if models.is_empty() {
                    ui.set_max_width(460.0);
                    ui.label(self.t.no_models());
                }
                for known in &models {
                    ui.horizontal(|ui| {
                        let is_chosen = chosen.as_deref() == Some(known.name.as_str());
                        if ui.radio(is_chosen, known.name.as_str()).clicked() {
                            self.act(|core| core.choose_model(&known.name));
                        }
                        ui.weak(if known.local {
                            self.t.local_model()
                        } else {
                            self.t.cloud_model()
                        });
                        if ui.small_button(self.t.remove()).clicked() {
                            self.act(|core| core.remove_model(&known.name));
                        }
                    });
                }
                ui.separator();
                ui.strong(self.t.connect_model());
                egui::Grid::new("model").num_columns(2).show(ui, |ui| {
                    let label = ui.label(self.t.model_name());
                    ui.add(egui::TextEdit::singleline(name).desired_width(320.0))
                        .labelled_by(label.id);
                    ui.end_row();
                    let label = ui.label(self.t.model_address());
                    ui.add(
                        egui::TextEdit::singleline(address)
                            .hint_text(self.t.model_address_hint())
                            .desired_width(320.0),
                    )
                    .labelled_by(label.id);
                    ui.end_row();
                    let label = ui.label(self.t.model_key());
                    ui.add(
                        egui::TextEdit::singleline(key)
                            .password(true)
                            .desired_width(320.0),
                    )
                    .labelled_by(label.id);
                    ui.end_row();
                    let label = ui.label(self.t.model_id());
                    ui.add(egui::TextEdit::singleline(model).desired_width(320.0))
                        .labelled_by(label.id);
                    ui.end_row();
                });
                ui.checkbox(local, self.t.model_is_local());
                ui.weak(self.t.model_stays_here());
                ui.add_space(8.0);
                let mut keep = true;
                ui.horizontal(|ui| {
                    let filled = !name.trim().is_empty() && !address.trim().is_empty();
                    let new = Model {
                        name: name.trim().to_string(),
                        address: address.trim().to_string(),
                        key: (!key.trim().is_empty()).then(|| key.trim().to_string()),
                        model: model.trim().to_string(),
                        local: *local,
                    };
                    let test = ui
                        .add_enabled(filled, egui::Button::new(self.t.test_and_add()))
                        .clicked();
                    let untested = ui
                        .add_enabled(filled, egui::Button::new(self.t.add_untested()))
                        .clicked();
                    if test || untested {
                        let added = self.act(|core| {
                            if test {
                                new.test()?;
                            }
                            core.save_model(new.clone())
                        });
                        if added.is_some() {
                            name.clear();
                            address.clear();
                            key.clear();
                            model.clear();
                            *local = false;
                        }
                    }
                    if ui.button(self.t.close()).clicked() {
                        keep = false;
                    }
                });
                keep
            }
            Dialog::RenameFolder { from, name } => {
                let label = ui.label(self.t.folder_name());
                ui.add(egui::TextEdit::singleline(name).desired_width(320.0))
                    .labelled_by(label.id);
                match self.confirm_row(ui, self.t.rename(), !name.trim().is_empty()) {
                    Some(true) => {
                        let wanted = name.trim().to_string();
                        match self.act(|core| core.rename_folder(from, &wanted)) {
                            Some(()) => {
                                self.folder = wanted;
                                self.open = None;
                                false
                            }
                            None => true,
                        }
                    }
                    Some(false) => false,
                    None => true,
                }
            }
        }
    }

    /// The buttons at the foot of a dialog: `Some(true)` when the
    /// person confirmed, `Some(false)` when they cancelled.
    pub(crate) fn confirm_row(&self, ui: &mut Ui, confirm: &str, enabled: bool) -> Option<bool> {
        let mut answer = None;
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(enabled, egui::Button::new(confirm))
                .clicked()
            {
                answer = Some(true);
            }
            if ui.button(self.t.cancel()).clicked() {
                answer = Some(false);
            }
        });
        answer
    }
}

/// What a Note is listed as: its title, or for an Unfiled Note without
/// one the start of its text.
fn list_title(note: &Note) -> String {
    let unnamed = note.unfiled() && note.id.ends_with(&format!("/{}.md", note.title));
    if !unnamed {
        return note.title.clone();
    }
    let first = note.text.lines().find(|l| !l.trim().is_empty());
    match first {
        Some(line) => line.trim().chars().take(60).collect(),
        None => note.title.clone(),
    }
}

/// A capture time as the person reads it: the day and the minute,
/// without seconds and offset.
fn when(captured: &str) -> String {
    let short: String = captured.chars().take(16).collect();
    short.replace('T', " ")
}

/// An error of the core in the person's language.
pub(crate) fn error_text(t: &L10n, error: &Error) -> String {
    match error {
        Error::Io(e) => t.error_file(&e.to_string()),
        Error::NotFound(what) => t.error_not_found(what),
        Error::Invalid(reason) => t.error_not_possible(reason),
        Error::Model(reason) => t.error_model(reason),
        Error::NoModel => t.error_no_model().to_string(),
        Error::Private => t.error_private().to_string(),
        other => t.error_other(&other.to_string()),
    }
}
