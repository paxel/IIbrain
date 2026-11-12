//! The core as the phone calls it: one request as JSON text in, one
//! answer as JSON text out (ADR 0011). The requests are the calls the
//! desktop makes directly; this is their one list.
//!
//! A request is `{"call": "<name>", ...arguments}`. The answer is
//! `{"ok": <value>}` or `{"error": {"kind": "<kind>", "message": "..."}}`.
//! Files are never passed through: a Capture names the paths of its
//! files.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::asking::{Answer, Asking, Found, Lookup};
use crate::brain::{Capture, Note};
use crate::core::Core;
use crate::device::Device;
use crate::error::{Error, Result};
use crate::filing::ProposalRequest;
use crate::model::Model;

/// The core of one app: closed until a Device is opened, and then with
/// or without a Brain.
#[derive(Default)]
pub struct Session {
    device_dir: Option<PathBuf>,
    core: Option<Core>,
}

/// A request taken as far as it goes without waiting for a Model.
pub enum Step {
    /// The answer.
    Done(String),
    /// A Proposal to ask a Model for.
    Propose(ProposalRequest),
    /// A Question to ask a Model.
    Ask(Box<Asking>),
}

impl Step {
    /// Waits for the Model where one is asked, and gives the answer.
    /// It needs no [`Session`], so the app's other calls go on
    /// meanwhile.
    pub fn finish(self) -> String {
        match self {
            Step::Done(answer) => answer,
            Step::Propose(request) => answer_of(request.send().map(|p| json!(p))),
            Step::Ask(asking) => answer_of(asking.send().map(|a| json!({ "answer": a }))),
        }
    }
}

#[derive(Deserialize)]
struct Filing {
    id: String,
    title: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    folder: String,
}

impl Session {
    /// Follows one request to its answer, waiting for a Model where one
    /// is asked.
    pub fn call(&mut self, request: &str) -> String {
        self.begin(request).finish()
    }

    /// Follows one request as far as it goes without a Model.
    pub fn begin(&mut self, request: &str) -> Step {
        let request: Value = match serde_json::from_str(request) {
            Ok(request) => request,
            Err(e) => return Step::Done(answer_of(Err(Error::Invalid(e.to_string())))),
        };
        match self.step(&request) {
            Ok(step) => step,
            Err(e) => Step::Done(answer_of(Err(e))),
        }
    }

    fn core(&mut self) -> Result<&mut Core> {
        self.core
            .as_mut()
            .ok_or_else(|| Error::NotFound("no Brain chosen".to_string()))
    }

    fn step(&mut self, request: &Value) -> Result<Step> {
        let call = text(request, "call")?;
        let done = |value: Value| Ok(Step::Done(answer_of(Ok(value))));
        match call.as_str() {
            // ------------------------------------------- the Device
            "open" => {
                let dir = PathBuf::from(text(request, "device_dir")?);
                let device = Device::load(&dir)?;
                self.device_dir = Some(dir);
                self.core = match device.settings.brain {
                    // A Brain whose folder is gone, or out of reach
                    // until the person grants access, is not an error
                    // of opening the Device.
                    Some(_) => Core::open(device).ok(),
                    None => None,
                };
                done(self.state())
            }
            "state" => done(self.state()),
            "choose_brain" => {
                let dir = self
                    .device_dir
                    .clone()
                    .ok_or_else(|| Error::Invalid("the Device is not open".to_string()))?;
                self.core = Some(Core::open_folder(
                    &dir,
                    Path::new(&text(request, "folder")?),
                )?);
                done(self.state())
            }
            "refresh" => {
                let changed = self.core()?.refresh()?;
                done(json!(changed))
            }
            // ------------------------------------------- reading
            "folders" => {
                let folders = self.core()?.folders()?;
                done(json!(folders))
            }
            "notes" => {
                let core = self.core()?;
                let notes = core.notes(&text_or(request, "folder"))?;
                done(Value::Array(notes.iter().map(|n| shown(core, n)).collect()))
            }
            "note" => {
                let core = self.core()?;
                let note = core.note(&text(request, "id")?)?;
                done(shown(core, &note))
            }
            "attachment_path" => {
                let core = self.core()?;
                let path = core
                    .brain()
                    .attachment_path(&text(request, "id")?, &text(request, "name")?)?;
                done(json!(path))
            }
            "search" => {
                let hits = self.core()?.search(&text_or(request, "query"))?;
                done(json!(hits))
            }
            // ------------------------------------------- writing
            "capture" => {
                let capture: Capture = parse(request.get("capture"))?;
                let id = self.core()?.capture(&capture)?;
                done(json!(id))
            }
            "file" => {
                let filing: Filing = parse(Some(request))?;
                let id =
                    self.core()?
                        .file(&filing.id, &filing.title, &filing.tags, &filing.folder)?;
                done(json!(id))
            }
            "set_text" => {
                self.core()?
                    .set_text(&text(request, "id")?, &text_or(request, "text"))?;
                done(Value::Null)
            }
            "delete" => {
                self.core()?.delete(&text(request, "id")?)?;
                done(Value::Null)
            }
            "create_folder" => {
                self.core()?.create_folder(&text(request, "name")?)?;
                done(Value::Null)
            }
            "rename_folder" => {
                self.core()?
                    .rename_folder(&text(request, "name")?, &text(request, "new_name")?)?;
                done(Value::Null)
            }
            "remove_folder" => {
                self.core()?.remove_folder(&text(request, "name")?)?;
                done(Value::Null)
            }
            "extract_one" => {
                let read = self.core()?.extract_one()?;
                done(json!(read))
            }
            // ------------------------------------------- privacy
            "set_private" => {
                self.core()?
                    .set_private(&text(request, "id")?, flag(request, "private")?)?;
                done(Value::Null)
            }
            "set_folder_private" => {
                self.core()?
                    .set_folder_private(&text(request, "folder")?, flag(request, "private")?)?;
                done(Value::Null)
            }
            "set_unfiled_private" => {
                self.core()?
                    .set_unfiled_private(flag(request, "private")?)?;
                done(self.state())
            }
            "set_lookup_to_cloud" => {
                let allowed = flag(request, "allowed")?;
                let kind = text(request, "kind")?;
                self.core()?.set_lookup_to_cloud(&kind, allowed)?;
                done(self.state())
            }
            // ------------------------------------------- models
            "save_model" => {
                let model: Model = parse(request.get("model"))?;
                self.core()?.save_model(model)?;
                done(self.state())
            }
            "test_model" => {
                let model: Model = parse(request.get("model"))?;
                model.test()?;
                done(Value::Null)
            }
            "remove_model" => {
                self.core()?.remove_model(&text(request, "name")?)?;
                done(self.state())
            }
            "choose_model" => {
                self.core()?.choose_model(&text(request, "name")?)?;
                done(self.state())
            }
            "propose" => Ok(Step::Propose(
                self.core()?.proposal_request(&text(request, "id")?)?,
            )),
            "ask" => {
                let lookups: Vec<Lookup> = match request.get("lookups") {
                    Some(lookups) => parse(Some(lookups))?,
                    None => Vec::new(),
                };
                match self
                    .core()?
                    .question(&text(request, "question")?, &lookups)?
                {
                    Found::Nothing { left_out } => done(json!({ "nothing": left_out })),
                    Found::Sources(asking) => Ok(Step::Ask(asking)),
                }
            }
            "save_answer" => {
                let answer: Answer = parse(request.get("answer"))?;
                let id = self.core()?.save_answer(&answer)?;
                done(json!(id))
            }
            // ------------------------------------------- conflicts
            "versions" => {
                let versions = self.core()?.versions(&text(request, "id")?)?;
                done(json!(versions))
            }
            "keep_version" => {
                self.core()?
                    .keep_version(&text(request, "id")?, &text(request, "version")?)?;
                done(Value::Null)
            }
            "resolve" => {
                self.core()?
                    .resolve(&text(request, "id")?, &text_or(request, "text"))?;
                done(Value::Null)
            }
            other => Err(Error::Invalid(format!("no call named {other}"))),
        }
    }

