//! One sentence, several steps across lights and covers in the same room:
//! "Turn on the bedroom light, close the bedroom blinds and close the bedroom curtains".
//! A second room with its own light and blinds catches steps leaking across rooms.

use std::collections::BTreeSet;

use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::speech::render_snapshot;
use klar_nlu::types::SpeechSnapshot;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, Intent, ParseDecision, ParseOutcome, Settings};

fn area(id: &str, name: &str) -> AreaRec {
    AreaRec { area_id: id.into(), name: name.into(), aliases: vec![], floor_id: None }
}

fn entity(id: &str, name: &str, area: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: id.split_once('.').map(|(domain, _)| domain).unwrap_or("").into(),
        platform: None,
        area: Some(area.into()),
        aliases: vec![],
        tags: vec![],
    }
}

struct Room {
    lang: &'static str,
    home: HomeGraph,
    room: &'static str,
    covers: [&'static str; 2],
}

fn en() -> Room {
    Room {
        lang: "en",
        home: HomeGraph {
            areas: vec![area("bedroom", "Bedroom"), area("living_room", "Living Room")],
            entities: vec![
                entity("light.bedroom_light", "Bedroom Light", "bedroom"),
                entity("light.bedroom_lamp", "Bedroom Lamp", "bedroom"),
                entity("cover.bedroom_blinds", "Bedroom Blinds", "bedroom"),
                entity("cover.bedroom_curtains", "Bedroom Curtains", "bedroom"),
                entity("light.living_room", "Living Room Light", "living_room"),
                entity("cover.living_room_blinds", "Living Room Blinds", "living_room"),
            ],
            ..HomeGraph::default()
        },
        room: "bedroom",
        covers: ["cover.bedroom_blinds", "cover.bedroom_curtains"],
    }
}

fn de() -> Room {
    Room {
        lang: "de",
        home: HomeGraph {
            areas: vec![area("schlafzimmer", "Schlafzimmer"), area("wohnzimmer", "Wohnzimmer")],
            entities: vec![
                entity("light.schlafzimmer_decke", "Schlafzimmer Deckenlicht", "schlafzimmer"),
                entity("light.schlafzimmer_nachttisch", "Schlafzimmer Nachttischlampe", "schlafzimmer"),
                entity("cover.schlafzimmer_rollo", "Schlafzimmer Rollo", "schlafzimmer"),
                entity("cover.schlafzimmer_vorhang", "Schlafzimmer Vorhang", "schlafzimmer"),
                entity("light.wohnzimmer", "Wohnzimmer Licht", "wohnzimmer"),
                entity("cover.wohnzimmer_rollo", "Wohnzimmer Rollo", "wohnzimmer"),
            ],
            ..HomeGraph::default()
        },
        room: "schlafzimmer",
        covers: ["cover.schlafzimmer_rollo", "cover.schlafzimmer_vorhang"],
    }
}

const EN_LIGHT_ON_COVERS_CLOSED: &[&str] = &[
    "Turn on the bedroom light, close the bedroom blinds and close the bedroom curtains",
    "turn on the bedroom light and close the bedroom blinds and curtains",
    "close the bedroom blinds and curtains and turn on the bedroom light",
    "Turn on the bedroom light, then close the bedroom blinds and the bedroom curtains.",
];

const DE_LIGHT_ON_COVERS_CLOSED: &[&str] = &[
    "Mach das Licht im Schlafzimmer an und Rollo und Vorhang im Schlafzimmer zu",
    "Mach Rollo und Vorhang im Schlafzimmer zu und das Licht im Schlafzimmer an",
];

fn no_confirm(lang: &str) -> Settings {
    Settings { confirm_risky_actions: false, ..Settings::pinned(lang) }
}

fn intents(out: &ParseOutcome) -> Vec<Intent> {
    out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default()
}

fn domain(intent: &Intent) -> &str {
    intent.slot("domain").or_else(|| intent.slot("entity_id").and_then(|id| id.split_once('.').map(|(head, _)| head))).unwrap_or("")
}

fn in_room(intent: &Intent, room: &Room) -> bool {
    intent.slot("area") == Some(room.room)
        || intent
            .slot("entity_id")
            .is_some_and(|id| room.home.entities.iter().any(|entity| entity.entity_id == id && entity.area.as_deref() == Some(room.room)))
}

/// Exactly: a light turned on, both covers closed, all in this room, nothing else.
fn assert_light_on_covers_closed(text: &str, room: &Room, found: &[Intent]) {
    assert!(!found.is_empty(), "{text}: nothing planned");
    for intent in found {
        assert!(in_room(intent, room), "{text}: step leaked out of {}: {intent:?}", room.room);
        match (intent.name.as_str(), domain(intent)) {
            ("HassTurnOn", "light") | ("HassTurnOff", "cover") => {}
            _ => panic!("{text}: unexpected step {intent:?} in {found:?}"),
        }
    }
    assert!(found.iter().any(|i| i.name == "HassTurnOn" && domain(i) == "light"), "{text}: light not turned on: {found:?}");
    let room_wide_close = found.iter().any(|i| i.name == "HassTurnOff" && i.slot("area") == Some(room.room) && domain(i) == "cover");
    let closes = |id: &str| found.iter().any(|i| i.name == "HassTurnOff" && i.slot("entity_id") == Some(id));
    assert!(room_wide_close || room.covers.iter().all(|id| closes(id)), "{text}: both covers must close: {found:?}");
}

fn cases() -> Vec<(Room, &'static [&'static str])> {
    vec![(en(), EN_LIGHT_ON_COVERS_CLOSED), (de(), DE_LIGHT_ON_COVERS_CLOSED)]
}

