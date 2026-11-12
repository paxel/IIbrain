//! Connecting Models on the desktop (#16).

mod common;

use common::{Reply, Server, TestResult, World, app, click, fill, harness, model_server, shows};

fn fill_model(
    harness: &mut egui_kittest::Harness<'_, iibrain_gui::App>,
    name: &str,
    address: &str,
) {
    fill(harness, "Name", name);
    fill(harness, "Address", address);
    fill(harness, "Key", "secret-key");
    fill(harness, "Model at the endpoint", "llama3");
}

#[test]
fn a_model_that_answers_is_added_and_shown_as_local_or_cloud() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let world = World::new();
    let mut harness = harness(app(&world, "en"));
    click(&mut harness, "Models…");
    assert!(shows(&harness, "No Model is connected."));
    assert!(shows(&harness, "never enter the Brain's folder"));

    fill_model(&mut harness, "home", &format!("{}/v1", server.url));
    click(&mut harness, "It runs under my own control (Local Model)");
    click(&mut harness, "Test and add");
    assert!(shows(&harness, "home"));
    assert!(shows(&harness, "Local Model"));
    assert_eq!(server.requests().len(), 1, "the connection was tried");

    fill_model(&mut harness, "big", &format!("{}/v1", server.url));
    click(&mut harness, "Test and add");
    assert!(shows(&harness, "Cloud Model"));

    let core = harness.state().core().ok_or("a core")?;
    assert_eq!(core.models().len(), 2);
    assert_eq!(core.chosen_model().map(|m| m.name.as_str()), Some("home"));

    click(&mut harness, "big");
    let core = harness.state().core().ok_or("a core")?;
    assert_eq!(core.chosen_model().map(|m| m.name.as_str()), Some("big"));
    Ok(())
}

#[test]
fn a_wrong_key_is_told_and_the_model_is_not_added() -> TestResult {
    let server = Server::start(|_| {
        Reply::of(
            401,
            "application/json",
            r#"{"error":{"message":"Incorrect API key"}}"#,
        )
    });
    let world = World::new();
    let mut harness = harness(app(&world, "en"));
    click(&mut harness, "Models…");
    fill_model(&mut harness, "cloud", &format!("{}/v1", server.url));
    click(&mut harness, "Test and add");

    assert!(shows(&harness, "The Model did not answer"));
    assert!(shows(&harness, "Incorrect API key"));
    assert!(harness.state().core().ok_or("a core")?.models().is_empty());

    // A Model that is off right now can be added all the same.
    click(&mut harness, "Add without testing");
    assert_eq!(harness.state().core().ok_or("a core")?.models().len(), 1);
    Ok(())
}

#[test]
fn a_model_is_removed_in_german() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let world = World::new();
    let mut harness = harness(app(&world, "de"));
    click(&mut harness, "Modelle…");
    fill(&mut harness, "Name", "daheim");
    fill(&mut harness, "Adresse", &format!("{}/v1", server.url));
    click(&mut harness, "Ohne Test hinzufügen");
    assert!(shows(&harness, "Cloud-Modell"));

    click(&mut harness, "Entfernen");
    assert!(shows(&harness, "Kein Modell ist verbunden."));
    Ok(())
}
