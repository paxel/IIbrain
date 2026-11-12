//! Images and files as Captures, with their Extracted Text (#12).

mod common;

use std::path::PathBuf;

use common::{TestResult, World, app, click, fill, harness, shows};

fn sample(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../iibrain-core/tests/samples")
        .join(name)
}

#[test]
fn an_added_photo_shows_its_extracted_text_and_is_found_by_it() -> TestResult {
    let world = World::new();
    let app = app(&world, "en").with_file_picker(Box::new(|| vec![sample("german-receipt.jpg")]));
    let mut harness = harness(app);

    click(&mut harness, "New Note");
    click(&mut harness, "Add files…");
    assert!(shows(&harness, "german-receipt.jpg"));
    click(&mut harness, "Save");

    assert!(shows(&harness, "Inbox (1)"));
    assert!(shows(&harness, "Extracted Text"));
    assert!(shows(&harness, "Bäckerei Müller GmbH"));

    fill(&mut harness, "Search", "Käsekuchen");
    assert!(shows(&harness, "1 Note found."));
    Ok(())
}

#[test]
fn a_file_without_text_says_so_in_german() -> TestResult {
    let world = World::new();
    let blank = world.dir.path().join("leer.png");
    std::fs::write(&blank, "kein Bild")?;
    let app = app(&world, "de").with_file_picker(Box::new(move || vec![blank.clone()]));
    let mut harness = harness(app);

    click(&mut harness, "Neue Notiz");
    click(&mut harness, "Dateien hinzufügen…");
    click(&mut harness, "Speichern");
    assert!(shows(&harness, "leer.png"));
    assert!(shows(&harness, "Darin wurde kein Text gefunden."));
    Ok(())
}
