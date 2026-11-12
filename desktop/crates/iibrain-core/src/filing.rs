//! Proposals: a Model's suggestion of title, tags and Folder for an
//! Unfiled Note (ADR 0005). A Proposal changes nothing; the person
//! files the Note, as proposed or corrected.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::brain::{INBOX, Note};
use crate::core::Core;
use crate::error::{Error, Result};
use crate::model::{Message, Model};

/// How much of a Note's text and Extracted Text a Model gets to make a
/// Proposal from.
const MAX_CHARS: usize = 6000;

/// What a Model suggests for an Unfiled Note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub title: String,
    pub tags: Vec<String>,
    /// One of the person's Folders, or a new one; empty for the
    /// Brain's root.
    pub folder: String,
    /// Whether `folder` does not exist yet.
    pub new_folder: bool,
    /// The name of the Model that made it.
    pub model: String,
}

/// A Proposal about to be asked for: everything the asking needs, so
/// that it can wait for the Model beside the app.
#[derive(Debug, Clone)]
pub struct ProposalRequest {
    model: Model,
    messages: Vec<Message>,
    folders: Vec<String>,
}

impl ProposalRequest {
    /// Asks the Model and reads its Proposal.
    pub fn send(self) -> Result<Proposal> {
        let answer = self.model.ask(&self.messages)?;
        read_proposal(&answer, &self.folders, &self.model.name)
    }
}

impl Core {
    /// Prepares the Proposal for the Note `id`. The chosen Model gets
    /// the Note only when the gate allows it: a Private Note goes to a
    /// Local Model alone.
    pub fn proposal_request(&self, id: &str) -> Result<ProposalRequest> {
        let note = self.note(id)?;
        let model = self.chosen_model().ok_or(Error::NoModel)?.clone();
        if !self.may_send(&note, &model) {
            return Err(Error::Private);
        }
        // The names of Private Folders stay away from a Cloud Model too.
        let folders: Vec<String> = self
            .folders()?
            .into_iter()
            .filter(|f| f.name != INBOX && (model.local || !f.private))
            .map(|f| f.name)
            .collect();
        let system = format!(
            "You file notes for a person's private knowledge base. Read the note and answer \
             with one JSON object and nothing else: {{\"title\": \"...\", \"tags\": [\"...\"], \
             \"folder\": \"...\"}}. The title is short and says what the note is about. Give at \
             most five tags, single lower-case words. For the folder choose the best fitting of \
             the person's folders; only when none fits, suggest a short name for a new one. \
             Write title and tags in the language of the note.\n\nThe person's folders:\n{}",
            if folders.is_empty() {
                "(none yet)".to_string()
            } else {
                folders
                    .iter()
                    .map(|f| format!("- {f}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        );
        Ok(ProposalRequest {
            messages: vec![Message::system(&system), Message::user(&whole_text(&note))],
            model,
            folders,
        })
    }

    /// Asks the chosen Model for a Proposal for the Note `id` and waits
    /// for it.
    pub fn propose(&self, id: &str) -> Result<Proposal> {
        self.proposal_request(id)?.send()
    }
}

/// Everything a Note says, for a Model to read: its text, its link, the
/// names of its Attachments and its Extracted Text, cut to a length.
pub(crate) fn whole_text(note: &Note) -> String {
    let mut parts = vec![note.text.trim().to_string()];
    if let Some(link) = &note.link {
        parts.push(format!("Link: {link}"));
    }
    if let Some(page) = note.page_text.as_deref().filter(|p| !p.is_empty()) {
        parts.push(format!("Text of the linked page:\n{page}"));
    }
    for attachment in &note.attachments {
        match attachment.extracted.as_deref().filter(|t| !t.is_empty()) {
            Some(text) => parts.push(format!("Attachment {}:\n{text}", attachment.name)),
            None => parts.push(format!("Attachment {}", attachment.name)),
        }
    }
    let text = parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    text.chars().take(MAX_CHARS).collect()
}

/// Reads a Proposal out of what a Model wrote: the JSON object in it,
/// whatever stands around it.
fn read_proposal(answer: &str, folders: &[String], model: &str) -> Result<Proposal> {
    let unreadable = || Error::Model("the Proposal was not in the expected form".to_string());
    let from = answer.find('{').ok_or_else(unreadable)?;
    let to = answer.rfind('}').ok_or_else(unreadable)?;
    let json = answer.get(from..=to).ok_or_else(unreadable)?;
    let value: Value = serde_json::from_str(json).map_err(|_| unreadable())?;
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or_else(unreadable)?
        .to_string();
    let tags: Vec<String> = match value.get("tags") {
        Some(Value::Array(tags)) => tags
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect(),
        Some(Value::String(tags)) => tags.split(',').map(String::from).collect(),
        _ => Vec::new(),
    };
    let tags = tags
        .iter()
        .map(|t| t.trim().trim_start_matches('#').to_string())
        .filter(|t| !t.is_empty())
        .collect();
    // A Folder is a path inside the Brain: nothing hidden, nothing that
    // leaves it, and never the Inbox, which a Note is filed out of.
    let folder = value
        .get("folder")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty() && !part.starts_with('.') && !part.contains('\\'))
        .collect::<Vec<_>>()
        .join("/");
    let folder = if folder == INBOX {
        String::new()
    } else {
        folder
    };
    // The person's own spelling of a Folder the Model named in another.
    let known = folders
        .iter()
        .find(|f| f.to_lowercase() == folder.to_lowercase());
    Ok(Proposal {
        title,
        tags,
        new_folder: known.is_none() && !folder.is_empty(),
        folder: known.cloned().unwrap_or(folder),
        model: model.to_string(),
    })
}
