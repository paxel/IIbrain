//! What a Live Lookup read on the phone, on its way to a Model (#27):
//! handed over beside the Sources, never kept, and to a Cloud Model
//! only once its switch is opened.

mod common;

use common::{TestResult, World, model, model_server, sent_to};
use iibrain_core::{Found, Lookup};

fn calendar() -> Lookup {
    Lookup {
        kind: "calendar".to_string(),
        text: "2026-11-16 14:00 to 15:00: Dentist".to_string(),
    }
}

fn contacts() -> Lookup {
    Lookup {
        kind: "contacts".to_string(),
        text: "Max Brandt: 0151 555".to_string(),
    }
}

#[test]
fn a_local_model_gets_what_was_looked_up_beside_the_sources() -> TestResult {
    let server = model_server(|_| "On Monday at two.".to_string());
    let world = World::new();
    world.write(
        "Health/Teeth.md",
        "The dentist wants a check-up twice a year.",
    );
    let mut core = world.core();
    core.save_model(model(&server, "home", true))?;

    let answer = core
        .ask("When is the dentist?", &[calendar(), contacts()])?
        .ok_or("an Answer")?;
    assert_eq!(answer.sources.len(), 1);
    let sent = sent_to(&server);
    assert!(sent.contains("From the person's calendar"), "{sent}");
    assert!(sent.contains("14:00 to 15:00: Dentist"));
    assert!(sent.contains("From the person's contacts"));
    assert!(sent.contains("Max Brandt: 0151 555"));
    Ok(())
}

#[test]
fn a_lookup_alone_is_enough_to_ask() -> TestResult {
    let server = model_server(|_| "On Monday at two.".to_string());
    let world = World::new();
    let mut core = world.core();
    core.save_model(model(&server, "home", true))?;

    let answer = core
        .ask("When is the dentist?", &[calendar()])?
        .ok_or("an Answer")?;
    assert!(answer.sources.is_empty());
    assert!(sent_to(&server).contains("Dentist"));
    Ok(())
}

#[test]
fn a_cloud_model_gets_nothing_looked_up_until_the_switch_is_opened() -> TestResult {
    let server = model_server(|_| "x".to_string());
    let world = World::new();
    world.write(
        "Health/Teeth.md",
        "The dentist wants a check-up twice a year.",
    );
    let mut core = world.core();
    core.save_model(model(&server, "big", false))?;

    core.ask("When is the dentist?", &[calendar(), contacts()])?;
    let sent = sent_to(&server);
    assert!(sent.contains("check-up"), "the Note is no secret");
    assert!(
        !sent.contains("14:00") && !sent.contains("Max Brandt"),
        "{sent}"
    );

    core.set_lookup_to_cloud("calendar", true)?;
    core.ask("When is the dentist?", &[calendar(), contacts()])?;
    let sent = sent_to(&server);
    assert!(sent.contains("14:00 to 15:00: Dentist"));
    assert!(!sent.contains("Max Brandt"), "each has its own switch");

    core.set_lookup_to_cloud("contacts", true)?;
    core.ask("When is the dentist?", &[contacts()])?;
    assert!(sent_to(&server).contains("Max Brandt"));
    assert!(core.set_lookup_to_cloud("photos", true).is_err());
    Ok(())
}

#[test]
fn a_lookup_a_cloud_model_may_not_see_does_not_make_it_asked() -> TestResult {
    let server = model_server(|_| "x".to_string());
    let world = World::new();
    let mut core = world.core();
    core.save_model(model(&server, "big", false))?;

    assert!(matches!(
        core.question("When is the dentist?", &[calendar()])?,
        Found::Nothing { .. }
    ));
    assert!(server.requests().is_empty());
    Ok(())
}

#[test]
fn nothing_of_a_lookup_is_kept_in_the_brain() -> TestResult {
    let server = model_server(|_| "On Monday at two.".to_string());
    let world = World::new();
    let mut core = world.core();
    core.save_model(model(&server, "home", true))?;
    core.ask("When is the dentist?", &[calendar(), contacts()])?;

    assert!(world.files().is_empty(), "{:?}", world.files());
    assert!(core.search("dentist")?.is_empty());
    Ok(())
}

#[test]
fn the_switches_belong_to_the_device() -> TestResult {
    let world = World::new();
    let mut core = world.core();
    core.set_lookup_to_cloud("calendar", true)?;
    drop(core);
    let settings = iibrain_core::Device::load(&world.device())?.settings;
    assert!(settings.calendar_to_cloud);
    assert!(!settings.contacts_to_cloud);
    assert!(world.files().is_empty(), "nothing of it lies in the Brain");
    Ok(())
}
