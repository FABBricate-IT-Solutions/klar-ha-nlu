//! Plan 2.2 Mengen-Lexer: duration and temperature tables (DE/EN) plus quantity findings.
use klar_nlu::nlu::parse_with_policies;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, ParseDecision, Settings, SpeechBank};
use std::collections::HashSet;

fn area(id: &str, name: &str) -> AreaRec {
    AreaRec { area_id: id.into(), name: name.into(), aliases: vec![], floor_id: None }
}

fn ent(id: &str, name: &str, area: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: id.split_once('.').map(|(d, _)| d).unwrap_or("").into(),
        platform: None,
        area: (!area.is_empty()).then(|| area.into()),
        aliases: vec![],
        tags: vec![],
    }
}

fn home() -> HomeGraph {
    let entities = vec![
        ent("light.stehlampe", "Stehlampe", "wohnzimmer"),
        ent("light.floor_lamp", "Floor Lamp", "living_room"),
        ent("light.hall_light", "Hall Light", "hall"),
        ent("light.office_light", "Office Light", "office"),
        ent("light.flur_licht", "Flur Licht", "hall"),
        ent("climate.wohnzimmer", "Heizung Wohnzimmer", "wohnzimmer"),
        ent("climate.living_room", "Living Room Thermostat", "living_room"),
        ent("climate.bedroom", "Bedroom Thermostat", "bedroom"),
        ent("climate.office", "Office Thermostat", "office"),
        ent("climate.buro", "Heizung Büro", "buro"),
        ent("media_player.living_room_tv", "Living Room TV", "living_room"),
        ent("media_player.kitchen_speaker", "Kitchen Speaker", "kitchen"),
        ent("media_player.bedroom_speaker", "Bedroom Speaker", "bedroom"),
    ];
    let areas = vec![
        area("wohnzimmer", "Wohnzimmer"),
        area("living_room", "Living Room"),
        area("bedroom", "Bedroom"),
        area("office", "Office"),
        area("hall", "Hall"),
        area("kitchen", "Kitchen"),
        area("buro", "Büro"),
    ];
    let assist: HashSet<_> = entities.iter().map(|e| e.entity_id.clone()).collect();
    HomeGraph { areas, entities, assist: Some(assist), ..HomeGraph::default() }
}

fn parse(lang: &str, text: &str) -> klar_nlu::types::ParseOutcome {
    let mut session = Session::new();
    let mut settings = Settings::pinned(lang);
    settings.confirm_risky_actions = false;
    parse_with_policies(text, &home(), &mut session, &[], &settings, &[], &SpeechBank::default())
}

fn slot(out: &klar_nlu::types::ParseOutcome, name: &str) -> Option<String> {
    out.plan.as_ref()?.intents().first()?.slot(name).map(str::to_string)
}

fn intent_name(out: &klar_nlu::types::ParseOutcome) -> Option<String> {
    out.plan.as_ref()?.intents().first().map(|i| i.name.clone())
}

fn slots_of(out: &klar_nlu::types::ParseOutcome, index: usize) -> Vec<(String, String)> {
    out.plan
        .as_ref()
        .map(|p| p.intents())
        .unwrap_or_default()
        .get(index)
        .map(|i| i.slots.iter().map(|s| (s.name.clone(), s.value.clone())).collect())
        .unwrap_or_default()
}

#[test]
fn p04_relative_temperature_is_a_step_not_absolute() {
    let warmer = parse("de", "Heizung Wohnzimmer zwei Grad wärmer");
    assert_eq!(intent_name(&warmer).as_deref(), Some("HassClimateSetTemperature"), "{warmer:?}");
    assert_eq!(slot(&warmer, "temperature_step").as_deref(), Some("2"), "{warmer:?}");
    assert!(slot(&warmer, "temperature").is_none(), "{warmer:?}");

    let mut session = Session::new();
    let mut settings = Settings::pinned("en");
    settings.confirm_risky_actions = false;
    let home = home();
    let _ = parse_with_policies("set the living room thermostat to 21", &home, &mut session, &[], &settings, &[], &SpeechBank::default());
    let follow = parse_with_policies("one degree warmer", &home, &mut session, &[], &settings, &[], &SpeechBank::default());
    assert_eq!(intent_name(&follow).as_deref(), Some("HassClimateSetTemperature"), "{follow:?}");
    assert_eq!(slot(&follow, "temperature_step").as_deref(), Some("1"), "{follow:?}");
    assert_ne!(slot(&follow, "temperature").as_deref(), Some("1"), "{follow:?}");

    // Statement mood keeps "zu kalt" as a status ask; it must never turn the heat off (P-04).
    let cold = parse("de", "im Büro ist es zu kalt");
    assert_ne!(intent_name(&cold).as_deref(), Some("HassTurnOff"), "{cold:?}");
    assert!(
        matches!(intent_name(&cold).as_deref(), Some("HassGetState" | "HassClimateGetTemperature" | "HassClimateSetTemperature")),
        "{cold:?}"
    );
}

