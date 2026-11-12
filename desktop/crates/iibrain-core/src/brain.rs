//! The Brain store: Notes as Markdown files with YAML frontmatter, their
//! Attachments beside them, in the Folders of the Brain's folder
//! (ADR 0001). The folder is the truth and is taken as it is found;
//! nothing about a Note is kept anywhere else.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::{Mapping, Value};

use crate::error::{Error, Result};

/// The one fixed Folder, where Unfiled Notes wait.
pub const INBOX: &str = "Inbox";
/// What the directory of a Note's Attachments ends with, after the
/// Note's file name without `.md`.
const FILES: &str = ".files";
/// What the file of an Attachment's Extracted Text ends with, after the
/// Attachment's name.
const EXTRACTED: &str = ".extracted.txt";
/// The Extracted Text of the page behind a Note's link.
const PAGE: &str = "page";
/// The marker file of a Private Folder (ADR 0004).
const PRIVATE_MARK: &str = ".iibrain-private";
/// The Brain's own settings, at its root.
const SETTINGS_DIR: &str = ".iibrain";
const SETTINGS_FILE: &str = "brain.yaml";

/// A file kept with a Note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// Its file name.
    pub name: String,
    /// Its Extracted Text: `None` while nothing was read, empty when
    /// the app read it and found no text.
    pub extracted: Option<String>,
}

/// One kept thing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Its file, relative to the Brain, with `/` between the parts.
    pub id: String,
    /// The Folder it lies in, empty for the Brain's root.
    pub folder: String,
    pub title: String,
    pub tags: Vec<String>,
    /// When it was captured, as RFC 3339.
    pub captured: Option<String>,
    /// How it was captured: `typed`, `file`, `photo`, `link`, `voice`,
    /// `print`, `share` or `answer`.
    pub via: Option<String>,
    /// The link of a link Capture.
    pub link: Option<String>,
    /// Its own Private mark; its Folder may make it Private too
    /// ([`Brain::is_private`]).
    pub private: bool,
    /// The Model that wrote it, for a Saved Answer.
    pub model: Option<String>,
    /// The Sources of a Saved Answer, as ids.
    pub sources: Vec<String>,
    /// Its text, without the frontmatter.
    pub text: String,
    pub attachments: Vec<Attachment>,
    /// The Extracted Text of the page behind its link: `None` while it
    /// was not fetched, empty when it could not be.
    pub page_text: Option<String>,
    /// The other versions a Sync Tool left of it, as ids. Not empty
    /// means the Note is in Conflict.
    pub conflicts: Vec<String>,
}

impl Note {
    /// Whether it waits in the Inbox.
    pub fn unfiled(&self) -> bool {
        self.folder == INBOX
    }
}

/// What a Capture brings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capture {
    pub text: String,
    /// How it is captured, see [`Note::via`].
    pub via: String,
    pub link: Option<String>,
    /// Whether the person marked it Private while capturing.
    pub private: bool,
    /// Files to keep as its Attachments; they are copied.
    pub files: Vec<PathBuf>,
}

/// A Folder of the Brain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    /// Its path relative to the Brain, with `/` between the parts.
    pub name: String,
    /// Whether it carries the Private mark itself or lies in a Folder
    /// that does.
    pub private: bool,
    /// How many Notes lie directly in it.
    pub notes: usize,
}

/// The Brain's folder.
#[derive(Debug, Clone)]
pub struct Brain {
    root: PathBuf,
}

/// A Note's file, read.
struct Parsed {
    front: Mapping,
    body: String,
}

