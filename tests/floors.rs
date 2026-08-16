use klar_nlu::home::load_home_config;
use klar_nlu::nlu;
use klar_nlu::parse::resolve::resolve;
use klar_nlu::session::Session;
use klar_nlu::types::{ParseDecision, Settings};

fn family_home(path: &str) -> klar_nlu::types::HomeGraph {
    load_home_config(std::path::Path::new(path)).expect("family home")
}

fn parse(text: &str, home: &klar_nlu::types::HomeGraph, language: &str) -> klar_nlu::types::ParseOutcome {
    let mut session = Session::new();
    let settings = Settings { languages: vec![language.into()], ..Settings::default() };
    nlu::parse(text, home, &mut session, &[], &settings)
}

fn execute_slots(text: &str, home: &klar_nlu::types::HomeGraph, language: &str) -> Vec<(String, Vec<(String, String)>)> {
    let outcome = parse(text, home, language);
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{text}: {outcome:#?}");
    outcome
        .plan
        .expect("execute plan")
        .steps
        .into_iter()
        .map(|step| (step.intent.name, step.intent.slots.into_iter().map(|slot| (slot.name, slot.value)).collect()))
        .collect()
}

#[test]
fn yaml_family_homes_load_floors() {
    for path in ["tests/datasets/family_home_en/home_config.yaml", "tests/datasets/familienhaus_de/home_config.yaml"] {
        let home = family_home(path);
        assert!(home.floors.iter().any(|floor| floor.floor_id == "upper"), "{path} missing upper floor");
        assert!(home.floors.iter().any(|floor| floor.floor_id == "ground"), "{path} missing ground floor");
        assert!(home.areas.iter().any(|area| area.floor_id.as_deref() == Some("upper")), "{path} areas lost floor_id");
        assert!(home.floors.iter().any(|floor| floor.aliases.iter().any(|alias| alias == "upstairs")));
        assert!(home.floors.iter().any(|floor| floor.aliases.iter().any(|alias| alias == "obergeschoss")));
    }
}

#[test]
fn resolve_scopes_assist_visible_lights_to_the_named_floor() {
    let home = family_home("tests/datasets/family_home_en/home_config.yaml");
    let hit = resolve(&["upstairs".into(), "lights".into()], &home, Some("light"));
    assert_eq!(hit.floors, ["upper"]);
    assert!(hit.areas.is_empty(), "{:?}", hit.areas);
    assert!(hit.entities.iter().all(|entity| home
        .areas
        .iter()
        .any(|area| { area.area_id == entity.area.as_deref().unwrap_or("") && area.floor_id.as_deref() == Some("upper") })));
}

#[test]
fn upstairs_lights_resolve_to_floor_in_english() {
    let home = family_home("tests/datasets/family_home_en/home_config.yaml");
    let found = execute_slots("Turn on the upstairs lights", &home, "en");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "HassTurnOn");
    assert!(found[0].1.iter().any(|(name, value)| name == "floor" && value == "upper"), "{found:?}");
    assert!(found[0].1.iter().any(|(name, value)| name == "domain" && value == "light"), "{found:?}");
}

#[test]
fn obergeschoss_lights_resolve_to_floor_in_german() {
    let home = family_home("tests/datasets/familienhaus_de/home_config.yaml");
    let found = execute_slots("Licht im Obergeschoss an", &home, "de");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "HassTurnOn");
    assert!(found[0].1.iter().any(|(name, value)| name == "floor" && value == "upper"), "{found:?}");
    assert!(found[0].1.iter().any(|(name, value)| name == "domain" && value == "light"), "{found:?}");
}
