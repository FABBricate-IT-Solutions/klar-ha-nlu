//! The engine rejects or asks instead of guessing a target.
//!
//! A word it does not know never picks the lights of a room, a scene or a script needs its
//! whole name, a command it cannot express is not answered as a status query, a sentence
//! that loses a clause or a target changes nothing, and rooms that share a word are told
//! apart by their full names.
mod common;

use common::acts::{intended_changes, reach};
use common::collision_home::collision_home;
use common::home::{area, describe, ent, home, Talk};
use klar_nlu::lang::{catalog_for, LangId, VerbKind};
use klar_nlu::types::{HomeGraph, Intent, ParseDecision};

/// What the last turn of a conversation decided, and every device it would change.
fn last_turn(talk: &mut Talk, turns: &[&str]) -> (String, Vec<String>) {
    let mut out = None;
    for turn in turns {
        out = Some(talk.say(turn));
    }
    let out = out.expect("at least one turn");
    let mut devices: Vec<String> =
        intended_changes(&out, &talk.session, talk.home).iter().flat_map(|step| reach(step, talk.home)).collect();
    devices.sort();
    devices.dedup();
    (describe(&out), devices)
}

fn changed(lang: &str, home: &HomeGraph, turns: &[&str]) -> (String, Vec<String>) {
    last_turn(&mut Talk::new(lang, home), turns)
}

fn assert_nothing(lang: &str, home: &HomeGraph, turns: &[&str]) {
    let (seen, devices) = changed(lang, home, turns);
    assert!(devices.is_empty(), "[{lang}] {turns:?} must change nothing, got {devices:?}: {seen}");
}

fn assert_devices(lang: &str, home: &HomeGraph, turns: &[&str], wanted: &[&str]) {
    let (seen, devices) = changed(lang, home, turns);
    assert_eq!(devices, wanted, "[{lang}] {turns:?}: {seen}");
}

fn lights(devices: &[String]) -> Vec<&String> {
    devices.iter().filter(|id| id.starts_with("light.")).collect()
}

// ---------- S-08: a scene or a script needs its whole name ----------

#[test]
fn one_shared_word_starts_no_scene_or_script() {
    let en = collision_home("en");
    for text in ["open the door", "open the blinds", "movie", "start the movie scene", "turn on everything in the kitchen"] {
        let (seen, devices) = changed("en", &en, &[text]);
        assert!(!devices.iter().any(|id| id.starts_with("scene.") || id.starts_with("script.")), "[en] {text:?}: {seen}");
    }
    let de = collision_home("de");
    for text in ["öffne die Tür", "Film", "mach alles in der Küche an", "schalte alles im Wohnzimmer aus"] {
        let (seen, devices) = changed("de", &de, &[text]);
        assert!(!devices.iter().any(|id| id.starts_with("scene.") || id.starts_with("script.")), "[de] {text:?}: {seen}");
    }
}

#[test]
fn the_whole_name_still_starts_the_scene_or_script() {
    let en = collision_home("en");
    for text in ["movie night", "start movie night", "activate the movie night scene"] {
        assert_devices("en", &en, &[text], &["scene.movie_night"]);
    }
    assert_devices("en", &en, &["all off"], &["scene.all_off"]);
    assert_devices("en", &en, &["run open garage"], &["script.open_garage"]);
    let de = collision_home("de");
    for text in ["Filmabend", "starte den Filmabend"] {
        assert_devices("de", &de, &[text], &["scene.filmabend"]);
    }
    assert_devices("de", &de, &["alles aus"], &["scene.alles_aus"]);
    assert_devices("de", &de, &["Garage öffnen"], &["script.garage_offnen"]);
}

/// (language, scene name, script name, a sentence with one word of each, the scene by name)
const ROUTINES: &[(&str, &str, &str, [&str; 2], &str)] = &[
    ("fr", "Soirée Cinéma", "Ouvrir Garage", ["lance la soirée", "ouvrir la porte"], "lance la soirée cinéma"),
    ("es", "Noche de Cine", "Abrir Garaje", ["activa la noche", "abrir la puerta"], "activa noche de cine"),
    ("it", "Serata Film", "Apri Garage", ["attiva serata", "apri la porta"], "attiva serata film"),
    ("nl", "Film Avond", "Open Garage", ["start film", "open de deur"], "start film avond"),
];

