//! Proposals for Unfiled Notes (#18).

mod common;

use common::{Reply, Server, TestResult, World, model, model_server, sent_to};
use iibrain_core::{Capture, Core, Error};

const PROPOSAL: &str = r##"Here you go:
{"title": "Plumber for the boiler", "tags": ["house", "#repair"], "folder": "home"}"##;

fn typed(text: &str) -> Capture {
    Capture {
        text: text.to_string(),
        via: "typed".to_string(),
        ..Capture::default()
    }
}

/// A Brain with two Folders and one Unfiled Note, Unfiled Notes open.
fn setup(world: &World) -> Result<(Core, String), Box<dyn std::error::Error>> {
    world.write("Home/keep.md", "x");
    world.write("Money/keep.md", "x");
    let mut core = world.core();
    core.set_unfiled_private(false)?;
    let id = core.capture(&typed("The boiler in the cellar leaks, call the plumber."))?;
    Ok((core, id))
}

#[test]
fn a_model_proposes_title_tags_and_one_of_the_folders() -> TestResult {
    let server = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let (mut core, id) = setup(&world)?;
    core.save_model(model(&server, "big", false))?;

    let proposal = core.propose(&id)?;
    assert_eq!(proposal.title, "Plumber for the boiler");
    assert_eq!(proposal.tags, ["house", "repair"]);
    assert_eq!(proposal.folder, "Home", "in the person's own spelling");
    assert!(!proposal.new_folder);
    assert_eq!(proposal.model, "big");

    let sent = sent_to(&server);
    assert!(sent.contains("The boiler in the cellar leaks"));
    assert!(
        sent.contains("- Home") && sent.contains("- Money"),
        "the Folders to choose from"
    );
    assert!(!sent.contains("- Inbox"));
    Ok(())
}

#[test]
fn a_proposal_changes_nothing_until_the_person_files() -> TestResult {
    let server = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let (mut core, id) = setup(&world)?;
    core.save_model(model(&server, "big", false))?;
    let before = (world.files(), world.read(&id));

    let proposal = core.propose(&id)?;
    assert_eq!(
        (world.files(), world.read(&id)),
        before,
        "nothing moved, nothing written"
    );
    assert!(core.note(&id)?.unfiled());

    // Accepted as it is.
    let filed = core.file(&id, &proposal.title, &proposal.tags, &proposal.folder)?;
    assert_eq!(filed, "Home/Plumber for the boiler.md");
    assert_eq!(core.note(&filed)?.tags, ["house", "repair"]);
    Ok(())
}

#[test]
fn a_proposal_may_suggest_a_new_folder() -> TestResult {
    let server = model_server(|_| {
        r#"{"title":"Apple tree","tags":"garden, trees","folder":"Garden"}"#.to_string()
    });
    let world = World::new();
    let (mut core, id) = setup(&world)?;
    core.save_model(model(&server, "big", false))?;

    let proposal = core.propose(&id)?;
    assert_eq!(proposal.folder, "Garden");
    assert!(proposal.new_folder);
    assert_eq!(proposal.tags, ["garden", "trees"]);
    assert!(!world.exists("Garden"), "suggested, not made");

    // Corrected before filing: another title, the new Folder kept.
    let filed = core.file(&id, "Old apple tree", &proposal.tags, &proposal.folder)?;
    assert_eq!(filed, "Garden/Old apple tree.md");
    Ok(())
}

