//! Capturing typed Notes into the Inbox and filing them by hand (#10).

mod common;

use common::{TestResult, World};
use iibrain_core::{Capture, INBOX};

fn typed(text: &str) -> Capture {
    Capture {
        text: text.to_string(),
        via: "typed".to_string(),
        ..Capture::default()
    }
}

fn tags(tags: &[&str]) -> Vec<String> {
    tags.iter().map(|t| t.to_string()).collect()
}

#[test]
fn a_typed_capture_becomes_one_markdown_file_in_the_inbox() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&typed("Call the plumber on Monday."))?;

    assert!(id.starts_with("Inbox/") && id.ends_with(".md"));
    assert_eq!(world.files(), std::slice::from_ref(&id));
    let file = world.read(&id);
    assert!(file.starts_with("---\n"), "it carries frontmatter: {file}");
    assert!(file.contains("via: typed"));
    assert!(file.contains("captured: "));
    assert!(file.ends_with("Call the plumber on Monday."));

    let note = core.note(&id)?;
    assert!(note.unfiled());
    assert_eq!(note.via.as_deref(), Some("typed"));
    assert!(note.captured.is_some());
    assert_eq!(note.text, "Call the plumber on Monday.");
    assert_eq!(core.unfiled()?, 1);
    Ok(())
}

#[test]
fn two_captures_in_the_same_second_are_two_notes() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let a = core.capture(&typed("one"))?;
    let b = core.capture(&typed("two"))?;
    assert_ne!(a, b);
    assert_eq!(core.unfiled()?, 2);
    Ok(())
}

#[test]
fn filing_by_hand_gives_title_tags_and_folder() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&typed("Call the plumber on Monday."))?;
    core.create_folder("Home")?;

    let filed = core.file(&id, "Plumber", &tags(&["house", "#todo"]), "Home")?;
    assert_eq!(filed, "Home/Plumber.md");
    assert!(!world.exists(&id), "it left the Inbox");
    assert_eq!(core.unfiled()?, 0);

    let note = core.note(&filed)?;
    assert_eq!(note.title, "Plumber");
    assert_eq!(note.tags, ["house", "todo"]);
    assert_eq!(note.folder, "Home");
    assert_eq!(note.text, "Call the plumber on Monday.");
    let file = world.read(&filed);
    assert!(file.contains("title: Plumber"));
    assert!(file.contains("- house"));
    Ok(())
}

#[test]
fn a_filed_note_can_be_moved_retitled_retagged_and_edited() -> TestResult {
    let world = World::new();
    world.write("Home/Plumber.files/card.jpg", "jpg");
    world.write(
        "Home/Plumber.md",
        "---\ntitle: Plumber\naliases: [pipes]\n---\n\nold\n",
    );
    let mut core = world.core();
    core.create_folder("Archive")?;

    let moved = core.file(
        "Home/Plumber.md",
        "Plumber 2026",
        &tags(&["done"]),
        "Archive",
    )?;
    assert_eq!(moved, "Archive/Plumber 2026.md");
    assert!(
        world.exists("Archive/Plumber 2026.files/card.jpg"),
        "its Attachments moved with it"
    );
    assert!(!world.exists("Home/Plumber.md"));
    assert!(!world.exists("Home/Plumber.files"));

    core.set_text(&moved, "new text\n")?;
    let note = core.note(&moved)?;
    assert_eq!(note.text, "new text\n");
    assert_eq!(note.tags, ["done"]);
    assert_eq!(note.attachments.len(), 1);
    assert!(
        world.read(&moved).contains("aliases"),
        "what another tool wrote into the frontmatter stays"
    );
    Ok(())
}

#[test]
fn a_title_that_is_taken_gets_a_number() -> TestResult {
    let world = World::new();
    world.write("Home/Plumber.md", "first");
    let mut core = world.core();
    let id = core.capture(&typed("second"))?;
    assert_eq!(core.file(&id, "Plumber", &[], "Home")?, "Home/Plumber 2.md");
    assert_eq!(core.note("Home/Plumber.md")?.text, "first");
    Ok(())
}

#[test]
fn a_title_with_signs_no_file_name_takes_still_files() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&typed("x"))?;
    let filed = core.file(&id, "What? A/B: \"test\"", &[], "")?;
    assert_eq!(core.note(&filed)?.title, "What? A/B: \"test\"");
    assert!(!filed.contains(['?', ':', '"']));
    assert!(
        core.file(&filed, "  ", &[], "").is_err(),
        "a Note needs a title"
    );
    Ok(())
}

#[test]
fn deleting_removes_the_note_and_its_attachments() -> TestResult {
    let world = World::new();
    world.write("Home/Plumber.md", "x");
    world.write("Home/Plumber.files/card.jpg", "jpg");
    world.write("Home/Other.md", "y");
    let mut core = world.core();

    core.delete("Home/Plumber.md")?;
    assert_eq!(world.files(), ["Home/Other.md"]);
    assert!(core.note("Home/Plumber.md").is_err());
    Ok(())
}

#[test]
fn folders_are_made_renamed_and_removed_but_the_inbox_stays() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    core.create_folder("Home")?;
    assert!(core.create_folder("Home").is_err(), "the name is taken");
    let id = core.capture(&typed("x"))?;
    let filed = core.file(&id, "Plumber", &[], "Home")?;

    assert!(core.remove_folder("Home").is_err(), "it still holds a Note");
    core.rename_folder("Home", "House")?;
    assert_eq!(core.note("House/Plumber.md")?.title, "Plumber");
    assert!(core.note(&filed).is_err());

    core.delete("House/Plumber.md")?;
    core.remove_folder("House")?;
    let names: Vec<String> = core.folders()?.into_iter().map(|f| f.name).collect();
    assert_eq!(names, [INBOX]);

    assert!(core.rename_folder(INBOX, "Stuff").is_err());
    assert!(core.remove_folder(INBOX).is_err());
    Ok(())
}

#[test]
fn nothing_reaches_outside_the_brain() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&typed("x"))?;
    assert!(core.file(&id, "t", &[], "../elsewhere").is_err());
    assert!(core.create_folder("../elsewhere").is_err());
    assert!(core.note("../device/device.json").is_err());
    Ok(())
}
