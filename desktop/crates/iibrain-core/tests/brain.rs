//! Opening a folder as the Brain and reading its Notes (#9).

mod common;

use std::fs;

use common::{TestResult, World};
use iibrain_core::{Core, INBOX};

const WITH_FRONT: &str = "---\ntitle: Tax return 2025\ntags: [tax, money]\naliases:\n  - taxes\n---\n\nSend it before May.\n";
const WITHOUT_FRONT: &str = "# Garden plan\n\nTomatoes on the south side.\n";

#[test]
fn a_folder_of_markdown_files_opens_as_it_is() -> TestResult {
    let world = World::new();
    world.write("Money/tax.md", WITH_FRONT);
    world.write("garden.md", WITHOUT_FRONT);
    world.write("Money/plain.md", "no heading here\n");
    let before = world.files();

    let core = world.core();
    let tax = core.note("Money/tax.md")?;
    assert_eq!(tax.title, "Tax return 2025");
    assert_eq!(tax.tags, ["tax", "money"]);
    assert_eq!(tax.folder, "Money");
    assert_eq!(tax.text, "Send it before May.\n");

    let garden = core.note("garden.md")?;
    assert_eq!(garden.title, "Garden plan");
    assert_eq!(garden.folder, "");
    assert_eq!(garden.text, WITHOUT_FRONT);

    assert_eq!(core.note("Money/plain.md")?.title, "plain");
    assert_eq!(core.all_notes()?.len(), 3);

    // Opening and reading changed nothing.
    assert_eq!(world.files(), before);
    assert_eq!(world.read("Money/tax.md"), WITH_FRONT);
    assert_eq!(world.read("garden.md"), WITHOUT_FRONT);
    Ok(())
}

#[test]
fn the_folders_are_listed_with_the_inbox_first() -> TestResult {
    let world = World::new();
    world.write("Money/tax.md", WITH_FRONT);
    world.write("Money/2025/receipt.md", "x");
    world.write("Albums/a.md", "x");
    world.write(".obsidian/config.md", "not a Note");

    let core = world.core();
    let names: Vec<String> = core.folders()?.into_iter().map(|f| f.name).collect();
    assert_eq!(names, [INBOX, "Albums", "Money", "Money/2025"]);
    assert_eq!(core.notes("Money")?.len(), 1);
    assert_eq!(core.notes("Money/2025")?.len(), 1);
    assert_eq!(
        core.all_notes()?.len(),
        3,
        "hidden directories hold no Notes"
    );
    Ok(())
}

#[test]
fn attachments_lie_beside_the_note_and_are_listed() -> TestResult {
    let world = World::new();
    world.write("Money/tax.md", WITH_FRONT);
    world.write("Money/tax.files/scan.pdf", "%PDF");
    world.write("Money/tax.files/scan.pdf.extracted.txt", "total 120 EUR");

    let core = world.core();
    let note = core.note("Money/tax.md")?;
    assert_eq!(note.attachments.len(), 1);
    let scan = note.attachments.first().ok_or("an Attachment")?;
    assert_eq!(scan.name, "scan.pdf");
    assert_eq!(scan.extracted.as_deref(), Some("total 120 EUR"));
    assert!(
        core.folders()?.iter().all(|f| !f.name.contains(".files")),
        "the directory of Attachments is no Folder"
    );
    Ok(())
}

#[test]
fn what_another_tool_changes_is_taken_in() -> TestResult {
    let world = World::new();
    world.write("a.md", "first\n");
    let mut core = world.core();
    assert!(!core.refresh()?, "nothing changed yet");

    world.write("b.md", "# Second\n");
    world.write("a.md", "first, now longer\n");
    assert!(core.refresh()?);
    assert_eq!(core.note("b.md")?.title, "Second");
    assert_eq!(core.note("a.md")?.text, "first, now longer\n");

    fs::remove_file(world.brain().join("a.md"))?;
    assert!(core.refresh()?);
    assert!(core.note("a.md").is_err());
    assert_eq!(core.all_notes()?.len(), 1);
    Ok(())
}

#[test]
fn the_index_is_rebuilt_from_the_folder_alone() -> TestResult {
    let world = World::new();
    world.write("Money/tax.md", WITH_FRONT);
    world.write("garden.md", WITHOUT_FRONT);
    let before = world.core().all_notes()?;

    fs::remove_dir_all(world.device())?;
    let after = world.core().all_notes()?;
    assert_eq!(before, after);
    Ok(())
}

#[test]
fn the_device_remembers_its_brain() -> TestResult {
    let world = World::new();
    world.write("a.md", "x");
    drop(world.core());

    let device = iibrain_core::Device::load(&world.device())?;
    assert_eq!(device.settings.brain, Some(world.brain()));
    assert_eq!(Core::open(device)?.all_notes()?.len(), 1);
    Ok(())
}

#[test]
fn a_device_without_a_brain_says_so() -> TestResult {
    let world = World::new();
    let device = iibrain_core::Device::load(&world.device())?;
    assert!(Core::open(device).is_err());
    Ok(())
}
