//! A regional pack (de-AT, de-CH, en-GB, pt-BR) is an overlay on its parent pack: it adds
//! regional words and never changes what the parent's own sentences do. Every conversation
//! of the parent's suites must end in the same decision under the regional code; only the
//! spoken reply may differ.
mod common;
mod voice_suite_support;

use common::home::{area, ent, home};
use klar_nlu::lang::LangId;
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, IntentPlan, ParseDecision, ParseOutcome, Settings};
use voice_suite_support::{conversations, suite_home};

const WOHNUNG_DE: &[&str] = &[
    "area",
    "assist",
    "clarifications",
    "conversation",
    "devices",
    "multiple_intents",
    "query_area",
    "query_devices",
    "state_persistance",
];
const WOHNUNG_EN: &[&str] =
    &["area", "assist", "conversation", "devices", "multiple_intents", "query_area", "query_devices", "state_persistance"];
const TIERS: &[&str] = &["quick", "full"];

fn steps(plan: Option<&IntentPlan>) -> String {
    let intents = plan.map(IntentPlan::intents).unwrap_or_default();
    let rows: Vec<String> = intents
        .iter()
        .map(|intent| {
            let slots: Vec<String> = intent.slots.iter().map(|slot| format!("{}={}", slot.name, slot.value)).collect();
            format!("{}({})", intent.name, slots.join(","))
        })
        .collect();
    rows.join(" + ")
}

/// What Klar decided, without the wording of the reply. A confirmation names the plan a "yes" would run.
fn verdict(out: &ParseOutcome, session: &Session) -> String {
    match &out.decision {
        ParseDecision::Execute => format!("execute {}", steps(out.plan.as_ref())),
        ParseDecision::Clarify { options, .. } => format!("clarify {options:?}"),
        ParseDecision::Confirm { .. } => format!("confirm {}", steps(session.pending_confirm().map(|pending| &pending.plan))),
        ParseDecision::Reject { reason } => format!("reject {reason:?}"),
        ParseDecision::Chat => "chat".into(),
        ParseDecision::Error { code, .. } => format!("error {code}"),
    }
}

fn talk(lang: &str, home: &HomeGraph, turns: &[String]) -> Vec<String> {
    let settings = Settings::pinned(lang);
    let mut session = Session::new();
    let mut verdicts = Vec::new();
    for turn in turns {
        let out = parse(turn, home, &mut session, &[], &settings);
        verdicts.push(verdict(&out, &session));
    }
    verdicts
}

fn say(lang: &str, home: &HomeGraph, text: &str) -> String {
    talk(lang, home, &[text.to_string()]).remove(0)
}

/// Runs every conversation of `suites` under `parent` and under `child` and lists the ones that part ways.
fn assert_inherits(parent: &str, child: &str, suites: &[(&str, &[&str])], at_least: usize) {
    let mut seen = 0;
    let mut differences = Vec::new();
    for (suite, groups) in suites {
        let home = suite_home(suite);
        for (label, turns) in conversations(suite, groups) {
            seen += 1;
            let want = talk(parent, &home, &turns);
            let got = talk(child, &home, &turns);
            if want != got {
                differences.push(format!("{label} {turns:?}\n    {parent}: {want:?}\n    {child}: {got:?}"));
            }
        }
    }
    assert!(seen >= at_least, "{child}: only {seen} conversations loaded from {suites:?}");
    assert!(
        differences.is_empty(),
        "{child} decides differently from {parent} in {} of {seen} conversations:\n{}",
        differences.len(),
        differences.join("\n")
    );
}

const GERMAN: &[(&str, &[&str])] = &[("wohnung_mittel", WOHNUNG_DE), ("full_home/de", TIERS)];
const ENGLISH: &[(&str, &[&str])] = &[("wohnung_en", WOHNUNG_EN), ("full_home/en", TIERS)];
const PORTUGUESE: &[(&str, &[&str])] = &[("full_home/pt", TIERS)];

#[test]
fn german_suites_decide_the_same_under_de_at() {
    assert_inherits("de", "de-AT", GERMAN, 1000);
}

#[test]
fn german_suites_decide_the_same_under_de_ch() {
    assert_inherits("de", "de-CH", GERMAN, 1000);
}

#[test]
fn english_suites_decide_the_same_under_en_gb() {
    assert_inherits("en", "en-GB", ENGLISH, 1000);
}

#[test]
fn portuguese_suites_decide_the_same_under_pt_br() {
    assert_inherits("pt", "pt-BR", PORTUGUESE, 100);
}

fn german_home() -> HomeGraph {
    home(
        vec![area("wohnzimmer", "Wohnzimmer"), area("schlafzimmer", "Schlafzimmer"), area("kueche", "Küche")],
        vec![
            ent("light.wohnzimmer", "Wohnzimmer Licht", "wohnzimmer"),
            ent("light.kueche", "Küche Licht", "kueche"),
            ent("cover.schlafzimmer", "Schlafzimmer Rollo", "schlafzimmer"),
        ],
        &[],
    )
}

/// L-01: the regional packs turned standard German around.
#[test]
fn standard_german_keeps_its_meaning_under_the_regional_codes() {
    let home = german_home();
    for lang in ["de", "de-AT", "de-CH"] {
        assert_eq!(
            say(lang, &home, "Schalte bitte das Licht im Wohnzimmer ein"),
            "execute HassTurnOn(entity_id=light.wohnzimmer)",
            "{lang}"
        );
        assert_eq!(say(lang, &home, "Schalte das Licht in der Küche aus"), "execute HassTurnOff(entity_id=light.kueche)", "{lang}");
        assert_eq!(
            say(lang, &home, "Mach das Rollo im Schlafzimmer zu"),
            "confirm HassTurnOff(domain=cover,entity_id=cover.schlafzimmer)",
            "{lang}: closing a cover asks first"
        );
    }
}

