//! Marking Notes and Folders as Private on the desktop (#17).

mod common;

use common::{TestResult, World, app, click, harness, shows};

#[test]
fn a_note_is_marked_private_and_shows_it_in_the_list() -> TestResult {
    let world = World::new();
    world.write("Health/Blood.md", "---\ntitle: Blood test\n---\n\nvalues\n");
    let mut harness = harness(app(&world, "en"));
    click(&mut harness, "Health (1)");
    click(&mut harness, "Blood test");

    click(&mut harness, "Private");
    assert!(world.read("Health/Blood.md").contains("private: true"));
    assert!(shows(&harness, "Blood test · Private"));
    Ok(())
}

#[test]
fn a_folder_is_marked_private_and_its_notes_say_why() -> TestResult {
    let world = World::new();
    world.write("Health/Blood.md", "---\ntitle: Blood test\n---\n\nvalues\n");
    let mut harness = harness(app(&world, "en"));
    click(&mut harness, "Health (1)");
    click(&mut harness, "Mark Private");
    assert!(world.exists("Health/.iibrain-private"));
    assert!(shows(&harness, "Health (1) · Private"));

    click(&mut harness, "Blood test · Private");
    assert!(shows(&harness, "Private because its Folder is."));

    click(&mut harness, "Lift Private mark");
    assert!(!world.exists("Health/.iibrain-private"));
    assert!(!shows(&harness, "Private because its Folder is."));
    Ok(())
}

#[test]
fn unfiled_notes_are_private_until_the_setting_is_cleared() -> TestResult {
    let world = World::new();
    world.write("Inbox/fresh.md", "fresh thought");
    let mut harness = harness(app(&world, "de"));
    assert!(shows(&harness, "fresh thought · privat"));
    click(&mut harness, "fresh thought · privat");
    assert!(shows(&harness, "Privat, solange sie nicht abgelegt ist."));

    click(&mut harness, "Nicht abgelegte Notizen gelten als privat");
    assert!(
        world
            .read(".iibrain/brain.yaml")
            .contains("unfiled_private: false")
    );
    assert!(!shows(&harness, "fresh thought · privat"));
    Ok(())
}
