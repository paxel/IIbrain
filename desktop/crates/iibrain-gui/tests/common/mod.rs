//! The desktop window driven headless over a Brain in a temporary
//! directory.

// Helpers of tests may unwrap, expect, panic and index, as tests may.
#![allow(
    dead_code,
    unused_imports,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use egui_kittest::Harness;
pub use egui_kittest::kittest::{NodeT, Queryable};
use iibrain_gui::App;

pub use iibrain_testing::{Reply, Server, TestResult, World, model, model_server, sent_to};

/// The app on a Device that has not chosen a Brain yet; its folder
/// dialog answers with this world's Brain.
pub fn fresh_app(world: &World, locale: &str) -> App {
    let brain = world.brain();
    App::new(&world.device(), locale)
        .with_folder_picker(Box::new(move || Some(brain.clone())))
        .reading_at_once()
}

/// The app with this world's Brain open.
pub fn app(world: &World, locale: &str) -> App {
    let mut app = fresh_app(world, locale);
    app.choose_brain(&world.brain());
    app
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

/// Types `text` into the field labelled `label`, after what it holds.
pub fn fill(harness: &mut Harness<'_, App>, label: &str, text: &str) {
    let field = harness
        .query_all_by_label(label)
        .find(|node| {
            matches!(
                node.accesskit_node().role(),
                egui::accesskit::Role::TextInput
                    | egui::accesskit::Role::MultilineTextInput
                    | egui::accesskit::Role::PasswordInput
            )
        })
        .unwrap_or_else(|| panic!("no field labelled {label}"));
    field.focus();
    harness.run();
    let field = harness
        .query_all_by_label(label)
        .find(|node| {
            matches!(
                node.accesskit_node().role(),
                egui::accesskit::Role::TextInput
                    | egui::accesskit::Role::MultilineTextInput
                    | egui::accesskit::Role::PasswordInput
            )
        })
        .unwrap_or_else(|| panic!("no field labelled {label}"));
    field.type_text(text);
    harness.run();
}

/// Whether a choice on the screen, as a combo box shows it, is `value`.
pub fn chosen(harness: &Harness<'_, App>, value: &str) -> bool {
    harness.query_all_by_value(value).next().is_some()
}
