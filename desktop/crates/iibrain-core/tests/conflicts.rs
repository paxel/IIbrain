//! Recognising and resolving Conflicts (#21): two versions of one Note,
//! left side by side by a Sync Tool. Recognised are the names Syncthing
//! gives (`name.sync-conflict-<date>-<time>-<device>.md`) and those of
//! Nextcloud, ownCloud and Dropbox (`name (conflicted copy <date>).md`,
//! `name (Ann's conflicted copy <date>).md`).

mod common;

use common::{TestResult, World};
use iibrain_core::brain::conflict_base;

const SYNCTHING: &str = "Home/Plumber.sync-conflict-20261112-153000-ABCDEFG.md";

fn world() -> World {
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\n---\n\nHe comes on Monday.\n",
    );
    world.write(
        SYNCTHING,
        "---\ntitle: Plumber\n---\n\nHe comes on Tuesday.\n",
    );
    world.write("Home/Other.md", "nothing to do with it\n");
    world
}

#[test]
fn a_pair_left_by_the_sync_tool_is_one_note_in_conflict() -> TestResult {
    let world = world();
    let core = world.core();
    assert_eq!(core.notes("Home")?.len(), 2, "not three Notes");
    let note = core.note("Home/Plumber.md")?;
    assert_eq!(note.conflicts, [SYNCTHING]);
    assert!(
        core.note(SYNCTHING).is_err(),
        "the other version is no Note of its own"
    );
    assert!(core.note("Home/Other.md")?.conflicts.is_empty());
    Ok(())
}

#[test]
fn the_names_of_several_sync_tools_are_recognised() {
    for (file, base) in [
        (
            "Home/Plumber.sync-conflict-20261112-153000-ABCDEFG.md",
            Some("Home/Plumber.md"),
        ),
        (
            "Plumber (conflicted copy 2026-11-12 153000).md",
            Some("Plumber.md"),
        ),
        (
            "Home/Plumber (Ann's conflicted copy 2026-11-12).md",
            Some("Home/Plumber.md"),
        ),
        (
            "A/B/Plumber (Conflicted Copy 2026-11-12).md",
            Some("A/B/Plumber.md"),
        ),
        ("Home/Plumber (draft).md", None),
        ("Home/Plumber 2.md", None),
        ("Home/Plumber.md", None),
        ("Home/.sync-conflict-20261112-153000-ABCDEFG.md", None),
    ] {
        assert_eq!(conflict_base(file).as_deref(), base, "{file}");
    }
}

#[test]
fn a_version_without_its_note_is_a_note_of_its_own() -> TestResult {
    let world = World::new();
    world.write(SYNCTHING, "He comes on Tuesday.\n");
    let core = world.core();
    assert_eq!(core.note(SYNCTHING)?.conflicts.len(), 0);
    Ok(())
}

#[test]
fn both_versions_can_be_read_side_by_side() -> TestResult {
    let world = world();
    let core = world.core();
    let versions = core.versions("Home/Plumber.md")?;
    let texts: Vec<&str> = versions.iter().map(|v| v.text.as_str()).collect();
    assert_eq!(texts, ["He comes on Monday.\n", "He comes on Tuesday.\n"]);
    Ok(())
}

#[test]
fn keeping_the_other_version_leaves_one_file_with_its_text() -> TestResult {
    let world = world();
    let mut core = world.core();
    core.keep_version("Home/Plumber.md", SYNCTHING)?;

    assert_eq!(world.files(), ["Home/Other.md", "Home/Plumber.md"]);
    let note = core.note("Home/Plumber.md")?;
    assert_eq!(note.text, "He comes on Tuesday.\n");
    assert!(note.conflicts.is_empty());
    Ok(())
}

#[test]
fn keeping_the_notes_own_version_removes_the_other() -> TestResult {
    let world = world();
    let mut core = world.core();
    core.keep_version("Home/Plumber.md", "Home/Plumber.md")?;
    assert_eq!(world.files(), ["Home/Other.md", "Home/Plumber.md"]);
    assert_eq!(core.note("Home/Plumber.md")?.text, "He comes on Monday.\n");
    assert!(
        core.keep_version("Home/Plumber.md", "Home/Other.md")
            .is_err(),
        "no version of it"
    );
    Ok(())
}

#[test]
fn a_merged_text_ends_the_conflict() -> TestResult {
    let world = world();
    let mut core = world.core();
    core.resolve("Home/Plumber.md", "He comes on Monday or Tuesday.\n")?;

    assert_eq!(world.files(), ["Home/Other.md", "Home/Plumber.md"]);
    let note = core.note("Home/Plumber.md")?;
    assert_eq!(note.text, "He comes on Monday or Tuesday.\n");
    assert_eq!(note.title, "Plumber", "its frontmatter stays");
    assert!(note.conflicts.is_empty());
    Ok(())
}

#[test]
fn search_finds_a_note_in_conflict_once() -> TestResult {
    let world = world();
    let core = world.core();
    assert_eq!(core.search("comes")?.len(), 1);
    assert!(
        core.search("Tuesday")?.is_empty(),
        "the other version is not searched until kept"
    );
    Ok(())
}

#[test]
fn a_conflict_that_arrives_later_is_taken_in() -> TestResult {
    let world = World::new();
    world.write("Home/Plumber.md", "He comes on Monday.\n");
    let mut core = world.core();
    assert!(core.note("Home/Plumber.md")?.conflicts.is_empty());

    world.write(
        "Home/Plumber (conflicted copy 2026-11-12 153000).md",
        "Tuesday.\n",
    );
    core.refresh()?;
    assert_eq!(core.note("Home/Plumber.md")?.conflicts.len(), 1);
    assert_eq!(core.notes("Home")?.len(), 1);
    Ok(())
}

#[test]
fn deleting_a_note_in_conflict_removes_every_version() -> TestResult {
    let world = world();
    let mut core = world.core();
    core.delete("Home/Plumber.md")?;
    assert_eq!(world.files(), ["Home/Other.md"]);
    Ok(())
}