#[test]
fn routines_need_their_whole_name_in_other_languages() {
    for (lang, scene, script, partial, whole) in ROUTINES {
        let rooms = vec![area("den", "Den"), area("garage", "Garage")];
        let devices = vec![
            ent("light.den_main", "Globe", "den"),
            ent("cover.garage_door", "Portal", "garage"),
            ent("scene.film_evening", scene, ""),
            ent("script.garage_opener", script, ""),
        ];
        let home = home(rooms, devices, &[]);
        for text in partial {
            let (seen, devices) = changed(lang, &home, &[text]);
            assert!(!devices.iter().any(|id| id.starts_with("scene.") || id.starts_with("script.")), "[{lang}] {text:?}: {seen}");
        }
        assert_devices(lang, &home, &[whole], &["scene.film_evening"]);
    }
}

// ---------- S-19: an unknown device word picks no room light ----------

#[test]
fn an_unknown_device_word_does_not_pick_the_room_lights() {
    let en = collision_home("en");
    for text in [
        "turn off the oven in the hall",
        "turn off the spaceship in the kitchen",
        "close the shutters in the kitchen",
        "switch on the dishwasher in the office",
    ] {
        assert_nothing("en", &en, &[text]);
    }
    let de = collision_home("de");
    for text in ["schalte das Raumschiff in der Küche aus", "Jalousien im Büro schließen", "mach den Whirlpool im Flur an"] {
        assert_nothing("de", &de, &[text]);
    }
}

#[test]
fn an_unknown_device_word_does_not_pick_the_room_of_the_satellite() {
    let en = collision_home("en");
    let mut talk = Talk::new("en", &en);
    talk.session.preferred_area = Some("kitchen".into());
    let (seen, devices) = last_turn(&mut talk, &["turn off the spaceship"]);
    assert!(devices.is_empty(), "{seen} {devices:?}");
    // The same room still answers to a plain command.
    let mut talk = Talk::new("en", &en);
    talk.session.preferred_area = Some("kitchen".into());
    let (seen, devices) = last_turn(&mut talk, &["turn off the lights"]);
    assert_eq!(devices, ["light.kitchen_ceiling", "light.kitchen_counter"], "{seen}");
}

#[test]
fn a_room_or_a_light_word_still_switches_the_room_lights() {
    let en = collision_home("en");
    let kitchen = ["light.kitchen_ceiling", "light.kitchen_counter"];
    for text in [
        "turn off the kitchen",
        "turn off the kitchen lights",
        "turn off the lighting in the kitchen",
        "turn the kitchen lights off for me real quick",
    ] {
        assert_devices("en", &en, &[text], &kitchen);
    }
    // A follow-up names nothing; it stays with the last target.
    // "Hall Light" is a device name (P-11): the full name is that lamp, not every light in the hall.
    assert_devices("en", &en, &["turn on the hall light", "turn it off"], &["light.hall_light"]);
    assert_devices("en", &en, &["turn on the hall light", "a bit brighter"], &["light.hall_light"]);
    // The plural still means the room.
    assert_devices("en", &en, &["turn on the hall lights", "turn them off"], &["light.floor_lamp", "light.hall_light"]);
    // A word the pack does not know next to the device's own name is not a second target.
    assert_devices("en", &en, &["switch on the coffee machine quickly"], &["switch.coffee_machine"]);
    let de = collision_home("de");
    let kueche = ["light.arbeitsplatte", "light.kuche_decke"];
    for text in ["mach die Küche aus", "schalte bitte mal eben schnell das Licht in der Küche aus"] {
        assert_devices("de", &de, &[text], &kueche);
    }
}

