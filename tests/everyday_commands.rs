//! Everyday voice commands in a realistic, messy home, in English and German.
//!
//! The homes copy what real installs look like: light groups next to their own bulbs,
//! screens and LED rings in the `light` domain (hidden from Assist), smart plugs that are
//! "lights", config switches next to the real one, heating and hot water in one room,
//! entity IDs that don't match names, and a room with an apostrophe.
//!
//! Each case checks what Home Assistant would actually do: every (action, device, value)
//! triple, however the plan is shaped. Known bugs are `#[ignore]` tests that assert the
//! correct behaviour; a fix removes the `ignore`.

use std::collections::BTreeSet;

mod common;
use common::home::{area, describe, effects, ent, home, run};
use klar_nlu::types::{EntityRec, HomeGraph, ParseDecision};

// ---------- homes ----------

fn en_home() -> HomeGraph {
    home(
        vec![
            area("kitchen", "Kitchen"),
            area("living_room", "Living Room"),
            area("bedroom", "Bedroom"),
            area("office", "Office"),
            area("hall", "Hall"),
            area("bathroom", "Bathroom"),
            area("back_garden", "Back Garden"),
            area("annas_office", "Anna's Office"),
            area("front_porch", "Front Porch"),
        ],
        vec![
            ent("light.main_kitchen_left", "Main Kitchen - Left", "kitchen"),
            ent("light.main_kitchen_right", "Main Kitchen - Right", "kitchen"),
            ent("light.main_kitchen_lights", "Main Kitchen Lights", "kitchen"),
            ent("light.kitchen_table_light_1", "Kitchen - Table Light 1", "kitchen"),
            ent("light.kitchen_table_light_2", "Kitchen - Table Light 2", "kitchen"),
            ent("light.kitchen_table_lights", "Kitchen Table Lights", "kitchen"),
            ent("light.esp32_s3_box_3_a8d314_screen", "Kitchen Screen", "kitchen"),
            ent("light.kitchen_voice_led_ring", "Kitchen Voice LED Ring", "kitchen"),
            ent("cover.kitchen_blinds", "Kitchen Blinds", "kitchen"),
            ent("switch.coffee_machine", "Coffee Machine", "kitchen"),
            ent("switch.coffee_machine_led", "Coffee Machine LED", "kitchen"),
            ent("switch.coffee_machine_auto_off_enabled", "Coffee Machine Auto-off enabled", "kitchen"),
            ent("fan.dehumidifier", "Dehumidifier Fan", "kitchen"),
            ent("media_player.kitchen_speaker", "Kitchen Speaker", "kitchen"),
            ent("media_player.kitchen_view_assist", "Kitchen View Assist Media Player", "kitchen"),
            ent("light.living_room_main_light", "Living Room Main Light", "living_room"),
            ent("light.living_room_lamp", "Living Room Lamp", "living_room"),
            ent("light.left_floor_light", "Left Floor Light", "living_room"),
            ent("light.right_floor_light", "Right Floor Light", "living_room"),
            ent("light.google_voice_led_ring", "Google Voice Living Room LED Ring", "living_room"),
            ent("cover.living_room_blinds", "Living Room blinds", "living_room"),
            ent("media_player.living_room_tv", "Living Room TV", "living_room"),
            ent("media_player.living_room_homepod", "Living Room HomePod", "living_room"),
            ent("light.bedroom_light", "Bedroom Light", "bedroom"),
            ent("light.bedroom_lamp", "Bedroom Lamp", "bedroom"),
            ent("cover.bedroom_blinds", "Bedroom Blinds", "bedroom"),
            ent("cover.bedroom_curtains", "Bedroom Curtains", "bedroom"),
            ent("switch.bedroom_tv", "Bedroom TV", "bedroom"),
            ent("switch.bedroom_tv_child_lock", "Bedroom TV Child lock", "bedroom"),
            ent("switch.phone_charger", "Phone Charger", "bedroom"),
            ent("light.office_light", "Office Light", "office"),
            ent("light.office_charger_socket", "Office Charger Socket", "office"),
            ent("cover.office_blinds", "Office Blinds", "office"),
            ent("media_player.office_speaker", "Office Speaker", "office"),
            ent("light.hall_light", "Hall Light", "hall"),
            ent("climate.hive_receiver_heat", "Hive Receiver Heat", "hall"),
            ent("climate.hive_receiver_water", "Hive Receiver Water", "hall"),
            ent("lock.front_door", "Front Door", "hall"),
            ent("light.0x84b4dbfffe246e9b", "Upstairs Bathroom Light", "bathroom"),
            ent("light.back_garden_floodlight", "Back Garden Floodlight", "back_garden"),
            ent("light.back_garden_status_led", "Back Garden Status LED", "back_garden"),
            ent("switch.back_garden_record", "Back Garden Record", "back_garden"),
            // The entity ID says "hall", but this is Anna's office light.
            ent("light.hall", "Anna's Office Light", "annas_office"),
            ent("cover.annas_office_blinds", "Anna's Office Blinds", "annas_office"),
            ent("light.front_porch_light", "Front Porch Light", "front_porch"),
        ],
        &[
            "light.esp32_s3_box_3_a8d314_screen",
            "light.kitchen_voice_led_ring",
            "light.google_voice_led_ring",
            "light.office_charger_socket",
            "light.back_garden_status_led",
            "switch.coffee_machine_led",
            "switch.coffee_machine_auto_off_enabled",
            "switch.bedroom_tv_child_lock",
            "switch.back_garden_record",
        ],
    )
}

