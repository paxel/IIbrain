//! Searching the Brain by words (#11).

mod common;

use common::{TestResult, World, click, fill, harness, shows};

fn world() -> World {
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\n---\n\nThe boiler in the cellar leaks.\n",
    );
    world.write(
        "Money/Tax.md",
        "---\ntitle: Tax return\n---\n\nBefore May.\n",
    );
    world
}

#[test]
fn typed_words_list_the_notes_that_hold_them_with_an_excerpt() -> TestResult {
    let mut harness = harness(world().app("en"));
    fill(&mut harness, "Search", "boiler");
    assert!(shows(&harness, "1 Note found."));
    assert!(shows(&harness, "boiler in the cellar leaks"));

    click(&mut harness, "Plumber");
    assert_eq!(harness.state().open_note(), Some("Home/Plumber.md"));
    Ok(())
}

#[test]
fn nothing_found_is_said_and_clearing_shows_the_folder_again() -> TestResult {
    let mut harness = harness(world().app("de"));
    fill(&mut harness, "Suche", "U-Boot");
    assert!(shows(&harness, "Nichts gefunden."));

    click(&mut harness, "Leeren");
    assert!(!shows(&harness, "Nichts gefunden."));
    assert!(shows(&harness, "Nichts wartet auf Ablage."));
    Ok(())
}