impl Brain {
    /// The Brain in `root`, which must be a directory.
    pub fn open(root: &Path) -> Result<Self> {
        if !root.is_dir() {
            return Err(Error::NotFound(root.display().to_string()));
        }
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    // ------------------------------------------------------- reading

    /// Every Folder, the Inbox first, the others by name. The Inbox is
    /// listed whether its directory exists or not.
    pub fn folders(&self) -> Result<Vec<Folder>> {
        let mut dirs = Vec::new();
        self.walk_dirs(&self.root, "", &mut dirs)?;
        if !dirs.iter().any(|d| d == INBOX) {
            dirs.push(INBOX.to_string());
        }
        dirs.sort_by_key(|a| (a != INBOX, a.to_lowercase()));
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for id in self.note_ids()? {
            *counts.entry(folder_of(&id)).or_default() += 1;
        }
        Ok(dirs
            .into_iter()
            .map(|name| Folder {
                private: self.folder_private(&name),
                notes: counts.get(&name).copied().unwrap_or(0),
                name,
            })
            .collect())
    }

    /// The ids of every Note, a Conflict's other versions left out.
    pub fn note_ids(&self) -> Result<Vec<String>> {
        Ok(self.note_files()?.into_keys().collect())
    }

    /// Every Note's id with the ids of its other versions.
    fn note_files(&self) -> Result<BTreeMap<String, Vec<String>>> {
        let mut files = Vec::new();
        self.walk_notes(&self.root, "", &mut files)?;
        let all: std::collections::BTreeSet<String> = files.iter().cloned().collect();
        let mut notes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut versions = Vec::new();
        for id in files {
            match conflict_base(&id).filter(|base| all.contains(base)) {
                Some(base) => versions.push((base, id)),
                None => {
                    notes.entry(id).or_default();
                }
            }
        }
        for (base, id) in versions {
            notes.entry(base).or_default().push(id);
        }
        Ok(notes)
    }

    /// The Note `id`.
    pub fn read(&self, id: &str) -> Result<Note> {
        let conflicts = self
            .note_files()?
            .remove(id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;
        self.read_with(id, conflicts)
    }

    /// One version of a Note in Conflict, or any Note's file, read as
    /// it is.
    pub fn read_version(&self, id: &str) -> Result<Note> {
        self.read_with(id, Vec::new())
    }

    fn read_with(&self, id: &str, conflicts: Vec<String>) -> Result<Note> {
        let parsed = self.parse(id)?;
        let front = &parsed.front;
        let (attachments, page_text) = self.read_files(id)?;
        let title = text_of(front, "title")
            .or_else(|| heading(&parsed.body))
            .unwrap_or_else(|| stem_of(id));
        Ok(Note {
            id: id.to_string(),
            folder: folder_of(id),
            title,
            tags: tags_of(front),
            captured: text_of(front, "captured"),
            via: text_of(front, "via"),
            link: text_of(front, "link"),
            private: front.get("private").and_then(Value::as_bool) == Some(true),
            model: text_of(front, "model"),
            sources: list_of(front, "sources"),
            text: parsed.body,
            attachments,
            page_text,
            conflicts,
        })
    }

    /// A Note's Attachments and the Extracted Text of its page.
    fn read_files(&self, id: &str) -> Result<(Vec<Attachment>, Option<String>)> {
        let dir = self.files_dir(id)?;
        let mut names = Vec::new();
        if dir.is_dir() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    names.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        names.sort();
        let extracted =
            |name: &str| fs::read_to_string(dir.join(format!("{name}{EXTRACTED}"))).ok();
        let attachments = names
            .iter()
            .filter(|n| !n.ends_with(EXTRACTED) && !n.starts_with('.'))
            .map(|n| Attachment {
                name: n.clone(),
                extracted: extracted(n),
            })
            .collect();
        Ok((attachments, extracted(PAGE)))
    }

    /// What tells whether a Note changed since it was last read: its
    /// file and everything in the directory of its Attachments, each
    /// with its size and the time it was last written.
    pub fn fingerprint(&self, id: &str) -> Result<String> {
        let mut parts = vec![stamp(&self.path_of(id)?)?];
        let dir = self.files_dir(id)?;
        if dir.is_dir() {
            let mut files = Vec::new();
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                files.push(format!(
                    "{}:{}",
                    entry.file_name().to_string_lossy(),
                    stamp(&entry.path())?
                ));
            }
            files.sort();
            parts.extend(files);
        }
        Ok(parts.join("|"))
    }

    /// Where an Attachment lies.
    pub fn attachment_path(&self, id: &str, name: &str) -> Result<PathBuf> {
        if name.contains(['/', '\\']) || name.starts_with('.') {
            return Err(Error::Invalid(name.to_string()));
        }
        let path = self.files_dir(id)?.join(name);
        if path.is_file() {
            Ok(path)
        } else {
            Err(Error::NotFound(name.to_string()))
        }
    }

    // ------------------------------------------------------- privacy

    /// Whether `note` is Private: by its own mark, by its Folder's, or
    /// as an Unfiled Note while those count as Private (ADR 0004).
    pub fn is_private(&self, note: &Note) -> bool {
        note.private
            || self.folder_private(&note.folder)
            || (note.unfiled() && self.unfiled_private())
    }

    /// Whether `folder` or a Folder it lies in carries the Private mark.
    pub fn folder_private(&self, folder: &str) -> bool {
        let mut dir = self.root.clone();
        for part in folder.split('/').filter(|p| !p.is_empty()) {
            dir.push(part);
            if dir.join(PRIVATE_MARK).is_file() {
                return true;
            }
        }
        false
    }

    /// Sets or lifts the Private mark of `folder` itself.
    pub fn set_folder_private(&self, folder: &str, private: bool) -> Result<()> {
        let mark = self.dir_of(folder)?.join(PRIVATE_MARK);
        if private {
            fs::create_dir_all(self.dir_of(folder)?)?;
            fs::write(mark, "")?;
        } else if mark.is_file() {
            fs::remove_file(mark)?;
        }
        Ok(())
    }

    /// Sets or lifts a Note's own Private mark.
    pub fn set_private(&self, id: &str, private: bool) -> Result<()> {
        let mut parsed = self.parse(id)?;
        if private {
            parsed.front.insert("private".into(), Value::Bool(true));
        } else {
            parsed.front.remove("private");
        }
        self.write(id, &parsed)
    }

    /// Whether Unfiled Notes count as Private; they do unless the
    /// Brain's settings say otherwise.
    pub fn unfiled_private(&self) -> bool {
        self.settings()
            .get("unfiled_private")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }

    pub fn set_unfiled_private(&self, private: bool) -> Result<()> {
        let mut settings = self.settings();
        settings.insert("unfiled_private".into(), Value::Bool(private));
        let dir = self.root.join(SETTINGS_DIR);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(SETTINGS_FILE), to_yaml(&settings)?)?;
        Ok(())
    }