#[test]
fn p05_signs_decimals_and_clock_times() {
    let neg = parse("en", "set the living room thermostat to -5 degrees");
    assert_eq!(slot(&neg, "temperature").as_deref(), Some("-5"), "{neg:?}");

    let minus = parse("en", "set the living room thermostat to minus 5 degrees");
    assert_eq!(slot(&minus, "temperature").as_deref(), Some("-5"), "{minus:?}");

    let dec = parse("en", "set the bedroom thermostat to 19.5 degrees");
    assert_eq!(slot(&dec, "temperature").as_deref(), Some("19.5"), "{dec:?}");

    let de = parse("de", "Stell die Heizung Wohnzimmer auf 19,5 Grad");
    assert_eq!(slot(&de, "temperature").as_deref(), Some("19.5"), "{de:?}");

    let clock_en = parse("en", "set a timer for 1:30");
    assert_eq!(intent_name(&clock_en).as_deref(), Some("HassStartTimer"), "{clock_en:?}");
    assert_eq!(slot(&clock_en, "hours").as_deref(), Some("1"), "{clock_en:?}");
    assert_eq!(slot(&clock_en, "minutes").as_deref(), Some("30"), "{clock_en:?}");

    let clock_de = parse("de", "Timer auf 1:30");
    assert_eq!(slot(&clock_de, "hours").as_deref(), Some("1"), "{clock_de:?}");
    assert_eq!(slot(&clock_de, "minutes").as_deref(), Some("30"), "{clock_de:?}");
}

#[test]
fn p06_duration_tables_de_en() {
    let cases_en = [
        ("set a timer for half an hour", None, Some("30"), None),
        ("set a timer for an hour and a half", Some("1"), Some("30"), None),
        ("set a timer for one and a half hours", Some("1"), Some("30"), None),
        ("set a timer for two hours and thirty minutes", Some("2"), Some("30"), None),
        ("set a timer for a quarter of an hour", None, Some("15"), None),
    ];
    for (text, hours, minutes, seconds) in cases_en {
        let out = parse("en", text);
        assert_eq!(intent_name(&out).as_deref(), Some("HassStartTimer"), "{text}: {out:?}");
        assert_eq!(slot(&out, "hours").as_deref(), hours, "{text}: {out:?}");
        assert_eq!(slot(&out, "minutes").as_deref(), minutes, "{text}: {out:?}");
        assert_eq!(slot(&out, "seconds").as_deref(), seconds, "{text}: {out:?}");
    }

    let cases_de = [
        ("Timer auf 2 Stunden 30 Minuten", Some("2"), Some("30")),
        ("Timer auf zwei Stunden und dreißig Minuten", Some("2"), Some("30")),
        ("Timer auf eineinhalb Stunden", Some("1"), Some("30")),
    ];
    for (text, hours, minutes) in cases_de {
        let out = parse("de", text);
        assert_eq!(intent_name(&out).as_deref(), Some("HassStartTimer"), "{text}: {out:?}");
        assert_eq!(slot(&out, "hours").as_deref(), hours, "{text}: {out:?}");
        assert_eq!(slot(&out, "minutes").as_deref(), minutes, "{text}: {out:?}");
    }

    let zero = parse("en", "set a timer for 0 minutes");
    assert!(
        matches!(zero.decision, ParseDecision::Clarify { .. } | ParseDecision::Reject { .. })
            || slot(&zero, "minutes").as_deref() != Some("0"),
        "zero-length timer must not execute: {zero:?}"
    );
}

