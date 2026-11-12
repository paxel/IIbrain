//! Capturing typed Notes into the Inbox and filing them by hand (#10).

mod common;

use common::{TestResult, World, app, click, fill, harness, shows};

#[test]
fn a_typed_note_lands_in_the_inbox_and_is_counted() -> TestResult {
    let world = World::new();
    let mut harness = harness(app(&world, "en"));
    assert!(shows(&harness, "Nothing waits Unfiled."));

    click(&mut harness, "New Note");
    assert!(shows(&harness, "found on the phone"));
    fill(&mut harness, "Text", "Call the plumber on Monday.");
    click(&mut harness, "Save");

    assert!(shows(&harness, "Inbox (1)"));
    assert!(shows(&harness, "1 Note waits Unfiled."));
    let id = harness
        .state()
        .open_note()
        .ok_or("the new Note is open")?
        .to_string();
    assert!(world.read(&id).ends_with("Call the plumber on Monday."));
    Ok(())
}

#[test]
fn an_unfiled_note_is_filed_by_hand() -> TestResult {
    let world = World::new();
    world.write("Home/keep.md", "x");
    let mut harness = harness(app(&world, "en"));
    click(&mut harness, "New Note");
    fill(&mut harness, "Text", "Call the plumber.");
    click(&mut harness, "Save");

    click(&mut harness, "File…");
    assert!(shows(&harness, "File this Note"));
    fill(&mut harness, "Title", "Plumber");
    fill(&mut harness, "Tags", "house, todo");
    click(&mut harness, "Save");

    assert!(
        world.exists("Plumber.md"),
        "filed into the Brain's root by default"
    );
    assert!(shows(&harness, "Inbox (0)"));
    assert!(shows(&harness, "#house #todo"));
    Ok(())
}

#[test]
fn a_note_is_edited_and_deleted() -> TestResult {
    let world = World::new();
    world.write("Home/Plumber.md", "---\ntitle: Plumber\n---\n\nold");
    let mut harness = harness(app(&world, "en"));
    click(&mut harness, "Home (1)");
    click(&mut harness, "Plumber");

    click(&mut harness, "Edit");
    fill(&mut harness, "Text", " and new");
    click(&mut harness, "Save");
    assert!(world.read("Home/Plumber.md").ends_with("old and new"));

    click(&mut harness, "Delete");
    assert!(shows(&harness, "Delete this Note with its Attachments?"));
    // The dialog's own Delete, not the Note's button under it.
    harness.state_mut().confirm_delete();
    harness.run();
    assert!(!world.exists("Home/Plumber.md"));
    assert!(shows(&harness, "Home (0)"));
    Ok(())
}

#[test]
fn folders_are_made_renamed_and_removed() -> TestResult {
    let world = World::new();
    let mut harness = harness(app(&world, "en"));
    assert!(!shows(&harness, "Rename…"), "the Inbox is fixed");

    click(&mut harness, "New Folder…");
    fill(&mut harness, "Name", "Home");
    click(&mut harness, "Create");
    assert!(world.exists("Home"));
    assert!(shows(&harness, "Home (0)"));

    click(&mut harness, "Rename…");
    fill(&mut harness, "Name", "stead");
    click(&mut harness, "Rename");
    assert!(world.exists("Homestead") && !world.exists("Home"));

    click(&mut harness, "Remove");
    assert!(!world.exists("Homestead"));
    Ok(())
}

#[test]
fn a_folder_that_holds_notes_is_not_removed() -> TestResult {
    let world = World::new();
    world.write("Home/keep.md", "x");
    let mut harness = harness(app(&world, "de"));
    click(&mut harness, "Home (1)");
    click(&mut harness, "Entfernen");
    assert!(world.exists("Home/keep.md"));
    assert!(shows(&harness, "Das geht nicht"));
    Ok(())
}
