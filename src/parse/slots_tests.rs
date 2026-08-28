use super::name_has_group_conj;
use crate::parse::parse;
use crate::session::Session;
use crate::types::{EntityRec, HomeGraph, Settings};
use std::collections::HashSet;

#[test]
fn group_conj_is_a_word_not_a_substring() {
    assert!(name_has_group_conj("Wohn und Esszimmer"));
    assert!(name_has_group_conj("Living and Dining"));
    assert!(!name_has_group_conj("Island Lights"));
    assert!(!name_has_group_conj("Insel Licht"));
    assert!(!name_has_group_conj("Standleuchte"));
}

#[test]
fn configured_todo_names_aliases_and_labels_route_without_object_id_evidence() {
    for (sentence, entity) in [
        ("Füge Milch zur Aufgabenliste hinzu", todo("todo.internal_de", "Aufgaben", &[], &[])),
        ("Add milk to the House Tasks list", todo("todo.internal_en", "House Tasks", &[], &[])),
        ("Add milk to the Errands list", todo("todo.internal_alias", "Internal", &["Errands"], &[])),
        ("Add milk to the Weekend list", todo("todo.internal_label", "Internal", &[], &["Weekend"])),
    ] {
        let result = parse_with_home(sentence, HomeGraph { entities: vec![entity.clone()], ..HomeGraph::default() });
        assert!(!result.clarify, "{sentence}: {result:?}");
        assert_eq!(result.intents.len(), 1, "{sentence}: {result:?}");
        assert_eq!(result.intents[0].slot("entity_id"), Some(entity.entity_id.as_str()), "{sentence}: {result:?}");
        assert_eq!(result.intents[0].slot("item"), sentence.starts_with("Füge").then_some("milch").or(Some("milk")));
    }
}

#[test]
fn generic_shopping_does_not_guess_todo_and_hidden_or_ambiguous_names_do_not_execute() {
    let first = todo("todo.first", "Errands", &[], &[]);
    let second = todo("todo.second", "Errands", &[], &[]);
    let generic = parse_with_home(
        "Add milk to the shopping list",
        HomeGraph { entities: vec![first.clone(), second.clone()], ..HomeGraph::default() },
    );
    assert_eq!(generic.intents.len(), 1, "{generic:?}");
    assert_eq!(generic.intents[0].slot("name"), Some("shopping_list"));
    assert_eq!(generic.intents[0].slot("entity_id"), None);

    let ambiguous =
        parse_with_home("Add milk to the Errands list", HomeGraph { entities: vec![first.clone(), second], ..HomeGraph::default() });
    assert!(ambiguous.clarify, "{ambiguous:?}");
    assert!(ambiguous.intents.is_empty(), "{ambiguous:?}");

    let hidden = parse_with_home(
        "Add milk to the Errands list",
        HomeGraph { entities: vec![first], assist: Some(HashSet::new()), ..HomeGraph::default() },
    );
    assert!(!hidden.clarify, "{hidden:?}");
    assert!(hidden.intents.is_empty(), "{hidden:?}");
}

fn parse_with_home(sentence: &str, home: HomeGraph) -> crate::types::ParseResult {
    let lang = if sentence.starts_with("Füge") { "de" } else { "en" };
    parse(sentence, &home, &mut Session::new(), &[], &Settings::pinned(lang))
}

#[test]
fn ja_yes_picks_laundry_switch_and_th_open_unlocks() {
    use crate::types::AreaRec;
    let laundry = HomeGraph {
        areas: vec![AreaRec { area_id: "laundry".into(), name: "Laundry".into(), aliases: vec!["sentakushitsu".into()], floor_id: None }],
        entities: vec![switch("switch.washing_machine", "laundry"), switch("switch.dryer", "laundry")],
        ..HomeGraph::default()
    };
    let mut session = Session::new();
    let first = parse("点ける kiki sentakushitsu", &laundry, &mut session, &[], &Settings::pinned("ja"));
    assert!(first.clarify, "{first:?}");
    let picked = parse("はい", &laundry, &mut session, &[], &Settings::pinned("ja"));
    assert!(!picked.clarify, "{picked:?}");
    assert_eq!(picked.intents[0].slot("entity_id"), Some("switch.washing_machine"), "{picked:?}");

    let locks = HomeGraph {
        areas: vec![AreaRec { area_id: "entryway".into(), name: "Entry".into(), aliases: vec!["thangkhau".into()], floor_id: None }],
        entities: vec![EntityRec {
            entity_id: "lock.front_door".into(),
            name: "Front".into(),
            domain: "lock".into(),
            platform: None,
            area: Some("entryway".into()),
            aliases: vec![],
            tags: vec![],
        }],
        ..HomeGraph::default()
    };
    let unlocked = parse("เปิด กุญแจ thangkhau", &locks, &mut Session::new(), &[], &Settings::pinned("th"));
    assert_eq!(unlocked.intents[0].name, "HassTurnOff", "{unlocked:?}");
    assert_eq!(unlocked.intents[0].slot("entity_id"), Some("lock.front_door"), "{unlocked:?}");

    let garage = HomeGraph {
        areas: vec![AreaRec { area_id: "garage".into(), name: "Garage".into(), aliases: vec!["garage".into()], floor_id: None }],
        entities: vec![
            EntityRec {
                entity_id: "lock.garage_entry".into(),
                name: "Garage lock".into(),
                domain: "lock".into(),
                platform: None,
                area: Some("garage".into()),
                aliases: vec![],
                tags: vec![],
            },
            EntityRec {
                entity_id: "cover.garage_door".into(),
                name: "Garage door".into(),
                domain: "cover".into(),
                platform: None,
                area: Some("garage".into()),
                aliases: vec![],
                tags: vec![],
            },
        ],
        ..HomeGraph::default()
    };
    let mut after_lock = Session::new();
    parse("กุญแจ กุญแจ garage", &garage, &mut after_lock, &[], &Settings::pinned("th"));
    let cover = parse("ปิด ม่าน garage", &garage, &mut after_lock, &[], &Settings::pinned("th"));
    assert_eq!(cover.intents[0].slot("entity_id"), Some("cover.garage_door"), "{cover:?}");
    assert_eq!(cover.intents[0].name, "HassTurnOff", "{cover:?}");

    let lists = HomeGraph { entities: vec![todo("todo.chores", "Aufgaben", &[], &[])], ..HomeGraph::default() };
    let done = parse("हो गया bread aufgabenliste", &lists, &mut Session::new(), &[], &Settings::pinned("hi"));
    assert_eq!(done.intents[0].name, "HassListCompleteItem", "{done:?}");
    assert_eq!(done.intents[0].slot("item"), Some("bread"), "{done:?}");
}

fn switch(id: &str, area: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: id.into(),
        domain: "switch".into(),
        platform: None,
        area: Some(area.into()),
        aliases: vec![],
        tags: vec![],
    }
}

fn todo(id: &str, name: &str, aliases: &[&str], tags: &[&str]) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: "todo".into(),
        platform: None,
        area: None,
        aliases: aliases.iter().map(|value| (*value).into()).collect(),
        tags: tags.iter().map(|value| (*value).into()).collect(),
    }
}