#[test]
fn p09_one_value_per_clause_and_color_with_brightness() {
    let temps = parse("en", "set the living room thermostat to 21 degrees and the bedroom thermostat to 18 degrees");
    let intents = temps.plan.as_ref().map(|p| p.intents()).unwrap_or_default();
    assert_eq!(intents.len(), 2, "{temps:?}");
    assert_eq!(intents[0].slot("temperature"), Some("21"), "{temps:?}");
    assert_eq!(intents[1].slot("temperature"), Some("18"), "{temps:?}");

    let lights = parse("de", "stell die Stehlampe auf 30 Prozent und das Flur Licht auf 80 Prozent");
    let intents = lights.plan.as_ref().map(|p| p.intents()).unwrap_or_default();
    assert_eq!(intents.len(), 2, "{lights:?}");
    assert_eq!(intents[0].slot("brightness"), Some("30"), "{lights:?}");
    assert_eq!(intents[1].slot("brightness"), Some("80"), "{lights:?}");

    let both = parse("en", "set the floor lamp to red and 30 percent");
    assert_eq!(intent_name(&both).as_deref(), Some("HassLightSet"), "{both:?}");
    assert_eq!(slot(&both, "color").as_deref(), Some("red"), "{both:?}");
    assert_eq!(slot(&both, "brightness").as_deref(), Some("30"), "{both:?}");

    let de_both = parse("de", "Stehlampe rot und 30 Prozent");
    assert_eq!(slot(&de_both, "color").as_deref(), Some("red"), "{de_both:?}");
    assert_eq!(slot(&de_both, "brightness").as_deref(), Some("30"), "{de_both:?}");
}

#[test]
fn p15_volume_out_of_range_rejects_and_no_silent_player() {
    let high = parse("en", "set the volume to 300");
    assert!(
        matches!(high.decision, ParseDecision::Reject { .. } | ParseDecision::Clarify { .. })
            || high.plan.as_ref().is_none_or(|p| p.intents().is_empty()),
        "volume 300 must not clamp to 100: {high:?}"
    );
    assert_ne!(slot(&high, "volume_level").as_deref(), Some("100"), "{high:?}");

    let de = parse("de", "Lautstärke auf 300");
    assert_ne!(slot(&de, "volume_level").as_deref(), Some("100"), "{de:?}");

    let bare = parse("en", "set the volume to 30 percent");
    // Ambiguous multi-player home: do not silently bind living_room_tv.
    if matches!(bare.decision, ParseDecision::Execute) {
        assert!(
            bare.plan.as_ref().is_some_and(|p| p.intents().iter().all(|i| i.slot("entity_id").is_none())),
            "must not silently pick a player: {bare:?}"
        );
    }
}

#[test]
fn p20_temp_unit_and_absurd_setpoint() {
    let f = parse("en", "set the living room thermostat to 70 fahrenheit");
    assert_eq!(slot(&f, "temperature").as_deref(), Some("21"), "{f:?}");

    let absurd = parse("en", "set the living room thermostat to 99 degrees");
    assert!(
        matches!(absurd.decision, ParseDecision::Reject { .. }) || absurd.plan.as_ref().is_none_or(|p| p.intents().is_empty()),
        "99 °C must not execute: {absurd:?}"
    );

    let mut session = Session::new();
    let mut settings = Settings::pinned("en");
    settings.unit_system = klar_nlu::types::UnitSystem::Imperial;
    settings.confirm_risky_actions = false;
    let out = parse_with_policies(
        "set the living room thermostat to 70 degrees",
        &home(),
        &mut session,
        &[],
        &settings,
        &[],
        &SpeechBank::default(),
    );
    // "degrees"/"Grad" follow the operator scale — 70 °F → 21 °C (P-20).
    assert_eq!(slot(&out, "temperature").as_deref(), Some("21"), "{out:?}");
}

#[test]
fn duration_and_temperature_slot_tables() {
    // Acceptance: exact slots for the DE/EN tables.
    assert_eq!(slots_of(&parse("en", "set a timer for half an hour"), 0), vec![("minutes".into(), "30".into())]);
    assert_eq!(
        slots_of(&parse("de", "Timer auf 2 Stunden 30 Minuten"), 0)
            .into_iter()
            .filter(|(k, _)| matches!(k.as_str(), "hours" | "minutes" | "seconds"))
            .collect::<Vec<_>>(),
        vec![("hours".into(), "2".into()), ("minutes".into(), "30".into())]
    );
    assert_eq!(
        slots_of(&parse("de", "Heizung Wohnzimmer zwei Grad wärmer"), 0).into_iter().find(|(k, _)| k == "temperature_step").map(|(_, v)| v),
        Some("2".into())
    );
    assert_eq!(
        slots_of(&parse("en", "set the bedroom thermostat to 19.5 degrees"), 0)
            .into_iter()
            .find(|(k, _)| k == "temperature")
            .map(|(_, v)| v),
        Some("19.5".into())
    );
}