#[test]
fn it_stands_for_the_last_target_and_for_nothing_else() {
    let en = collision_home("en");
    // P-11: "Hall Light" is the named lamp, so "it" follows that lamp alone.
    let hall = ["light.hall_light"];
    // The verb is not in the pack, but "it" says what is meant.
    assert_devices("en", &en, &["turn on the hall light", "nudge it to 40 percent"], &hall);
    assert_devices("en", &en, &["turn on the coffee machine", "zonk it off"], &["switch.coffee_machine"]);
    // Without "it" the unknown word could be a device, and so could a word further away.
    assert_nothing("en", &en, &["turn on the hall light", "nudge to 40 percent"]);
    assert_nothing("en", &en, &["turn on the hall light", "turn off the spaceship, do it"]);
    assert_nothing("en", &en, &["turn on the hall light", "close the blinds, do it"]);
    assert_nothing("en", &en, &["nudge it to 40 percent"]);
    let de = collision_home("de");
    assert_devices("de", &de, &["mach die Kaffeemaschine an", "knips sie aus"], &["switch.kaffeemaschine"]);
    assert_nothing("de", &de, &["mach das Flur Licht an", "Rollläden schließen, mach es"]);
    assert_nothing("de", &de, &["mach das Flur Licht an", "Raumschiff aus, mach es"]);
}

/// (language, kitchen, living room, an unknown device in the kitchen, the kitchen lights)
const UNKNOWN: &[(&str, &str, &str, &str, &str)] = &[
    ("fr", "Cuisine", "Salon", "éteins le vaisseau dans la cuisine", "éteins la lumière de la cuisine"),
    ("es", "Cocina", "Sala", "apaga la nave en la cocina", "apaga las luces de la cocina"),
    ("it", "Cucina", "Sala", "spegni l'astronave in cucina", "spegni le luci in cucina"),
    ("nl", "Keuken", "Woonkamer", "doe het ruimteschip in de keuken uit", "doe de lampen in de keuken uit"),
    ("pl", "Kuchnia", "Salon", "zamknij rolety w kuchnia", "wyłącz światło w kuchnia"),
    ("tr", "Mutfak", "Salon", "mutfak panjurları kapat", "mutfak ışıklarını kapat"),
];

#[test]
fn an_unknown_device_word_picks_no_light_in_other_languages() {
    for (lang, kitchen, living, unknown, known) in UNKNOWN {
        let rooms = vec![area("kitchen", kitchen), area("living", living)];
        let home = home(rooms, vec![ent("light.kitchen_main", "Opal", "kitchen"), ent("light.living_main", "Globe", "living")], &[]);
        assert_nothing(lang, &home, &[unknown]);
        assert_devices(lang, &home, &[known], &["light.kitchen_main"]);
    }
}

// ---------- acceptance: "close the blinds" touches no light in any locale ----------

/// Two rooms with lights and nothing that could be closed.
fn home_without_covers() -> HomeGraph {
    let rooms = vec![area("atelier", "Atelier"), area("veranda", "Veranda")];
    let devices = vec![
        ent("light.atelier_main", "Opal", "atelier"),
        ent("light.atelier_wall", "Sconce", "atelier"),
        ent("light.veranda_main", "Lantern", "veranda"),
    ];
    home(rooms, devices, &[])
}

/// "close the blinds" as each pack would say it: every closing word with every cover noun,
/// verb first and verb last, alone and with a room.
fn close_the_blinds(lang: &str) -> Vec<String> {
    let cat = catalog_for(&[lang.to_string()]);
    let mut verbs: Vec<&str> =
        cat.verb_entries().filter(|(_, kind)| matches!(kind, VerbKind::Close | VerbKind::Roll)).map(|(word, _)| word).collect();
    verbs.extend(cat.close_words().iter().copied());
    let mut nouns: Vec<&str> = cat.cover_nouns().iter().copied().collect();
    verbs.sort_unstable();
    verbs.dedup();
    nouns.sort_unstable();
    let mut sentences = Vec::new();
    for verb in &verbs {
        for noun in &nouns {
            sentences.extend([
                format!("{verb} {noun}"),
                format!("{noun} {verb}"),
                format!("{verb} {noun} Atelier"),
                format!("Atelier {noun} {verb}"),
            ]);
        }
    }
    sentences
}

