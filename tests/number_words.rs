//! Spoken numbers are read whole or not at all. A number the language's rules cannot add
//! up is rejected instead of acting on its first word, and German and English read hundreds.
use std::collections::BTreeSet;

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::types::{HomeGraph, ParseDecision};

/// One room with a lamp and a heater, named in the language under test.
fn test_home(lang: &str) -> HomeGraph {
    let (light, climate) = match lang {
        "de" => ("Salon Licht", "Heizung"),
        "fr" => ("Lampe Salon", "Chauffage"),
        "es" => ("Lampara Salon", "Calefaccion"),
        "it" => ("Lampada Salon", "Riscaldamento"),
        "nl" => ("Lamp Salon", "Verwarming"),
        _ => ("Salon Light", "Heating"),
    };
    home(vec![area("salon", "Salon")], vec![ent("light.salon", light, "salon"), ent("climate.salon", climate, "salon")], &[])
}

fn sets(lang: &str, turns: &[&str], wanted: &str) {
    let home = test_home(lang);
    let mut talk = Talk::new(lang, &home);
    let mut out = None;
    for turn in turns {
        out = Some(talk.say(turn));
    }
    let out = out.expect("at least one turn");
    assert_eq!(out.decision, ParseDecision::Execute, "{lang} {turns:?}: {}", describe(&out));
    assert_eq!(effects(&out, &home), BTreeSet::from([wanted.to_string()]), "{lang} {turns:?}: {}", describe(&out));
}

fn starts_timer(lang: &str, text: &str, slot: &str, wanted: &str) {
    let home = test_home(lang);
    let out = Talk::new(lang, &home).say(text);
    let step = out.plan.as_ref().and_then(|plan| plan.intents().into_iter().next());
    assert_eq!(out.decision, ParseDecision::Execute, "{lang} {text}: {}", describe(&out));
    assert_eq!(step.as_ref().map(|intent| intent.name.as_str()), Some("HassStartTimer"), "{lang} {text}: {}", describe(&out));
    assert_eq!(step.as_ref().and_then(|intent| intent.slot(slot)), Some(wanted), "{lang} {text}: {}", describe(&out));
}

fn refuses(lang: &str, turns: &[&str]) {
    let home = test_home(lang);
    let mut talk = Talk::new(lang, &home);
    let (last, before) = turns.split_last().expect("at least one turn");
    for turn in before {
        talk.say(turn);
    }
    let out = talk.say(last);
    assert!(matches!(out.decision, ParseDecision::Reject { .. }), "{lang} {turns:?}: {}", describe(&out));
    assert!(out.plan.is_none() && effects(&out, &home).is_empty(), "{lang} {turns:?}: {}", describe(&out));
}

#[test]
fn english_reads_hundreds() {
    sets("en", &["set the salon light to one hundred percent"], "HassLightSet light.salon brightness=100");
    sets("en", &["set the salon light to a hundred percent"], "HassLightSet light.salon brightness=100");
    sets("en", &["set the salon light to twenty five percent"], "HassLightSet light.salon brightness=25");
    sets("en", &["set the heating to twenty-one degrees"], "HassClimateSetTemperature climate.salon temperature=21");
    starts_timer("en", "set a timer for one hundred twenty seconds", "seconds", "120");
    starts_timer("en", "set a timer for two hundred seconds", "seconds", "200");
}

#[test]
fn german_reads_hundreds_and_split_compounds() {
    sets("de", &["Stell das Salon Licht auf einhundert Prozent"], "HassLightSet light.salon brightness=100");
    sets("de", &["Stell das Salon Licht auf hundert Prozent"], "HassLightSet light.salon brightness=100");
    sets("de", &["Stell das Salon Licht auf ein hundert Prozent"], "HassLightSet light.salon brightness=100");
    sets("de", &["Stell das Salon Licht auf fünfundzwanzig Prozent"], "HassLightSet light.salon brightness=25");
    sets("de", &["Stell das Salon Licht auf fünf und zwanzig Prozent"], "HassLightSet light.salon brightness=25");
    sets("de", &["Stell die Heizung auf einundzwanzig Grad"], "HassClimateSetTemperature climate.salon temperature=21");
    // The "und" inside the number is no clause boundary and the "ein" no article to drop.
    sets("de", &["Stell die Heizung auf ein und zwanzig Grad"], "HassClimateSetTemperature climate.salon temperature=21");
    starts_timer("de", "Stell einen Timer auf hundertzwanzig Sekunden", "seconds", "120");
}

