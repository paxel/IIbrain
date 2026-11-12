//! Connecting Models and declaring them local or cloud (#16).

mod common;

use std::fs;

use common::{Reply, Server, TestResult, World, model, model_server};
use iibrain_core::{Capture, Core, Device, Message, Model};

#[test]
fn a_model_is_asked_in_the_openai_form_with_its_key() -> TestResult {
    let server = model_server(|_| "  Hello there.  ".to_string());
    let llama = model(&server, "llama", true);
    assert_eq!(
        llama.ask(&[Message::system("Be brief."), Message::user("Hi")])?,
        "Hello there."
    );

    let requests = server.requests();
    let request = requests.first().ok_or("a request")?;
    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/v1/chat/completions");
    assert_eq!(request.header("authorization"), Some("Bearer key-of-llama"));
    let body: serde_json::Value = serde_json::from_str(&request.body)?;
    assert_eq!(body["model"], "llama-large");
    assert_eq!(body["messages"][0]["role"], "system");
    assert_eq!(body["messages"][1]["content"], "Hi");
    Ok(())
}

#[test]
fn a_model_without_a_key_sends_none_and_a_trailing_slash_is_no_matter() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let plain = Model {
        key: None,
        address: format!("{}/v1/", server.url),
        ..model(&server, "home", true)
    };
    plain.test()?;
    let requests = server.requests();
    let request = requests.first().ok_or("a request")?;
    assert_eq!(request.path, "/v1/chat/completions");
    assert_eq!(request.header("authorization"), None);
    Ok(())
}

#[test]
fn testing_tells_a_wrong_key_a_wrong_address_and_a_wrong_name_apart() -> TestResult {
    let refusing = Server::start(|request| match request.path.as_str() {
        "/v1/chat/completions" if request.header("authorization") != Some("Bearer right") => {
            Reply::of(
                401,
                "application/json",
                r#"{"error":{"message":"Incorrect API key"}}"#,
            )
        }
        "/v1/chat/completions" => Reply::of(
            404,
            "application/json",
            r#"{"error":{"message":"model not found"}}"#,
        ),
        _ => Reply::status(404),
    });
    let base = model(&refusing, "cloud", false);

    let wrong_key = base.test().err().ok_or("a refusal")?.to_string();
    assert!(
        wrong_key.contains("key was refused") && wrong_key.contains("Incorrect API key"),
        "{wrong_key}"
    );

    let wrong_name = Model {
        key: Some("right".to_string()),
        ..base.clone()
    }
    .test()
    .err()
    .ok_or("a refusal")?
    .to_string();
    assert!(wrong_name.contains("model not found"), "{wrong_name}");

    let wrong_address = Model {
        address: "http://127.0.0.1:9/v1".to_string(),
        ..base.clone()
    }
    .test()
    .err()
    .ok_or("a failure")?
    .to_string();
    assert!(wrong_address.contains("not reached"), "{wrong_address}");

    assert!(
        Model {
            address: "localhost:11434".to_string(),
            ..base
        }
        .test()
        .is_err()
    );
    Ok(())
}

#[test]
fn an_answer_in_no_known_form_is_an_error() {
    let server = Server::start(|_| Reply::json(r#"{"unexpected":true}"#));
    assert!(
        model(&server, "odd", true)
            .ask(&[Message::user("Hi")])
            .is_err()
    );
}

#[test]
fn several_models_are_kept_and_one_is_chosen() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let world = World::new();
    let mut core = world.core();
    assert!(core.chosen_model().is_none());

    core.save_model(model(&server, "home", true))?;
    core.save_model(model(&server, "cloud", false))?;
    assert_eq!(core.models().len(), 2);
    assert_eq!(
        core.chosen_model().map(|m| m.name.as_str()),
        Some("home"),
        "the first is chosen"
    );

    core.choose_model("cloud")?;
    assert_eq!(
        core.chosen_model().map(|m| (m.name.as_str(), m.local)),
        Some(("cloud", false))
    );
    assert!(core.choose_model("unknown").is_err());

    // The same name again replaces the Model.
    core.save_model(Model {
        local: true,
        ..model(&server, "cloud", false)
    })?;
    assert_eq!(core.models().len(), 2);
    assert_eq!(core.chosen_model().map(|m| m.local), Some(true));

    core.remove_model("cloud")?;
    assert_eq!(core.chosen_model().map(|m| m.name.as_str()), Some("home"));
    core.remove_model("home")?;
    assert!(core.chosen_model().is_none());
    assert!(core.remove_model("home").is_err());
    Ok(())
}

#[test]
fn models_belong_to_the_device_and_never_enter_the_brain() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let world = World::new();
    world.write("a.md", "a Note");
    let mut core = world.core();
    core.save_model(model(&server, "cloud", false))?;
    drop(core);

    for file in world.files() {
        let content = fs::read(world.brain().join(&file))?;
        let content = String::from_utf8_lossy(&content);
        assert!(!content.contains("key-of-cloud"), "{file} holds the key");
        assert!(!content.contains(&server.url), "{file} holds the address");
    }
    // The Device remembers them at the next start.
    let again = Core::open(Device::load(&world.device())?)?;
    assert_eq!(again.models().len(), 1);
    // Another Device on the same folder knows none of them.
    let other = Core::open_folder(&world.other_device(), &world.brain())?;
    assert!(other.models().is_empty());
    Ok(())
}

#[test]
fn a_model_needs_a_name_and_an_address() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let world = World::new();
    let mut core = world.core();
    assert!(
        core.save_model(Model {
            name: " ".to_string(),
            ..model(&server, "x", true)
        })
        .is_err()
    );
    assert!(
        core.save_model(Model {
            address: String::new(),
            ..model(&server, "x", true)
        })
        .is_err()
    );
    Ok(())
}

#[test]
fn without_a_model_capturing_filing_and_search_work() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&Capture {
        text: "Tomatoes need a stake.".to_string(),
        via: "typed".to_string(),
        ..Capture::default()
    })?;
    let filed = core.file(&id, "Tomatoes", &[], "")?;
    assert_eq!(
        core.search("stake")?.first().map(|h| h.id.clone()),
        Some(filed)
    );
    Ok(())
}
