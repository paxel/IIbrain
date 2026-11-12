//! Keeping an Answer as a Saved Answer on the desktop (#20).

mod common;

use common::{Queryable, TestResult, World, app, click, fill, harness, model, model_server, shows};

#[test]
fn an_answer_is_kept_and_shows_its_mark_and_its_sources() -> TestResult {
    let server = model_server(|_| "Mr Brandt comes on Monday [1].".to_string());
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\n---\n\nThe boiler leaks. Mr Brandt comes on Monday.\n",
    );
    let mut app = app(&world, "en");
    assert!(app.connect(model(&server, "big", false)));
    let mut harness = harness(app);
    click(&mut harness, "Ask");
    fill(&mut harness, "Question", "When does the plumber come?");
    click(&mut harness, "Ask");

    click(&mut harness, "Keep as a Note");
    let id = harness
        .state()
        .open_note()
        .ok_or("the Saved Answer is open")?
        .to_string();
    assert!(world.read(&id).contains("model: big"));
    assert!(shows(
        &harness,
        "Written by the Model big, from these Sources:"
    ));
    assert!(
        shows(&harness, "When does the plumber come? · Answer · Private"),
        "in the Inbox's list"
    );

    harness
        .get_by_role_and_label(egui::accesskit::Role::Link, "Plumber")
        .click();
    harness.run();
    assert_eq!(harness.state().open_note(), Some("Home/Plumber.md"));
    Ok(())
}

#[test]
fn a_source_that_is_gone_is_said_in_german() -> TestResult {
    let world = World::new();
    world.write(
        "Antwort.md",
        "---\ntitle: Wann kommt er?\nmodel: daheim\nsources:\n- Haus/Klempner.md\n---\n\nAm Montag.\n",
    );
    let mut harness = harness(app(&world, "de"));
    click(&mut harness, "Brain (1)");
    click(&mut harness, "Wann kommt er? · Antwort");
    assert!(shows(
        &harness,
        "Vom Modell daheim geschrieben, aus diesen Quellen:"
    ));
    assert!(shows(&harness, "Haus/Klempner.md (nicht mehr vorhanden)"));
    Ok(())
}
