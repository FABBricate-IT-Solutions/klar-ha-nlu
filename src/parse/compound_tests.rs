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
