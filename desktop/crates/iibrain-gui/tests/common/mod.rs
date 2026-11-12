//! The desktop window driven headless over a Brain in a temporary
//! directory.

// Helpers of tests may unwrap, expect, panic and index, as tests may.
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::fs;
use std::path::PathBuf;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use iibrain_gui::App;
use tempfile::TempDir;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct World {
    pub dir: TempDir,
}

impl World {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("brain")).unwrap();
        Self { dir }
    }

    pub fn brain(&self) -> PathBuf {
        self.dir.path().join("brain")
    }

    pub fn device(&self) -> PathBuf {
        self.dir.path().join("device")
    }

    /// Writes a file into the Brain as another tool would.
    pub fn write(&self, relative: &str, content: &str) {
        let path = self.brain().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.brain().join(relative)).unwrap()
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.brain().join(relative).exists()
    }

    /// The app on a Device that has not chosen a Brain yet; its folder
    /// dialog answers with this world's Brain.
    pub fn fresh_app(&self, locale: &str) -> App {
        let brain = self.brain();
        App::new(&self.device(), locale).with_folder_picker(Box::new(move || Some(brain.clone())))
    }

    /// The app with this world's Brain open.
    pub fn app(&self, locale: &str) -> App {
        let mut app = self.fresh_app(locale);
        app.choose_brain(&self.brain());
        app
    }
}

pub fn harness(app: App) -> Harness<'static, App> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    harness.run();
    harness
}

/// Whether anything on the screen carries `label` in its text.
pub fn shows(harness: &Harness<'_, App>, label: &str) -> bool {
    harness.query_all_by_label_contains(label).next().is_some()
}

/// Clicks the one thing labelled exactly `label` and lets the window
/// settle.
pub fn click(harness: &mut Harness<'_, App>, label: &str) {
    harness.get_by_label(label).click();
    harness.run();
}
