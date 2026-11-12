//! Recognising and resolving Conflicts on the desktop (#21).

mod common;

use common::{TestResult, World, app, click, fill, harness, shows};

const OTHER: &str = "Home/Plumber.sync-conflict-20261112-153000-ABCDEFG.md";

fn world() -> World {
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\n---\n\nHe comes on Monday.",
    );
    world.write(OTHER, "---\ntitle: Plumber\n---\n\nHe comes on Tuesday.");
    world
}

#[test]
fn a_conflict_is_one_marked_note_with_both_versions_side_by_side() -> TestResult {
    let world = world();
    let mut harness = harness(app(&world, "en"));
    assert!(shows(&harness, "Home (1)"), "one Note, not two");
    click(&mut harness, "Home (1)");
    click(&mut harness, "Plumber · Conflict");
    assert!(shows(&harness, "2 versions of this Note lie side by side"));

    click(&mut harness, "Resolve…");
    assert!(shows(&harness, "Version 1") && shows(&harness, "Version 2"));
    assert!(shows(&harness, "He comes on Tuesday."));
    Ok(())
}

#[test]
fn keeping_one_version_leaves_one_file() -> TestResult {
    let world = world();
    let mut harness = harness(app(&world, "en"));
    click(&mut harness, "Home (1)");
    click(&mut harness, "Plumber · Conflict");
    click(&mut harness, "Resolve…");

    click(&mut harness, "Keep version 2");
    assert!(!world.exists(OTHER));
    assert!(
        world
            .read("Home/Plumber.md")
            .ends_with("He comes on Tuesday.")
    );
    assert!(!shows(&harness, "versions of this Note"));
    assert!(shows(&harness, "Plumber"));
    Ok(())
}

#[test]
fn a_merged_text_ends_the_conflict_in_german() -> TestResult {
    let world = world();
    let mut harness = harness(app(&world, "de"));
    click(&mut harness, "Home (1)");
    click(&mut harness, "Plumber · Konflikt");
    click(&mut harness, "Auflösen…");

    fill(&mut harness, "Zusammengeführter Text", " Or on Tuesday.");
    click(&mut harness, "Zusammengeführten Text behalten");
    assert!(!world.exists(OTHER));
    assert!(
        world
            .read("Home/Plumber.md")
            .ends_with("He comes on Monday. Or on Tuesday.")
    );
    Ok(())
}
