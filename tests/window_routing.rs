//! Window questions go to window sensors only when the room has them; rooms whose
//! "windows" are blinds/curtains get a cover answer. English and German.
//!
//! Room layout (both languages):
//!   sensor room  – window contact + door contact, no covers
//!   blinds room  – blinds + curtains + a motion sensor (a non-window binary_sensor)
//!   both room    – window contact + blinds
//!   bare room    – a light and a motion sensor, no windows of any kind

use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, Intent, ParseDecision, ParseOutcome, Settings};

fn area(id: &str, name: &str) -> AreaRec {
    AreaRec { area_id: id.into(), name: name.into(), aliases: vec![], floor_id: None }
}

fn entity(id: &str, name: &str, area: Option<&str>) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: id.split_once('.').map(|(domain, _)| domain).unwrap_or("").into(),
        platform: None,
        area: area.map(Into::into),
        aliases: vec![],
        tags: vec![],
    }
}

fn en_home() -> HomeGraph {
    HomeGraph {
        areas: vec![area("kitchen", "Kitchen"), area("bedroom", "Bedroom"), area("office", "Office"), area("hall", "Hall")],
        entities: vec![
            entity("binary_sensor.kitchen_window", "Kitchen Window", Some("kitchen")),
            entity("binary_sensor.kitchen_door", "Kitchen Door", Some("kitchen")),
            entity("light.kitchen", "Kitchen Light", Some("kitchen")),
            entity("cover.bedroom_blinds", "Bedroom Blinds", Some("bedroom")),
            entity("cover.bedroom_curtains", "Bedroom Curtains", Some("bedroom")),
            entity("binary_sensor.bedroom_motion", "Bedroom Motion", Some("bedroom")),
            entity("light.bedroom_light", "Bedroom Light", Some("bedroom")),
            entity("light.bedroom_lamp", "Bedroom Lamp", Some("bedroom")),
            entity("binary_sensor.office_window", "Office Window", Some("office")),
            entity("cover.office_blinds", "Office Blinds", Some("office")),
            entity("light.hall", "Hall Light", Some("hall")),
            entity("binary_sensor.hall_motion", "Hall Motion", Some("hall")),
            entity("cover.garage_door", "Garage Door", None),
        ],
        ..HomeGraph::default()
    }
}

fn de_home() -> HomeGraph {
    HomeGraph {
        areas: vec![area("kuche", "Küche"), area("schlafzimmer", "Schlafzimmer"), area("buro", "Büro"), area("flur", "Flur")],
        entities: vec![
            entity("binary_sensor.kuche_fenster", "Küchenfenster", Some("kuche")),
            entity("binary_sensor.kuche_tur", "Küchentür", Some("kuche")),
            entity("light.kuche", "Küchenlicht", Some("kuche")),
            entity("cover.schlafzimmer_rollo", "Schlafzimmer Rollo", Some("schlafzimmer")),
            entity("cover.schlafzimmer_vorhang", "Schlafzimmer Vorhang", Some("schlafzimmer")),
            entity("binary_sensor.schlafzimmer_bewegung", "Schlafzimmer Bewegung", Some("schlafzimmer")),
            entity("light.schlafzimmer_decke", "Schlafzimmer Deckenlicht", Some("schlafzimmer")),
            entity("binary_sensor.buro_fenster", "Bürofenster", Some("buro")),
            entity("cover.buro_rollo", "Büro Rollo", Some("buro")),
            entity("light.flur", "Flurlicht", Some("flur")),
            entity("binary_sensor.flur_bewegung", "Flur Bewegung", Some("flur")),
            entity("cover.garagentor", "Garagentor", None),
        ],
        ..HomeGraph::default()
    }
}

fn run(text: &str, home: &HomeGraph, lang: &str) -> ParseOutcome {
    parse(text, home, &mut Session::new(), &[], &Settings::pinned(lang))
}

/// With the default "Confirm risky actions" setting, closing covers and setting their
/// position ask "Should I really do that?" first. Answer yes and return what executes.
fn through_confirm(text: &str, home: &HomeGraph, lang: &str, session: &mut Session) -> ParseOutcome {
    let settings = Settings::pinned(lang);
    let first = parse(text, home, session, &[], &settings);
    if !matches!(first.decision, ParseDecision::Confirm { .. }) {
        return first;
    }
    assert!(first.plan.is_none(), "{text}: plan must stay hidden until confirmed: {first:?}");
    let yes = if lang == "de" { "ja" } else { "yes" };
    parse(yes, home, session, &[], &settings)
}

