//! The index: what the Brain's folder holds, read once and kept in a
//! database on the Device, with a full-text index over it (ADR 0001,
//! ADR 0006). It holds nothing the folder lacks and is rebuilt from the
//! folder alone.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::brain::{Brain, Note};
use crate::error::{Error, Result};

/// A Note found by a search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hit {
    pub id: String,
    pub title: String,
    pub folder: String,
    /// The text around the found words.
    pub excerpt: String,
}

pub struct Index {
    db: Connection,
}

impl Index {
    /// The index in the file `path` for the Brain in `brain`. An index
    /// made for another Brain is emptied.
    pub fn open(path: &Path, brain: &Path) -> Result<Self> {
        let db = Connection::open(path)?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS about(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS notes(
                 id TEXT PRIMARY KEY, stamp TEXT NOT NULL, note TEXT NOT NULL);
             CREATE VIRTUAL TABLE IF NOT EXISTS words USING fts5(
                 id UNINDEXED, title, tags, body, extracted,
                 tokenize = 'unicode61 remove_diacritics 2');",
        )?;
        let brain = brain.to_string_lossy().to_string();
        let known: Option<String> = db
            .query_row("SELECT value FROM about WHERE key = 'brain'", [], |r| {
                r.get(0)
            })
            .optional()?;
        if known.as_deref() != Some(brain.as_str()) {
            db.execute_batch("DELETE FROM notes; DELETE FROM words;")?;
            db.execute(
                "INSERT OR REPLACE INTO about(key, value) VALUES ('brain', ?1)",
                params![brain],
            )?;
        }
        Ok(Self { db })
    }

    /// Brings the index in step with the folder and says whether
    /// anything had changed there.
    pub fn refresh(&mut self, brain: &Brain) -> Result<bool> {
        let mut known: BTreeMap<String, String> = BTreeMap::new();
        {
            let mut rows = self.db.prepare("SELECT id, stamp FROM notes")?;
            for row in rows.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))? {
                let (id, stamp) = row?;
                known.insert(id, stamp);
            }
        }
        let mut changed = false;
        let tx = self.db.transaction()?;
        for (id, versions) in brain.note_files()? {
            // A Note another tool removes between the listing and the
            // reading is gone; the next refresh sees that.
            let Ok(mut stamp) = brain.fingerprint(&id) else {
                continue;
            };
            for version in &versions {
                stamp.push('|');
                stamp.push_str(version);
            }
            // Only a Note that changed is read; the others cost one
            // look at their files' sizes and times.
            if known.remove(&id).as_deref() == Some(stamp.as_str()) {
                continue;
            }
            let Ok(note) = brain.read_with(&id, versions) else {
                continue;
            };
            changed = true;
            let json =
                serde_json::to_string(&note).map_err(|e| Error::Unreadable(e.to_string()))?;
            tx.execute(
                "INSERT OR REPLACE INTO notes(id, stamp, note) VALUES (?1, ?2, ?3)",
                params![id, stamp, json],
            )?;
            tx.execute("DELETE FROM words WHERE id = ?1", params![id])?;
            let mut extracted: Vec<&str> = note
                .attachments
                .iter()
                .filter_map(|a| a.extracted.as_deref())
                .collect();
            extracted.extend(note.page_text.as_deref());
            let names: Vec<&str> = note.attachments.iter().map(|a| a.name.as_str()).collect();
            tx.execute(
                "INSERT INTO words(id, title, tags, body, extracted) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    id,
                    note.title,
                    note.tags.join(" "),
                    note.text,
                    format!("{}\n{}", extracted.join("\n"), names.join("\n")),
                ],
            )?;
        }
        for id in known.keys() {
            changed = true;
            tx.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
            tx.execute("DELETE FROM words WHERE id = ?1", params![id])?;
        }
        tx.commit()?;
        Ok(changed)
    }

    /// The Note `id` as the index last read it.
    pub fn note(&self, id: &str) -> Result<Note> {
        let json: Option<String> = self
            .db
            .query_row("SELECT note FROM notes WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .optional()?;
        let json = json.ok_or_else(|| Error::NotFound(id.to_string()))?;
        serde_json::from_str(&json).map_err(|e| Error::Unreadable(e.to_string()))
    }

    /// Every Note, newest Capture first; Notes without a capture time
    /// come last, by title.
    pub fn notes(&self) -> Result<Vec<Note>> {
        self.notes_where("?1 IS NOT NULL", "")
    }

    /// The Notes lying directly in `folder`, in the same order.
    pub fn notes_in(&self, folder: &str) -> Result<Vec<Note>> {
        self.notes_where("json_extract(note, '$.folder') = ?1", folder)
    }

    fn notes_where(&self, condition: &str, value: &str) -> Result<Vec<Note>> {
        let mut rows = self
            .db
            .prepare(&format!("SELECT note FROM notes WHERE {condition}"))?;
        let mut notes = Vec::new();
        for row in rows.query_map(params![value], |r| r.get::<_, String>(0))? {
            let note: Note =
                serde_json::from_str(&row?).map_err(|e| Error::Unreadable(e.to_string()))?;
            notes.push(note);
        }
        notes.sort_by(|a, b| {
            b.captured
                .cmp(&a.captured)
                .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
        Ok(notes)
    }

    /// The Notes holding the `words`: all of them, or with `any` at
    /// least one, best match first. A word matches as the start of a
    /// word too, so "rece" finds "receipt".
    pub fn search(&self, words: &[String], any: bool, limit: usize) -> Result<Vec<Hit>> {
        let terms: Vec<String> = words
            .iter()
            .flat_map(|w| w.split(|c: char| !c.is_alphanumeric()))
            .filter(|w| !w.is_empty())
            .map(|w| format!("\"{w}\"*"))
            .collect();
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let query = terms.join(if any { " OR " } else { " AND " });
        let mut rows = self.db.prepare(
            "SELECT words.id, snippet(words, -1, '', '', ' … ', 24), notes.note
             FROM words JOIN notes ON notes.id = words.id
             WHERE words MATCH ?1 ORDER BY bm25(words, 0, 8, 4, 1, 1) LIMIT ?2",
        )?;
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut hits = Vec::new();
        for row in rows.query_map(params![query, limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })? {
            let (id, excerpt, json) = row?;
            let note: Note =
                serde_json::from_str(&json).map_err(|e| Error::Unreadable(e.to_string()))?;
            hits.push(Hit {
                id,
                title: note.title,
                folder: note.folder,
                excerpt: excerpt.split_whitespace().collect::<Vec<_>>().join(" "),
            });
        }
        Ok(hits)
    }
}