/// A regional word does what its standard counterpart does.
#[test]
fn regional_words_act_like_their_standard_counterparts() {
    let home = german_home();
    for (lang, regional, standard) in [
        ("de-CH", "Mach s Liecht im Wohnzimmer aa", "Mach das Licht im Wohnzimmer an"),
        ("de-CH", "Mach s Liecht im Wohnzimmer uus", "Mach das Licht im Wohnzimmer aus"),
        ("de-CH", "Liecht i de Chuchi uusschalte", "Licht in der Küche ausschalten"),
        ("de-CH", "Mach d Store im Schlafzimmer zue", "Mach das Rollo im Schlafzimmer zu"),
        ("de-CH", "Mach d Store im Schlafzimmer uf", "Mach das Rollo im Schlafzimmer auf"),
        ("de-AT", "Licht im Wohnzimmer aufdrehen", "Licht im Wohnzimmer einschalten"),
        ("de-AT", "Licht in der Küche abdrehen", "Licht in der Küche ausschalten"),
    ] {
        let want = say("de", &home, standard);
        assert!(!want.starts_with("reject") && !want.starts_with("clarify"), "de does not act on {standard:?}: {want}");
        assert_eq!(say(lang, &home, regional), want, "{lang} {regional:?} against de {standard:?}");
    }
    let flat = home_en();
    for (regional, standard) in
        [("turn on the telly in the lounge", "turn on the tv in the lounge"), ("switch off the telly", "switch off the tv")]
    {
        let want = say("en", &flat, standard);
        assert!(want.starts_with("execute "), "en does not act on {standard:?}: {want}");
        assert_eq!(say("en-GB", &flat, regional), want, "en-GB {regional:?} against en {standard:?}");
    }
}

fn home_en() -> HomeGraph {
    home(
        vec![area("lounge", "Lounge"), area("kitchen", "Kitchen")],
        vec![ent("media_player.lounge_tv", "Lounge TV", "lounge"), ent("light.kitchen", "Kitchen Light", "kitchen")],
        &[],
    )
}

/// Found while binding en-GB alone: the English pack left "thermostat" to the German one, so a
/// home pinned to English set a light to 23 percent.
#[test]
fn english_alone_reads_a_thermostat_as_climate() {
    let flat = home(
        vec![area("living_room", "Living Room")],
        vec![
            ent("climate.living_room", "Living Room Thermostat", "living_room"),
            ent("light.living_room", "Living Room Light", "living_room"),
        ],
        &[],
    );
    for lang in ["en", "en-GB"] {
        for (text, degrees) in [("Living room thermostat 23", 23), ("set the living room thermostat to 21", 21)] {
            assert_eq!(
                say(lang, &flat, text),
                format!("execute HassClimateSetTemperature(entity_id=climate.living_room,temperature={degrees})"),
                "{lang} {text:?}"
            );
        }
    }
}

/// The registry names the parent, and no regional verb contradicts the verb kind its parent gives the same word.
#[test]
fn regional_packs_name_their_parent_and_never_reclassify_its_verbs() {
    for (code, parent) in [("de-AT", "de"), ("de-CH", "de"), ("en-GB", "en"), ("pt-BR", "pt")] {
        let lang = LangId::from_code(code).expect(code);
        let base = lang.parent().unwrap_or_else(|| panic!("{code} has no parent"));
        assert_eq!(base.code(), parent);
        assert_eq!(lang.meta().and_then(|meta| meta.parent), Some(parent));
        for (word, kind) in lang.pack().verbs {
            if let Some((_, standard)) = base.pack().verbs.iter().find(|(known, _)| known == word) {
                assert_eq!(kind, standard, "{code} verb {word:?} is {kind:?}, {parent} says {standard:?}");
            }
        }
    }
    for lang in LangId::all() {
        if let Some(base) = lang.parent() {
            assert!(base.parent().is_none(), "{} inherits from a regional pack", lang.code());
        }
    }
    assert!(LangId::De.parent().is_none() && LangId::En.parent().is_none());
}

/// A regional pack has no confirmation wording of its own, so it asks with its parent's: the
/// question names the action instead of "Should I really do that?".
#[test]
fn regional_confirmation_names_the_action_like_its_parent() {
    let home = german_home();
    let ask = |lang: &str| {
        let mut session = Session::new();
        let out = parse("Mach das Rollo im Schlafzimmer zu", &home, &mut session, &[], &Settings::pinned(lang));
        match out.decision {
            ParseDecision::Confirm { prompt, .. } => prompt,
            other => panic!("{lang}: expected a confirmation, got {other:?}"),
        }
    };
    let standard = ask("de");
    assert!(standard.contains("Rollo"), "de names the cover: {standard}");
    for lang in ["de-AT", "de-CH"] {
        assert_eq!(ask(lang), standard, "{lang}");
    }
}

/// A regional negation forbids the action like the standard one: without it "nöd" and
/// "ned" were unknown words and the lamp was switched on.
#[test]
fn regional_negation_stops_the_command() {
    let home = german_home();
    let standard = say("de", &home, "Mach das Licht im Wohnzimmer nicht an");
    assert!(standard.starts_with("reject"), "de: {standard}");
    for (lang, text) in [
        ("de-CH", "Mach s Liecht im Wohnzimmer nöd aa"),
        ("de-CH", "Mach s Liecht im Wohnzimmer nid aa"),
        ("de-AT", "Schalte das Licht im Wohnzimmer ned ein"),
        ("de-AT", "Schalte das Licht im Wohnzimmer net ein"),
    ] {
        assert_eq!(say(lang, &home, text), standard, "{lang} {text:?}");
    }
}