fn intents(out: &ParseOutcome) -> Vec<Intent> {
    out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default()
}

fn is_window_list(intent: &Intent) -> bool {
    intent.slot("device_class") == Some("window")
}

/// The intent is about this room: an area slot, or an entity id that lives there.
fn targets_room(intent: &Intent, home: &HomeGraph, room: &str) -> bool {
    intent.slot("area") == Some(room)
        || intent
            .slot("entity_id")
            .is_some_and(|id| home.entities.iter().any(|entity| entity.entity_id == id && entity.area.as_deref() == Some(room)))
}

fn has_target(intent: &Intent) -> bool {
    intent.slot("area").is_some() || intent.slot("entity_id").is_some() || intent.slot("floor").is_some()
}

/// Window-sensor list for `room`, with the given open/closed filter.
fn assert_window_list(text: &str, home: &HomeGraph, lang: &str, room: Option<&str>, state: Option<&str>) {
    let out = run(text, home, lang);
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let found = intents(&out);
    assert_eq!(found.len(), 1, "{text}: {found:?}");
    let intent = &found[0];
    assert_eq!(intent.name, "HassGetState", "{text}");
    assert!(is_window_list(intent), "{text}: expected window sensors, got {intent:?}");
    assert_eq!(intent.slot("domain"), Some("binary_sensor"), "{text}");
    assert_eq!(intent.slot("area"), room, "{text}");
    assert_eq!(intent.slot("state"), state, "{text}");
}

/// A status question about `room`'s covers, never the window-sensor list.
fn assert_cover_status(text: &str, home: &HomeGraph, lang: &str, room: &str) {
    let out = run(text, home, lang);
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let found = intents(&out);
    assert!(!found.is_empty(), "{text}: no intent");
    for intent in &found {
        assert_eq!(intent.name, "HassGetState", "{text}: {intent:?}");
        assert!(!is_window_list(intent), "{text}: hijacked by window sensors: {intent:?}");
        assert!(targets_room(intent, home, room), "{text}: wrong room: {intent:?}");
    }
}

/// Never answers about the whole house or a different room.
fn assert_no_wrong_room(text: &str, home: &HomeGraph, lang: &str, room: &str) {
    let out = run(text, home, lang);
    for intent in intents(&out) {
        assert!(has_target(&intent) || !is_window_list(&intent), "{text}: whole-home answer for one room: {intent:?}");
        if has_target(&intent) {
            assert!(targets_room(&intent, home, room), "{text}: answered for another room: {intent:?}");
        }
    }
}

/// A command must not become a read-only window query.
fn assert_command_not_query(text: &str, home: &HomeGraph, lang: &str) -> Vec<Intent> {
    let out = run(text, home, lang);
    let found = intents(&out);
    for intent in &found {
        assert!(!is_window_list(intent), "{text}: command became a window query: {intent:?}");
        assert_ne!(intent.name, "HassGetState", "{text}: command became a query: {intent:?}");
    }
    found
}

// ===================== English =====================

#[test]
fn en_sensor_room_lists_window_sensors() {
    let home = en_home();
    for (text, state) in [
        ("are the kitchen windows open", Some("on")),
        ("is the kitchen window open", Some("on")),
        ("are the windows in the kitchen open?", Some("on")),
        ("are the kitchen windows closed", Some("off")),
        ("what's the status of the kitchen windows", None),
        ("kitchen window status", None),
    ] {
        assert_window_list(text, &home, "en", Some("kitchen"), state);
    }
}

#[test]
fn en_blinds_room_with_motion_sensor_answers_from_covers() {
    let home = en_home();
    for text in [
        "are the bedroom windows open",
        "are the windows in the bedroom open?",
        "what's the status of the bedroom windows?",
        "bedroom window status please",
        "tell me the current state of the bedroom windows",
        "if you don't mind, can you tell me if the bedroom windows are open",
        "ARE THE BEDROOM WINDOWS OPEN???",
    ] {
        assert_cover_status(text, &home, "en", "bedroom");
    }
}

