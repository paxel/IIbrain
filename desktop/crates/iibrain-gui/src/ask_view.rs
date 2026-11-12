//! The ask screen: a Question in the person's own words, and the
//! Answer with its Sources.

use std::time::Duration;

use egui::Ui;
use iibrain_core::{Answer, Error, Found};

use crate::app::{App, Dialog, error_text};
use crate::job::Job;

/// What the ask screen shows for the last Question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Asked {
    /// Nothing the Model may see fits; that many Private Notes did.
    Nothing {
        left_out: usize,
    },
    Answer(Answer),
    /// No Model is connected.
    NoModel,
}

impl App {
    /// Asks the Question in the field.
    pub(crate) fn ask_question(&mut self) {
        let question = self.question.trim().to_string();
        if question.is_empty() || self.asking.is_some() {
            return;
        }
        self.asked = None;
        self.error = None;
        let found = match self.core.as_ref().map(|core| core.question(&question, &[])) {
            Some(Ok(found)) => found,
            Some(Err(Error::NoModel)) => {
                self.asked = Some(Asked::NoModel);
                return;
            }
            Some(Err(e)) => {
                self.error = Some(error_text(&self.t, &e));
                return;
            }
            None => return,
        };
        match found {
            Found::Nothing { left_out } => self.asked = Some(Asked::Nothing { left_out }),
            Found::Sources(asking) => {
                self.asking = Some(Job::start(self.beside, move || asking.send()));
            }
        }
    }

    pub(crate) fn show_ask(&mut self, ui: &mut Ui) {
        match self.asking.as_ref().and_then(Job::take) {
            Some(Ok(answer)) => {
                self.asked = Some(Asked::Answer(answer));
                self.asking = None;
            }
            Some(Err(e)) => {
                self.error = Some(error_text(&self.t, &e));
                self.asking = None;
            }
            None => {}
        }
        ui.heading(self.t.ask_title());
        let label = ui.label(self.t.question_label());
        ui.add(
            egui::TextEdit::multiline(&mut self.question)
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(label.id);
        ui.horizontal(|ui| {
            let ready = !self.question.trim().is_empty() && self.asking.is_none();
            if ui
                .add_enabled(ready, egui::Button::new(self.t.ask()))
                .clicked()
            {
                self.ask_question();
            }
            if self.asking.is_some() {
                ui.spinner();
                ui.weak(self.t.model_working());
                ui.ctx().request_repaint_after(Duration::from_millis(200));
            }
        });
        ui.separator();
        let Some(asked) = self.asked.clone() else {
            return;
        };
        egui::ScrollArea::vertical()
            .id_salt("answer")
            .auto_shrink(false)
            .show(ui, |ui| match asked {
                Asked::NoModel => {
                    ui.label(self.t.error_no_model());
                    if ui.button(self.t.models()).clicked() {
                        self.dialog = Some(Dialog::new_model());
                    }
                }
                Asked::Nothing { left_out } => {
                    ui.label(self.t.nothing_fits());
                    self.show_left_out(ui, left_out);
                }
                Asked::Answer(answer) => {
                    ui.weak(self.t.answer_by(&answer.model));
                    ui.add(egui::Label::new(answer.text.as_str()).selectable(true));
                    self.show_left_out(ui, answer.left_out);
                    if ui.button(self.t.keep_answer()).clicked()
                        && let Some(id) = self.act(|core| core.save_answer(&answer))
                    {
                        self.folder = iibrain_core::INBOX.to_string();
                        self.open = Some(id);
                        self.ask_view = false;
                        self.search.clear();
                    }
                    ui.separator();
                    ui.strong(self.t.sources());
                    for source in &answer.sources {
                        if ui.link(source.title.as_str()).clicked() {
                            self.folder = source.folder.clone();
                            self.open = Some(source.id.clone());
                            self.ask_view = false;
                            self.search.clear();
                        }
                    }
                }
            });
    }

    fn show_left_out(&self, ui: &mut Ui, left_out: usize) {
        if left_out > 0 {
            let count = i64::try_from(left_out).unwrap_or(i64::MAX);
            ui.weak(self.t.private_left_out(count));
        }
    }
}
