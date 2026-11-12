//! Asking a Question and getting an Answer with its Sources (#19).

mod common;

use common::{Reply, Server, TestResult, World, model, model_server, sent_to};
use iibrain_core::{Core, Error, Found, asking};

fn world() -> World {
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\n---\n\nThe boiler in the cellar leaks. Mr Brandt, phone 0151 555, comes on Monday.\n",
    );
    world.write(
        "Home/Boiler manual.md",
        "---\ntitle: Boiler manual\n---\n\nVaillant ecoTEC, serviced every two years.\n",
    );
    world.write(
        "Money/Tax.md",
        "---\ntitle: Tax return\n---\n\nSend it before May.\n",
    );
    world.write(
        "Health/Blood.md",
        "---\ntitle: Blood test\nprivate: true\n---\n\nThe boiler of the body: iron is low.\n",
    );
    world
}

fn with_model(
    world: &World,
    server: &Server,
    local: bool,
) -> Result<Core, Box<dyn std::error::Error>> {
    let mut core = world.core();
    core.save_model(model(server, if local { "home" } else { "big" }, local))?;
    Ok(core)
}

#[test]
fn an_answer_lists_exactly_the_notes_handed_over() -> TestResult {
    let server = model_server(|_| "Mr Brandt comes on Monday [1].".to_string());
    let world = world();
    let core = with_model(&world, &server, false)?;

    let answer = core
        .ask("When does the plumber come for the boiler?", &[])?
        .ok_or("an Answer")?;
    assert_eq!(answer.text, "Mr Brandt comes on Monday [1].");
    assert_eq!(answer.model, "big");
    let mut ids: Vec<&str> = answer.sources.iter().map(|s| s.id.as_str()).collect();
    ids.sort();
    assert_eq!(ids, ["Home/Boiler manual.md", "Home/Plumber.md"]);

    let sent = sent_to(&server);
    assert!(sent.contains("Mr Brandt, phone 0151 555"));
    assert!(sent.contains("Vaillant ecoTEC"));
    assert!(
        !sent.contains("Send it before May"),
        "a Note that does not fit is not sent"
    );
    assert_eq!(server.requests().len(), 1);
    Ok(())
}

#[test]
fn the_model_is_told_to_answer_from_the_sources_alone() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let world = world();
    let core = with_model(&world, &server, false)?;
    core.ask("boiler?", &[])?;

    let requests = server.requests();
    let body: serde_json::Value = serde_json::from_str(&requests.first().ok_or("a request")?.body)?;
    let system = body["messages"][0]["content"]
        .as_str()
        .ok_or("a system message")?;
    assert_eq!(body["messages"][0]["role"], "system");
    assert!(
        system.contains("Use only what stands in the notes"),
        "{system}"
    );
    assert!(
        system.contains("nothing you know from elsewhere"),
        "{system}"
    );
    let user = body["messages"][1]["content"]
        .as_str()
        .ok_or("a user message")?;
    assert!(user.ends_with("The question: boiler?"), "{user}");
    Ok(())
}

#[test]
fn a_question_that_fits_nothing_asks_no_model() -> TestResult {
    let server = model_server(|_| "I can invent something.".to_string());
    let world = world();
    let core = with_model(&world, &server, false)?;

    assert_eq!(core.ask("What is the capital of Peru?", &[])?, None);
    assert!(matches!(
        core.question("Who was it?", &[])?,
        Found::Nothing { left_out: 0 }
    ));
    assert!(server.requests().is_empty(), "no Model was asked");
    Ok(())
}

#[test]
fn private_notes_stay_away_from_a_cloud_model_and_are_counted() -> TestResult {
    let server = model_server(|_| "From the manual [1].".to_string());
    let world = world();
    let core = with_model(&world, &server, false)?;

    let answer = core.ask("boiler iron", &[])?.ok_or("an Answer")?;
    assert_eq!(answer.left_out, 1);
    assert!(!answer.private);
    assert!(answer.sources.iter().all(|s| s.id != "Health/Blood.md"));
    let sent = sent_to(&server);
    assert!(!sent.contains("iron is low"), "{sent}");
    assert!(!sent.contains("Blood test"), "not even its title: {sent}");
    Ok(())
}

#[test]
fn only_private_notes_fitting_means_nothing_is_sent_at_all() -> TestResult {
    let server = model_server(|_| "x".to_string());
    let world = world();
    let core = with_model(&world, &server, false)?;
    assert!(matches!(
        core.question("iron low?", &[])?,
        Found::Nothing { left_out: 1 }
    ));
    assert_eq!(core.ask("iron low?", &[])?, None);
    assert!(server.requests().is_empty());
    Ok(())
}

#[test]
fn a_local_model_gets_the_private_notes_too() -> TestResult {
    let server = model_server(|_| "Iron is low [1].".to_string());
    let world = world();
    let core = with_model(&world, &server, true)?;

    let answer = core.ask("iron low?", &[])?.ok_or("an Answer")?;
    assert_eq!(answer.left_out, 0);
    assert!(answer.private, "a Source was Private");
    assert_eq!(
        answer.sources.first().map(|s| s.id.as_str()),
        Some("Health/Blood.md")
    );
    assert!(sent_to(&server).contains("iron is low"));
    Ok(())
}

#[test]
fn unfiled_notes_count_as_private_for_the_gate() -> TestResult {
    let server = model_server(|_| "x".to_string());
    let world = world();
    world.write("Inbox/fresh.md", "The chimney sweep comes in March.");
    let mut core = with_model(&world, &server, false)?;
    assert!(matches!(
        core.question("chimney sweep?", &[])?,
        Found::Nothing { left_out: 1 }
    ));

    core.set_unfiled_private(false)?;
    assert!(matches!(
        core.question("chimney sweep?", &[])?,
        Found::Sources(_)
    ));
    Ok(())
}

#[test]
fn without_a_model_or_with_a_failing_one_the_person_is_told() -> TestResult {
    let world = world();
    let mut core = world.core();
    assert!(matches!(core.ask("boiler?", &[]), Err(Error::NoModel)));

    let failing = Server::start(|_| Reply::status(500));
    core.save_model(model(&failing, "broken", false))?;
    assert!(matches!(core.ask("boiler?", &[]), Err(Error::Model(_))));
    Ok(())
}

#[test]
fn a_question_is_searched_by_its_telling_words() {
    assert_eq!(
        asking::search_words("When does the plumber come for the boiler?"),
        ["plumber", "come", "boiler"]
    );
    assert_eq!(
        asking::search_words("Wann kommt der Klempner wegen des Boilers, und was kostet es?"),
        ["wann", "kommt", "klempner", "wegen", "boilers", "kostet"]
    );
    assert!(asking::search_words("Is it?").is_empty());
}
