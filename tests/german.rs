use klar_nlu::lexicon::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, Settings};

fn run(text: &str) -> (Vec<String>, bool) {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    (result.intents.iter().map(|i| i.name.clone()).collect(), result.clarify)
}

fn slots(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let home = default_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    result.intents.into_iter().map(|i| (i.name, i.slots.into_iter().map(|s| (s.name, s.value)).collect())).collect()
}

#[test]
fn wohnzimmer_licht_an() {
    let (names, _) = run("Mach das Licht im Wohnzimmer an");
    assert_eq!(names, vec!["HassTurnOn"]);
}

#[test]
fn zwei_raeume_und_heizung() {
    let found = slots("Mach das Licht im Wohnzimmer und in der Küche an und stell die Heizung auf 23");
    let names: Vec<_> = found.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names.iter().filter(|n| **n == "HassTurnOn").count(), 2, "{found:?}");
    assert!(names.contains(&"HassClimateSetTemperature"));
    let areas: Vec<_> = found
        .iter()
        .filter(|(n, _)| n == "HassTurnOn")
        .flat_map(|(_, slots)| slots.iter().filter(|(k, _)| k == "area").map(|(_, v)| v.as_str()))
        .collect();
    assert!(
        areas.contains(&"wohnzimmer") || found.iter().any(|(_, s)| { s.iter().any(|(k, v)| k == "entity_id" && v.contains("wohnzimmer")) })
    );
}

#[test]
fn temperatur_wohnung() {
    let found = slots("Wie warm ist es in der Wohnung");
    assert_eq!(found[0].0, "HassGetState");
    assert!(found[0].1.iter().any(|(k, v)| k == "area" && v == "wohnung"));
}

#[test]
fn alle_lichter_aus() {
    let found = slots("Alle Lichter aus");
    assert_eq!(found[0].0, "HassTurnOff");
}

#[test]
fn heizung_zahlwort() {
    let found = slots("Heizung Wohnzimmer auf dreiundzwanzig Grad");
    assert_eq!(found[0].0, "HassClimateSetTemperature");
    assert!(found[0].1.iter().any(|(k, v)| k == "temperature" && v == "23"));
}

#[test]
fn staubsauger() {
    let (names, _) = run("R2D2 soll bitte saugen");
    assert_eq!(names, vec!["HassVacuumStart"]);
}

#[test]
fn licht_nicht_mehrdeutig() {
    let (names, clarify) = run("Mach das Licht im Wohnzimmer an");
    assert!(!clarify);
    assert_eq!(names, vec!["HassTurnOn"]);
}

#[test]
fn licht_ohne_raum_fragt_nach() {
    let (names, clarify) = run("Mach das Licht an");
    assert!(clarify, "{names:?}");
    assert!(names.is_empty(), "{names:?}");
}

#[test]
fn sauger_status_dock_kein_return() {
    let found = slots("Ist R2D2 am Dock");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "HassGetState");
}

#[test]
fn timer_nutzt_minutes() {
    let found = slots("Stell einen Timer auf fünf Minuten");
    assert_eq!(found[0].0, "HassStartTimer", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "minutes" && v == "5"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "duration"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "entity_id"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "domain"), "{found:?}");
}

#[test]
fn timer_abbrechen() {
    let found = slots("Timer abbrechen");
    assert_eq!(found[0].0, "HassCancelTimer", "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "minutes"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "hours"), "{found:?}");
    assert!(!found[0].1.iter().any(|(k, _)| k == "seconds"), "{found:?}");
}

#[test]
fn timer_aus_bricht_ab() {
    let found = slots("Timer aus");
    assert_eq!(found[0].0, "HassCancelTimer", "{found:?}");
}

#[test]
fn cancel_the_timer() {
    let found = slots("Cancel the timer");
    assert_eq!(found[0].0, "HassCancelTimer", "{found:?}");
}

#[test]
fn timer_pausieren() {
    let found = slots("Timer pausieren");
    assert_eq!(found[0].0, "HassPauseTimer", "{found:?}");
}

#[test]
fn timer_eine_minute() {
    let found = slots("Stell einen Timer auf eine Minute");
    assert_eq!(found[0].0, "HassStartTimer", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "minutes" && v == "1"), "{found:?}");
}