    /// What the app needs to know about the Device and its Brain at a
    /// glance.
    fn state(&self) -> Value {
        match &self.core {
            None => json!({ "brain": null }),
            Some(core) => {
                let settings = &core.device().settings;
                json!({
                    "brain": core.brain().root(),
                    "models": core.models(),
                    "chosen": core.chosen_model().map(|m| m.name.clone()),
                    "unfiled_private": core.unfiled_private(),
                    "calendar_to_cloud": settings.calendar_to_cloud,
                    "contacts_to_cloud": settings.contacts_to_cloud,
                    "version": crate::VERSION,
                })
            }
        }
    }
}

/// A Note as the phone shows it: with whether it is Private now, by its
/// own mark, its Folder's or the Inbox's rule.
fn shown(core: &Core, note: &Note) -> Value {
    let mut value = json!(note);
    if let Some(fields) = value.as_object_mut() {
        fields.insert("is_private".to_string(), json!(core.is_private(note)));
        fields.insert("unfiled".to_string(), json!(note.unfiled()));
    }
    value
}

fn answer_of(result: Result<Value>) -> String {
    match result {
        Ok(value) => json!({ "ok": value }),
        Err(e) => {
            let kind = match &e {
                Error::Io(_) => "file",
                Error::Index(_) => "index",
                Error::NotFound(_) => "not_found",
                Error::Invalid(_) => "invalid",
                Error::Unreadable(_) => "unreadable",
                Error::Model(_) => "model",
                Error::NoModel => "no_model",
                Error::Private => "private",
            };
            // The words after the kind, which the app says itself.
            let message = match &e {
                Error::Io(inner) => inner.to_string(),
                Error::NotFound(m) | Error::Invalid(m) | Error::Unreadable(m) | Error::Model(m) => {
                    m.clone()
                }
                other => other.to_string(),
            };
            json!({ "error": { "kind": kind, "message": message } })
        }
    }
    .to_string()
}

fn text(request: &Value, key: &str) -> Result<String> {
    request
        .get(key)
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| Error::Invalid(format!("the request names no {key}")))
}

fn text_or(request: &Value, key: &str) -> String {
    request
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn flag(request: &Value, key: &str) -> Result<bool> {
    request
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| Error::Invalid(format!("the request names no {key}")))
}

fn parse<T: serde::de::DeserializeOwned>(value: Option<&Value>) -> Result<T> {
    let value = value.ok_or_else(|| Error::Invalid("the request is incomplete".to_string()))?;
    serde_json::from_value(value.clone()).map_err(|e| Error::Invalid(e.to_string()))
}
