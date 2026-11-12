//! Asking: a Question in the person's own words, answered by a Model
//! from Notes of the Brain alone (ADR 0006). The app finds the Notes
//! itself, by full text, and hands them over as Sources; when it finds
//! nothing, no Model is asked.

use serde::{Deserialize, Serialize};

use crate::brain::Capture;
use crate::core::Core;
use crate::error::{Error, Result};
use crate::filing::whole_text;
use crate::model::{Message, Model};

/// How many Notes at most become the Sources of one Answer.
const MAX_SOURCES: usize = 8;

/// The filler words of English and German that say nothing about what
/// a Question is after.
const FILLERS: &str = "a an and are as at be been but by can could did do does for from had has \
    have how i if in is it its me my of on or our she so that the their them then there these \
    they this to was we were what when where which who whom why will with would you your about \
    into over than too very any some no not yes he his her him us also just more most much many \
    aber als am an auch auf aus bei bin bis bist da damit dann das dass dein deine dem den der \
    des dessen die dies diese dieser dieses dir doch du durch ein eine einem einen einer eines \
    er es euch euer für hat hatte hatten habe haben hast ich ihm ihn ihr ihre im in ins ist ja \
    kann kein keine können man mein meine mich mir mit nach nicht noch nun nur ob oder ohne sehr \
    sein seine sich sie sind so über um und uns unser unter vom von vor war waren was weil welche \
    welcher welches wem wen wenn wer werde werden wie wieder will wir wird wo wurde wurden zu zum \
    zur mal denn etwas gibt";

/// Something a Live Lookup read on the phone for one Question: never
/// kept in the Brain (ADR 0010).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lookup {
    /// `calendar` or `contacts`.
    pub kind: String,
    /// What was read, as text.
    pub text: String,
}

/// A Note handed to the Model for a Question.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub folder: String,
}

/// What a Model wrote in reply to a Question, from the Sources alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    pub question: String,
    pub text: String,
    /// The Notes that were handed over, all of them.
    pub sources: Vec<Source>,
    /// The name of the Model that wrote it.
    pub model: String,
    /// How many fitting Private Notes were not handed to a Cloud Model.
    pub left_out: usize,
    /// Whether a Source was Private; a Saved Answer then is too.
    pub private: bool,
}

/// A Question about to be asked: everything the asking needs, so that
/// it can wait for the Model beside the app.
#[derive(Debug, Clone)]
pub struct Asking {
    model: Model,
    messages: Vec<Message>,
    answer: Answer,
}

impl Asking {
    /// Asks the Model and gives its Answer with the Sources.
    pub fn send(self) -> Result<Answer> {
        let text = self.model.ask(&self.messages)?;
        Ok(Answer {
            text,
            ..self.answer
        })
    }
}

/// What the app found for a Question before any Model is asked.
#[derive(Debug, Clone)]
pub enum Found {
    /// Nothing the Model may see fits. `left_out` Private Notes did fit
    /// but stay away from a Cloud Model.
    Nothing { left_out: usize },
    /// Sources to ask the Model with.
    Sources(Box<Asking>),
}

/// The words of `question` a search goes by.
pub fn search_words(question: &str) -> Vec<String> {
    let fillers: Vec<&str> = FILLERS.split_whitespace().collect();
    let mut words: Vec<String> = Vec::new();
    for word in question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
    {
        if word.chars().count() > 1 && !fillers.contains(&word) && !words.iter().any(|w| w == word)
        {
            words.push(word.to_string());
        }
    }
    words
}

impl Core {
    /// Finds the Sources for `question` and prepares the asking. No
    /// Model is contacted here. `lookups` is what the phone read live
    /// for this Question, if anything.
    pub fn question(&self, question: &str, lookups: &[Lookup]) -> Result<Found> {
        let model = self.chosen_model().ok_or(Error::NoModel)?.clone();
        let hits = self
            .index
            .search(&search_words(question), true, MAX_SOURCES)?;
        let mut sources = Vec::new();
        let mut texts = Vec::new();
        let mut left_out = 0;
        let mut private = false;
        for hit in hits {
            let note = self.note(&hit.id)?;
            if !self.may_send(&note, &model) {
                left_out += 1;
                continue;
            }
            private |= self.is_private(&note);
            texts.push(format!(
                "[{}] {}\n{}",
                sources.len() + 1,
                note.title,
                whole_text(&note)
            ));
            sources.push(Source {
                id: note.id,
                title: note.title,
                folder: note.folder,
            });
        }
        // What a Live Lookup read passes the same gate, by its switch.
        let settings = &self.device.settings;
        let lookups: Vec<&Lookup> = lookups
            .iter()
            .filter(|l| !l.text.trim().is_empty())
            .filter(|l| {
                model.local
                    || match l.kind.as_str() {
                        "calendar" => settings.calendar_to_cloud,
                        "contacts" => settings.contacts_to_cloud,
                        _ => false,
                    }
            })
            .collect();
        if sources.is_empty() && lookups.is_empty() {
            return Ok(Found::Nothing { left_out });
        }
        let mut given = texts;
        for lookup in lookups {
            let heading = match lookup.kind.as_str() {
                "calendar" => "From the person's calendar",
                _ => "From the person's contacts",
            };
            given.push(format!("{heading}:\n{}", lookup.text.trim()));
        }
        let system = "You answer a person's question from their own notes. Use only what stands \
            in the notes given to you, and nothing you know from elsewhere. When the notes do \
            not hold the answer, say that the notes do not say. Name the notes you used by \
            their numbers, like [1]. Answer in the language of the question.";
        let user = format!(
            "The notes:\n\n{}\n\nThe question: {}",
            given.join("\n\n"),
            question.trim()
        );
        Ok(Found::Sources(Box::new(Asking {
            messages: vec![Message::system(system), Message::user(&user)],
            answer: Answer {
                question: question.trim().to_string(),
                text: String::new(),
                sources,
                model: model.name.clone(),
                left_out,
                private,
            },
            model,
        })))
    }

    /// Asks `question` and waits for the Answer; `None` when nothing in
    /// the Brain fits, and then no Model was asked.
    pub fn ask(&self, question: &str, lookups: &[Lookup]) -> Result<Option<Answer>> {
        match self.question(question, lookups)? {
            Found::Nothing { .. } => Ok(None),
            Found::Sources(asking) => asking.send().map(Some),
        }
    }

    /// Keeps an Answer as a Saved Answer: an Unfiled Note titled with
    /// the Question, marked as written by the Model, linking its
    /// Sources. With a Private Source it is Private (ADR 0004).
    pub fn save_answer(&mut self, answer: &Answer) -> Result<String> {
        let mut text = answer.text.trim().to_string();
        if !answer.sources.is_empty() {
            text.push_str("\n\nSources:\n");
            for source in &answer.sources {
                text.push_str(&format!("- {} ({})\n", source.title, source.id));
            }
        }
        let capture = Capture {
            text,
            via: "answer".to_string(),
            private: answer.private,
            ..Capture::default()
        };
        let id = self.brain.capture(&capture, chrono::Local::now())?;
        let title: String = answer.question.chars().take(80).collect();
        let sources: Vec<String> = answer.sources.iter().map(|s| s.id.clone()).collect();
        self.brain
            .set_answer(&id, title.trim(), &answer.model, &sources)?;
        self.refresh()?;
        Ok(id)
    }
}
