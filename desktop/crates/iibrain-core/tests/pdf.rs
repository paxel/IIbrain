//! PDFs made searchable (#13). The sample PDFs are made here: pages
//! with a text layer, and pages that are nothing but the photo of a
//! page, as a scanner writes them.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::{TestResult, World};
use iibrain_core::Capture;
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream, dictionary};

fn sample(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/samples")
        .join(name)
}

/// What a page of a sample PDF holds.
enum Page<'a> {
    Text(&'a str),
    /// A JPEG covering the page.
    Scan(&'a Path),
}

fn write_pdf(path: &Path, pages: &[Page]) -> TestResult {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Courier",
        "Encoding" => "WinAnsiEncoding",
    });
    let mut kids = Vec::new();
    for page in pages {
        let (content, resources) = match page {
            Page::Text(text) => (
                Content {
                    operations: vec![
                        Operation::new("BT", vec![]),
                        Operation::new("Tf", vec!["F1".into(), 12.into()]),
                        Operation::new("Td", vec![72.into(), 720.into()]),
                        Operation::new("Tj", vec![Object::string_literal(*text)]),
                        Operation::new("ET", vec![]),
                    ],
                },
                dictionary! { "Font" => dictionary! { "F1" => font } },
            ),
            Page::Scan(photo) => {
                let bytes = fs::read(photo)?;
                let (width, height) = image::load_from_memory(&bytes)?.into_rgb8().dimensions();
                let picture = doc.add_object(Stream::new(
                    dictionary! {
                        "Type" => "XObject", "Subtype" => "Image",
                        "Width" => i64::from(width), "Height" => i64::from(height),
                        "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8,
                        "Filter" => "DCTDecode",
                    },
                    bytes,
                ));
                (
                    Content {
                        operations: vec![
                            Operation::new("q", vec![]),
                            Operation::new(
                                "cm",
                                vec![
                                    595.into(),
                                    0.into(),
                                    0.into(),
                                    842.into(),
                                    0.into(),
                                    0.into(),
                                ],
                            ),
                            Operation::new("Do", vec!["Im1".into()]),
                            Operation::new("Q", vec![]),
                        ],
                    },
                    dictionary! { "XObject" => dictionary! { "Im1" => picture } },
                )
            }
        };
        let content = doc.add_object(Stream::new(dictionary! {}, content.encode()?));
        kids.push(doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content,
            "Resources" => resources,
        }));
    }
    let count = i64::try_from(kids.len())?;
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => kids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
            "Count" => count,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    doc.save(path)?;
    Ok(())
}

fn capture_pdf(
    world: &World,
    name: &str,
    pages: &[Page],
) -> Result<(iibrain_core::Core, String), Box<dyn std::error::Error>> {
    let pdf = world.dir.path().join(name);
    write_pdf(&pdf, pages)?;
    let mut core = world.core();
    let id = core.capture(&Capture {
        via: "file".to_string(),
        files: vec![pdf],
        ..Capture::default()
    })?;
    core.extract_pending()?;
    Ok((core, id))
}

fn found(
    core: &iibrain_core::Core,
    words: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    Ok(core.search(words)?.into_iter().map(|h| h.id).collect())
}

#[test]
fn a_pdf_with_a_text_layer_is_found_by_words_from_any_page() -> TestResult {
    let world = World::new();
    let (core, id) = capture_pdf(
        &world,
        "contract.pdf",
        &[
            Page::Text("Rental agreement for the flat in Lindenstrasse"),
            Page::Text("The deposit amounts to three monthly rents"),
        ],
    )?;
    assert_eq!(found(&core, "rental agreement")?, std::slice::from_ref(&id));
    assert_eq!(found(&core, "deposit monthly")?, std::slice::from_ref(&id));
    Ok(())
}

#[test]
fn a_scanned_pdf_is_found_by_the_words_on_its_pages() -> TestResult {
    let world = World::new();
    let receipt = sample("german-receipt.jpg");
    let book = sample("rust-book.jpg");
    let (core, id) = capture_pdf(
        &world,
        "scan.pdf",
        &[Page::Scan(&receipt), Page::Scan(&book)],
    )?;
    assert_eq!(found(&core, "Käsekuchen")?, std::slice::from_ref(&id));
    assert_eq!(
        found(&core, "programming language")?,
        std::slice::from_ref(&id)
    );
    Ok(())
}

#[test]
fn a_pdf_mixing_both_kinds_of_pages_is_found_by_either() -> TestResult {
    let world = World::new();
    let receipt = sample("german-receipt.jpg");
    let (core, id) = capture_pdf(
        &world,
        "mixed.pdf",
        &[
            Page::Text("Expenses of the bakery visit in October"),
            Page::Scan(&receipt),
        ],
    )?;
    assert_eq!(found(&core, "expenses bakery")?, std::slice::from_ref(&id));
    assert_eq!(found(&core, "Bäckerei")?, std::slice::from_ref(&id));
    let text = core
        .note(&id)?
        .attachments
        .first()
        .and_then(|a| a.extracted.clone())
        .ok_or("Extracted Text")?;
    let first = text.find("Expenses").ok_or("the first page's text")?;
    let second = text.find("Bäckerei").ok_or("the second page's text")?;
    assert!(first < second, "the pages keep their order: {text}");
    Ok(())
}

#[test]
fn a_pdf_that_cannot_be_read_is_kept_and_says_that_no_text_was_found() -> TestResult {
    let world = World::new();
    let broken = world.dir.path().join("broken.pdf");
    fs::write(&broken, "%PDF-1.5 and then nothing a reader understands")?;
    let mut core = world.core();
    let id = core.capture(&Capture {
        via: "file".to_string(),
        files: vec![broken],
        ..Capture::default()
    })?;
    assert_eq!(core.extract_pending()?, 1);
    let note = core.note(&id)?;
    let kept = note.attachments.first().ok_or("an Attachment")?;
    assert_eq!(kept.name, "broken.pdf");
    assert_eq!(kept.extracted.as_deref(), Some(""));
    Ok(())
}
