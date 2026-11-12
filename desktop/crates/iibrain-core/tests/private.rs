//! Marking Notes and Folders as Private (#17).

mod common;

use common::{TestResult, World, model, model_server};
use iibrain_core::{Capture, Core, INBOX};

fn typed(text: &str) -> Capture {
    Capture {
        text: text.to_string(),
        via: "typed".to_string(),
        ..Capture::default()
    }
}

#[test]
fn a_note_is_marked_private_and_unmarked() -> TestResult {
    let world = World::new();
    world.write("Health/Blood.md", "---\ntitle: Blood test\n---\n\nvalues\n");
    let mut core = world.core();
    assert!(!core.is_private(&core.note("Health/Blood.md")?));

    core.set_private("Health/Blood.md", true)?;
    assert!(core.is_private(&core.note("Health/Blood.md")?));
    assert!(world.read("Health/Blood.md").contains("private: true"));

    core.set_private("Health/Blood.md", false)?;
    assert!(!core.is_private(&core.note("Health/Blood.md")?));
    assert!(!world.read("Health/Blood.md").contains("private"));
    Ok(())
}

#[test]
fn a_private_folder_makes_everything_in_it_private() -> TestResult {
    let world = World::new();
    world.write("Health/Blood.md", "x");
    world.write("Health/2026/Scan.md", "x");
    world.write("Garden/Plan.md", "x");
    let mut core = world.core();
    core.set_folder_private("Health", true)?;

    assert!(core.is_private(&core.note("Health/Blood.md")?));
    assert!(
        core.is_private(&core.note("Health/2026/Scan.md")?),
        "Folders inside it too"
    );
    assert!(!core.is_private(&core.note("Garden/Plan.md")?));
    let private: Vec<String> = core
        .folders()?
        .into_iter()
        .filter(|f| f.private)
        .map(|f| f.name)
        .collect();
    assert_eq!(private, ["Health", "Health/2026"]);
    assert!(
        !world.read("Health/Blood.md").contains("private"),
        "the Note's file is untouched"
    );

    core.set_folder_private("Health", false)?;
    assert!(!core.is_private(&core.note("Health/Blood.md")?));
    assert!(
        core.set_folder_private("", true).is_err(),
        "the root is no Folder"
    );
    Ok(())
}

#[test]
fn the_mark_moves_with_a_renamed_folder() -> TestResult {
    let world = World::new();
    world.write("Health/Blood.md", "x");
    let mut core = world.core();
    core.set_folder_private("Health", true)?;
    core.rename_folder("Health", "Body")?;
    assert!(core.is_private(&core.note("Body/Blood.md")?));
    Ok(())
}

#[test]
fn unfiled_notes_are_private_until_the_setting_opens_them() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    let id = core.capture(&typed("fresh"))?;
    assert!(core.unfiled_private());
    assert!(core.is_private(&core.note(&id)?));
    assert!(
        !world.read(&id).contains("private"),
        "by the setting, not by a mark of its own"
    );

    core.set_unfiled_private(false)?;
    assert!(!core.is_private(&core.note(&id)?));

    // A Capture the person marked stays Private whatever the setting.
    let marked = core.capture(&Capture {
        private: true,
        ..typed("secret")
    })?;
    assert!(core.is_private(&core.note(&marked)?));
    assert!(world.read(&marked).contains("private: true"));

    // Filing does not lift a Note's own mark, and ends the Inbox's rule.
    core.set_unfiled_private(true)?;
    let filed = core.file(&id, "Fresh", &[], "")?;
    assert!(!core.is_private(&core.note(&filed)?));
    let filed_marked = core.file(&marked, "Secret", &[], "")?;
    assert!(core.is_private(&core.note(&filed_marked)?));
    Ok(())
}

#[test]
fn the_gate_lets_private_notes_through_to_local_models_only() -> TestResult {
    let server = model_server(|_| "ok".to_string());
    let (local, cloud) = (model(&server, "home", true), model(&server, "big", false));
    let world = World::new();
    world.write("Health/Blood.md", "---\nprivate: true\n---\n\nvalues\n");
    world.write("Garden/Plan.md", "x");
    let core = world.core();
    let private = core.note("Health/Blood.md")?;
    let open = core.note("Garden/Plan.md")?;

    assert!(core.may_send(&private, &local));
    assert!(!core.may_send(&private, &cloud));
    assert!(core.may_send(&open, &local));
    assert!(core.may_send(&open, &cloud));
    Ok(())
}

#[test]
fn every_mark_holds_on_another_device_of_the_same_brain() -> TestResult {
    let world = World::new();
    world.write("Health/Blood.md", "x");
    world.write("Garden/Plan.md", "x");
    let mut phone = world.core();
    let fresh = phone.capture(&typed("fresh"))?;
    phone.set_private("Garden/Plan.md", true)?;
    phone.set_folder_private("Health", true)?;

    let mut desktop = Core::open_folder(&world.other_device(), &world.brain())?;
    assert!(desktop.is_private(&desktop.note("Garden/Plan.md")?));
    assert!(desktop.is_private(&desktop.note("Health/Blood.md")?));
    assert!(desktop.is_private(&desktop.note(&fresh)?));
    assert!(
        desktop
            .folders()?
            .iter()
            .any(|f| f.name == "Health" && f.private)
    );

    // And the setting for Unfiled Notes, set on the desktop, on the phone.
    desktop.set_unfiled_private(false)?;
    assert!(!phone.unfiled_private());
    assert!(!phone.is_private(&phone.note(&fresh)?));
    assert_eq!(phone.note(&fresh)?.folder, INBOX);
    Ok(())
}

#[test]
fn the_marks_are_no_notes_and_no_folders() -> TestResult {
    let world = World::new();
    world.write("Health/Blood.md", "x");
    let mut core = world.core();
    core.set_folder_private("Health", true)?;
    core.set_unfiled_private(false)?;
    core.refresh()?;
    assert_eq!(core.all_notes()?.len(), 1);
    let names: Vec<String> = core.folders()?.into_iter().map(|f| f.name).collect();
    assert_eq!(names, [INBOX, "Health"]);
    Ok(())
}
