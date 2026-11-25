//! What the speaker takes out of a command is never part of the plan (plan package 1.3,
//! finding S-06).
//!
//! "Turn on everything in the kitchen except the oven" must not switch the oven, and "the
//! hall light but not the office light" must not switch the office light. Where the engine
//! can read the sentence it acts on the rest; where it cannot, it changes nothing.
//! English and German run in the collision home, the other languages in a small home with
//! names of their own.
mod common;
use common::acts::{intended_changes, reach, show};
use common::collision_home::collision_home;
use common::home::{area, describe, ent, home, Talk};
use klar_nlu::types::{HomeGraph, ParseDecision, ParseOutcome};

/// The outcome and every device the sentence would change, now or after one more word.
fn touched(lang: &str, home: &HomeGraph, text: &str) -> (ParseOutcome, Vec<String>) {
    let mut talk = Talk::new(lang, home);
    let out = talk.say(text);
    let devices = intended_changes(&out, &talk.session, home).iter().flat_map(|step| reach(step, home)).collect();
    (out, devices)
}

/// The sentence switches every device in `acts_on` and none in `spared`.
fn assert_spares(lang: &str, home: &HomeGraph, text: &str, acts_on: &[&str], spared: &[&str]) {
    let (out, devices) = touched(lang, home, text);
    for device in spared {
        assert!(!devices.iter().any(|id| id == device), "[{lang}] {text:?} excepted {device} and still reaches it: {}", describe(&out));
    }
    for device in acts_on {
        assert!(devices.iter().any(|id| id == device), "[{lang}] {text:?} must still reach {device}, got {devices:?}: {}", describe(&out));
    }
}

fn assert_changes_nothing(lang: &str, home: &HomeGraph, text: &str) {
    let (out, devices) = touched(lang, home, text);
    assert!(devices.is_empty(), "[{lang}] {text:?} must not change anything, got {devices:?}: {}", describe(&out));
}

#[test]
fn an_excepted_device_stays_out_of_a_room_command() {
    let en = collision_home("en");
    let kitchen = ["light.kitchen_ceiling", "light.kitchen_counter"];
    assert_spares("en", &en, "turn on everything in the kitchen except the oven", &kitchen, &["switch.oven"]);
    assert_spares(
        "en",
        &en,
        "turn on the kitchen lights excluding the counter light",
        &["light.kitchen_ceiling"],
        &["light.kitchen_counter"],
    );
    assert_spares(
        "en",
        &en,
        "turn on all the lights in the kitchen apart from the counter light",
        &["light.kitchen_ceiling"],
        &["light.kitchen_counter"],
    );
    assert_spares("en", &en, "turn off the lights in the hall, but not the floor lamp", &["light.hall_light"], &["light.floor_lamp"]);
    assert_spares("en", &en, "turn on the lights in the garage but not the workbench light", &["light.garage_light"], &["light.workbench"]);
    assert_spares(
        "en",
        &en,
        "turn off all lights in the office except the desk lamp",
        &["light.office_light", "light.on_air"],
        &["light.desk_lamp"],
    );
    assert_spares("en", &en, "turn off everything in the living room except the tv", &["light.lamp_1"], &["media_player.living_room_tv"]);

    let de = collision_home("de");
    assert_spares("de", &de, "mach die Lichter im Flur aus, aber nicht die Stehlampe", &["light.flur_licht"], &["light.stehlampe"]);
    assert_spares(
        "de",
        &de,
        "mach die Lichter in der Küche an, nur nicht die Arbeitsplatte",
        &["light.kuche_decke"],
        &["light.arbeitsplatte"],
    );
    assert_spares(
        "de",
        &de,
        "mach die Lichter in der Garage an aber nicht das Werkbank Licht",
        &["light.garagenlicht"],
        &["light.werkbank"],
    );
    assert_spares(
        "de",
        &de,
        "alle Lichter im Büro bis auf die Schreibtischlampe aus",
        &["light.buro_licht", "light.auf_sendung"],
        &["light.schreibtischlampe"],
    );
    assert_spares(
        "de",
        &de,
        "mach die Lichter im Büro an mit Ausnahme von der Schreibtischlampe",
        &["light.buro_licht"],
        &["light.schreibtischlampe"],
    );
}

#[test]
fn but_not_takes_the_second_device_out() {
    let en = collision_home("en");
    assert_spares("en", &en, "turn on the hall light but not the office light", &["light.hall_light"], &["light.office_light"]);
    assert_spares("en", &en, "turn on lamp 1 but not lamp 2", &[], &["light.lamp_2"]);
    let de = collision_home("de");
    assert_spares("de", &de, "mach das Flur Licht an aber nicht das Büro Licht", &["light.flur_licht"], &["light.buro_licht"]);
    assert_spares("de", &de, "mach Lampe 1 an aber nicht Lampe 2", &[], &["light.lampe_2"]);
}