    fn settings(&self) -> Mapping {
        fs::read_to_string(self.root.join(SETTINGS_DIR).join(SETTINGS_FILE))
            .ok()
            .and_then(|text| serde_yaml_ng::from_str::<Mapping>(&text).ok())
            .unwrap_or_default()
    }

    // ------------------------------------------------------- writing

    /// Keeps a Capture as an Unfiled Note in the Inbox and says its id.
    pub fn capture(&self, capture: &Capture, now: DateTime<Local>) -> Result<String> {
        let inbox = self.root.join(INBOX);
        fs::create_dir_all(&inbox)?;
        let name = free_name(&inbox, &now.format("%Y-%m-%d %H.%M.%S").to_string(), None);
        let id = format!("{INBOX}/{name}");
        let mut front = Mapping::new();
        front.insert("captured".into(), now.to_rfc3339().into());
        front.insert("via".into(), capture.via.clone().into());
        if let Some(link) = &capture.link {
            front.insert("link".into(), link.clone().into());
        }
        if capture.private {
            front.insert("private".into(), Value::Bool(true));
        }
        self.write(
            &id,
            &Parsed {
                front,
                body: capture.text.clone(),
            },
        )?;
        for file in &capture.files {
            self.attach(&id, file)?;
        }
        Ok(id)
    }

    /// Copies `file` to the Attachments of `id` and says the name it
    /// got there.
    pub fn attach(&self, id: &str, file: &Path) -> Result<String> {
        let dir = self.files_dir(id)?;
        fs::create_dir_all(&dir)?;
        let wanted = file
            .file_name()
            .map(|n| clean_name(&n.to_string_lossy()))
            .unwrap_or_else(|| "file".to_string());
        let (stem, ext) = match wanted.rsplit_once('.') {
            Some((stem, ext)) if !stem.is_empty() => (stem.to_string(), format!(".{ext}")),
            _ => (wanted.clone(), String::new()),
        };
        let mut name = wanted;
        let mut n = 2;
        while dir.join(&name).exists() {
            name = format!("{stem} {n}{ext}");
            n += 1;
        }
        fs::copy(file, dir.join(&name))?;
        Ok(name)
    }

