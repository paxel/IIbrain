//! Extracted Text: what the app itself reads out of an Attachment, on
//! the Device (ADR 0007). The text lies in the Brain's folder beside
//! the Attachment, so every Device and every other tool has it.

use std::collections::HashMap;
use std::path::Path;

use ndarray::ArrayD;
use rten_tensor::prelude::*;
use rusto::engine::{EngineError, InferenceSession};

use crate::brain::Brain;
use crate::error::Result;

/// What an Attachment is, by the ending of its name, as far as reading
/// its text goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A photo, a screenshot or a scanned page.
    Picture,
    /// A PDF: its text layer, and where a page has none, the text on
    /// the page's images.
    Pdf,
    /// Plain text or Markdown; its text is its content.
    Text,
    /// Anything else: kept, and found by its name and its Note only.
    Other,
}

pub fn kind_of(name: &str) -> Kind {
    let ending = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase())
        .unwrap_or_default();
    match ending.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" => Kind::Picture,
        "pdf" => Kind::Pdf,
        "txt" | "md" | "markdown" | "text" | "csv" => Kind::Text,
        _ => Kind::Other,
    }
}

/// The engines that read, each loaded when first needed.
#[derive(Default)]
pub struct Engines {
    pictures: Option<rusto::RustO>,
}

impl Engines {
    /// The text of the file at `path`: `None` for a kind the app does
    /// not read, empty when it read and found no text.
    pub fn read(&mut self, path: &Path) -> Option<String> {
        let name = path.file_name()?.to_string_lossy().to_string();
        match kind_of(&name) {
            Kind::Picture => Some(self.read_picture(path).unwrap_or_default()),
            Kind::Pdf => Some(self.read_pdf(path).unwrap_or_default()),
            Kind::Text => Some(
                std::fs::read(path)
                    .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                    .unwrap_or_default(),
            ),
            Kind::Other => None,
        }
    }

    /// The text on the picture in the file at `path`.
    fn read_picture(&mut self, path: &Path) -> Option<String> {
        self.read_picture_bytes(&std::fs::read(path).ok()?)
    }

    /// The text of the PDF in the file at `path`, page after page: a
    /// page's text layer, or for a page with next to none, as a scanned
    /// page has, the text on its images. A page image the app cannot
    /// decode (JBIG2, CCITT fax, JPEG 2000) is left out.
    fn read_pdf(&mut self, path: &Path) -> Option<String> {
        let document = lopdf::Document::load(path).ok()?;
        let mut pages = Vec::new();
        for (number, page) in document.get_pages() {
            let layer = document.extract_text(&[number]).unwrap_or_default();
            let layer = join_lines(layer.lines().map(String::from));
            if layer.chars().filter(|c| c.is_alphanumeric()).count() >= SCANNED_BELOW {
                pages.push(layer);
                continue;
            }
            let mut read = vec![layer];
            for picture in document.get_page_images(page).unwrap_or_default() {
                if let Some(pixels) = page_picture(&document, &picture) {
                    read.push(self.read_pixels(pixels).unwrap_or_default());
                }
            }
            pages.push(join_lines(read));
        }
        Some(join_lines(pages))
    }

    /// The text on a picture, as lines laid out as on the picture.
    pub fn read_picture_bytes(&mut self, bytes: &[u8]) -> Option<String> {
        self.read_pixels(image::load_from_memory(bytes).ok()?.into_rgb8())
    }

    fn read_pixels(&mut self, pixels: image::RgbImage) -> Option<String> {
        if self.pictures.is_none() {
            self.pictures = picture_engine();
        }
        let engine = self.pictures.as_mut()?;
        let (width, height) = pixels.dimensions();
        let stride = usize::try_from(width).ok()?.checked_mul(3)?;
        let result = engine
            .detect_text_rgb(pixels.as_raw(), width, height, stride, &picture_options())
            .ok()?;
        Some(match result {
            rusto::DetectTextResult::Spatial(text) => join_lines(text.lines().map(String::from)),
            rusto::DetectTextResult::Structured(lines) => {
                join_lines(lines.into_iter().map(|line| line.text))
            }
        })
    }
}

/// A page with fewer letters and digits in its text layer than this is
/// taken for a scanned one.
const SCANNED_BELOW: usize = 16;

/// A picture on a PDF page as pixels: a JPEG as it is, or 8-bit samples
/// in grey or RGB.
fn page_picture(
    document: &lopdf::Document,
    picture: &lopdf::xobject::PdfImage,
) -> Option<image::RgbImage> {
    let filters = picture.filters.clone().unwrap_or_default();
    if filters.iter().any(|f| f == "DCTDecode") {
        return Some(image::load_from_memory(picture.content).ok()?.into_rgb8());
    }
    if filters.iter().any(|f| f != "FlateDecode") || picture.bits_per_component != Some(8) {
        return None;
    }
    let samples = document
        .get_object(picture.id)
        .ok()?
        .as_stream()
        .ok()?
        .decompressed_content()
        .ok()?;
    let width = u32::try_from(picture.width).ok()?;
    let height = u32::try_from(picture.height).ok()?;
    match picture.color_space.as_deref() {
        Some("DeviceRGB") => image::RgbImage::from_raw(width, height, samples),
        Some("DeviceGray") => image::GrayImage::from_raw(width, height, samples)
            .map(|grey| image::DynamicImage::ImageLuma8(grey).into_rgb8()),
        _ => None,
    }
}

