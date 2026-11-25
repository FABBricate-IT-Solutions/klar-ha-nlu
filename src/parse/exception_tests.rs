use super::*;
use crate::lang::{bind_catalog, catalog_for};
use crate::types::{AreaRec, EntityRec};

fn home() -> HomeGraph {
    let light = |id: &str, name: &str| EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: "light".into(),
        platform: None,
        area: Some("office".into()),
        aliases: Vec::new(),
        tags: Vec::new(),
    };
    HomeGraph {
        entities: vec![light("light.on_air", "On Air"), light("light.away", "Außer Haus"), light("light.desk", "Desk Lamp")],
        areas: vec![AreaRec { area_id: "office".into(), name: "Office".into(), aliases: Vec::new(), floor_id: None }],
        ..HomeGraph::default()
    }
}

fn parts(lang: &str, text: &str) -> Option<Result<(String, Vec<String>), Unreadable>> {
    let cat = catalog_for(&[lang.into()]);
    let _bound = bind_catalog(cat.clone());
    let read = read(&tokenize(text), &cat, &Names::of(&home()))?;
    Some(read.map(|found| (found.command.join(" "), found.items.iter().map(|item| item.join(" ")).collect())))
}

fn split_of(lang: &str, text: &str) -> (String, Vec<String>) {
    parts(lang, text).expect("an exception").expect("a readable exception")
}

#[test]
fn the_exception_follows_its_word() {
    assert_eq!(
        split_of("en", "turn off all the lights except the hall light"),
        ("turn off all the lights".into(), vec!["the hall light".into()])
    );
    assert_eq!(
        split_of("en", "turn on the kitchen lights apart from the counter light"),
        ("turn on the kitchen lights".into(), vec!["the counter light".into()])
    );
    assert_eq!(split_of("en", "turn on lamp 1 but not lamp 2"), ("turn on lamp 1".into(), vec!["lamp 2".into()]));
}

#[test]
fn a_list_of_exceptions_is_split_at_and() {
    let (command, items) = split_of("en", "turn off everything in the office except the desk lamp and the printer");
    assert_eq!(command, "turn off everything in the office");
    assert_eq!(items, vec!["the desk lamp".to_string(), "the printer".to_string()]);
}

#[test]
fn a_second_command_is_not_part_of_the_exception() {
    let (command, items) = split_of("en", "turn on the lights in the hall except the floor lamp and turn off the tv");
    assert_eq!(command, "turn on the lights in the hall and turn off the tv");
    assert_eq!(items, vec!["the floor lamp".to_string()]);
}

#[test]
fn a_verb_at_the_end_goes_back_to_the_command() {
    assert_eq!(
        split_of("de", "alle Lichter außer dem Flurlicht ausschalten"),
        ("alle lichter ausschalten".into(), vec!["dem flurlicht".into()])
    );
    assert_eq!(
        split_of("de", "mach alle Lichter bis auf die Stehlampe aus"),
        ("mach alle lichter aus".into(), vec!["die stehlampe".into()])
    );
    assert_eq!(
        split_of("de", "mach die Lichter im Büro an, nur nicht die Stehlampe"),
        ("mach die lichter im buero an".into(), vec!["die stehlampe".into()])
    );
}

#[test]
fn words_inside_a_name_are_the_name() {
    assert_eq!(parts("de", "schalte Außer Haus ein"), None);
    assert_eq!(
        split_of("en", "turn off the lights in the office except On Air"),
        ("turn off the lights in the office".into(), vec!["on air".into()])
    );
}

#[test]
fn an_exception_before_its_command_is_read_only_when_the_order_is_clear() {
    assert_eq!(split_of("en", "the desk lamp except all lights off"), ("all lights off".into(), vec!["the desk lamp".into()]));
    assert_eq!(parts("en", "the desk lamp except the office light"), Some(Err(Unreadable::WhichSide)));
}

#[test]
fn an_exception_that_names_nothing_is_unreadable() {
    assert_eq!(parts("en", "turn off all the lights except"), Some(Err(Unreadable::Incomplete)));
    assert_eq!(parts("en", "except the desk lamp"), Some(Err(Unreadable::Incomplete)));
}

#[test]
fn a_sentence_without_an_exception_word_has_none() {
    assert_eq!(parts("en", "turn off all the lights"), None);
    assert_eq!(parts("en", "do not turn on the desk lamp"), None);
}
