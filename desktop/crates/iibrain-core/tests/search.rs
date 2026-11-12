//! Searching the Brain by words (#11).

mod common;

use std::fs;

use common::{TestResult, World};

fn ids(hits: &[iibrain_core::Hit]) -> Vec<&str> {
    let mut ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    ids.sort();
    ids
}

fn world() -> World {
    let world = World::new();
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\ntags: [house]\n---\n\nThe boiler in the cellar leaks since the cold week in January, call before Monday.\n",
    );
    world.write(
        "Home/Garden.md",
        "# Garden plan\n\nTomatoes on the south side, the boiler room window stays shut.\n",
    );
    world.write(
        "Money/Tax.md",
        "---\ntitle: Tax return\ntags: [money]\n---\n\nSend it before May.\n",
    );
    world
}

#[test]
fn a_search_finds_the_notes_that_hold_every_word() -> TestResult {
    let world = world();
    let core = world.core();
    assert_eq!(
        ids(&core.search("boiler")?),
        ["Home/Garden.md", "Home/Plumber.md"]
    );
    assert_eq!(ids(&core.search("boiler cellar")?), ["Home/Plumber.md"]);
    assert_eq!(
        ids(&core.search("BOILER")?),
        ["Home/Garden.md", "Home/Plumber.md"]
    );
    assert!(core.search("submarine")?.is_empty());
    assert!(core.search("   ")?.is_empty());
    Ok(())
}

#[test]
fn titles_and_tags_are_searched_too() -> TestResult {
    let world = world();
    let core = world.core();
    assert_eq!(ids(&core.search("plumber")?), ["Home/Plumber.md"]);
    assert_eq!(ids(&core.search("money")?), ["Money/Tax.md"]);
    assert_eq!(ids(&core.search("garden")?), ["Home/Garden.md"]);
    Ok(())
}

#[test]
fn the_start_of_a_word_is_enough() -> TestResult {
    let world = world();
    let core = world.core();
    assert_eq!(ids(&core.search("tomat")?), ["Home/Garden.md"]);
    Ok(())
}

#[test]
fn a_hit_shows_the_text_around_the_found_words() -> TestResult {
    let world = world();
    let core = world.core();
    let hits = core.search("cellar")?;
    let hit = hits.first().ok_or("a hit")?;
    assert_eq!(hit.title, "Plumber");
    assert_eq!(hit.folder, "Home");
    assert!(
        hit.excerpt.contains("boiler in the cellar leaks"),
        "{}",
        hit.excerpt
    );
    Ok(())
}

#[test]
fn signs_in_the_query_do_no_harm() -> TestResult {
    let world = world();
    let core = world.core();
    assert_eq!(
        ids(&core.search("\"boiler\" + (cellar*")?),
        ["Home/Plumber.md"]
    );
    assert!(core.search("-- ;").is_ok());
    Ok(())
}

#[test]
fn what_another_tool_changed_is_found_by_its_new_content() -> TestResult {
    let world = world();
    let mut core = world.core();
    world.write("Home/Garden.md", "# Garden plan\n\nCucumbers instead.\n");
    world.write("Home/New.md", "A greenhouse would be nice.\n");
    core.refresh()?;
    assert_eq!(ids(&core.search("cucumbers")?), ["Home/Garden.md"]);
    assert_eq!(ids(&core.search("greenhouse")?), ["Home/New.md"]);
    assert_eq!(ids(&core.search("boiler")?), ["Home/Plumber.md"]);
    Ok(())
}

#[test]
fn search_finds_the_same_after_the_index_was_rebuilt() -> TestResult {
    let world = world();
    let before = world.core().search("boiler")?;
    fs::remove_dir_all(world.device())?;
    assert_eq!(world.core().search("boiler")?, before);
    Ok(())
}
