//! The core as the phone calls it: JSON in, JSON out (#22, ADR 0011).

mod common;

use common::{TestResult, World, model, model_server};
use iibrain_core::api::Session;
use serde_json::{Value, json};

/// Sends one request and gives what stands under `ok`.
fn ok(session: &mut Session, request: Value) -> Result<Value, Box<dyn std::error::Error>> {
    let answer: Value = serde_json::from_str(&session.call(&request.to_string()))?;
    match answer.get("ok") {
        Some(value) => Ok(value.clone()),
        None => Err(format!("{request} was answered with {answer}").into()),
    }
}

/// Sends one request and gives the kind of its error.
fn error(session: &mut Session, request: Value) -> Result<String, Box<dyn std::error::Error>> {
    let answer: Value = serde_json::from_str(&session.call(&request.to_string()))?;
    Ok(answer
        .pointer("/error/kind")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{request} was answered with {answer}"))?
        .to_string())
}

fn opened(world: &World) -> Result<Session, Box<dyn std::error::Error>> {
    let mut session = Session::default();
    ok(
        &mut session,
        json!({"call": "open", "device_dir": world.device()}),
    )?;
    ok(
        &mut session,
        json!({"call": "choose_brain", "folder": world.brain()}),
    )?;
    Ok(session)
}

#[test]
fn a_device_opens_without_a_brain_and_then_chooses_one() -> TestResult {
    let world = World::new();
    world.write("Home/Plumber.md", "---\ntitle: Plumber\n---\n\nMonday.\n");
    let mut session = Session::default();
    assert_eq!(
        error(&mut session, json!({"call": "folders"}))?,
        "not_found"
    );

    let state = ok(
        &mut session,
        json!({"call": "open", "device_dir": world.device()}),
    )?;
    assert_eq!(state["brain"], Value::Null);

    let state = ok(
        &mut session,
        json!({"call": "choose_brain", "folder": world.brain()}),
    )?;
    assert_eq!(state["brain"], json!(world.brain()));
    assert_eq!(state["unfiled_private"], true);

    // The next start finds the Brain again.
    let mut again = Session::default();
    let state = ok(
        &mut again,
        json!({"call": "open", "device_dir": world.device()}),
    )?;
    assert_eq!(state["brain"], json!(world.brain()));
    Ok(())
}

#[test]
fn the_phone_reads_what_the_desktop_wrote_and_the_other_way_round() -> TestResult {
    let world = World::new();
    let mut desktop = world.core();
    let from_desktop = desktop.capture(&iibrain_core::Capture {
        text: "written on the desktop".to_string(),
        via: "typed".to_string(),
        ..Default::default()
    })?;

    let mut phone = Session::default();
    ok(
        &mut phone,
        json!({"call": "open", "device_dir": world.other_device()}),
    )?;
    ok(
        &mut phone,
        json!({"call": "choose_brain", "folder": world.brain()}),
    )?;
    let notes = ok(&mut phone, json!({"call": "notes", "folder": "Inbox"}))?;
    assert_eq!(notes[0]["id"], json!(from_desktop));
    assert_eq!(notes[0]["text"], "written on the desktop");
    assert_eq!(notes[0]["unfiled"], true);
    assert_eq!(notes[0]["is_private"], true);

    let from_phone = ok(
        &mut phone,
        json!({"call": "capture", "capture": {"text": "written on the phone", "via": "typed"}}),
    )?;
    let from_phone = from_phone.as_str().ok_or("an id")?;
    desktop.refresh()?;
    assert_eq!(desktop.note(from_phone)?.text, "written on the phone");
    assert!(
        world.read(from_phone).starts_with("---\n"),
        "the same kind of file"
    );
    Ok(())
}

#[test]
fn filing_editing_searching_and_deleting_go_through_the_call() -> TestResult {
    let world = World::new();
    let mut session = opened(&world)?;
    let id = ok(
        &mut session,
        json!({"call": "capture", "capture": {"text": "The boiler leaks.", "via": "typed"}}),
    )?;
    ok(
        &mut session,
        json!({"call": "create_folder", "name": "Home"}),
    )?;
    let filed = ok(
        &mut session,
        json!({"call": "file", "id": id, "title": "Boiler", "tags": ["house"], "folder": "Home"}),
    )?;
    assert_eq!(filed, "Home/Boiler.md");

    ok(
        &mut session,
        json!({"call": "set_text", "id": filed, "text": "The boiler leaks badly."}),
    )?;
    let note = ok(&mut session, json!({"call": "note", "id": filed}))?;
    assert_eq!(note["text"], "The boiler leaks badly.");
    assert_eq!(note["tags"], json!(["house"]));

    let hits = ok(&mut session, json!({"call": "search", "query": "badly"}))?;
    assert_eq!(hits[0]["id"], "Home/Boiler.md");
    let folders = ok(&mut session, json!({"call": "folders"}))?;
    assert_eq!(folders[1]["name"], "Home");
    assert_eq!(folders[1]["notes"], 1);

    ok(
        &mut session,
        json!({"call": "set_private", "id": filed, "private": true}),
    )?;
    assert_eq!(
        ok(&mut session, json!({"call": "note", "id": filed}))?["is_private"],
        true
    );

    ok(&mut session, json!({"call": "delete", "id": filed}))?;
    assert_eq!(
        error(&mut session, json!({"call": "note", "id": filed}))?,
        "not_found"
    );
    assert_eq!(
        error(
            &mut session,
            json!({"call": "remove_folder", "name": "Inbox"})
        )?,
        "invalid"
    );
    Ok(())
}