#[test]
fn light_on_and_covers_closed_run_together_when_confirm_is_off() {
    for (room, texts) in cases() {
        for text in texts {
            let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm(room.lang));
            assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
            assert_light_on_covers_closed(text, &room, &intents(&out));
        }
    }
}

#[test]
fn default_settings_hold_the_whole_sentence_until_yes() {
    // The light must not switch on while the cover close waits for confirmation.
    for (room, texts) in cases() {
        let settings = Settings::pinned(room.lang);
        let yes = if room.lang == "de" { "ja" } else { "yes" };
        for text in texts {
            let mut session = Session::new();
            let asked = parse(text, &room.home, &mut session, &[], &settings);
            assert!(matches!(asked.decision, ParseDecision::Confirm { .. }), "{text}: {asked:?}");
            assert!(asked.plan.is_none(), "{text}: something ran before yes: {asked:?}");
            let done = parse(yes, &room.home, &mut session, &[], &settings);
            assert_eq!(done.decision, ParseDecision::Execute, "{text} → {yes}: {done:?}");
            assert_light_on_covers_closed(text, &room, &intents(&done));
        }
    }
}

const LIGHT_OFF_COVERS_OPEN: [(&str, &str); 2] = [
    ("en", "Turn off the bedroom light and open the bedroom blinds and curtains"),
    ("de", "Mach das Licht im Schlafzimmer aus und Rollo und Vorhang im Schlafzimmer auf"),
];

fn room_for(lang: &str) -> Room {
    if lang == "de" {
        de()
    } else {
        en()
    }
}

#[test]
fn room_wide_cover_open_asks_by_default() {
    // Any room-wide cover command counts as risky, opening included (risky_intent in validation.rs).
    for (lang, text) in LIGHT_OFF_COVERS_OPEN {
        let room = room_for(lang);
        let out = parse(text, &room.home, &mut Session::new(), &[], &Settings::pinned(lang));
        assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{text}: {out:?}");
        assert!(out.plan.is_none(), "{text}: nothing may run before yes: {out:?}");
    }
}

#[test]
fn light_off_and_covers_opened_run_together_when_confirm_is_off() {
    for (lang, text) in LIGHT_OFF_COVERS_OPEN {
        let room = room_for(lang);
        let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm(lang));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        let found = intents(&out);
        for intent in &found {
            assert!(in_room(intent, &room), "{text}: leaked: {intent:?}");
            match (intent.name.as_str(), domain(intent)) {
                ("HassTurnOff", "light") | ("HassTurnOn", "cover") => {}
                _ => panic!("{text}: unexpected step {intent:?} in {found:?}"),
            }
        }
        assert!(found.iter().any(|i| i.name == "HassTurnOff" && domain(i) == "light"), "{text}: {found:?}");
        assert!(found.iter().any(|i| i.name == "HassTurnOn" && domain(i) == "cover"), "{text}: {found:?}");
        // Several steps always get a spoken reply, never just the "Short confirmation" chime.
        assert!(!out.quiet_ack_eligible, "{text}: multi-step command must be spoken: {out:?}");
    }
}

#[test]
fn de_three_clauses_with_comma_close_both_covers() {
    let room = de();
    let text = "Mach das Licht im Schlafzimmer an, mach das Rollo im Schlafzimmer zu und mach den Vorhang im Schlafzimmer zu";
    let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm("de"));
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    assert_light_on_covers_closed(text, &room, &intents(&out));
}

/// German room with heating, for set-point sentences.
fn de_with_heating() -> Room {
    let mut room = de();
    room.home.entities.push(entity("climate.schlafzimmer", "Schlafzimmer Heizung", "schlafzimmer"));
    room
}

