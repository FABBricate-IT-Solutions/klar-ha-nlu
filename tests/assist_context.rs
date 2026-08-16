use klar_nlu::home::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{Intent, Settings};

fn one(text: &str, preferred_area: &str) -> Intent {
    let home = default_home();
    let mut session = Session::new();
    session.preferred_area = Some(preferred_area.into());
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    assert_eq!(result.intents.len(), 1, "{text}: {:?}", result.intents);
    result.intents.into_iter().next().unwrap()
}

fn slot<'a>(intent: &'a Intent, name: &str) -> Option<&'a str> {
    intent.slot(name)
}

fn targets(intent: &Intent, area: &str, entity: &str) -> bool {
    slot(intent, "area") == Some(area) || slot(intent, "entity_id") == Some(entity)
}

#[test]
fn preferred_area_targets_room_light_on_and_off() {
    let on = one("Licht an", "wohnzimmer");
    assert_eq!(on.name, "HassTurnOn");
    assert!(targets(&on, "wohnzimmer", "light.wohnzimmer"), "{on:?}");

    let off = one("Licht aus", "wohnzimmer");
    assert_eq!(off.name, "HassTurnOff");
    assert!(targets(&off, "wohnzimmer", "light.wohnzimmer"), "{off:?}");
}

#[test]
fn explicit_light_area_overrides_preferred_area() {
    let intent = one("Licht in der Küche an", "wohnzimmer");
    assert_eq!(intent.name, "HassTurnOn");
    assert!(targets(&intent, "kuche", "light.kuche_kuche"), "{intent:?}");
}

#[test]
fn all_lights_ignore_preferred_area() {
    let intent = one("Alle Lichter aus", "wohnzimmer");
    assert_eq!(intent.name, "HassTurnOff");
    assert_eq!(slot(&intent, "entity_id"), Some("light.alle_lichter"));
}

#[test]
fn named_tv_ignores_preferred_area() {
    let intent = one("Schlafzimmer TV an", "wohnzimmer");
    assert_eq!(intent.name, "HassTurnOn");
    assert_eq!(slot(&intent, "entity_id"), Some("switch.schlafzimmer_tv"));
}
