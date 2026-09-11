use super::*;
use crate::home::default_home;

#[test]
fn fuzzy_split_transposes_room_before_licht() {
    let home = default_home();
    let split = expand_compounds(&["wonhzimmerlicht".into()], &home);
    assert!(split.tokens.iter().any(|token| token == "wohnzimmer"), "{:?}", split.tokens);
    assert!(split.tokens.iter().any(|token| token == "licht"), "{:?}", split.tokens);
    assert_eq!(split.light_areas, ["wohnzimmer"]);
}

#[test]
fn fuzzy_split_rejects_unrelated_licht() {
    let home = default_home();
    let split = expand_compounds(&["fensterbanklicht".into()], &home);
    assert_eq!(split.tokens, ["fensterbanklicht"]);
    assert!(split.light_areas.is_empty());
}

#[test]
fn nachttisch_is_not_a_whole_word_nacht_script() {
    let _bind = crate::lang::bind(&["de".into()]);
    let home = crate::home::load_home_config(std::path::Path::new("tests/datasets/full_home/de/home_config.yaml")).expect("home");
    assert_eq!(named_scene_or_script(&["nachttisch".into(), "schlafzimmer".into()], &home), None);
    assert_eq!(named_scene_or_script(&["nacht".into(), "an".into()], &home).as_deref(), Some("script.good_night"));
}

#[test]
fn short_script_alias_is_an_exact_token_match() {
    let _bind = crate::lang::bind(&["af".into()]);
    let home = crate::types::HomeGraph {
        entities: vec![crate::types::EntityRec {
            entity_id: "script.good_night".into(),
            name: "Good Night".into(),
            domain: "script".into(),
            platform: None,
            area: None,
            aliases: vec!["nag".into()],
            tags: vec![],
        }],
        ..HomeGraph::default()
    };
    assert_eq!(named_scene_or_script(&["nag".into()], &home).as_deref(), Some("script.good_night"));
}

#[test]
fn dishwasher_off_is_not_the_all_off_scene() {
    let _bind = crate::lang::bind(&["de".into()]);
    let home = crate::home::load_home_config(std::path::Path::new("tests/datasets/wohnung_mittel/home_config.yaml")).expect("home");
    assert_eq!(named_scene_or_script(&["spuelmaschine".into(), "aus".into()], &home), None);
    let settings = crate::types::Settings { languages: vec!["de".into()], ..crate::types::Settings::default() };
    let outcome = crate::nlu::parse("Spülmaschine aus", &home, &mut crate::session::Session::new(), &[], &settings);
    let names: Vec<_> = outcome.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    assert!(
        names.iter().any(|intent| intent.name == "HassTurnOff" && intent.slot("entity_id") == Some("switch.kuche_spulmaschine")),
        "{:?} {:?}",
        outcome.decision,
        names
    );
}

#[test]
fn all_lamps_off_is_not_the_all_off_scene() {
    let _bind = crate::lang::bind(&["en".into()]);
    let home = crate::home::load_home_config(std::path::Path::new("tests/datasets/wohnung_mittel/home_config.yaml")).expect("home");
    assert_eq!(named_scene_or_script(&["turn".into(), "off".into(), "all".into(), "lamps".into()], &home), None);
}