    /// Keeps the Extracted Text of an Attachment; empty says that the
    /// app read it and found none.
    pub fn set_extracted(&self, id: &str, attachment: &str, text: &str) -> Result<()> {
        self.attachment_path(id, attachment)?;
        fs::write(
            self.files_dir(id)?.join(format!("{attachment}{EXTRACTED}")),
            text,
        )?;
        Ok(())
    }

    /// Keeps the Extracted Text of the page behind the Note's link;
    /// empty says that the page could not be fetched.
    pub fn set_page_text(&self, id: &str, text: &str) -> Result<()> {
        let dir = self.files_dir(id)?;
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(format!("{PAGE}{EXTRACTED}")), text)?;
        Ok(())
    }

    /// Replaces a Note's text.
    pub fn set_text(&self, id: &str, text: &str) -> Result<()> {
        let mut parsed = self.parse(id)?;
        parsed.body = text.to_string();
        self.write(id, &parsed)
    }

    /// Marks a Note as a Saved Answer: written by `model`, from
    /// `sources`.
    pub fn set_answer(&self, id: &str, model: &str, sources: &[String]) -> Result<()> {
        let mut parsed = self.parse(id)?;
        parsed.front.insert("model".into(), model.into());
        parsed.front.insert(
            "sources".into(),
            Value::Sequence(sources.iter().map(|s| s.clone().into()).collect()),
        );
        self.write(id, &parsed)
    }

    /// Gives a Note its title, tags and Folder, and says its id from
    /// then on. This is Filing for an Unfiled Note, and moving,
    /// retitling and retagging for a filed one. The file is named after
    /// the title and moves with its Attachments.
    pub fn file(&self, id: &str, title: &str, tags: &[String], folder: &str) -> Result<String> {
        let title = title.trim();
        if title.is_empty() {
            return Err(Error::Invalid("a Note needs a title".to_string()));
        }
        let mut parsed = self.parse(id)?;
        parsed.front.insert("title".into(), title.into());
        let tags: Vec<Value> = tags
            .iter()
            .map(|t| t.trim().trim_start_matches('#'))
            .filter(|t| !t.is_empty())
            .map(Value::from)
            .collect();
        if tags.is_empty() {
            parsed.front.remove("tags");
        } else {
            parsed.front.insert("tags".into(), Value::Sequence(tags));
        }
        let dir = self.dir_of(folder)?;
        fs::create_dir_all(&dir)?;
        let from = self.path_of(id)?;
        let name = free_name(&dir, &clean_name(title), Some(&from));
        let new_id = join_id(folder, &name);
        let to = self.path_of(&new_id)?;
        let old_files = self.files_dir(id)?;
        self.write(id, &parsed)?;
        if from != to {
            fs::rename(&from, &to)?;
            if old_files.is_dir() {
                fs::rename(&old_files, self.files_dir(&new_id)?)?;
            }
        }
        Ok(new_id)
    }

    /// Removes a Note with its Attachments and its other versions.
    pub fn delete(&self, id: &str) -> Result<()> {
        let note = self.read(id)?;
        for version in note.conflicts.iter().chain(std::iter::once(&note.id)) {
            fs::remove_file(self.path_of(version)?)?;
            let files = self.files_dir(version)?;
            if files.is_dir() {
                fs::remove_dir_all(files)?;
            }
        }
        Ok(())
    }

    /// Ends a Conflict: the Note keeps `text`, and the files of its
    /// other versions are removed.
    pub fn resolve(&self, id: &str, text: &str) -> Result<()> {
        let note = self.read(id)?;
        self.set_text(id, text)?;
        for version in &note.conflicts {
            fs::remove_file(self.path_of(version)?)?;
            let files = self.files_dir(version)?;
            if files.is_dir() {
                fs::remove_dir_all(files)?;
            }
        }
        Ok(())
    }

    // ------------------------------------------------------- folders

    pub fn create_folder(&self, name: &str) -> Result<()> {
        let dir = self.dir_of(name)?;
        if name.trim().is_empty() || dir.exists() {
            return Err(Error::Invalid(name.to_string()));
        }
        fs::create_dir_all(dir)?;
        Ok(())
    }

    /// Renames a Folder; the Inbox keeps its name.
    pub fn rename_folder(&self, name: &str, new_name: &str) -> Result<()> {
        let from = self.dir_of(name)?;
        let to = self.dir_of(new_name)?;
        if name == INBOX || new_name.trim().is_empty() || !from.is_dir() || to.exists() {
            return Err(Error::Invalid(new_name.to_string()));
        }
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(from, to)?;
        Ok(())
    }

    /// Removes a Folder that holds no Notes and no Folders; the Inbox
    /// stays.
    pub fn remove_folder(&self, name: &str) -> Result<()> {
        let dir = self.dir_of(name)?;
        if name == INBOX || name.is_empty() || !dir.is_dir() {
            return Err(Error::Invalid(name.to_string()));
        }
        let inside = format!("{name}/");
        let holds_notes = self.note_ids()?.iter().any(|id| id.starts_with(&inside));
        let holds_folders = self.folders()?.iter().any(|f| f.name.starts_with(&inside));
        if holds_notes || holds_folders {
            return Err(Error::Invalid(format!("{name} is not empty")));
        }
        fs::remove_dir_all(dir)?;
        Ok(())
    }

    // ------------------------------------------------------- inside

    fn parse(&self, id: &str) -> Result<Parsed> {
        let path = self.path_of(id)?;
        if !path.is_file() {
            return Err(Error::NotFound(id.to_string()));
        }
        let content = String::from_utf8_lossy(&fs::read(path)?).to_string();
        Ok(match split_front(&content) {
            Some((yaml, body)) => Parsed {
                // Frontmatter another tool wrote that is no mapping is
                // kept as text of the Note rather than lost.
                front: match serde_yaml_ng::from_str::<Mapping>(yaml) {
                    Ok(front) => front,
                    Err(_) => {
                        return Ok(Parsed {
                            front: Mapping::new(),
                            body: content,
                        });
                    }
                },
                body: body.to_string(),
            },
            None => Parsed {
                front: Mapping::new(),
                body: content,
            },
        })
    }

    fn write(&self, id: &str, parsed: &Parsed) -> Result<()> {
        let path = self.path_of(id)?;
        let content = if parsed.front.is_empty() {
            parsed.body.clone()
        } else {
            format!("---\n{}---\n\n{}", to_yaml(&parsed.front)?, parsed.body)
        };
        // Written beside the file and moved over it, so that a Sync
        // Tool never carries half a Note.
        let part = path.with_extension("md.part");
        fs::write(&part, content)?;
        fs::rename(part, path)?;
        Ok(())
    }

    /// The file of the Note `id`, which must lie inside the Brain.
    fn path_of(&self, id: &str) -> Result<PathBuf> {
        if !id.ends_with(".md") {
            return Err(Error::Invalid(id.to_string()));
        }
        self.inside(id)
    }

    fn dir_of(&self, folder: &str) -> Result<PathBuf> {
        if folder.split('/').any(|p| p.ends_with(FILES)) {
            return Err(Error::Invalid(folder.to_string()));
        }
        self.inside(folder)
    }

    fn inside(&self, relative: &str) -> Result<PathBuf> {
        let mut path = self.root.clone();
        for part in relative.split('/').filter(|p| !p.is_empty()) {
            if part == ".." || part == "." || part.starts_with('.') || part.contains('\\') {
                return Err(Error::Invalid(relative.to_string()));
            }
            path.push(part);
        }
        Ok(path)
    }

    /// The directory of the Attachments of `id`.
    fn files_dir(&self, id: &str) -> Result<PathBuf> {
        let path = self.path_of(id)?;
        let stem = id
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".md"))
            .unwrap_or_default();
        Ok(path.with_file_name(format!("{stem}{FILES}")))
    }

    fn walk_dirs(&self, dir: &Path, relative: &str, out: &mut Vec<String>) -> Result<()> {
        for (name, path) in sub_dirs(dir)? {
            let id = join_id(relative, &name);
            self.walk_dirs(&path, &id, out)?;
            out.push(id);
        }
        Ok(())
    }

    fn walk_notes(&self, dir: &Path, relative: &str, out: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type()?.is_file() && name.ends_with(".md") && !name.starts_with('.') {
                out.push(join_id(relative, &name));
            }
        }
        for (name, path) in sub_dirs(dir)? {
            self.walk_notes(&path, &join_id(relative, &name), out)?;
        }
        Ok(())
    }
}