#[test]
fn timer_haengt_fremden_helper_nicht_an() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "timer.5min_warten".into(),
        name: "5min warten".into(),
        domain: "timer".into(),
        area: None,
        aliases: vec!["5min".into()],
        tags: Vec::new(),
    });
    let mut session = Session::new();
    let result = parse("Stell einen Timer auf 5 Minuten", &home, &mut session, &[], &Settings::default());
    let slots: Vec<_> = result.intents[0].slots.iter().map(|s| (s.name.as_str(), s.value.as_str())).collect();
    assert_eq!(result.intents[0].name, "HassStartTimer", "{slots:?}");
    assert!(slots.iter().any(|(k, v)| *k == "minutes" && *v == "5"), "{slots:?}");
    assert!(!slots.iter().any(|(k, v)| *k == "entity_id" && *v == "timer.5min_warten"), "{slots:?}");
}

#[test]
fn einkaufsliste_nutzt_todo_ohne_shopping_list_name() {
    let mut home = default_home();
    home.entities.push(EntityRec {
        entity_id: "todo.einkaufsliste".into(),
        name: "Einkaufsliste".into(),
        domain: "todo".into(),
        area: None,
        aliases: vec!["einkaufsliste".into(), "einkauf".into()],
        tags: Vec::new(),
    });
    let mut session = Session::new();
    let result = parse("Setze Milch auf die Einkaufsliste", &home, &mut session, &[], &Settings::default());
    let slots: Vec<_> = result.intents[0].slots.iter().map(|s| (s.name.as_str(), s.value.as_str())).collect();
    assert_eq!(result.intents[0].name, "HassListAddItem", "{slots:?}");
    assert!(slots.iter().any(|(k, v)| *k == "entity_id" && *v == "todo.einkaufsliste"), "{slots:?}");
    assert!(!slots.iter().any(|(k, v)| *k == "name" && *v == "shopping_list"), "{slots:?}");
}

#[test]
fn einkaufsliste_heisst_list_add() {
    let found = slots("Setze Milch auf die Einkaufsliste");
    assert_eq!(found[0].0, "HassListAddItem", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "item" && v.contains("milch")), "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "name" && v == "shopping_list"), "{found:?}");
}

#[test]
fn smalltalk_hat_keinen_home_intent() {
    for text in ["Was ist die Hauptstadt von Frankreich", "Erzähl einen Katzenwitz", "asdfghjkl qwerty"] {
        let (names, clarify) = run(text);
        assert!(!clarify, "{text}: {names:?}");
        assert!(names.is_empty(), "{text}: {names:?}");
    }
}

#[test]
fn casual_und_sonderfaelle_gehen_an_llm() {
    let home = default_home();
    let settings = Settings::default();
    for text in [
        "Erzähle eine Geschichte",
        "Erzähle einen Witz",
        "Erzähl einen Katzenwitz",
        "Wie geht es dir",
        "Guten Morgen",
        "Danke",
        "Was ist die Hauptstadt von Frankreich",
        "Wie ist das Wetter",
        "Was soll ich kochen",
        "Wer bist du",
        "Unterhalte mich",
    ] {
        let mut session = Session::new();
        let result = parse(text, &home, &mut session, &[], &settings);
        assert!(!result.clarify, "{text}: {}", result.speech);
        assert!(result.intents.is_empty(), "{text}: {:?}", result.intents);
        assert!(result.chat, "{text}: chat fehlt");
    }
    for text in ["Licht im Wohnzimmer an", "Wie ist der Status der Küche", "Wie warm ist es im Schlafzimmer", "asdfghjkl qwerty"] {
        let mut session = Session::new();
        let result = parse(text, &home, &mut session, &[], &settings);
        assert!(!result.chat, "{text}: NLU/Müll darf nicht chat sein {:?}", result.intents);
    }
}