/// The same request the way a person writes it.
const CLOSE_THE_BLINDS: &[(&str, &[&str])] = &[
    ("en", &["close the blinds", "close the blinds in the atelier", "shut the shutters", "lower the shades in the atelier"]),
    ("de", &["Rollläden schließen", "mach die Rollos im Atelier zu", "Jalousien runter", "fahr die Rollläden im Atelier herunter"]),
    ("fr", &["ferme les volets", "ferme les stores dans l'atelier"]),
    ("es", &["cierra las persianas", "baja las persianas del atelier"]),
    ("it", &["chiudi le tapparelle", "abbassa le tapparelle in atelier"]),
    ("nl", &["doe de rolluiken dicht", "sluit de gordijnen in het atelier"]),
    ("pt", &["fecha as persianas", "fecha os estores no atelier"]),
    ("da", &["luk persiennerne", "luk persiennerne i atelier"]),
    ("sv", &["stäng persiennerna", "stäng persiennerna i atelier"]),
    ("nb", &["lukk persiennene", "lukk persiennene i atelier"]),
    ("pl", &["zamknij rolety", "zamknij rolety w atelier"]),
    ("tr", &["panjurları kapat", "atelier panjurları kapat"]),
    ("he", &["סגור את התריסים", "סגור את התריסים atelier"]),
    ("ja", &["ブラインドを閉めて", "atelier の ブラインド を 閉めて"]),
];

#[test]
fn closing_the_blinds_touches_no_light_in_any_locale() {
    let home = home_without_covers();
    let mut failures = Vec::new();
    let mut tried = 0;
    for id in LangId::all() {
        let lang = id.code();
        let written =
            CLOSE_THE_BLINDS.iter().filter(|(code, _)| *code == lang).flat_map(|(_, texts)| texts.iter().map(|text| text.to_string()));
        for text in close_the_blinds(lang).into_iter().chain(written) {
            // Said out of the blue, to the satellite of a room, and right after a light was switched.
            let fresh = Talk::new(lang, &home);
            let mut satellite = Talk::new(lang, &home);
            satellite.session.preferred_area = Some("atelier".into());
            let mut after = Talk::new(lang, &home);
            after.session.remember(&Intent::new("HassTurnOn").with("entity_id", "light.atelier_main"));
            for (state, mut talk) in [("fresh", fresh), ("satellite", satellite), ("after a light", after)] {
                tried += 1;
                let (seen, devices) = last_turn(&mut talk, &[text.as_str()]);
                if !lights(&devices).is_empty() {
                    failures.push(format!("[{lang}, {state}] {text:?} -> {devices:?}: {seen}"));
                }
            }
        }
    }
    assert!(tried > 1000, "only {tried} sentences were built from the packs");
    assert!(failures.is_empty(), "{} of {tried} touched a light:\n{}", failures.len(), failures.join("\n"));
}

// ---------- S-20: a command that cannot be expressed is not a status query ----------

#[test]
fn a_command_without_an_expressible_value_is_refused() {
    let en = collision_home("en");
    for text in ["set the fan to high", "set the bedroom fan to high", "set the fan speed to turbo"] {
        let out = Talk::new("en", &en).say(text);
        assert!(matches!(out.decision, ParseDecision::Reject { .. }), "[en] {text:?} is an order, not a question: {}", describe(&out));
    }
    // A question about the same device is still answered.
    for text in ["is the fan on", "fan speed", "what is the fan set to?"] {
        let out = Talk::new("en", &en).say(text);
        let reads = out.plan.as_ref().is_some_and(|plan| plan.intents().iter().any(|step| step.name == "HassGetState"));
        assert!(matches!(out.decision, ParseDecision::Execute) && reads, "[en] {text:?}: {}", describe(&out));
    }
}

#[test]
fn a_capital_letter_with_an_accent_opens_a_command() {
    let rooms = vec![area("dining", "Salle à manger"), area("salon", "Salon")];
    let home = home(rooms, vec![ent("light.dining", "Plafond", "dining"), ent("light.salon", "Globe", "salon")], &[]);
    // "Éteins" used to stay unfolded, so the room was read as a status question.
    assert_devices("fr", &home, &["Éteins la salle à manger"], &["light.dining"]);
    assert_devices("fr", &home, &["éteins la salle à manger"], &["light.dining"]);
}

// ---------- S-21: a lost clause or a lost target changes nothing ----------