fn de_home() -> HomeGraph {
    home(
        vec![
            area("kuche", "Küche"),
            area("wohnzimmer", "Wohnzimmer"),
            area("schlafzimmer", "Schlafzimmer"),
            area("buro", "Büro"),
            area("flur", "Flur"),
            area("bad", "Bad"),
            area("garten", "Garten"),
        ],
        vec![
            ent("light.kuche_links", "Küche Links", "kuche"),
            ent("light.kuche_rechts", "Küche Rechts", "kuche"),
            ent("light.kuche_alle", "Küchenlichter", "kuche"),
            ent("light.esp32_s3_box_3_screen", "Küche Bildschirm", "kuche"),
            ent("light.kuche_voice_led_ring", "Küche Voice LED Ring", "kuche"),
            ent("cover.kuche_rollo", "Küche Rollo", "kuche"),
            ent("switch.kaffeemaschine", "Kaffeemaschine", "kuche"),
            ent("switch.kaffeemaschine_led", "Kaffeemaschine LED", "kuche"),
            ent("fan.luftentfeuchter", "Luftentfeuchter", "kuche"),
            ent("media_player.kuche_lautsprecher", "Küche Lautsprecher", "kuche"),
            ent("light.wohnzimmer_decke", "Wohnzimmer Deckenlicht", "wohnzimmer"),
            ent("light.wohnzimmer_stehlampe", "Wohnzimmer Stehlampe", "wohnzimmer"),
            ent("light.google_voice_led_ring", "Google Voice Wohnzimmer LED Ring", "wohnzimmer"),
            ent("cover.wohnzimmer_rollo", "Wohnzimmer Rollo", "wohnzimmer"),
            ent("media_player.wohnzimmer_fernseher", "Wohnzimmer Fernseher", "wohnzimmer"),
            ent("media_player.wohnzimmer_homepod", "Wohnzimmer HomePod", "wohnzimmer"),
            ent("light.schlafzimmer_decke", "Schlafzimmer Deckenlicht", "schlafzimmer"),
            ent("light.schlafzimmer_nachttisch", "Schlafzimmer Nachttischlampe", "schlafzimmer"),
            ent("cover.schlafzimmer_rollo", "Schlafzimmer Rollo", "schlafzimmer"),
            ent("cover.schlafzimmer_vorhang", "Schlafzimmer Vorhang", "schlafzimmer"),
            ent("switch.schlafzimmer_tv", "Schlafzimmer Fernseher", "schlafzimmer"),
            ent("switch.schlafzimmer_tv_child_lock", "Schlafzimmer Fernseher Kindersicherung", "schlafzimmer"),
            ent("light.buro", "Büro Licht", "buro"),
            ent("light.buro_ladesteckdose", "Büro Ladesteckdose", "buro"),
            ent("cover.buro_rollo", "Büro Rollo", "buro"),
            ent("light.flur", "Flur Licht", "flur"),
            ent("climate.heizung", "Heizung", "flur"),
            ent("climate.warmwasser", "Warmwasser", "flur"),
            ent("lock.haustur", "Haustür", "flur"),
            ent("light.0x84b4dbfffe246e9b", "Bad Licht", "bad"),
            ent("light.garten_strahler", "Garten Strahler", "garten"),
        ],
        &[
            "light.esp32_s3_box_3_screen",
            "light.kuche_voice_led_ring",
            "light.google_voice_led_ring",
            "light.buro_ladesteckdose",
            "switch.kaffeemaschine_led",
            "switch.schlafzimmer_tv_child_lock",
        ],
    )
}