#[test]
fn a_room_or_a_floor_can_be_the_exception() {
    let en = collision_home("en");
    let office = ["light.office_light", "light.desk_lamp", "light.on_air"];
    assert_spares("en", &en, "turn off the lights upstairs except the office", &["light.bedroom_ceiling", "light.guest_light"], &office);
    assert_spares("en", &en, "turn off all the lights except the hall light", &["light.kitchen_ceiling"], &["light.hall_light"]);
    assert_spares(
        "en",
        &en,
        "turn off all lights except the kitchen",
        &["light.hall_light"],
        &["light.kitchen_ceiling", "light.kitchen_counter"],
    );
    let de = collision_home("de");
    let buero = ["light.buro_licht", "light.schreibtischlampe", "light.auf_sendung"];
    assert_spares("de", &de, "schalte oben die Lichter aus außer im Büro", &["light.schlafzimmer_decke", "light.gastelicht"], &buero);
    assert_spares("de", &de, "schalte alle Lichter aus außer dem Flur Licht", &["light.kuche_decke"], &["light.flur_licht"]);
    assert_spares(
        "de",
        &de,
        "alle Lichter außer Schlafzimmer ausschalten",
        &["light.kuche_decke"],
        &["light.schlafzimmer_decke", "light.nachttischlampe"],
    );
}

#[test]
fn several_exceptions_are_all_spared() {
    let en = collision_home("en");
    assert_spares(
        "en",
        &en,
        "turn off the lights in the office except the desk lamp and On Air",
        &["light.office_light"],
        &["light.desk_lamp", "light.on_air"],
    );
    let de = collision_home("de");
    assert_spares(
        "de",
        &de,
        "mach die Lichter im Büro aus außer der Schreibtischlampe und Auf Sendung",
        &["light.buro_licht"],
        &["light.schreibtischlampe", "light.auf_sendung"],
    );
}

#[test]
fn a_command_after_the_exception_is_still_a_command() {
    let en = collision_home("en");
    assert_spares(
        "en",
        &en,
        "turn on the lights in the hall except the floor lamp and turn off the tv",
        &["light.hall_light", "media_player.living_room_tv"],
        &["light.floor_lamp"],
    );
}

/// A scene or a script switches devices the engine cannot see. Nothing can be taken out of
/// it, so the sentence changes nothing.
#[test]
fn nothing_is_taken_out_of_a_scene() {
    let en = collision_home("en");
    assert_changes_nothing("en", &en, "start movie night except the tv");
    let de = collision_home("de");
    assert_changes_nothing("de", &de, "schalte alles aus außer dem Fernseher");
    assert_changes_nothing("de", &de, "starte den Filmabend außer dem Fernseher");
}

#[test]
fn an_exception_that_cannot_be_read_changes_nothing() {
    let en = collision_home("en");
    // The exception names nothing in this home.
    assert_changes_nothing("en", &en, "turn on the lights in the kitchen except the banana");
    // Nothing is left once the exception is out.
    assert_changes_nothing("en", &en, "turn on the lights in the kitchen except the kitchen");
    // The command itself is not understood; its exception must not become the target.
    assert_changes_nothing("en", &en, "turn off everything except the oven");
    // P-21 made "close all the blinds" readable, so the bedroom exception is spared.
    assert_spares(
        "en",
        &en,
        "close all the blinds except the ones in the bedroom",
        &["cover.living_room_blinds"],
        &["cover.bedroom_blinds", "cover.garage_door"],
    );
    // The exception leads and no command follows that says "all".
    assert_changes_nothing("en", &en, "the floor lamp except the hall light off");
    let de = collision_home("de");
    assert_changes_nothing("de", &de, "mach die Lichter in der Küche an außer der Banane");
    assert_spares(
        "de",
        &de,
        "mach alle Rollos zu außer im Schlafzimmer",
        &["cover.wohnzimmer_rollo"],
        &["cover.schlafzimmer_rollo", "cover.garagentor"],
    );
}

/// An exception word inside a device name is the name.
#[test]
fn a_name_that_holds_an_exception_word_is_a_name() {
    let rooms = vec![area("flur", "Flur")];
    let devices = vec![ent("light.ausser_haus", "Außer Haus", "flur"), ent("light.flur", "Flurlicht", "flur")];
    let de = home(rooms, devices, &[]);
    let (out, devices) = touched("de", &de, "schalte Außer Haus ein");
    assert_eq!(devices, ["light.ausser_haus"], "{}", describe(&out));
}