#[test]
fn a_sentence_that_loses_a_part_changes_nothing() {
    let en = collision_home("en");
    for text in [
        "turn on the hall light and the spaceship",
        "turn on the hall light and feed the cat",
        "turn on the hall light, close the blinds",
        "turn off the kitchen and the hall",
    ] {
        assert_nothing("en", &en, &[text]);
    }
    let de = collision_home("de");
    for text in ["mach das Licht im Flur an und füttere die Katze", "mach das Flur Licht und das Raumschiff an"] {
        assert_nothing("de", &de, &[text]);
    }
    let rooms = vec![area("salon", "Salon"), area("cuisine", "Cuisine")];
    let french = home(rooms, vec![ent("light.salon", "Plafonnier", "salon"), ent("light.cuisine", "Suspension", "cuisine")], &[]);
    assert_nothing("fr", &french, &["allume la lumière du salon et nourris le chat"]);
    let rooms = vec![area("sala", "Sala"), area("cocina", "Cocina")];
    let spanish = home(rooms, vec![ent("light.sala", "Techo", "sala"), ent("light.cocina", "Colgante", "cocina")], &[]);
    assert_nothing("es", &spanish, &["enciende la luz de la sala y da de comer al gato"]);
}

#[test]
fn a_sentence_whose_parts_are_all_understood_still_acts() {
    let en = collision_home("en");
    assert_devices(
        "en",
        &en,
        &["turn off the kitchen and hall lights"],
        &["light.floor_lamp", "light.hall_light", "light.kitchen_ceiling", "light.kitchen_counter"],
    );
    assert_devices("en", &en, &["turn on the hall light and the floor lamp"], &["light.floor_lamp", "light.hall_light"]);
    // The second command needs a "yes"; both are in the plan that waits.
    let (seen, devices) = changed("en", &en, &["turn on the hall light and lock the back door"]);
    assert!(devices.iter().any(|id| id == "lock.back_door") && devices.iter().any(|id| id == "light.hall_light"), "{seen} {devices:?}");
    let de = collision_home("de");
    assert_devices(
        "de",
        &de,
        &["mach das Licht in der Küche und im Flur an"],
        &["light.arbeitsplatte", "light.flur_licht", "light.kuche_decke", "light.stehlampe"],
    );
}

// ---------- P-24: rooms that share a word ----------

/// (language, room, the room whose name shares a word with it, a sentence for the second room)
const SHARED: &[(&str, &str, &str, &str)] = &[
    ("en", "Guest Room", "Guest Bathroom", "turn off the guest bathroom light"),
    ("en", "Guest Room", "Guest Bathroom", "turn off the light in the guest bathroom"),
    ("en", "Guest Room", "Guest Bathroom", "turn off the guest bathroom"),
    ("it", "Camera Ospiti", "Bagno Ospiti", "spegni la luce bagno ospiti"),
    ("es", "Cuarto Invitados", "Bano Invitados", "apaga la luz bano invitados"),
    ("tr", "Misafir Odasi", "Misafir Banyosu", "misafir banyosu isik kapat"),
    ("fr", "Chambre Amis", "Chambre Amis Bain", "éteins la lumière chambre amis bain"),
    ("id", "Kamar Tamu", "Kamar Mandi Tamu", "matikan lampu kamar mandi tamu"),
];

/// The first room's id is one word of the second room's name, as Home Assistant slugs them.
fn shared_word_home(room: &str, other: &str) -> HomeGraph {
    let rooms = vec![area("guest", room), area("guestbath", other), area("living", "Lounge")];
    let devices =
        vec![ent("light.guest", "Opal", "guest"), ent("light.guestbath", "Sconce", "guestbath"), ent("light.living", "Globe", "living")];
    home(rooms, devices, &[])
}

#[test]
fn rooms_that_share_a_word_are_told_apart() {
    for (lang, room, other, text) in SHARED {
        assert_devices(lang, &shared_word_home(room, other), &[text], &["light.guestbath"]);
    }
}

#[test]
fn both_rooms_are_switched_only_when_both_are_named() {
    let home = shared_word_home("Guest Room", "Guest Bathroom");
    assert_devices("en", &home, &["turn off the guest room light"], &["light.guest"]);
    assert_devices("en", &home, &["turn off the lights in the guest room and the guest bathroom"], &["light.guest", "light.guestbath"]);
}
