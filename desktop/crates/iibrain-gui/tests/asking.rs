//! The ask screen (#19).

mod common;

use common::{
    Queryable, Reply, Server, TestResult, World, app, click, fill, harness, model, model_server,
    sent_to, shows,
};
use iibrain_gui::App;

fn world() -> World {
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\n---\n\nThe boiler leaks. Mr Brandt comes on Monday.\n",
    );
    world.write(
        "Health/Blood.md",
        "---\ntitle: Blood test\nprivate: true\n---\n\nThe boiler of the body: iron is low.\n",
    );
    world
}

fn ask(harness: &mut egui_kittest::Harness<'_, App>, question: &str) {
    click(harness, "Ask");
    fill(harness, "Question", question);
    // The screen's own Ask button; the bar's reads "Back to the Notes" now.
    click(harness, "Ask");
}

#[test]
fn an_answer_shows_with_its_sources_and_a_source_opens_its_note() -> TestResult {
    let server = model_server(|_| "Mr Brandt comes on Monday [1].".to_string());
    let world = world();
    let mut app = app(&world, "en");
    assert!(app.connect(model(&server, "big", false)));
    let mut harness = harness(app);

    ask(&mut harness, "When does the plumber come?");
    assert!(shows(
        &harness,
        "Answer by big, from the Sources below alone:"
    ));
    assert!(shows(&harness, "Mr Brandt comes on Monday [1]."));
    assert!(shows(&harness, "Sources"));
    let sources = harness.state().answer().ok_or("an Answer")?.sources.clone();
    assert_eq!(sources.len(), 1);

    // The Source, not the Note's row in a list: the Inbox is listed.
    click(&mut harness, "Plumber");
    assert_eq!(harness.state().open_note(), Some("Home/Plumber.md"));
    assert!(shows(
        &harness,
        "The boiler leaks. Mr Brandt comes on Monday."
    ));
    Ok(())
}

#[test]
fn nothing_found_is_said_and_no_model_is_asked() -> TestResult {
    let server = model_server(|_| "invented".to_string());
    let world = world();
    let mut app = app(&world, "en");
    assert!(app.connect(model(&server, "big", false)));
    let mut harness = harness(app);

    ask(&mut harness, "What is the capital of Peru?");
    assert!(shows(&harness, "Nothing in your Brain fits this Question."));
    assert!(server.requests().is_empty());
    Ok(())
}

#[test]
fn private_notes_left_out_are_told() -> TestResult {
    let server = model_server(|_| "The boiler leaks [1].".to_string());
    let world = world();
    let mut app = app(&world, "en");
    assert!(app.connect(model(&server, "big", false)));
    let mut harness = harness(app);

    ask(&mut harness, "What about the boiler?");
    assert!(shows(&harness, "1 Private Note was left out"));
    assert!(!sent_to(&server).contains("iron is low"));
    Ok(())
}

#[test]
fn without_a_model_the_screen_says_so_and_leads_to_the_models() -> TestResult {
    let world = world();
    let mut harness = harness(app(&world, "de"));
    click(&mut harness, "Fragen");
    fill(&mut harness, "Frage", "Was ist mit dem Boiler?");
    click(&mut harness, "Fragen");
    assert!(shows(&harness, "Kein Modell ist verbunden."));

    // The button under the message; the bar has one of the same name.
    let buttons: Vec<_> = harness.query_all_by_label("Modelle…").collect();
    buttons.last().ok_or("a button")?.click();
    harness.run();
    assert!(shows(&harness, "Modell verbinden"));
    Ok(())
}

#[test]
fn a_failing_model_is_told() -> TestResult {
    let server = Server::start(|_| Reply::status(500));
    let world = world();
    let mut app = app(&world, "en");
    assert!(app.connect(model(&server, "broken", false)));
    let mut harness = harness(app);

    ask(&mut harness, "What about the boiler?");
    assert!(shows(&harness, "The Model did not answer"));
    Ok(())
}