/// The directories in `dir` that are Folders: not hidden, and not the
/// Attachments of a Note.
fn sub_dirs(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.file_type()?.is_dir() && !name.starts_with('.') && !name.ends_with(FILES) {
            dirs.push((name, entry.path()));
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn join_id(folder: &str, name: &str) -> String {
    if folder.is_empty() {
        name.to_string()
    } else {
        format!("{folder}/{name}")
    }
}

/// The Folder of the Note `id`.
pub fn folder_of(id: &str) -> String {
    id.rsplit_once('/')
        .map(|(folder, _)| folder.to_string())
        .unwrap_or_default()
}

/// The file name of `id` without `.md`.
fn stem_of(id: &str) -> String {
    let name = id.rsplit('/').next().unwrap_or(id);
    name.strip_suffix(".md").unwrap_or(name).to_string()
}

/// The id of the Note that `id` is another version of, when its name is
/// one a Sync Tool gives the loser of a Conflict: Syncthing's
/// `name.sync-conflict-20261112-153000-ABCDEFG.md`, and the
/// `name (conflicted copy 2026-11-12 153000).md` or
/// `name (Ann's conflicted copy 2026-11-12).md` of Nextcloud, ownCloud
/// and Dropbox.
pub fn conflict_base(id: &str) -> Option<String> {
    let stem = id.strip_suffix(".md")?;
    let (folder, name) = match stem.rsplit_once('/') {
        Some((folder, name)) => (folder, name),
        None => ("", stem),
    };
    let base = if let Some((base, _)) = name.split_once(".sync-conflict-") {
        base
    } else {
        let (base, bracket) = name.strip_suffix(')')?.rsplit_once(" (")?;
        if !bracket.to_lowercase().contains("conflicted copy") {
            return None;
        }
        base
    };
    if base.is_empty() {
        return None;
    }
    Some(join_id(folder, &format!("{base}.md")))
}

/// The frontmatter and the text after it, when `content` starts with a
/// frontmatter block.
fn split_front(content: &str) -> Option<(&str, &str)> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']) == "---" {
            let yaml = rest.get(..offset)?;
            let body = rest.get(offset + line.len()..)?;
            let body = body
                .strip_prefix("\r\n")
                .or_else(|| body.strip_prefix('\n'))
                .unwrap_or(body);
            return Some((yaml, body));
        }
        offset += line.len();
    }
    None
}

