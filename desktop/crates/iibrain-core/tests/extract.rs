//! Images and files as Captures, and photos found by the text on them
//! (#12). The samples: `rust-book.jpg` from the examples of the ocrs
//! crate, `german-receipt.jpg` rendered in Courier with umlauts and the
//! euro sign, `curved-page.jpg` the page of RustO's issue on dropped
//! lines; all three as Old Brain Fix keeps them.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{TestResult, World};
use iibrain_core::Capture;

fn sample(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/samples")
        .join(name)
}

fn files(files: &[PathBuf]) -> Capture {
    Capture {
        via: "file".to_string(),
        files: files.to_vec(),
        ..Capture::default()
    }
}

#[test]
fn an_added_image_becomes_a_note_with_the_image_beside_it() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&files(&[sample("rust-book.jpg")]))?;

    let note = core.note(&id)?;
    assert!(note.unfiled());
    assert_eq!(note.attachments.len(), 1);
    let stem = id.strip_suffix(".md").ok_or("an id")?;
    assert!(world.exists(&format!("{stem}.files/rust-book.jpg")));
    assert_eq!(
        fs::read(world.brain().join(format!("{stem}.files/rust-book.jpg")))?,
        fs::read(sample("rust-book.jpg"))?,
        "the file is kept as it was"
    );
    Ok(())
}

#[test]
fn a_photo_is_found_by_the_text_on_it() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&files(&[sample("rust-book.jpg")]))?;
    assert!(core.search("programming")?.is_empty(), "not read yet");

    assert_eq!(core.extract_pending()?, 1);
    let note = core.note(&id)?;
    let text = note
        .attachments
        .first()
        .and_then(|a| a.extracted.clone())
        .ok_or("Extracted Text")?;
    assert!(text.contains("THE RUST\nPROGRAMMING\nLANGUAGE"), "{text}");

    let hits = core.search("programming language")?;
    assert_eq!(hits.len(), 1);
    assert_eq!(hits.first().map(|h| h.id.as_str()), Some(id.as_str()));

    let stem = id.strip_suffix(".md").ok_or("an id")?;
    assert_eq!(
        world.read(&format!("{stem}.files/rust-book.jpg.extracted.txt")),
        text,
        "the text lies in the folder, for every Device and every tool"
    );
    assert_eq!(core.extract_pending()?, 0, "nothing is read twice");
    Ok(())
}

#[test]
fn german_letters_and_the_euro_sign_are_read() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&files(&[sample("german-receipt.jpg")]))?;
    core.extract_pending()?;
    let note = core.note(&id)?;
    let text = note
        .attachments
        .first()
        .and_then(|a| a.extracted.clone())
        .ok_or("Extracted Text")?;
    assert!(text.contains("Bäckerei Müller GmbH"), "{text}");
    assert!(text.contains("Hauptstraße 7, 80331 München"), "{text}");
    assert!(
        text.lines()
            .any(|line| line.starts_with("Käsekuchen") && line.ends_with("3,90 €")),
        "{text}"
    );
    assert_eq!(core.search("käsekuchen")?.len(), 1);
    Ok(())
}

#[test]
fn every_line_of_a_curved_page_is_read() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&files(&[sample("curved-page.jpg")]))?;
    core.extract_pending()?;
    let note = core.note(&id)?;
    let text = note
        .attachments
        .first()
        .and_then(|a| a.extracted.clone())
        .ok_or("Extracted Text")?;
    assert_eq!(text.lines().count(), 12, "{text}");
    assert!(text.contains("once or twice"), "{text}");
    Ok(())
}

#[test]
fn a_picture_without_text_stays_and_says_that_none_was_found() -> TestResult {
    let world = World::new();
    let blank = world.dir.path().join("blank.png");
    image::RgbImage::from_pixel(64, 64, image::Rgb([255, 255, 255])).save(&blank)?;
    let broken = world.dir.path().join("broken.jpg");
    fs::write(&broken, "no picture at all")?;
    let mut core = world.core();
    let id = core.capture(&files(&[blank, broken]))?;
    assert_eq!(core.extract_pending()?, 2);

    let note = core.note(&id)?;
    assert_eq!(note.attachments.len(), 2, "both files stay");
    for attachment in &note.attachments {
        assert_eq!(
            attachment.extracted.as_deref(),
            Some(""),
            "{}",
            attachment.name
        );
    }
    Ok(())
}

#[test]
fn text_and_markdown_files_are_found_by_their_content() -> TestResult {
    let world = World::new();
    let list = world.dir.path().join("packing list.txt");
    fs::write(&list, "sleeping bag, headlamp, gas stove")?;
    let mut core = world.core();
    let id = core.capture(&files(&[list]))?;
    core.extract_pending()?;
    assert_eq!(
        core.search("headlamp")?.first().map(|h| h.id.clone()),
        Some(id)
    );
    Ok(())
}

#[test]
fn a_file_the_app_cannot_read_is_kept_and_found_by_its_name() -> TestResult {
    let world = World::new();
    let sheet = world.dir.path().join("budget 2026.xlsx");
    fs::write(&sheet, [0u8, 1, 2, 3])?;
    let mut core = world.core();
    let id = core.capture(&files(&[sheet]))?;
    assert_eq!(core.extract_pending()?, 0, "nothing to read");

    let note = core.note(&id)?;
    let kept = note.attachments.first().ok_or("an Attachment")?;
    assert_eq!(kept.name, "budget 2026.xlsx");
    assert_eq!(kept.extracted, None);
    assert_eq!(
        core.search("budget")?.first().map(|h| h.id.clone()),
        Some(id)
    );
    Ok(())
}

#[test]
fn two_files_of_one_name_are_both_kept() -> TestResult {
    let world = World::new();
    let a = world.dir.path().join("a/scan.txt");
    let b = world.dir.path().join("b/scan.txt");
    fs::create_dir_all(a.parent().ok_or("a parent")?)?;
    fs::create_dir_all(b.parent().ok_or("a parent")?)?;
    fs::write(&a, "first")?;
    fs::write(&b, "second")?;
    let mut core = world.core();
    let id = core.capture(&files(&[a, b]))?;
    let names: Vec<String> = core
        .note(&id)?
        .attachments
        .into_iter()
        .map(|a| a.name)
        .collect();
    assert_eq!(names, ["scan 2.txt", "scan.txt"]);
    Ok(())
}