/// A question with an exception stays a question.
#[test]
fn a_question_with_an_exception_changes_nothing() {
    let en = collision_home("en");
    assert_changes_nothing("en", &en, "are all the lights off except the hall light");
    assert_changes_nothing("en", &en, "which lights are on except the hall light");
}

/// (language, kitchen, living room, strip light, the two sentences)
type Case = (&'static str, &'static str, &'static str, &'static str, [&'static str; 2]);

/// "Kitchen lights except the strip" and "all lights except the living room" in languages
/// whose packs list an exception word. The home is the same; only the names change.
const ELSEWHERE: &[Case] = &[
    (
        "fr",
        "Cuisine",
        "Salon",
        "Bandeau",
        ["allume les lumières de la cuisine sauf le bandeau", "éteins toutes les lumières sauf le salon"],
    ),
    ("es", "Cocina", "Salón", "Tira", ["enciende las luces de la cocina excepto la tira", "apaga todas las luces excepto el salón"]),
    (
        "it",
        "Cucina",
        "Soggiorno",
        "Striscia",
        ["accendi le luci della cucina tranne la striscia", "spegni tutte le luci tranne il soggiorno"],
    ),
    ("nl", "Keuken", "Woonkamer", "Strip", ["doe de lampen in de keuken aan behalve de strip", "doe alle lampen uit behalve de woonkamer"]),
    ("pt", "Cozinha", "Sala", "Fita", ["liga as luzes da cozinha exceto a fita", "desliga todas as luzes exceto a sala"]),
    ("sv", "Kök", "Vardagsrum", "Slinga", ["tänd lamporna i köket utom slingan", "släck alla lampor utom vardagsrum"]),
    // Turkish and Japanese put the exception word after what it leaves out.
    ("tr", "Mutfak", "Salon", "Şerit", ["şerit hariç mutfak ışıklarını aç", "salon hariç tüm ışıkları kapat"]),
    ("ja", "Kitchin", "Ribingu", "Tepu", ["kitchin raito tsukete tepu 以外", "全部 raito keshite ribingu 以外"]),
    ("pl", "Kuchnia", "Salon", "Listwa", ["włącz światła w kuchni oprócz listwy", "wyłącz wszystkie światła oprócz salonu"]),
];

fn small_home(kitchen: &str, living: &str, strip: &str) -> HomeGraph {
    let rooms = vec![area("kitchen", kitchen), area("living", living)];
    let devices = vec![
        ent("light.kitchen_main", "Plafond", "kitchen"),
        ent("light.kitchen_strip", strip, "kitchen"),
        ent("light.living_main", "Lustre", "living"),
        ent("light.living_corner", "Globe", "living"),
    ];
    home(rooms, devices, &[])
}

#[test]
fn the_exception_is_spared_in_other_languages() {
    let mut read = Vec::new();
    for (lang, kitchen, living, strip, [in_room, everywhere]) in ELSEWHERE {
        let home = small_home(kitchen, living, strip);
        let (out, devices) = touched(lang, &home, in_room);
        assert!(!devices.iter().any(|id| id == "light.kitchen_strip"), "[{lang}] {in_room:?} reaches the strip: {}", describe(&out));
        if devices.iter().any(|id| id == "light.kitchen_main") {
            read.push(format!("{lang} room"));
        }
        let (out, devices) = touched(lang, &home, everywhere);
        let spared = !devices.iter().any(|id| id.starts_with("light.living"));
        assert!(spared, "[{lang}] {everywhere:?} reaches the living room: {}", describe(&out));
        if devices.iter().any(|id| id == "light.kitchen_main") {
            read.push(format!("{lang} all"));
        }
    }
    // The guarantee above holds even where a sentence is not understood. These are the
    // ones that are understood and act on the rest; the list may only grow.
    let understood: Vec<&str> = read.iter().map(String::as_str).collect();
    for expected in UNDERSTOOD {
        assert!(understood.contains(expected), "{expected:?} no longer acts on the rest; understood: {understood:?}");
    }
}

const UNDERSTOOD: &[&str] = &["fr room", "es room", "it room", "nl room", "nl all", "pt room"];

#[test]
fn the_plan_shows_what_it_does() {
    let en = collision_home("en");
    let mut talk = Talk::new("en", &en);
    let out = talk.say("turn on everything in the kitchen except the oven");
    let steps: Vec<String> = intended_changes(&out, &talk.session, &en).iter().map(show).collect();
    assert!(matches!(out.decision, ParseDecision::Execute), "{}", describe(&out));
    assert!(steps.iter().all(|step| !step.contains("oven")), "{steps:?}");
}
