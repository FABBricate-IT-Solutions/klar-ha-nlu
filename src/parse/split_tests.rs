use super::{split_clauses, verb_final_boundary};
use crate::parse::parse;
use crate::session::Session;
use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph, Settings};

fn rec(id: &str, name: &str, domain: &str, area: &str, aliases: &[&str]) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: domain.into(),
        platform: None,
        area: Some(area.into()),
        aliases: aliases.iter().map(|value| (*value).into()).collect(),
        tags: Vec::new(),
    }
}

fn leftover_home() -> HomeGraph {
    HomeGraph {
        floors: vec![FloorRec {
            floor_id: "basement".into(),
            name: "Keller".into(),
            aliases: vec!["keller".into(), "basement".into()],
            level: Some(-1),
        }],
        areas: vec![
            AreaRec {
                area_id: "living".into(),
                name: "Wohnzimmer".into(),
                aliases: vec!["wohnzimmer".into(), "living".into(), "ribingu".into()],
                floor_id: Some("ground".into()),
            },
            AreaRec {
                area_id: "office".into(),
                name: "Büro".into(),
                aliases: vec!["arbeitszimmer".into(), "office".into()],
                floor_id: Some("ground".into()),
            },
            AreaRec {
                area_id: "garden".into(),
                name: "Garten".into(),
                aliases: vec!["garten".into(), "garden".into()],
                floor_id: Some("ground".into()),
            },
            AreaRec {
                area_id: "basement".into(),
                name: "Keller".into(),
                aliases: vec!["keller".into(), "basement".into()],
                floor_id: Some("basement".into()),
            },
            AreaRec {
                area_id: "kitchen".into(),
                name: "Küche".into(),
                aliases: vec!["kuche".into(), "kitchen".into()],
                floor_id: Some("ground".into()),
            },
        ],
        entities: vec![
            rec("cover.living_blinds", "Rollo", "cover", "living", &["rollo"]),
            rec("light.living_ceiling", "Wohnzimmer Decke", "light", "living", &["decke", "deckenlampe"]),
            rec("light.living_globe", "Kugel", "light", "living", &["kugel", "globe"]),
            rec("light.living_lamp", "Wohnzimmer Lampe", "light", "living", &["lampe", "lamp"]),
            rec("light.office_ceiling", "Büro Decke", "light", "office", &["decke"]),
            rec("light.garden", "Garten Licht", "light", "garden", &["garten licht"]),
            rec("light.basement", "Keller Licht", "light", "basement", &["keller licht"]),
            rec("light.kitchen", "Küche Licht", "light", "kitchen", &["kuche licht"]),
        ],
        ..HomeGraph::default()
    }
}

fn parse_de(sentence: &str, home: HomeGraph) -> crate::types::ParseResult {
    parse(sentence, &home, &mut Session::new(), &[], &Settings::pinned("de"))
}

#[test]
fn trailing_particle_clause_cuts() {
    let _bound = crate::lang::bind_catalog(crate::lang::catalog_for(&["de".into(), "en".into()]));
    let home = leftover_home();
    let one = |words: &[&str]| words.iter().map(|word| (*word).to_string()).collect::<Vec<_>>();
    assert_eq!(split_clauses(&one(&["activate", "all", "off"]), &home).len(), 1);
    assert_eq!(split_clauses(&one(&["mach", "heizung", "schlafzimmer", "und", "wohnzimmer", "an"]), &home).len(), 1);
    assert_eq!(split_clauses(&one(&["wie", "warm", "ist", "wohnzimmer", "und", "mach", "lichter", "aus"]), &home).len(), 2);
    assert_eq!(split_clauses(&one(&["ist", "kinderzimmer", "an", "und", "mach", "lichter", "aus"]), &home).len(), 2);
}

