//! A link captured on the desktop (#14). Nothing listens at the link's
//! address here, so the page's text is missing; the fetching itself is
//! tested in the core.

mod common;

use common::{TestResult, World, click, fill, harness, shows};

#[test]
fn a_link_on_its_own_is_kept_as_a_link_and_a_missing_page_is_said() -> TestResult {
    let world = World::new();
    let mut harness = harness(world.app("en"));
    click(&mut harness, "New Note");
    assert!(shows(&harness, "kept together with the text of its page"));
    fill(&mut harness, "Text", "http://127.0.0.1:9/nothing");
    click(&mut harness, "Save");

    let id = harness
        .state()
        .open_note()
        .ok_or("the Note is open")?
        .to_string();
    assert!(world.read(&id).contains("via: link"));
    assert!(world.read(&id).contains("link: http://127.0.0.1:9/nothing"));
    assert!(shows(&harness, "The text of the page is missing"));
    Ok(())
}

#[test]
fn words_with_a_link_in_them_are_a_typed_note() -> TestResult {
    let world = World::new();
    let mut harness = harness(world.app("de"));
    click(&mut harness, "Neue Notiz");
    fill(&mut harness, "Text", "siehe http://127.0.0.1:9/nothing");
    click(&mut harness, "Speichern");

    let id = harness
        .state()
        .open_note()
        .ok_or("the Note is open")?
        .to_string();
    assert!(world.read(&id).contains("via: typed"));
    assert!(!shows(&harness, "Der Text der Seite fehlt"));
    Ok(())
}