#[test]
fn en_blinds_room_open_or_closed_answers_from_covers() {
    assert_cover_status("are the bedroom windows open or closed?", &en_home(), "en", "bedroom");
}

#[test]
fn en_blinds_room_mixed_nouns_answer_from_covers() {
    let home = en_home();
    for text in ["what's the status of the windows and blinds in the bedroom", "are the bedroom windows, blinds and curtains open"] {
        assert_cover_status(text, &home, "en", "bedroom");
    }
}

#[test]
fn en_room_with_sensor_and_blinds_prefers_the_sensor() {
    assert_window_list("are the office windows open", &en_home(), "en", Some("office"), Some("on"));
}

#[test]
fn en_open_or_closed_on_real_sensors_is_not_guessed() {
    let out = run("are the kitchen windows open or closed?", &en_home(), "en");
    assert!(intents(&out).iter().all(|intent| intent.slot("state").is_none()), "must not pick one state: {out:?}");
}

#[test]
fn en_whole_home_questions_use_sensors() {
    let home = en_home();
    assert_window_list("are any windows open", &home, "en", None, Some("on"));
    assert_window_list("which windows are open", &home, "en", None, Some("on"));
    assert_window_list("are all the windows closed", &home, "en", None, Some("off"));
}

#[test]
fn en_bare_room_never_answers_for_another_room() {
    let home = en_home();
    for text in ["are the hall windows open", "hall window status"] {
        assert_no_wrong_room(text, &home, "en", "hall");
    }
}

#[test]
fn en_unknown_room_is_not_a_whole_home_answer() {
    let home = en_home();
    for text in ["are the attic windows open", "what's the status of the windows in the attic"] {
        let out = run(text, &home, "en");
        for intent in intents(&out) {
            assert!(has_target(&intent), "{text}: unknown room became whole home: {intent:?}");
        }
    }
}

#[test]
fn en_two_rooms_are_not_silently_dropped() {
    let out = run("are the bedroom and kitchen windows open", &en_home(), "en");
    assert!(!matches!(out.decision, ParseDecision::Reject { .. }), "{out:?}");
}

#[test]
fn en_window_commands_stay_commands() {
    let home = en_home();
    for text in ["close the bedroom windows", "open the bedroom windows", "open the kitchen window", "shut the bedroom windows"] {
        assert_command_not_query(text, &home, "en");
    }
}

#[test]
fn en_close_blinds_and_curtains_targets_both() {
    let home = en_home();
    let text = "Close the bedroom blinds and curtains.";
    let out = through_confirm(text, &home, "en", &mut Session::new());
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let found = intents(&out);
    assert!(found.iter().all(|intent| !is_window_list(intent)), "{found:?}");
    assert!(found.iter().all(|intent| intent.name == "HassTurnOff"), "{found:?}");
    let covers_area = found.iter().any(|intent| intent.slot("area") == Some("bedroom") && intent.slot("domain") == Some("cover"));
    let named = |id: &str| found.iter().any(|intent| intent.slot("entity_id") == Some(id));
    assert!(covers_area || (named("cover.bedroom_blinds") && named("cover.bedroom_curtains")), "both covers: {found:?}");
}

#[test]
fn en_follow_up_sets_blind_position() {
    let home = en_home();
    let settings = Settings::pinned("en");
    let mut session = Session::new();
    let first = parse("what's the position of the bedroom windows?", &home, &mut session, &[], &settings);
    assert_eq!(first.decision, ParseDecision::Execute, "{first:?}");
    let out = through_confirm("set them to 50", &home, "en", &mut session);
    assert_eq!(out.decision, ParseDecision::Execute, "{out:?}");
    let found = intents(&out);
    assert!(found.iter().all(|intent| intent.name == "HassSetPosition" && targets_room(intent, &home, "bedroom")), "{found:?}");
}

#[test]
fn en_garage_door_is_never_a_window() {
    let out = run("is the garage door open", &en_home(), "en");
    assert!(intents(&out).iter().all(|intent| !is_window_list(intent)), "{out:?}");
}

// ===================== German =====================