#[test]
fn de_set_point_then_light_sets_the_heating_not_the_light() {
    // "auf 20" must stay with the heating; it once became 20 % brightness on the light.
    let room = de_with_heating();
    for text in [
        "Stell die Heizung im Schlafzimmer auf 20 und mach das Licht im Schlafzimmer an",
        "Stell die Heizung im Schlafzimmer auf 20 Grad und das Licht im Schlafzimmer an",
    ] {
        let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm("de"));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        let found = intents(&out);
        let heat =
            found.iter().find(|i| i.name == "HassClimateSetTemperature").unwrap_or_else(|| panic!("{text}: no set-point: {found:?}"));
        assert_eq!(heat.slot("temperature"), Some("20"), "{text}: {found:?}");
        assert!(found.iter().any(|i| i.name == ON && domain(i) == "light"), "{text}: light not on: {found:?}");
        assert!(!found.iter().any(|i| i.name == "HassLightSet" || i.name == "HassGetState"), "{text}: wrong step: {found:?}");
    }
}

#[test]
fn de_curtain_open_clause_is_a_command_not_a_status_check() {
    // "Vorhang" is a curtain noun; "auf" with only a curtain once fell through to a status check.
    let room = de();
    for (text, wanted) in [
        ("Mach den Vorhang im Schlafzimmer auf", vec![(ON, "cover.schlafzimmer_vorhang")]),
        (
            "Mach das Rollo im Schlafzimmer zu und den Vorhang im Schlafzimmer auf",
            vec![(OFF, "cover.schlafzimmer_rollo"), (ON, "cover.schlafzimmer_vorhang")],
        ),
        (
            "Mach das Licht im Schlafzimmer aus, das Rollo im Schlafzimmer zu und den Vorhang im Schlafzimmer auf",
            vec![
                (OFF, "light.schlafzimmer_decke"),
                (OFF, "light.schlafzimmer_nachttisch"),
                (OFF, "cover.schlafzimmer_rollo"),
                (ON, "cover.schlafzimmer_vorhang"),
            ],
        ),
    ] {
        let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm("de"));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        assert_eq!(effects(&out, &room), expect(&wanted), "{text}\nplan: {:?}", intents(&out));
    }
}

#[test]
fn de_curtain_questions_stay_questions() {
    // "offen" and "geschlossen" are now open/close words; a question must not become a command.
    let room = de();
    for text in
        ["Ist der Vorhang im Schlafzimmer offen?", "Ist das Rollo im Schlafzimmer geschlossen?", "Ist der Vorhang im Schlafzimmer auf?"]
    {
        let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm("de"));
        let found = intents(&out);
        assert!(!found.is_empty() && found.iter().all(|i| i.name == "HassGetState"), "{text}: {found:?}");
    }
}

/// Devices a step acts on, as Home Assistant reports them back: the named entity,
/// or every device of that domain in the area.
fn acted_on<'a>(intent: &Intent, room: &'a Room) -> Vec<&'a EntityRec> {
    if let Some(id) = intent.slot("entity_id") {
        return room.home.entities.iter().filter(|entity| entity.entity_id == id).collect();
    }
    room.home.entities.iter().filter(|entity| entity.area.as_deref() == intent.slot("area") && entity.domain == domain(intent)).collect()
}