fn home_for(lang: &str) -> HomeGraph {
    if lang == "de" {
        de_home()
    } else {
        en_home()
    }
}

// ---------- case kinds ----------

enum Want {
    /// Runs, acting on exactly these devices.
    Act(&'static [&'static str]),
    /// A question: only reads state, never acts.
    Ask,
    /// Asks the user which device or room they meant.
    Clarify,
    /// Does nothing: nothing in the home matches.
    Nothing,
}

/// Why a case failed, or `None` when it passed.
fn check(lang: &str, home: &HomeGraph, text: &str, want: &Want) -> Option<String> {
    let out = run(lang, home, text);
    let intents = out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default();
    let reads_only = !intents.is_empty() && intents.iter().all(|i| i.name.contains("Get"));
    let ok = match want {
        Want::Act(wanted) => {
            out.decision == ParseDecision::Execute && effects(&out, home) == wanted.iter().map(|s| (*s).to_string()).collect()
        }
        Want::Ask => reads_only,
        Want::Clarify => matches!(out.decision, ParseDecision::Clarify { .. }),
        Want::Nothing => out.decision != ParseDecision::Execute && intents.is_empty(),
    };
    (!ok).then(|| format!("[{lang}] {text}\n   acted on {:?}\n   {}", effects(&out, home), describe(&out)))
}

fn assert_all(lang: &str, cases: &[(&str, Want)]) {
    let home = home_for(lang);
    let failures: Vec<String> = cases.iter().filter_map(|(text, want)| check(lang, &home, text, want)).collect();
    assert!(failures.is_empty(), "{} of {} failed:\n{}", failures.len(), cases.len(), failures.join("\n"));
}

use Want::{Act, Ask, Clarify, Nothing};

const KITCHEN_LIGHTS_OFF: &[&str] = &[
    "HassTurnOff light.kitchen_table_light_1",
    "HassTurnOff light.kitchen_table_light_2",
    "HassTurnOff light.kitchen_table_lights",
    "HassTurnOff light.main_kitchen_left",
    "HassTurnOff light.main_kitchen_lights",
    "HassTurnOff light.main_kitchen_right",
];

const LIVING_LIGHTS_OFF: &[&str] = &[
    "HassTurnOff light.left_floor_light",
    "HassTurnOff light.living_room_lamp",
    "HassTurnOff light.living_room_main_light",
    "HassTurnOff light.right_floor_light",
];

// ---------- English ----------

