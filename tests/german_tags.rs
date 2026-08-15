mod common;

use common::run;
use klar_nlu::home::default_home;
use klar_nlu::parse::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, Settings};

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
    session.remember_entity("light.wohnzimmer");
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