#[test]
fn a_listed_number_is_read_with_or_without_its_hyphen() {
    sets("fr", &["mets le chauffage à dix-sept degrés"], "HassClimateSetTemperature climate.salon temperature=17");
    sets("fr", &["mets le chauffage à dix sept degrés"], "HassClimateSetTemperature climate.salon temperature=17");
    sets("fr", &["mets la lampe salon à quatre-vingts pour cent"], "HassLightSet light.salon brightness=80");
    sets("fr", &["allume le chauffage", "dix-sept"], "HassClimateSetTemperature climate.salon temperature=17");
}

#[test]
fn a_number_the_language_cannot_add_up_is_rejected() {
    // Package 4.2 replaced the 1.14 blanket reject for tens+ones that the locale
    // can now compose. Forms with no grammar (wrong order, missing link) still refuse.
    for (lang, text) in [
        ("it", "imposta lampada salon al cinque venti per cento"),
        ("en", "set the salon light to five ten percent"),
        ("en", "set the salon light to one hundred and five percent"),
        ("de", "Stell die Heizung auf zwei drei Grad"),
    ] {
        refuses(lang, &[text]);
    }
    refuses("en", &["turn on the heating", "twenty twenty"]);
}

#[test]
fn package_4_2_reads_composed_tens_and_ones() {
    sets("fr", &["mets le chauffage à vingt-deux degrés"], "HassClimateSetTemperature climate.salon temperature=22");
    sets("fr", &["mets le chauffage à vingt et un degrés"], "HassClimateSetTemperature climate.salon temperature=21");
    sets("fr", &["mets la lampe salon à vingt-cinq pour cent"], "HassLightSet light.salon brightness=25");
    sets("es", &["pon la calefaccion a treinta y cinco grados"], "HassClimateSetTemperature climate.salon temperature=35");
    sets("es", &["pon la lampara salon al treinta y cinco por ciento"], "HassLightSet light.salon brightness=35");
    sets("nl", &["zet lamp salon op vijf en twintig procent"], "HassLightSet light.salon brightness=25");
    starts_timer("fr", "mets un minuteur de vingt cinq minutes", "minutes", "25");
}

#[test]
fn article_one_does_not_steal_timer_duration() {
    // L-05: "um" / "یک" is the article, "cinco" / "پنج" is the duration.
    starts_timer("pt", "Define um temporizador de cinco minutos", "minutes", "5");
    starts_timer("pt-BR", "Coloca um temporizador de cinco minutos", "minutes", "5");
    starts_timer("fa", "یک تایمر پنج دقیقه بگذار", "minutes", "5");
    starts_timer("pt", "temporizador de um minuto", "minutes", "1");
}

#[test]
fn plain_numbers_next_to_other_words_still_work() {
    sets("fr", &["mets le chauffage à vingt degrés"], "HassClimateSetTemperature climate.salon temperature=20");
    sets("es", &["pon la lampara salon al treinta por ciento"], "HassLightSet light.salon brightness=30");
    // "per cento" holds the word for a hundred; it is not part of the value.
    sets("it", &["imposta lampada salon al venti per cento"], "HassLightSet light.salon brightness=20");
    sets("nl", &["zet lamp salon op twintig procent"], "HassLightSet light.salon brightness=20");
    // The article is not a one in front of the ten.
    starts_timer("nl", "zet een tien minuten timer", "minutes", "10");
    starts_timer("fr", "mets un minuteur de dix minutes", "minutes", "10");
    sets("en", &["turn on the heating", "twenty one"], "HassClimateSetTemperature climate.salon temperature=21");
}