/// What you hear: each step rendered on its own, then joined with a space,
/// the same way `execute_plan` in custom_components/klar_nlu/executor.py builds the reply.
fn spoken_reply(out: &ParseOutcome, room: &Room) -> String {
    intents(out)
        .iter()
        .map(|intent| {
            let entities: Vec<_> = acted_on(intent, room)
                .into_iter()
                .map(|entity| {
                    serde_json::json!({
                        "entity_id": entity.entity_id, "name": entity.name, "domain": entity.domain, "state": "unknown"
                    })
                })
                .collect();
            let slots: Vec<_> = intent.slots.iter().map(|slot| serde_json::json!({"name": slot.name, "value": slot.value})).collect();
            let snapshot: SpeechSnapshot = serde_json::from_value(serde_json::json!({
                "language": room.lang, "outcome": "success",
                "intent": {"name": intent.name, "slots": slots},
                "entities": entities
            }))
            .expect("valid speech snapshot");
            render_snapshot(&snapshot).speech
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn light_off_and_covers_opened_reply_names_every_device() {
    for (room, text, expected) in [
        (
            en(),
            "Turn off the bedroom light and open the bedroom blinds and curtains",
            // "the bedroom light" means every light in the room, so the lamp is named too.
            "Bedroom Light and Bedroom Lamp are off. Opened Bedroom Blinds and Bedroom Curtains.",
        ),
        (
            de(),
            "Mach das Licht im Schlafzimmer aus und Rollo und Vorhang im Schlafzimmer auf",
            "Schlafzimmer Deckenlicht und Schlafzimmer Nachttischlampe sind aus. Schlafzimmer Rollo und Schlafzimmer Vorhang sind geöffnet.",
        ),
    ] {
        let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm(room.lang));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        assert_eq!(spoken_reply(&out, &room), expected, "{text}\nplan: {:?}", intents(&out));
    }
}

/// What Home Assistant would actually do: every (action, device) pair, however the plan
/// is shaped. One room-wide step and two named steps give the same effects.
fn effects(out: &ParseOutcome, room: &Room) -> BTreeSet<(String, String)> {
    intents(out)
        .iter()
        .flat_map(|intent| acted_on(intent, room).into_iter().map(move |entity| (intent.name.clone(), entity.entity_id.clone())))
        .collect()
}

fn expect(pairs: &[(&str, &str)]) -> BTreeSet<(String, String)> {
    pairs.iter().map(|(action, id)| ((*action).to_string(), (*id).to_string())).collect()
}

const ON: &str = "HassTurnOn";
const OFF: &str = "HassTurnOff";

#[test]
fn every_step_hits_exactly_the_right_devices_en() {
    let room = en();
    let lights = |action| [(action, "light.bedroom_light"), (action, "light.bedroom_lamp")];
    for (text, wanted) in [
        (
            "Turn on the bedroom light, close the bedroom blinds and close the bedroom curtains",
            [&lights(ON)[..], &[(OFF, "cover.bedroom_blinds"), (OFF, "cover.bedroom_curtains")]].concat(),
        ),
        (
            "close the bedroom blinds and curtains and turn on the bedroom light",
            [&lights(ON)[..], &[(OFF, "cover.bedroom_blinds"), (OFF, "cover.bedroom_curtains")]].concat(),
        ),
        (
            "Turn off the bedroom light and open the bedroom blinds and curtains",
            [&lights(OFF)[..], &[(ON, "cover.bedroom_blinds"), (ON, "cover.bedroom_curtains")]].concat(),
        ),
        ("turn on the bedroom lamp and close the bedroom blinds", vec![(ON, "light.bedroom_lamp"), (OFF, "cover.bedroom_blinds")]),
        ("close the bedroom curtains and turn off the bedroom lamp", vec![(OFF, "cover.bedroom_curtains"), (OFF, "light.bedroom_lamp")]),
        ("open the bedroom blinds and turn on the living room light", vec![(ON, "cover.bedroom_blinds"), (ON, "light.living_room")]),
        (
            "turn off the living room light and close the bedroom curtains",
            vec![(OFF, "light.living_room"), (OFF, "cover.bedroom_curtains")],
        ),
    ] {
        let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm("en"));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        assert_eq!(effects(&out, &room), expect(&wanted), "{text}\nplan: {:?}", intents(&out));
    }
}

#[test]
fn every_step_hits_exactly_the_right_devices_de() {
    let room = de();
    let lights = |action| [(action, "light.schlafzimmer_decke"), (action, "light.schlafzimmer_nachttisch")];
    let covers = |action| [(action, "cover.schlafzimmer_rollo"), (action, "cover.schlafzimmer_vorhang")];
    for (text, wanted) in [
        (
            "Mach das Licht im Schlafzimmer an, mach das Rollo im Schlafzimmer zu und mach den Vorhang im Schlafzimmer zu",
            [&lights(ON)[..], &covers(OFF)[..]].concat(),
        ),
        ("Mach das Licht im Schlafzimmer an und Rollo und Vorhang im Schlafzimmer zu", [&lights(ON)[..], &covers(OFF)[..]].concat()),
        ("Mach Rollo und Vorhang im Schlafzimmer zu und das Licht im Schlafzimmer an", [&lights(ON)[..], &covers(OFF)[..]].concat()),
        ("Mach das Licht im Schlafzimmer aus und Rollo und Vorhang im Schlafzimmer auf", [&lights(OFF)[..], &covers(ON)[..]].concat()),
        ("Licht im Schlafzimmer an, Rollo im Schlafzimmer zu", [&lights(ON)[..], &[(OFF, "cover.schlafzimmer_rollo")]].concat()),
        (
            "Mach die Nachttischlampe im Schlafzimmer an und den Vorhang im Schlafzimmer zu",
            vec![(ON, "light.schlafzimmer_nachttisch"), (OFF, "cover.schlafzimmer_vorhang")],
        ),
        (
            "Mach das Rollo im Wohnzimmer auf und das Licht im Schlafzimmer aus",
            [&[(ON, "cover.wohnzimmer_rollo")][..], &lights(OFF)[..]].concat(),
        ),
        (
            "Schalte das Licht im Wohnzimmer aus, mach den Vorhang im Schlafzimmer zu",
            vec![(OFF, "light.wohnzimmer"), (OFF, "cover.schlafzimmer_vorhang")],
        ),
    ] {
        let out = parse(text, &room.home, &mut Session::new(), &[], &no_confirm("de"));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        assert_eq!(effects(&out, &room), expect(&wanted), "{text}\nplan: {:?}", intents(&out));
    }
}