#[test]
fn cover_and_ceiling_split_on_und() {
    let tokens = ["mach", "rollo", "im", "wohnzimmer", "an", "und", "decke", "im", "wohnzimmer", "an"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let clauses = split_clauses(&tokens, &leftover_home());
    assert_eq!(clauses.len(), 2, "{clauses:?}");
    let result = parse_de("mach rollo im wohnzimmer an und decke im wohnzimmer an", leftover_home());
    let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
    assert!(ids.contains(&"cover.living_blinds"), "{result:?}");
    assert!(ids.contains(&"light.living_ceiling"), "{result:?}");
    assert!(!ids.contains(&"light.office_ceiling"), "{result:?}");
}

#[test]
fn living_and_kitchen_off_stays_areas() {
    let result = parse_de("licht wohnzimmer und kuche aus", leftover_home());
    assert!(!result.clarify, "{result:?}");
    let hit = |needle: &str| {
        result
            .intents
            .iter()
            .any(|intent| intent.slot("area") == Some(needle) || intent.slot("entity_id").is_some_and(|id| id.contains(needle)))
    };
    assert!(hit("living") || hit("wohnzimmer"), "{result:?}");
    assert!(hit("kitchen") || hit("kuche"), "{result:?}");
    assert!(result.intents.iter().all(|intent| intent.name == "HassTurnOff"), "{result:?}");
}

#[test]
fn basement_does_not_drop_garden() {
    let result = parse_de("licht keller und garten an", leftover_home());
    let hit = |needle: &str| {
        result
            .intents
            .iter()
            .any(|intent| intent.slot("area") == Some(needle) || intent.slot("entity_id").is_some_and(|id| id.contains(needle)))
    };
    assert!(hit("garden") || hit("garten"), "{result:?}");
    assert!(hit("basement") || hit("keller"), "{result:?}");
}

#[test]
fn globe_and_ceiling_share_trailing_off() {
    let result = parse_de("kugel und decke aus", leftover_home());
    let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
    assert!(ids.contains(&"light.living_globe"), "{result:?}");
    assert!(ids.contains(&"light.living_ceiling"), "{result:?}");
    assert!(result.intents.iter().all(|intent| intent.name == "HassTurnOff"), "{result:?}");
}

#[test]
fn except_office_ceiling_skips_office_area() {
    let result = parse_de("alle lichter aus ausser decke im arbeitszimmer", leftover_home());
    assert!(!result.intents.iter().any(|intent| intent.slot("area") == Some("office")), "{result:?}");
    assert!(!result.intents.iter().any(|intent| intent.slot("entity_id") == Some("light.office_ceiling")), "{result:?}");
}

#[test]
fn except_lamp_token_skips_living_lamp() {
    let result = parse("keshite 全部 raito 以外 lamp ribingu", &leftover_home(), &mut Session::new(), &[], &Settings::pinned("ja"));
    assert!(!result.intents.iter().any(|intent| intent.slot("entity_id") == Some("light.living_lamp")), "{result:?}");
    assert!(
        result
            .intents
            .iter()
            .any(|intent| intent.slot("entity_id") == Some("light.living_ceiling") || intent.slot("area") == Some("living")),
        "{result:?}"
    );
}

#[test]
fn except_garden_turns_off_others() {
    let result = parse("keshite 全部 raito 以外 garden", &leftover_home(), &mut Session::new(), &[], &Settings::pinned("ja"));
    assert!(!result.clarify, "{result:?}");
    assert!(result.intents.iter().any(|intent| intent.name == "HassTurnOff"), "{result:?}");
    assert!(
        !result.intents.iter().any(|intent| intent.slot("area") == Some("garden") || intent.slot("entity_id") == Some("light.garden")),
        "{result:?}"
    );
    assert!(
        result
            .intents
            .iter()
            .any(|intent| intent.slot("area") == Some("living") || intent.slot("entity_id").is_some_and(|id| id.contains("living"))),
        "{result:?}"
    );
}

fn clauses_of(sentence: &str) -> Vec<String> {
    let _bound = crate::lang::bind_catalog(crate::lang::catalog_for(&["de".into(), "en".into()]));
    let tokens: Vec<String> = sentence.split_whitespace().map(str::to_string).collect();
    split_clauses(&tokens, &leftover_home()).iter().map(|clause| clause.join(" ")).collect()
}

#[test]
fn verb_final_clauses_split_where_each_particle_ends_its_clause() {
    for (sentence, expected) in [
        // Comma between clauses (gone after tokenising), then "und".
        (
            "mach licht schlafzimmer an mach rollo schlafzimmer zu und mach vorhang schlafzimmer zu",
            vec!["mach licht schlafzimmer an", "mach rollo schlafzimmer zu und", "mach vorhang schlafzimmer zu"],
        ),
        // "Rollo und Vorhang" share one particle and must stay together.
        (
            "mach licht schlafzimmer aus und rollo und vorhang schlafzimmer auf",
            vec!["mach licht schlafzimmer aus und", "rollo und vorhang schlafzimmer auf"],
        ),
        (
            "mach licht schlafzimmer an und rollo und vorhang schlafzimmer zu",
            vec!["mach licht schlafzimmer an und", "rollo und vorhang schlafzimmer zu"],
        ),
        (
            "mach rollo und vorhang schlafzimmer zu und licht schlafzimmer an",
            vec!["mach rollo und vorhang schlafzimmer zu und", "licht schlafzimmer an"],
        ),
        // No verb, comma only.
        ("licht schlafzimmer an rollo schlafzimmer zu", vec!["licht schlafzimmer an", "rollo schlafzimmer zu"]),
        // Two rooms.
        ("schalte licht wohnzimmer aus mach vorhang schlafzimmer zu", vec!["schalte licht wohnzimmer aus", "mach vorhang schlafzimmer zu"]),
    ] {
        assert_eq!(clauses_of(sentence), expected, "{sentence}");
    }
}

#[test]
fn verb_final_rule_leaves_single_clauses_alone() {
    for sentence in ["mach rollo und vorhang schlafzimmer zu", "mach licht schlafzimmer an", "mach heizung schlafzimmer und wohnzimmer an"]
    {
        assert_eq!(clauses_of(sentence).len(), 1, "{sentence}: {:?}", clauses_of(sentence));
    }
}

#[test]
fn verb_final_rule_ignores_verb_first_english() {
    let _bound = crate::lang::bind_catalog(crate::lang::catalog_for(&["de".into(), "en".into()]));
    let tokens = |sentence: &str| sentence.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    // "open" is a particle in English too, but it starts its clause, so the rule must not fire.
    let open = tokens("off bedroom light and open bedroom blinds and curtains");
    assert_eq!(verb_final_boundary(&open, 0, 4), None);
    let close = tokens("on bedroom light close bedroom blinds and close bedroom curtains");
    assert_eq!(verb_final_boundary(&close, 0, 3), None);
    assert_eq!(verb_final_boundary(&close, 3, 7), None);
}

#[test]
fn verb_final_rule_keeps_a_set_point_with_its_device() {
    // "auf 20" is a set-point: cutting after "auf" sent 20 to the light and left the heating unset.
    for (sentence, expected) in [
        (
            "stell heizung schlafzimmer auf 20 und mach licht schlafzimmer an",
            vec!["stell heizung schlafzimmer auf 20 und", "mach licht schlafzimmer an"],
        ),
        ("mach rollo schlafzimmer auf 50 und licht schlafzimmer an", vec!["mach rollo schlafzimmer auf 50", "licht schlafzimmer an"]),
    ] {
        assert_eq!(clauses_of(sentence), expected, "{sentence}");
    }
}
