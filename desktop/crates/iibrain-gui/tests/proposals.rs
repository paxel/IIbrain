//! Proposals for Unfiled Notes on the desktop (#18).

mod common;

use common::{
    Reply, Server, TestResult, World, app, chosen, click, fill, harness, model, model_server,
    sent_to, shows,
};
use iibrain_gui::App;

const PROPOSAL: &str =
    r#"{"title": "Plumber for the boiler", "tags": ["house", "repair"], "folder": "Home"}"#;

/// The app on a Brain with one Folder and one Unfiled Note, with the
/// Model at `server` connected; Unfiled Notes open or Private.
fn setup(world: &World, server: &Server, local: bool, unfiled_private: bool, locale: &str) -> App {
    world.write("Home/keep.md", "x");
    world.write("Inbox/boiler.md", "The boiler leaks, call the plumber.");
    world.write(
        ".iibrain/brain.yaml",
        &format!("unfiled_private: {unfiled_private}\n"),
    );
    let mut app = app(world, locale);
    let model = model(server, "big", local);
    assert!(app.connect(model), "the Model is added");
    app
}

fn open_filing(harness: &mut egui_kittest::Harness<'_, App>, list_label: &str, file_label: &str) {
    click(harness, list_label);
    click(harness, file_label);
}

#[test]
fn a_proposal_fills_the_dialog_and_is_accepted_as_it_is() -> TestResult {
    let server = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let mut harness = harness(setup(&world, &server, false, false, "en"));
    open_filing(&mut harness, "The boiler leaks, call the plumber.", "File…");

    click(&mut harness, "Propose");
    assert!(shows(&harness, "Proposal by big."));
    assert!(world.exists("Inbox/boiler.md"), "nothing is filed yet");

    click(&mut harness, "Save");
    assert!(world.exists("Home/Plumber for the boiler.md"));
    assert!(!world.exists("Inbox/boiler.md"));
    assert!(shows(&harness, "#house #repair"));
    Ok(())
}

#[test]
fn a_proposal_is_corrected_before_it_is_saved() -> TestResult {
    let server = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let mut harness = harness(setup(&world, &server, false, false, "en"));
    open_filing(&mut harness, "The boiler leaks, call the plumber.", "File…");
    click(&mut harness, "Propose");

    fill(&mut harness, "Title", " (urgent)");
    click(&mut harness, "Save");
    assert!(world.exists("Home/Plumber for the boiler (urgent).md"));
    Ok(())
}

#[test]
fn a_proposal_is_dismissed_and_the_note_stays_unfiled() -> TestResult {
    let server = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let mut harness = harness(setup(&world, &server, false, false, "en"));
    open_filing(&mut harness, "The boiler leaks, call the plumber.", "File…");
    click(&mut harness, "Propose");
    click(&mut harness, "Cancel");

    assert!(world.exists("Inbox/boiler.md"));
    assert!(!world.exists("Home/Plumber for the boiler.md"));
    assert!(shows(&harness, "1 Note waits Unfiled."));
    Ok(())
}

#[test]
fn a_suggested_new_folder_is_shown_as_new() -> TestResult {
    let server =
        model_server(|_| r#"{"title":"Apple tree","tags":[],"folder":"Garden"}"#.to_string());
    let world = World::new();
    let mut harness = harness(setup(&world, &server, false, false, "en"));
    open_filing(&mut harness, "The boiler leaks, call the plumber.", "File…");
    click(&mut harness, "Propose");
    assert!(chosen(&harness, "Garden (new Folder)"));
    assert!(!world.exists("Garden"));

    click(&mut harness, "Save");
    assert!(world.exists("Garden/Apple tree.md"));
    Ok(())
}

#[test]
fn a_private_unfiled_note_gets_no_proposal_from_a_cloud_model_and_says_why() -> TestResult {
    let server = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let mut harness = harness(setup(&world, &server, false, true, "de"));
    open_filing(
        &mut harness,
        "The boiler leaks, call the plumber. · privat",
        "Ablegen…",
    );

    click(&mut harness, "Vorschlagen");
    assert!(shows(
        &harness,
        "Das ist privat und dein Modell ist ein Cloud-Modell"
    ));
    assert!(
        !sent_to(&server).contains("boiler"),
        "nothing of the Note was sent"
    );
    Ok(())
}

#[test]
fn a_failing_model_is_told_and_filing_by_hand_goes_on() -> TestResult {
    let server = Server::start(|_| Reply::status(500));
    let world = World::new();
    let mut harness = harness(setup(&world, &server, true, true, "en"));
    open_filing(
        &mut harness,
        "The boiler leaks, call the plumber. · Private",
        "File…",
    );

    click(&mut harness, "Propose");
    assert!(shows(&harness, "The Model did not answer"));
    fill(&mut harness, "Title", "Plumber");
    click(&mut harness, "Save");
    assert!(world.exists("Plumber.md"));
    Ok(())
}