/// Reads the Extracted Text of one Attachment that still waits for it
/// and keeps it in the Brain's folder; says whether there was one.
/// It needs nothing but the folder, so it can run beside the app.
pub fn extract_one(brain: &Brain, engines: &mut Engines) -> Result<bool> {
    let Some((id, name)) = brain.waiting_for_text()?.into_iter().next() else {
        return Ok(false);
    };
    let path = brain.attachment_path(&id, &name)?;
    let text = engines.read(&path).unwrap_or_default();
    brain.set_extracted(&id, &name, &text)?;
    Ok(true)
}

/// The PP-OCRv6 Tiny text-detection and text-recognition models and
/// their character list (Apache-2.0, see `models/rusto/NOTICE`), built
/// in, so the phone and the desktop read with the same files.
static DETECTION: &[u8] = include_bytes!("../models/rusto/det.rten");
static RECOGNITION: &[u8] = include_bytes!("../models/rusto/rec.rten");
static CHARACTERS: &str = include_str!("../models/rusto/dict.txt");

fn picture_engine() -> Option<rusto::RustO> {
    let detection = ModelSession::new(DETECTION.to_vec(), None)?;
    let characters = CHARACTERS.lines().map(String::from).collect();
    let recognition = ModelSession::new(RECOGNITION.to_vec(), Some(characters))?;
    // The PP-OCRv6 settings; the paths are not read, the sessions hold
    // the models.
    let config = rusto::InitializeConfig::ppv6("det.rten", "rec.rten", "dict.txt");
    rusto::RustO::with_custom_engines(config, detection, recognition).ok()
}

/// How RustO reads: the text laid out as on the picture, and a box kept
/// from a probability of 0.3 instead of 0.6, so the lines of a curved
/// page are not dropped. The threshold is the top-level `postprocess`;
/// RustO 0.3.2 ignores `detection.postprocess`
/// (https://github.com/byrizki/rusto-rs/issues/21).
pub fn picture_options() -> rusto::OcrRunOptions {
    rusto::OcrRunOptions {
        output: rusto::OutputGranularity::Spatial,
        postprocess: Some(rusto::PostprocessRunOptions {
            box_threshold: Some(0.3),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// One model run as RustO runs its own models from a file, but loaded
/// from the bytes built in: the first input in, the first output out.
/// The recognition model also holds its characters.
struct ModelSession {
    model: rten::Model,
    characters: Option<Vec<String>>,
}

impl ModelSession {
    fn new(bytes: Vec<u8>, characters: Option<Vec<String>>) -> Option<ModelSession> {
        let model = rten::Model::load(bytes).ok()?;
        Some(ModelSession { model, characters })
    }
}

impl InferenceSession for ModelSession {
    fn run(&mut self, input: ArrayD<f32>) -> std::result::Result<ArrayD<f32>, EngineError> {
        let shape = input.shape().to_vec();
        let tensor = rten_tensor::Tensor::<f32>::from_data(
            &shape,
            input.iter().copied().collect::<Vec<_>>(),
        );
        let input_id = *self
            .model
            .input_ids()
            .first()
            .ok_or_else(|| EngineError::OutputError("Model has no inputs".into()))?;
        let outputs = self.model.run(
            vec![(input_id, tensor.view().into())],
            self.model.output_ids(),
            None,
        )?;
        match outputs.first() {
            Some(rten::Value::FloatTensor(output)) => Ok(ArrayD::from_shape_vec(
                output.shape().to_vec(),
                output.iter().copied().collect(),
            )?),
            Some(_) => Err(EngineError::OutputError(
                "Unsupported output tensor type".into(),
            )),
            None => Err(EngineError::OutputError("Model produced no outputs".into())),
        }
    }

    fn run_all(
        &mut self,
        _input: ArrayD<f32>,
    ) -> std::result::Result<HashMap<String, ArrayD<f32>>, EngineError> {
        Err(EngineError::Engine("not used for text".into()))
    }

    fn run_with_inputs(
        &mut self,
        _inputs: HashMap<String, ArrayD<f32>>,
    ) -> std::result::Result<HashMap<String, ArrayD<f32>>, EngineError> {
        Err(EngineError::Engine("not used for text".into()))
    }

    fn get_character_list(&self, key: &str) -> Option<Vec<String>> {
        (key == "character")
            .then(|| self.characters.clone())
            .flatten()
    }

    fn have_key(&self, key: &str) -> bool {
        key == "character" && self.characters.is_some()
    }
}

/// The recognised `lines` as one text: each trimmed, blank ones left
/// out, one per line.
pub fn join_lines(lines: impl IntoIterator<Item = String>) -> String {
    lines
        .into_iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