#[test]
fn a_proposed_folder_never_leaves_the_brain_and_is_never_the_inbox() -> TestResult {
    for (answer, folder) in [
        (r#"{"title":"t","folder":"../../etc"}"#, "etc"),
        (r#"{"title":"t","folder":"/Inbox"}"#, ""),
        (r#"{"title":"t","folder":".hidden/x"}"#, "x"),
        (r#"{"title":"t"}"#, ""),
    ] {
        let server = model_server(move |_| answer.to_string());
        let world = World::new();
        let (mut core, id) = setup(&world)?;
        core.save_model(model(&server, "big", false))?;
        assert_eq!(core.propose(&id)?.folder, folder, "{answer}");
    }
    Ok(())
}

#[test]
fn a_private_unfiled_note_reaches_a_local_model_only() -> TestResult {
    let cloud = model_server(|_| PROPOSAL.to_string());
    let local = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let (mut core, id) = setup(&world)?;
    core.set_unfiled_private(true)?;
    core.save_model(model(&cloud, "big", false))?;
    core.save_model(model(&local, "home", true))?;

    core.choose_model("big")?;
    assert!(matches!(core.propose(&id), Err(Error::Private)));
    assert!(cloud.requests().is_empty(), "nothing at all was sent");

    core.choose_model("home")?;
    assert_eq!(core.propose(&id)?.model, "home");
    assert!(sent_to(&local).contains("boiler"));
    Ok(())
}

#[test]
fn a_note_the_person_marked_stays_away_from_a_cloud_model_whatever_the_setting() -> TestResult {
    let cloud = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    let (mut core, _) = setup(&world)?;
    core.save_model(model(&cloud, "big", false))?;
    let marked = core.capture(&Capture {
        private: true,
        ..typed("my diagnosis")
    })?;
    assert!(matches!(core.propose(&marked), Err(Error::Private)));
    assert!(!sent_to(&cloud).contains("diagnosis"));
    Ok(())
}

#[test]
fn the_names_of_private_folders_stay_away_from_a_cloud_model() -> TestResult {
    let cloud = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    world.write("Therapy/keep.md", "x");
    let (mut core, id) = setup(&world)?;
    core.set_folder_private("Therapy", true)?;
    core.save_model(model(&cloud, "big", false))?;
    core.propose(&id)?;
    let sent = sent_to(&cloud);
    assert!(!sent.contains("Therapy"), "{sent}");
    assert!(sent.contains("- Home"));
    Ok(())
}

#[test]
fn extracted_text_and_the_link_are_part_of_what_the_model_reads() -> TestResult {
    let server = model_server(|_| PROPOSAL.to_string());
    let world = World::new();
    world.write(
        "Inbox/scan.md",
        "---\nlink: https://example.org/boiler\n---\n\nsee photo\n",
    );
    world.write("Inbox/scan.files/label.jpg", "jpg");
    world.write(
        "Inbox/scan.files/label.jpg.extracted.txt",
        "Vaillant ecoTEC 2019",
    );
    world.write(
        "Inbox/scan.files/page.extracted.txt",
        "Manual for the boiler series",
    );
    let mut core = world.core();
    core.set_unfiled_private(false)?;
    core.save_model(model(&server, "big", false))?;
    core.propose("Inbox/scan.md")?;
    let sent = sent_to(&server);
    for part in [
        "see photo",
        "Vaillant ecoTEC 2019",
        "example.org/boiler",
        "Manual for the boiler series",
        "label.jpg",
    ] {
        assert!(sent.contains(part), "{part} is missing in {sent}");
    }
    Ok(())
}

#[test]
fn without_a_model_or_with_a_failing_one_the_person_is_told() -> TestResult {
    let world = World::new();
    let (mut core, id) = setup(&world)?;
    assert!(matches!(core.propose(&id), Err(Error::NoModel)));

    let failing = Server::start(|_| Reply::status(500));
    core.save_model(model(&failing, "broken", false))?;
    assert!(matches!(core.propose(&id), Err(Error::Model(_))));

    let babbling = model_server(|_| "I would file it under home, I think.".to_string());
    core.save_model(model(&babbling, "chatty", false))?;
    core.choose_model("chatty")?;
    assert!(matches!(core.propose(&id), Err(Error::Model(_))));

    // Filing by hand stays possible.
    assert_eq!(core.file(&id, "Plumber", &[], "Home")?, "Home/Plumber.md");
    Ok(())
}
