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