#[test]
fn smalltalk_nach_geraet_geht_an_llm() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings::default();
    let first = parse("Licht im Arbeitszimmer an", &home, &mut session, &[], &settings);
    assert_eq!(first.intents[0].name, "HassTurnOn", "{}", first.speech);
    for text in ["Erzähle mir eine Geschichte", "Erzähl einen Katzenwitz", "Was ist die Hauptstadt von Frankreich"] {
        let result = parse(text, &home, &mut session, &[], &settings);
        assert!(!result.clarify, "{text}: {}", result.speech);
        assert!(result.intents.is_empty(), "{text}: {:?} {}", result.intents, result.speech);
    }
    let status = parse("wie ist der Status?", &home, &mut session, &[], &settings);
    assert_eq!(status.intents[0].name, "HassGetState", "{} {:?}", status.speech, status.intents);
    let off = parse("mach es aus", &home, &mut session, &[], &settings);
    assert_eq!(off.intents[0].name, "HassTurnOff", "{}", off.speech);
}

#[test]
fn schlafzimmerlicht_auf_prozent() {
    let found = slots("Schlafzimmerlicht auf 50%");
    assert_eq!(found[0].0, "HassLightSet", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "brightness" && v == "50"), "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "entity_id" && v == "light.schlafzimmer_kugel"), "{found:?}");
}

#[test]
fn schlafzimmerlicht_status_ist_das_licht() {
    let found = slots("Wie ist der Status vom Schlafzimmerlicht");
    assert_eq!(found[0].0, "HassGetState", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "entity_id" && v == "light.schlafzimmer_kugel"), "{found:?}");
    assert!(found[0].1.iter().all(|(k, _)| k != "area"), "{found:?}");
}

#[test]
fn schlafzimmerlicht_an_ohne_raumwort() {
    let found = slots("schalte das schlafzimmerlicht an");
    assert_eq!(found[0].0, "HassTurnOn", "{found:?}");
    assert!(
        found[0].1.iter().any(|(k, v)| k == "entity_id" && v == "light.schlafzimmer_kugel")
            || found[0].1.iter().any(|(k, v)| k == "area" && v == "schlafzimmer"),
        "{found:?}"
    );
    assert!(!found[0].1.is_empty(), "{found:?}");
}

#[test]
fn schlafzimmerlicht_im_raum() {
    let found = slots("schalte im schlafzimmer das schlafzimmerlicht an");
    assert_eq!(found[0].0, "HassTurnOn", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "entity_id" && v == "light.schlafzimmer_kugel"), "{found:?}");
}

#[test]
fn kuechenlicht_trifft_raum() {
    let found = slots("Küchenlicht an");
    assert_eq!(found[0].0, "HassTurnOn", "{found:?}");
    assert!(found[0].1.iter().any(|(k, v)| k == "area" && v == "kuche") || found[0].1.iter().any(|(k, _)| k == "entity_id"), "{found:?}");
}

/// Live-Wohnung: Hue-Raum `light.schlafzimmer` ist nur die Kugel.
/// Die Gruppe `Schlafzimmer Licht` darf nicht das Ziel sein.
#[test]
fn schlafzimmerlicht_trifft_hue_kugel_nicht_gruppe() {
    let mut home = default_home();
    home.entities.retain(|e| e.entity_id != "light.schlafzimmer_kugel");
    home.entities.push(EntityRec {
        entity_id: "light.schlafzimmer".into(),
        name: "Kugel".into(),
        domain: "light".into(),
        area: Some("schlafzimmer".into()),
        aliases: vec!["kugel".into()],
        tags: Vec::new(),
    });
    home.entities.push(EntityRec {
        entity_id: "light.hue_color_lamp_2".into(),
        name: "Kugel".into(),
        domain: "light".into(),
        area: Some("schlafzimmer".into()),
        aliases: vec!["kugel".into()],
        tags: Vec::new(),
    });
    let mut session = Session::new();
    let result = parse("Schlafzimmerlicht auf 50%", &home, &mut session, &[], &Settings::default());
    let slots: Vec<_> = result.intents[0].slots.iter().map(|s| (s.name.as_str(), s.value.as_str())).collect();
    assert_eq!(result.intents[0].name, "HassLightSet", "{slots:?}");
    assert!(slots.contains(&("brightness", "50")), "{slots:?}");
    assert!(slots.contains(&("entity_id", "light.schlafzimmer")), "{slots:?}");
    assert!(!slots.iter().any(|(k, v)| *k == "entity_id" && *v == "light.schlafzimmer_licht"), "{slots:?}");
}