#[test]
fn en_lights() {
    assert_all(
        "en",
        &[
            ("turn on the office light", Act(&["HassTurnOn light.office_light"])),
            ("turn off the office lights", Act(&["HassTurnOff light.office_light"])),
            ("could you turn on the office light please", Act(&["HassTurnOn light.office_light"])),
            ("turn off the kitchen lights", Act(KITCHEN_LIGHTS_OFF)),
            ("turn off the living room lights", Act(LIVING_LIGHTS_OFF)),
            ("switch off the lights in the living room", Act(LIVING_LIGHTS_OFF)),
            ("lights on in the bedroom", Act(&["HassTurnOn light.bedroom_lamp", "HassTurnOn light.bedroom_light"])),
            ("turn on the bathroom light", Act(&["HassTurnOn light.0x84b4dbfffe246e9b"])),
            ("turn on the upstairs bathroom light", Act(&["HassTurnOn light.0x84b4dbfffe246e9b"])),
            ("turn on the hall light", Act(&["HassTurnOn light.hall_light"])),
            ("turn on the porch light", Act(&["HassTurnOn light.front_porch_light"])),
            ("turn on the front porch light", Act(&["HassTurnOn light.front_porch_light"])),
            ("turn on the floodlight", Act(&["HassTurnOn light.back_garden_floodlight"])),
            ("turn on the living room lamp", Act(&["HassTurnOn light.living_room_lamp"])),
            ("turn on the kitchen table lights", Act(&["HassTurnOn light.kitchen_table_lights"])),
            ("kill the hall light", Act(&["HassTurnOff light.hall_light"])),
        ],
    );
}

#[test]
fn en_brightness_and_colour() {
    assert_all(
        "en",
        &[
            ("set the living room lamp to 50%", Act(&["HassLightSet light.living_room_lamp brightness=50"])),
            ("set the living room lamp to 50 percent", Act(&["HassLightSet light.living_room_lamp brightness=50"])),
            ("dim the office light to 20 percent", Act(&["HassLightSet light.office_light brightness=20"])),
            ("set the bedroom lamp to blue", Act(&["HassLightSet light.bedroom_lamp color=blue"])),
            ("make the living room lamp red", Act(&["HassLightSet light.living_room_lamp color=red"])),
        ],
    );
}

#[test]
fn en_covers_switches_media_locks() {
    assert_all(
        "en",
        &[
            ("open the kitchen blinds", Act(&["HassTurnOn cover.kitchen_blinds"])),
            ("close the office blinds", Act(&["HassTurnOff cover.office_blinds"])),
            ("close the living room blinds", Act(&["HassTurnOff cover.living_room_blinds"])),
            ("set the bedroom blinds to 30%", Act(&["HassSetPosition cover.bedroom_blinds position=30"])),
            ("close the bedroom blinds and curtains", Act(&["HassTurnOff cover.bedroom_blinds", "HassTurnOff cover.bedroom_curtains"])),
            ("turn on the coffee machine", Act(&["HassTurnOn switch.coffee_machine"])),
            ("turn off the coffee machine", Act(&["HassTurnOff switch.coffee_machine"])),
            ("turn off the bedroom TV", Act(&["HassTurnOff switch.bedroom_tv"])),
            ("turn on the phone charger", Act(&["HassTurnOn switch.phone_charger"])),
            ("turn on the dehumidifier", Act(&["HassTurnOn fan.dehumidifier"])),
            ("turn off the living room TV", Act(&["HassTurnOff media_player.living_room_tv"])),
            ("turn off the TV in the living room", Act(&["HassTurnOff media_player.living_room_tv"])),
            ("lock the front door", Act(&["HassTurnOn lock.front_door"])),
            ("unlock the front door", Act(&["HassTurnOff lock.front_door"])),
        ],
    );
}

#[test]
fn en_questions_never_act() {
    assert_all(
        "en",
        &[
            ("is the front door locked", Ask),
            ("are the kitchen lights on", Ask),
            ("is the office light on", Ask),
            ("are the bedroom blinds open", Ask),
            ("is the coffee machine on", Ask),
            ("what is the temperature in the hall", Ask),
        ],
    );
}

#[test]
fn en_several_steps_and_rooms() {
    assert_all(
        "en",
        &[
            (
                "turn off the lights in the kitchen and the living room",
                Act(&[
                    "HassTurnOff light.kitchen_table_light_1",
                    "HassTurnOff light.kitchen_table_light_2",
                    "HassTurnOff light.kitchen_table_lights",
                    "HassTurnOff light.left_floor_light",
                    "HassTurnOff light.living_room_lamp",
                    "HassTurnOff light.living_room_main_light",
                    "HassTurnOff light.main_kitchen_left",
                    "HassTurnOff light.main_kitchen_lights",
                    "HassTurnOff light.main_kitchen_right",
                    "HassTurnOff light.right_floor_light",
                ]),
            ),
            (
                "turn on the office light and open the office blinds",
                Act(&["HassTurnOn cover.office_blinds", "HassTurnOn light.office_light"]),
            ),
            (
                "turn off the living room TV, turn off the office light and turn on the hall light",
                Act(&["HassTurnOff light.office_light", "HassTurnOff media_player.living_room_tv", "HassTurnOn light.hall_light"]),
            ),
        ],
    );
}