#[test]
fn models_proposals_and_answers_go_through_the_call() -> TestResult {
    let server = model_server(|request| {
        if request.body.contains("You file notes") {
            r#"{"title": "Boiler", "tags": ["house"], "folder": "Home"}"#.to_string()
        } else {
            "It leaks [1].".to_string()
        }
    });
    let world = World::new();
    world.write("Home/keep.md", "The boiler was installed in 2019.");
    let mut session = opened(&world)?;
    assert_eq!(
        error(&mut session, json!({"call": "ask", "question": "boiler?"}))?,
        "no_model"
    );

    let big = serde_json::to_value(model(&server, "big", false))?;
    ok(&mut session, json!({"call": "test_model", "model": big}))?;
    let state = ok(&mut session, json!({"call": "save_model", "model": big}))?;
    assert_eq!(state["chosen"], "big");
    assert_eq!(state["models"][0]["local"], false);

    let id = ok(
        &mut session,
        json!({"call": "capture", "capture": {"text": "The boiler leaks.", "via": "typed"}}),
    )?;
    assert_eq!(
        error(&mut session, json!({"call": "propose", "id": id}))?,
        "private"
    );
    ok(
        &mut session,
        json!({"call": "set_unfiled_private", "private": false}),
    )?;
    let proposal = ok(&mut session, json!({"call": "propose", "id": id}))?;
    assert_eq!(proposal["title"], "Boiler");
    assert_eq!(proposal["new_folder"], false);

    let asked = ok(
        &mut session,
        json!({"call": "ask", "question": "What about the boiler?"}),
    )?;
    assert_eq!(asked["answer"]["text"], "It leaks [1].");
    assert_eq!(asked["answer"]["sources"].as_array().map(Vec::len), Some(2));
    let nothing = ok(&mut session, json!({"call": "ask", "question": "Peru?"}))?;
    assert_eq!(nothing, json!({"nothing": 0}));

    let saved = ok(
        &mut session,
        json!({"call": "save_answer", "answer": asked["answer"]}),
    )?;
    let note = ok(&mut session, json!({"call": "note", "id": saved}))?;
    assert_eq!(note["model"], "big");

    let state = ok(&mut session, json!({"call": "remove_model", "name": "big"}))?;
    assert_eq!(state["models"], json!([]));
    Ok(())
}

#[test]
fn a_question_waits_for_the_model_without_holding_the_session() -> TestResult {
    let server = model_server(|_| "It leaks [1].".to_string());
    let world = World::new();
    world.write("Home/keep.md", "The boiler was installed in 2019.");
    let mut session = opened(&world)?;
    ok(
        &mut session,
        json!({"call": "save_model", "model": serde_json::to_value(model(&server, "big", false))?}),
    )?;

    let step = session.begin(&json!({"call": "ask", "question": "boiler?"}).to_string());
    assert!(
        server.requests().is_empty(),
        "nothing is sent while the session is held"
    );
    // The session is free for the app's other calls meanwhile.
    ok(&mut session, json!({"call": "folders"}))?;
    let answer: Value = serde_json::from_str(&step.finish())?;
    assert_eq!(answer["ok"]["answer"]["text"], "It leaks [1].");
    Ok(())
}

#[test]
fn conflicts_and_lookup_switches_go_through_the_call() -> TestResult {
    let world = World::new();
    world.write("Home/Plumber.md", "Monday.");
    world.write("Home/Plumber (conflicted copy 2026-11-12).md", "Tuesday.");
    let mut session = opened(&world)?;
    let versions = ok(
        &mut session,
        json!({"call": "versions", "id": "Home/Plumber.md"}),
    )?;
    assert_eq!(versions[1]["text"], "Tuesday.");
    ok(
        &mut session,
        json!({"call": "resolve", "id": "Home/Plumber.md", "text": "Monday or Tuesday."}),
    )?;
    assert_eq!(world.files(), ["Home/Plumber.md"]);

    let state = ok(
        &mut session,
        json!({"call": "set_lookup_to_cloud", "kind": "calendar", "allowed": true}),
    )?;
    assert_eq!(state["calendar_to_cloud"], true);
    assert_eq!(state["contacts_to_cloud"], false);
    Ok(())
}

#[test]
fn a_request_nobody_understands_is_an_error_not_a_crash() -> TestResult {
    let mut session = Session::default();
    assert_eq!(error(&mut session, json!({"call": "open"}))?, "invalid");
    assert_eq!(error(&mut session, json!({"call": "fly"}))?, "invalid");
    let answer: Value = serde_json::from_str(&session.call("this is no JSON"))?;
    assert_eq!(answer["error"]["kind"], "invalid");
    Ok(())
}