#[test]
fn follow_up_aus() {
    let home = default_home();
    let mut session = Session::new();
    let settings = Settings::default();
    parse("Licht im Wohnzimmer an", &home, &mut session, &[], &settings);
    let second = parse("mach sie aus", &home, &mut session, &[], &settings);
    assert_eq!(second.intents[0].name, "HassTurnOff");
    let third = parse("schalte es wieder an", &home, &mut session, &[], &settings);
    assert_eq!(third.intents[0].name, "HassTurnOn", "{:?} {}", third.intents, third.speech);
}

fn tagged(id: &str, name: &str, domain: &str, area: &str, tags: &[&str]) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: domain.into(),
        area: Some(area.into()),
        aliases: vec![name.to_ascii_lowercase()],
        tags: tags.iter().map(|t| (*t).to_string()).collect(),
    }
}

fn role_home() -> klar_nlu::types::HomeGraph {
    let mut home = default_home();
    home.areas.push(AreaRec { area_id: "abstellkammer".into(), name: "Abstellkammer".into(), aliases: vec!["abstell".into()] });
    home.entities.push(tagged("switch.abstell_steckdose", "Abstellkammer", "switch", "abstellkammer", &["licht"]));
    home.entities.push(tagged("switch.balkon_heizstab", "Heizstab", "switch", "balkon", &["heizung"]));
    home.entities.push(tagged("switch.kuche_radio", "Küchenradio", "switch", "kuche", &["tv"]));
    home.entities.push(tagged("switch.flur_wichtig", "Flur Kiste", "switch", "flur", &["wichtig"]));
    home.entities.push(tagged("switch.esszimmer_box", "Lüfterbox", "switch", "esszimmer", &["lüfter"]));
    home
}

fn role_slots(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let home = role_home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &Settings::default());
    result.intents.into_iter().map(|i| (i.name, i.slots.into_iter().map(|s| (s.name, s.value)).collect())).collect()
}

fn has_entity(found: &[(String, Vec<(String, String)>)], id: &str) -> bool {
    found.iter().any(|(_, slots)| slots.iter().any(|(k, v)| k == "entity_id" && v == id))
}

#[test]
fn licht_tag_macht_steckdose_zum_licht() {
    let found = role_slots("Licht in der Abstellkammer an");
    assert!(found.iter().any(|(n, _)| n == "HassTurnOn"), "{found:?}");
    assert!(has_entity(&found, "switch.abstell_steckdose"), "{found:?}");
}

#[test]
fn adaptive_lighting_ist_kein_licht() {
    let mut home = default_home();
    home.entities.push(tagged(
        "switch.adaptive_lighting_adaptiv_wohnzimmer_adaptive_lighting_sleep_mode_adaptiv_wohnzimmer",
        "Adaptive Lighting Sleep Mode: Adaptiv Wohnzimmer",
        "switch",
        "wohnzimmer",
        &["licht"],
    ));
    home.entities.push(tagged(
        "switch.adaptiv_wohnzimmer_adaptive_lighting_adaptiv_wohnzimmer",
        "Adaptive Lighting: Adaptiv Wohnzimmer",
        "switch",
        "wohnzimmer",
        &["licht"],
    ));
    let mut session = Session::new();
    let first = parse("Licht im Wohnzimmer an", &home, &mut session, &[], &Settings::default());
    assert!(first.intents.iter().any(|i| i.slot("entity_id") == Some("light.wohnzimmer")), "{:?}", first.intents);
    assert!(
        first.intents.iter().all(|i| i.slot("entity_id").is_none_or(|id| !id.contains("adaptive") && !id.contains("adaptiv_"))),
        "{:?}",
        first.intents
    );
    let second = parse("mach sie aus", &home, &mut session, &[], &Settings::default());
    assert_eq!(second.intents[0].slot("entity_id"), Some("light.wohnzimmer"), "{:?}", second.intents);
}

#[test]
fn wichtig_tag_ist_kein_licht() {
    let found = role_slots("Licht im Flur an");
    assert!(!has_entity(&found, "switch.flur_wichtig"), "{found:?}");
}