#[test]
fn en_asks_or_does_nothing_when_unclear() {
    assert_all("en", &[("turn on the light", Clarify), ("turn on the jacuzzi", Nothing)]);
}

// ---------- German ----------

#[test]
fn de_lights() {
    assert_all(
        "de",
        &[
            ("Mach das Licht im Büro an", Act(&["HassTurnOn light.buro"])),
            ("Schalte das Licht im Flur aus", Act(&["HassTurnOff light.flur"])),
            ("Schalte das Licht im Bad ein", Act(&["HassTurnOn light.0x84b4dbfffe246e9b"])),
            (
                "Mach das Licht in der Küche aus",
                Act(&["HassTurnOff light.kuche_alle", "HassTurnOff light.kuche_links", "HassTurnOff light.kuche_rechts"]),
            ),
            ("Mach die Lichter im Wohnzimmer aus", Act(&["HassTurnOff light.wohnzimmer_decke", "HassTurnOff light.wohnzimmer_stehlampe"])),
            ("Mach die Stehlampe im Wohnzimmer an", Act(&["HassTurnOn light.wohnzimmer_stehlampe"])),
            ("Stell die Stehlampe im Wohnzimmer auf 50 Prozent", Act(&["HassLightSet light.wohnzimmer_stehlampe brightness=50"])),
            ("Mach die Nachttischlampe im Schlafzimmer blau", Act(&["HassLightSet light.schlafzimmer_nachttisch color=blue"])),
        ],
    );
}

#[test]
fn de_covers_switches_media_locks() {
    assert_all(
        "de",
        &[
            ("Mach das Rollo im Büro zu", Act(&["HassTurnOff cover.buro_rollo"])),
            ("Fahr das Rollo im Schlafzimmer auf 30 Prozent", Act(&["HassSetPosition cover.schlafzimmer_rollo position=30"])),
            ("Schalte den Fernseher im Schlafzimmer aus", Act(&["HassTurnOff switch.schlafzimmer_tv"])),
            ("Mach den Luftentfeuchter an", Act(&["HassTurnOn fan.luftentfeuchter"])),
            ("Mach den Fernseher im Wohnzimmer aus", Act(&["HassTurnOff media_player.wohnzimmer_fernseher"])),
            ("Schließ die Haustür ab", Act(&["HassTurnOn lock.haustur"])),
        ],
    );
}

#[test]
fn de_questions_never_act() {
    assert_all(
        "de",
        &[
            ("Ist die Haustür abgeschlossen", Ask),
            ("Ist das Licht im Büro an", Ask),
            ("Ist das Rollo im Schlafzimmer offen", Ask),
            ("Wie warm ist es im Flur", Ask),
        ],
    );
}

#[test]
fn de_several_steps() {
    assert_all(
        "de",
        &[(
            "Mach den Fernseher im Wohnzimmer aus, mach das Licht im Büro aus und mach das Licht im Flur an",
            Act(&["HassTurnOff light.buro", "HassTurnOff media_player.wohnzimmer_fernseher", "HassTurnOn light.flur"]),
        )],
    );
}

#[test]
fn de_asks_or_does_nothing_when_unclear() {
    assert_all("de", &[("Mach das Licht an", Clarify), ("Schalte den Whirlpool an", Nothing)]);
}

// ---------- bugs found by this file: one still open (ignored), the rest fixed ----------

#[test]
#[ignore = "bug: an unknown device in a room falls back to the room's lights; a word-list guard also blocks synonyms like \"lighting\", so this needs a design decision"]
fn unknown_device_in_a_room_does_nothing() {
    assert_all("en", &[("turn off the oven in the kitchen", Nothing), ("turn off the spaceship in the kitchen", Nothing)]);
}