#[test]
fn de_sensor_room_lists_window_sensors() {
    let home = de_home();
    for (text, state) in [
        ("Sind die Fenster in der Küche auf?", Some("on")),
        ("Sind die Fenster in der Küche offen?", Some("on")),
        ("Ist das Fenster in der Küche offen?", Some("on")),
        ("Sind die Fenster in der Küche zu?", Some("off")),
        ("Sind die Fenster in der Küche geschlossen?", Some("off")),
        ("Wie ist der Status der Fenster in der Küche?", None),
    ] {
        assert_window_list(text, &home, "de", Some("kuche"), state);
    }
}

#[test]
fn de_ascii_room_spelling_matches_umlaut_room() {
    assert_window_list("Sind die Fenster in der Kueche auf?", &de_home(), "de", Some("kuche"), Some("on"));
    assert_window_list("Sind die Fenster im Buero auf?", &de_home(), "de", Some("buro"), Some("on"));
}

#[test]
fn de_compound_kuechenfenster_question_is_answered() {
    let home = de_home();
    let text = "Ist das Küchenfenster offen?";
    let out = run(text, &home, "de");
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let found = intents(&out);
    assert!(found.iter().all(|intent| intent.name == "HassGetState" && targets_room(intent, &home, "kuche")), "{found:?}");
}

#[test]
fn de_blinds_room_with_motion_sensor_answers_from_covers() {
    let home = de_home();
    for text in [
        "Sind die Fenster im Schlafzimmer auf?",
        "Sind die Fenster im Schlafzimmer offen?",
        "Wie ist der Status der Fenster im Schlafzimmer?",
        "Sind die Fenster im Schlafzimmer zu?",
    ] {
        assert_cover_status(text, &home, "de", "schlafzimmer");
    }
}

#[test]
fn de_blinds_room_auf_oder_zu_answers_from_covers() {
    assert_cover_status("Sind die Fenster im Schlafzimmer auf oder zu?", &de_home(), "de", "schlafzimmer");
}

#[test]
fn de_room_with_sensor_and_blinds_prefers_the_sensor() {
    assert_window_list("Sind die Fenster im Büro auf?", &de_home(), "de", Some("buro"), Some("on"));
}

#[test]
fn de_whole_home_questions_use_sensors() {
    let home = de_home();
    assert_window_list("Sind Fenster offen?", &home, "de", None, Some("on"));
    assert_window_list("Welche Fenster sind auf?", &home, "de", None, Some("on"));
    assert_window_list("Sind alle Fenster zu?", &home, "de", None, Some("off"));
}

#[test]
fn de_bare_room_never_answers_for_another_room() {
    assert_no_wrong_room("Sind die Fenster im Flur auf?", &de_home(), "de", "flur");
}

#[test]
fn de_unknown_room_is_not_a_whole_home_answer() {
    let out = run("Sind die Fenster im Dachboden auf?", &de_home(), "de");
    for intent in intents(&out) {
        assert!(has_target(&intent), "unknown room became whole home: {intent:?}");
    }
}

#[test]
fn de_window_commands_stay_commands() {
    let home = de_home();
    for text in ["Mach die Fenster im Schlafzimmer zu", "Schließ die Fenster im Schlafzimmer", "Mach die Fenster im Schlafzimmer auf"] {
        assert_command_not_query(text, &home, "de");
    }
}

#[test]
fn de_close_rollo_und_vorhang_targets_both() {
    let home = de_home();
    let text = "Mach Rollo und Vorhang im Schlafzimmer zu";
    let out = through_confirm(text, &home, "de", &mut Session::new());
    assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
    let found = intents(&out);
    assert!(found.iter().all(|intent| !is_window_list(intent)), "{found:?}");
    assert!(found.iter().all(|intent| intent.name == "HassTurnOff"), "{found:?}");
    let covers_area = found.iter().any(|intent| intent.slot("area") == Some("schlafzimmer") && intent.slot("domain") == Some("cover"));
    let named = |id: &str| found.iter().any(|intent| intent.slot("entity_id") == Some(id));
    assert!(covers_area || (named("cover.schlafzimmer_rollo") && named("cover.schlafzimmer_vorhang")), "both covers: {found:?}");
}

#[test]
fn de_follow_up_sets_rollo_position() {
    let home = de_home();
    let settings = Settings::pinned("de");
    let mut session = Session::new();
    let first = parse("Wie ist die Position der Fenster im Schlafzimmer?", &home, &mut session, &[], &settings);
    assert_eq!(first.decision, ParseDecision::Execute, "{first:?}");
    let out = through_confirm("Stell sie auf 50 Prozent", &home, "de", &mut session);
    assert_eq!(out.decision, ParseDecision::Execute, "{out:?}");
    let found = intents(&out);
    assert!(found.iter().all(|intent| intent.name == "HassSetPosition" && targets_room(intent, &home, "schlafzimmer")), "{found:?}");
}

