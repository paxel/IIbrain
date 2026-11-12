//! A link captured together with the readable text of its page (#14).

mod common;

use common::{Reply, Server, TestResult, World};
use iibrain_core::{Capture, web};

const PAGE: &str = r#"<!doctype html>
<html><head><title>How to  prune
  an apple tree</title>
<style>body { color: red; }</style>
<script>trackEverything("visitor");</script></head>
<body>
<header><a href="/">Garden Weekly</a></header>
<nav><ul><li>Subscribe now</li><li>Shop</li></ul></nav>
<article>
<h1>How to prune an apple tree</h1>
<p>Cut in late winter, before the buds <em>swell</em>.</p>
<p>Remove crossing branches first.</p>
<aside>Advertisement: buy our secateurs</aside>
</article>
<footer>Imprint and cookies</footer>
</body></html>"#;

fn link(url: &str) -> Capture {
    Capture {
        via: "link".to_string(),
        link: Some(url.to_string()),
        ..Capture::default()
    }
}

#[test]
fn a_link_is_kept_with_the_readable_text_of_its_page() -> TestResult {
    let server = Server::start(|_| Reply::html(PAGE));
    let world = World::new();
    let mut core = world.core();
    let url = format!("{}/apple-tree", server.url);
    let id = core.capture(&link(&url))?;

    let note = core.note(&id)?;
    assert!(note.unfiled());
    assert_eq!(note.link.as_deref(), Some(url.as_str()));
    assert_eq!(note.page_text, None, "not fetched yet");
    assert_eq!(
        note.text, url,
        "the link is the Note's text until the person writes one"
    );

    assert_eq!(core.extract_pending()?, 1);
    let text = core.note(&id)?.page_text.ok_or("the page's text")?;
    assert!(text.starts_with("How to prune an apple tree\n"), "{text}");
    assert!(
        text.contains("Cut in late winter, before the buds swell."),
        "{text}"
    );
    assert!(text.contains("Remove crossing branches first."), "{text}");
    for unwanted in [
        "Subscribe",
        "trackEverything",
        "color: red",
        "secateurs",
        "Imprint",
        "Garden Weekly",
    ] {
        assert!(
            !text.contains(unwanted),
            "{unwanted} is no text of the page: {text}"
        );
    }
    assert_eq!(
        core.search("crossing branches")?
            .first()
            .map(|h| h.id.clone()),
        Some(id)
    );
    Ok(())
}

#[test]
fn only_the_host_of_the_link_is_asked() -> TestResult {
    let server = Server::start(|_| Reply::html(PAGE));
    let world = World::new();
    let mut core = world.core();
    core.capture(&link(&format!("{}/apple-tree", server.url)))?;
    core.extract_pending()?;
    core.extract_pending()?;

    let requests = server.requests();
    assert_eq!(requests.len(), 1, "one request, and none again later");
    let request = requests.first().ok_or("a request")?;
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/apple-tree");
    Ok(())
}

#[test]
fn a_redirect_is_followed() -> TestResult {
    let server = Server::start(|request| match request.path.as_str() {
        "/short" => Reply::redirect("/apple-tree"),
        _ => Reply::html(PAGE),
    });
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&link(&format!("{}/short", server.url)))?;
    core.extract_pending()?;
    assert!(
        core.note(&id)?
            .page_text
            .unwrap_or_default()
            .contains("crossing branches")
    );
    Ok(())
}

#[test]
fn a_page_that_cannot_be_fetched_leaves_the_link_and_says_so() -> TestResult {
    let server = Server::start(|_| Reply::status(403));
    let world = World::new();
    let mut core = world.core();
    let refused = core.capture(&link(&format!("{}/paywall", server.url)))?;
    // Nothing listens there.
    let gone = core.capture(&link("http://127.0.0.1:9/nothing"))?;
    assert_eq!(core.extract_pending()?, 2);

    for id in [refused, gone] {
        let note = core.note(&id)?;
        assert!(note.link.is_some(), "the link stays");
        assert_eq!(
            note.page_text.as_deref(),
            Some(""),
            "its text is said to be missing"
        );
    }
    assert_eq!(
        core.extract_pending()?,
        0,
        "it is not tried again on its own"
    );
    Ok(())
}

#[test]
fn words_of_the_person_stay_the_text_of_the_note() -> TestResult {
    let server = Server::start(|_| Reply::html(PAGE));
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&Capture {
        text: "For the old tree behind the shed".to_string(),
        ..link(&format!("{}/apple-tree", server.url))
    })?;
    assert_eq!(core.note(&id)?.text, "For the old tree behind the shed");
    Ok(())
}

#[test]
fn a_page_without_an_article_gives_the_text_of_its_body() {
    let text = web::readable(
        "<html><body><div>Opening hours</div><div>Mon to Fri, 9 to 17</div><script>x()</script></body></html>",
    );
    assert_eq!(text, "Opening hours\nMon to Fri, 9 to 17");
}

#[test]
fn only_one_web_link_alone_counts_as_a_link() {
    assert!(web::is_link("https://example.org/a?b=c"));
    assert!(web::is_link("  http://example.org  "));
    assert!(!web::is_link("see https://example.org"));
    assert!(!web::is_link("https://"));
    assert!(!web::is_link("ftp://example.org/file"));
    assert!(!web::is_link("example.org"));
}