#[test]
fn possessive_room_names_resolve() {
    assert_all(
        "en",
        &[
            ("turn on Anna's office light", Act(&["HassTurnOn light.hall"])),
            ("turn on the light in Anna's office", Act(&["HassTurnOn light.hall"])),
            ("open the blinds in Anna's office", Act(&["HassTurnOn cover.annas_office_blinds"])),
        ],
    );
}

#[test]
fn heating_set_point_with_hot_water_in_the_same_room() {
    assert_all(
        "en",
        &[
            ("set the heating to 20 degrees", Act(&["HassClimateSetTemperature climate.hive_receiver_heat temperature=20"])),
            ("set the hot water to 55 degrees", Act(&["HassClimateSetTemperature climate.hive_receiver_water temperature=55"])),
        ],
    );
    assert_all(
        "de",
        &[
            ("Stell die Heizung auf 20 Grad", Act(&["HassClimateSetTemperature climate.heizung temperature=20"])),
            ("Stell das Warmwasser auf 55 Grad", Act(&["HassClimateSetTemperature climate.warmwasser temperature=55"])),
        ],
    );
}

#[test]
fn de_rollo_hoch_opens_the_cover() {
    assert_all(
        "de",
        &[
            ("Mach das Rollo in der Küche hoch", Act(&["HassTurnOn cover.kuche_rollo"])),
            ("Mach das Rollo in der Küche runter", Act(&["HassTurnOff cover.kuche_rollo"])),
            ("Fahr den Vorhang im Schlafzimmer hoch", Act(&["HassTurnOn cover.schlafzimmer_vorhang"])),
            ("Mach das Licht im Büro an und das Rollo im Büro hoch", Act(&["HassTurnOn cover.buro_rollo", "HassTurnOn light.buro"])),
        ],
    );
}

#[test]
fn left_and_right_are_kept() {
    assert_all("en", &[("turn on the left floor light", Act(&["HassTurnOn light.left_floor_light"]))]);
}

#[test]
fn exact_device_name_beats_a_longer_one() {
    // Both exposed, as in a real home: "Kaffeemaschine" once asked "or Kaffeemaschine LED?".
    let pairs = [
        ("de", "kuche", "Küche", "switch.kaffeemaschine", "Kaffeemaschine", "Schalte die Kaffeemaschine an"),
        ("en", "kitchen", "Kitchen", "switch.coffee_machine", "Coffee Machine", "turn on the coffee machine"),
    ];
    for (lang, area_id, area_name, id, name, text) in pairs {
        let led = ent(&format!("{id}_led"), &format!("{name} LED"), area_id);
        let both_exposed = home(vec![area(area_id, area_name)], vec![ent(id, name, area_id), led], &[]);
        let out = run(lang, &both_exposed, text);
        assert_eq!(effects(&out, &both_exposed), BTreeSet::from([format!("HassTurnOn {id}")]), "[{lang}] {text}: {}", describe(&out));
    }
}

#[test]
fn named_room_beats_a_device_elsewhere() {
    // The kitchen's own "main" lights are hidden from Assist.
    let entities = vec![
        ent("light.main_kitchen_lights", "Main Kitchen Lights", "kitchen"),
        ent("light.kitchen_table_lights", "Kitchen Table Lights", "kitchen"),
        ent("light.living_room_main_light", "Living Room Main Light", "living_room"),
    ];
    let home = home(vec![area("kitchen", "Kitchen"), area("living_room", "Living Room")], entities, &["light.main_kitchen_lights"]);
    for (text, wanted) in [
        ("Turn off the main light in the kitchen", "HassTurnOff light.kitchen_table_lights"),
        ("turn off the kitchen main light", "HassTurnOff light.kitchen_table_lights"),
        ("turn off the living room main light", "HassTurnOff light.living_room_main_light"),
        ("turn off the main light", "HassTurnOff light.living_room_main_light"),
    ] {
        let out = run("en", &home, text);
        assert_eq!(effects(&out, &home), BTreeSet::from([wanted.to_string()]), "{text}: {}", describe(&out));
    }
}