#[test]
fn de_garagentor_is_never_a_window() {
    let out = run("Ist das Garagentor offen?", &de_home(), "de");
    assert!(intents(&out).iter().all(|intent| !is_window_list(intent)), "{out:?}");
}

// ===================== "Confirm risky actions" setting =====================

fn no_confirm(lang: &str) -> Settings {
    Settings { confirm_risky_actions: false, ..Settings::pinned(lang) }
}

/// Both bedroom covers are closed in one go: room-wide, or each one by id.
fn closes_both(found: &[Intent], room: &str, ids: [&str; 2]) -> bool {
    let room_wide = found.iter().any(|intent| intent.slot("area") == Some(room) && intent.slot("domain") == Some("cover"));
    let named = |id: &str| found.iter().any(|intent| intent.slot("entity_id") == Some(id));
    found.iter().all(|intent| intent.name == "HassTurnOff") && (room_wide || ids.iter().all(|id| named(id)))
}

#[test]
fn default_settings_ask_before_closing_covers() {
    for (text, lang, home) in
        [("Close the bedroom blinds and curtains.", "en", en_home()), ("Mach Rollo und Vorhang im Schlafzimmer zu", "de", de_home())]
    {
        let out = run(text, &home, lang);
        assert!(matches!(out.decision, ParseDecision::Confirm { .. }), "{text}: default must ask first: {out:?}");
        assert!(out.plan.is_none(), "{text}: nothing may run before yes: {out:?}");
    }
}

#[test]
fn confirm_off_closes_blinds_and_curtains_immediately() {
    for (text, lang, home, room, ids) in [
        ("Close the bedroom blinds and curtains.", "en", en_home(), "bedroom", ["cover.bedroom_blinds", "cover.bedroom_curtains"]),
        (
            "Mach Rollo und Vorhang im Schlafzimmer zu",
            "de",
            de_home(),
            "schlafzimmer",
            ["cover.schlafzimmer_rollo", "cover.schlafzimmer_vorhang"],
        ),
    ] {
        let out = parse(text, &home, &mut Session::new(), &[], &no_confirm(lang));
        assert_eq!(out.decision, ParseDecision::Execute, "{text}: {out:?}");
        assert!(closes_both(&intents(&out), room, ids), "{text}: {out:?}");
    }
}

#[test]
fn confirm_off_sets_blind_position_immediately() {
    for (question, follow_up, lang, home, room) in [
        ("what's the position of the bedroom windows?", "set them to 50", "en", en_home(), "bedroom"),
        ("Wie ist die Position der Fenster im Schlafzimmer?", "Stell sie auf 50 Prozent", "de", de_home(), "schlafzimmer"),
    ] {
        let settings = no_confirm(lang);
        let mut session = Session::new();
        let first = parse(question, &home, &mut session, &[], &settings);
        assert_eq!(first.decision, ParseDecision::Execute, "{question}: {first:?}");
        let out = parse(follow_up, &home, &mut session, &[], &settings);
        assert_eq!(out.decision, ParseDecision::Execute, "{follow_up}: {out:?}");
        let found = intents(&out);
        assert!(found.iter().all(|intent| intent.name == "HassSetPosition" && targets_room(intent, &home, room)), "{follow_up}: {found:?}");
    }
}

#[test]
fn confirm_off_never_changes_read_only_answers() {
    // Turning confirmations off must not change what a question does.
    for (text, lang, home, room) in [
        ("are the bedroom windows open", "en", en_home(), "bedroom"),
        ("Sind die Fenster im Schlafzimmer auf?", "de", de_home(), "schlafzimmer"),
    ] {
        let with = run(text, &home, lang);
        let without = parse(text, &home, &mut Session::new(), &[], &no_confirm(lang));
        assert_eq!(intents(&with), intents(&without), "{text}");
        assert!(intents(&without).iter().all(|intent| intent.name == "HassGetState" && targets_room(intent, &home, room)), "{text}");
    }
}