fn to_yaml(front: &Mapping) -> Result<String> {
    serde_yaml_ng::to_string(front).map_err(|e| Error::Unreadable(e.to_string()))
}

fn text_of(front: &Mapping, key: &str) -> Option<String> {
    match front.get(key)? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn list_of(front: &Mapping, key: &str) -> Vec<String> {
    match front.get(key) {
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.trim().to_string()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .filter(|s| !s.is_empty())
            .collect(),
        // Other tools also write `tags: a, b`.
        Some(Value::String(s)) => s
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn tags_of(front: &Mapping) -> Vec<String> {
    list_of(front, "tags")
        .into_iter()
        .map(|t| t.trim_start_matches('#').to_string())
        .collect()
}

/// The first Markdown heading of `body`, as a title for a Note whose
/// frontmatter names none.
fn heading(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.strip_prefix("# "))
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
}

/// `name` as a file name every system takes.
fn clean_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let cleaned: String = cleaned.trim_matches('.').chars().take(80).collect();
    let cleaned = cleaned.trim().to_string();
    if cleaned.is_empty() {
        "Note".to_string()
    } else {
        cleaned
    }
}

/// A file name `<stem>.md` free in `dir`, counting up from `<stem> 2`;
/// the file `own` counts as free, so a Note can keep its name.
fn free_name(dir: &Path, stem: &str, own: Option<&Path>) -> String {
    let mut name = format!("{stem}.md");
    let mut n = 2;
    while dir.join(&name).exists() && own != Some(dir.join(&name).as_path()) {
        name = format!("{stem} {n}.md");
        n += 1;
    }
    name
}

/// A file's size and the time it was last written.
fn stamp(path: &Path) -> Result<String> {
    let meta = fs::metadata(path)?;
    let written = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    Ok(format!("{}@{written}", meta.len()))
}
