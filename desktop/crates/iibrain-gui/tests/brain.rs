//! Opening a folder as the Brain and reading its Notes (#9).

mod common;

use common::{TestResult, World, app, click, fresh_app, harness, shows};

#[test]
fn the_first_start_asks_for_the_folder_and_opens_it() -> TestResult {
    let world = World::new();
    world.write(
        "Money/tax.md",
        "---\ntitle: Tax return\n---\n\nBefore May.\n",
    );
    let mut harness = harness(fresh_app(&world, "en"));
    assert!(shows(&harness, "Where is your Brain?"));

    click(&mut harness, "Choose folder…");
    assert!(!shows(&harness, "Where is your Brain?"));
    assert!(shows(&harness, "Money (1)"));
    Ok(())
}

#[test]
fn the_brain_is_remembered_at_the_next_start() -> TestResult {
    let world = World::new();
    world.write("a.md", "# Alpha\n");
    drop(app(&world, "en"));

    let harness = harness(fresh_app(&world, "en"));
    assert!(!shows(&harness, "Where is your Brain?"));
    assert!(shows(&harness, "Brain (1)"));
    Ok(())
}

#[test]
fn a_folder_lists_its_notes_and_a_note_opens_with_its_attachments() -> TestResult {
    let world = World::new();
    world.write(
        "Money/tax.md",
        "---\ntitle: Tax return\ntags: [tax]\n---\n\nBefore May.\n",
    );
    world.write("Money/tax.files/scan.jpg", "jpg");
    world.write("Money/tax.files/scan.jpg.extracted.txt", "total 120 EUR");
    let mut harness = harness(app(&world, "en"));

    assert!(
        shows(&harness, "Nothing waits Unfiled."),
        "the Inbox is empty"
    );
    click(&mut harness, "Money (1)");
    click(&mut harness, "Tax return");
    assert!(shows(&harness, "Before May."));
    assert!(shows(&harness, "#tax"));
    assert!(shows(&harness, "scan.jpg"));
    assert!(shows(&harness, "total 120 EUR"));
    Ok(())
}

#[test]
fn what_another_tool_adds_shows_without_a_restart() -> TestResult {
    let world = World::new();
    world.write("Money/tax.md", "x");
    let mut harness = harness(app(&world, "en"));
    assert!(shows(&harness, "Money (1)"));

    world.write("Money/receipt.md", "y");
    harness.state_mut().refresh();
    harness.run();
    assert!(shows(&harness, "Money (2)"));
    Ok(())
}

#[test]
fn the_screens_speak_german() -> TestResult {
    let world = World::new();
    let mut harness = harness(fresh_app(&world, "de"));
    assert!(shows(&harness, "Wo liegt dein Brain?"));
    click(&mut harness, "Ordner wählen…");
    assert!(shows(&harness, "Eingang (0)"));
    assert!(shows(&harness, "Nichts wartet auf Ablage."));
    Ok(())
}
