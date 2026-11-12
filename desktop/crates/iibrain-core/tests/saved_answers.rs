//! Keeping an Answer as a Saved Answer (#20).

mod common;

use common::{TestResult, World, model, model_server, sent_to};
use iibrain_core::Core;

fn world() -> World {
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\n---\n\nThe boiler leaks. Mr Brandt comes on Monday.\n",
    );
    world.write(
        "Health/Blood.md",
        "---\ntitle: Blood test\nprivate: true\n---\n\nIron is low, ferritin 12.\n",
    );
    world
}

fn with_model(
    world: &World,
    server: &common::Server,
    local: bool,
) -> Result<Core, Box<dyn std::error::Error>> {
    let mut core = world.core();
    core.save_model(model(server, if local { "home" } else { "big" }, local))?;
    Ok(core)
}

#[test]
fn a_saved_answer_is_a_note_marked_as_written_by_the_model_linking_its_sources() -> TestResult {
    let server = model_server(|_| "Mr Brandt comes on Monday [1].".to_string());
    let world = world();
    let mut core = with_model(&world, &server, false)?;
    let answer = core
        .ask("When does the plumber come?", &[])?
        .ok_or("an Answer")?;

    let id = core.save_answer(&answer)?;
    let note = core.note(&id)?;
    assert!(note.unfiled());
    assert_eq!(note.title, "When does the plumber come?");
    assert_eq!(note.model.as_deref(), Some("big"));
    assert_eq!(note.sources, ["Home/Plumber.md"]);
    assert_eq!(note.via.as_deref(), Some("answer"));
    assert!(note.text.starts_with("Mr Brandt comes on Monday [1]."));
    assert!(
        note.text.contains("- Plumber (Home/Plumber.md)"),
        "readable in any tool: {}",
        note.text
    );

    let file = world.read(&id);
    assert!(file.contains("model: big"), "{file}");
    assert!(file.contains("- Home/Plumber.md"), "{file}");
    assert!(!file.contains("private"), "no Source was Private");
    Ok(())
}

#[test]
fn a_saved_answer_is_found_and_serves_as_a_source_again() -> TestResult {
    let server = model_server(|_| {
        "The craftsman called Brandt arrives at the start of the week.".to_string()
    });
    let world = world();
    let mut core = with_model(&world, &server, false)?;
    core.set_unfiled_private(false)?;
    let answer = core
        .ask("When does the plumber come?", &[])?
        .ok_or("an Answer")?;
    let id = core.save_answer(&answer)?;

    assert_eq!(
        core.search("craftsman")?.first().map(|h| h.id.clone()),
        Some(id.clone())
    );
    let again = core
        .ask("Which craftsman arrives?", &[])?
        .ok_or("an Answer")?;
    assert!(
        again.sources.iter().any(|s| s.id == id),
        "the Saved Answer is a Source now"
    );
    Ok(())
}

#[test]
fn a_saved_answer_with_a_private_source_is_private_until_the_person_lifts_the_mark() -> TestResult {
    let local = model_server(|_| "Ferritin is 12, which is low [1].".to_string());
    let cloud = model_server(|_| "x".to_string());
    let world = world();
    let mut core = with_model(&world, &local, true)?;
    core.set_unfiled_private(false)?;
    let answer = core.ask("How is my ferritin?", &[])?.ok_or("an Answer")?;
    assert!(answer.private);

    let id = core.save_answer(&answer)?;
    assert!(world.read(&id).contains("private: true"));
    assert!(core.is_private(&core.note(&id)?));
    let filed = core.file(&id, "Ferritin", &[], "")?;
    assert!(
        core.is_private(&core.note(&filed)?),
        "filing does not lift it"
    );

    // So the summary of a Private Note does not leave through a Cloud Model.
    core.save_model(model(&cloud, "big", false))?;
    core.choose_model("big")?;
    assert_eq!(core.ask("ferritin?", &[])?, None);
    assert!(!sent_to(&cloud).contains("Ferritin"));

    // The person decides when a summary is harmless.
    core.set_private(&filed, false)?;
    assert!(!core.is_private(&core.note(&filed)?));
    assert!(core.ask("ferritin?", &[])?.is_some());
    Ok(())
}

#[test]
fn a_long_question_is_cut_for_the_title() -> TestResult {
    let server = model_server(|_| "Monday.".to_string());
    let world = world();
    let mut core = with_model(&world, &server, false)?;
    let question = format!("plumber {}", "and then ".repeat(30));
    let answer = core.ask(&question, &[])?.ok_or("an Answer")?;
    let id = core.save_answer(&answer)?;
    assert!(core.note(&id)?.title.chars().count() <= 80);
    Ok(())
}