#[test]
fn all_lights_without_a_whole_home_group() {
    // "All kitchen lights" is a group in one room, not the whole home; the strip has no room.
    let mut entities = vec![
        ent("light.all_kitchen_lights", "All kitchen lights", "kitchen"),
        ent("light.living_room_lamp", "Living Room Lamp", "living_room"),
        ent("light.hall_light", "Hall Light", "hall"),
    ];
    entities.push(EntityRec { area: None, ..ent("light.h6076", "H6076", "") });
    let areas = vec![area("kitchen", "Kitchen"), area("living_room", "Living Room"), area("hall", "Hall")];
    let home = home(areas, entities, &[]);
    let everything: BTreeSet<String> = ["light.all_kitchen_lights", "light.h6076", "light.hall_light", "light.living_room_lamp"]
        .map(|id| format!("HassTurnOff {id}"))
        .into();
    for (text, wanted) in [
        ("turn off all the lights", everything),
        ("turn off all kitchen lights", BTreeSet::from(["HassTurnOff light.all_kitchen_lights".to_string()])),
    ] {
        let out = run("en", &home, text);
        assert_eq!(effects(&out, &home), wanted, "{text}: {}", describe(&out));
    }
}

#[test]
fn vacuum_off_stop_and_home_send_it_to_the_dock() {
    let start = "HassVacuumStart vacuum.kitchen_vacuum";
    let dock = "HassVacuumReturnToBase vacuum.kitchen_vacuum";
    for (lang, text, wanted) in [
        ("en", "turn off the vacuum", dock),
        ("en", "stop the vacuum", dock),
        ("en", "pause the vacuum", dock),
        ("en", "stop vacuuming", dock),
        ("en", "send the vacuum home", dock),
        ("en", "turn on the vacuum", start),
        ("en", "vacuum the kitchen", start),
        ("en", "vacuum my home", start),
        ("de", "Schalte den Staubsauger aus", dock),
        ("de", "Staubsauger starten", start),
    ] {
        let home = home(
            vec![area("kitchen", if lang == "de" { "Küche" } else { "Kitchen" })],
            vec![
                ent("vacuum.kitchen_vacuum", if lang == "de" { "Staubsauger" } else { "Vacuum" }, "kitchen"),
                ent("light.kitchen_light", "Kitchen Light", "kitchen"),
            ],
            &[],
        );
        let out = run(lang, &home, text);
        assert_eq!(effects(&out, &home), BTreeSet::from([wanted.to_string()]), "[{lang}] {text}: {}", describe(&out));
    }
}

#[test]
fn a_room_name_inside_another_room_name_stays_with_its_room() {
    let synced = serde_json::from_value(serde_json::json!({
        "schema_version": "1",
        "areas": [{"id": "office", "name": "Office"}, {"id": "ela_s_office", "name": "Ela's Office"}],
        "devices": [{"id": "tuya_bulb", "name": "Ela’s office light", "name_by_user": "Ela's Office Light", "area_id": "ela_s_office"}],
        "entities": [
            {"entity_id": "light.office_light", "name": "Office Light", "area_id": "office"},
            {"entity_id": "light.hall", "has_entity_name": true, "device_id": "tuya_bulb"}
        ]
    }))
    .expect("valid snapshot");
    let home = klar_nlu::home::snapshot::ingest(synced).expect("ingest");
    let office = "HassTurnOn light.office_light";
    let ela = "HassTurnOn light.hall";
    for (text, wanted) in [
        ("turn on the light in the office", vec![office]),
        ("turn on the light in Ela's office", vec![ela]),
        ("turn on everything in my office", vec![office]),
        ("turn on the light in the office and turn on the light in Ela's office", vec![office, ela]),
    ] {
        let out = run("en", &home, text);
        let wanted: BTreeSet<String> = wanted.iter().map(|s| s.to_string()).collect();
        assert_eq!(effects(&out, &home), wanted, "{text}: {}", describe(&out));
        assert_eq!(out.plan.as_ref().map_or(0, |plan| plan.intents().len()), wanted.len(), "{text}: {}", describe(&out));
    }
}