#[test]
fn licht_tag_ist_kein_geraetename() {
    let found = role_slots("Licht an");
    assert!(!has_entity(&found, "switch.abstell_steckdose"), "{found:?}");
}

#[test]
fn tv_tag_macht_schalter_zum_fernseher() {
    let found = role_slots("TV in der Küche an");
    assert!(has_entity(&found, "switch.kuche_radio"), "{found:?}");
}

#[test]
fn heizung_tag_macht_schalter_zur_heizung() {
    let found = role_slots("Heizung auf dem Balkon an");
    assert!(has_entity(&found, "switch.balkon_heizstab"), "{found:?}");
}

#[test]
fn luefter_tag_macht_schalter_zum_luefter() {
    let found = role_slots("Lüfter im Esszimmer an");
    assert!(has_entity(&found, "switch.esszimmer_box"), "{found:?}");
}

#[test]
fn steckdose_bleibt_schalter_trotz_licht_tag() {
    let found = role_slots("Steckdose in der Abstellkammer an");
    assert!(found.iter().any(|(n, _)| n == "HassTurnOn"), "{found:?}");
    assert!(
        has_entity(&found, "switch.abstell_steckdose")
            || found.iter().any(|(_, slots)| slots.iter().any(|(k, v)| k == "domain" && v == "switch")),
        "{found:?}"
    );
}

#[test]
fn replay_skips_entity_not_exposed_to_assist() {
    let mut home = default_home();
    home.assist = Some(std::collections::HashSet::new());
    let mut session = Session::new();
    session.last_entities.push("light.wohnzimmer".into());
    let result = parse("aus", &home, &mut session, &[], &Settings::default());
    assert!(result.intents.iter().all(|i| i.slot("entity_id") != Some("light.wohnzimmer")), "{:?}", result.intents);
}

#[test]
fn news_briefing_then_followup_not_device_replay() {
    let home = default_home();
    let settings = Settings::default();
    let mut session = Session::new();
    let light = parse("Licht im Wohnzimmer an", &home, &mut session, &[], &settings);
    assert_eq!(light.intents[0].name, "HassTurnOn", "{}", light.speech);

    let news = parse("Was sind die aktuellen Nachrichten", &home, &mut session, &[], &settings);
    assert!(news.chat, "{}", news.speech);
    assert!(news.briefing, "{}", news.speech);
    assert!(news.intents.is_empty(), "{:?}", news.intents);
    assert!(news.speech.contains("Nachrichten"), "{}", news.speech);

    let yes = parse("ja", &home, &mut session, &[], &settings);
    assert!(yes.chat, "ja nach News darf kein Gerät schalten: {:?}", yes.intents);
    assert!(yes.briefing);
    assert!(yes.intents.is_empty(), "{:?}", yes.intents);

    let more = parse("erzähl mehr zur ersten", &home, &mut session, &[], &settings);
    assert!(more.chat, "{:?}", more.intents);
    assert!(more.intents.is_empty(), "{:?}", more.intents);

    let wetter = parse("Wie ist das Wetter", &home, &mut Session::new(), &[], &settings);
    assert!(wetter.chat);
    assert!(!wetter.briefing, "{}", wetter.speech);

    let off = parse("Licht im Wohnzimmer aus", &home, &mut session, &[], &settings);
    assert!(!off.chat, "{:?}", off.intents);
    assert!(!session.briefing);
    assert_eq!(off.intents[0].name, "HassTurnOff", "{}", off.speech);

    let again = parse("aktuelle News", &home, &mut session, &[], &settings);
    assert!(again.briefing);
    let stop = parse("nein danke", &home, &mut session, &[], &settings);
    assert!(!stop.chat, "{}", stop.speech);
    assert!(stop.briefing);
    assert!(!session.briefing);
    assert!(stop.speech.contains("klar") || stop.speech.contains("Klar"), "{}", stop.speech);

    let replay = parse("ja", &home, &mut session, &[], &settings);
    assert!(!replay.chat, "{:?}", replay.intents);
    assert!(replay.intents.iter().any(|i| i.name.starts_with("Hass")), "{:?}", replay.intents);
}
