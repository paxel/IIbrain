//! The page behind a link: fetched by the Device itself and reduced to
//! its readable text (ADR 0007). No archive service and no other third
//! party is asked; the only hosts contacted are the one the link names
//! and those it redirects to.

use std::time::Duration;

use scraper::{ElementRef, Html, Node, Selector};

/// A page larger than this is cut off.
const MAX_BYTES: u64 = 5 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(20);

/// What is no text of the page: scripts, styles, and what frames it.
const SKIPPED: [&str; 15] = [
    "script", "style", "noscript", "template", "nav", "header", "footer", "aside", "form",
    "button", "svg", "iframe", "select", "dialog", "figure",
];

/// What starts a line of its own.
const BLOCKS: [&str; 22] = [
    "p",
    "div",
    "br",
    "li",
    "ul",
    "ol",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "tr",
    "table",
    "section",
    "article",
    "blockquote",
    "pre",
    "dd",
    "dt",
    "hr",
    "main",
];

/// Whether `text` is nothing but one web link.
pub fn is_link(text: &str) -> bool {
    let text = text.trim();
    (text.starts_with("http://") || text.starts_with("https://"))
        && !text.contains(char::is_whitespace)
        && text.len() > "https://".len()
}

/// The readable text of the page at `link`, its title first; `None`
/// when the page cannot be fetched or is no text.
pub fn page_text(link: &str) -> Option<String> {
    if !is_link(link) {
        return None;
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .user_agent("IIbrain")
        .build()
        .into();
    let mut response = agent.get(link.trim()).call().ok()?;
    let kind = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html")
        .to_lowercase();
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .lossy_utf8(true)
        .read_to_string()
        .ok()?;
    let text = if kind.contains("html") || kind.contains("xml") {
        readable(&body)
    } else if kind.starts_with("text/") {
        body.trim().to_string()
    } else {
        return None;
    };
    (!text.is_empty()).then_some(text)
}

/// The readable text of an HTML page: its title, then the text of its
/// article or main part, without navigation, ads and scripts.
pub fn readable(html: &str) -> String {
    let page = Html::parse_document(html);
    let first = |selector: &str| {
        Selector::parse(selector)
            .ok()
            .and_then(|s| page.select(&s).next())
    };
    let title = first("title")
        .map(|t| squeeze(&t.text().collect::<String>()))
        .unwrap_or_default();
    let mut text = String::new();
    let root = ["article", "main", "[role=main]", "body"]
        .into_iter()
        .find_map(first);
    if let Some(root) = root {
        collect(root, &mut text);
    }
    let lines: Vec<String> = text
        .lines()
        .map(squeeze)
        .filter(|line| !line.is_empty())
        .collect();
    let body = lines.join("\n");
    match (title.is_empty(), body.is_empty()) {
        (true, _) => body,
        (false, true) => title,
        (false, false) => format!("{title}\n\n{body}"),
    }
}

fn collect(element: ElementRef, out: &mut String) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => out.push_str(text),
            Node::Element(inner) => {
                let name = inner.name();
                if SKIPPED.contains(&name) {
                    continue;
                }
                let block = BLOCKS.contains(&name);
                if block {
                    out.push('\n');
                }
                if let Some(inner) = ElementRef::wrap(child) {
                    collect(inner, out);
                }
                if block {
                    out.push('\n');
                }
            }
            _ => {}
        }
    }
}

/// `line` with every run of white space as one space.
fn squeeze(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}
