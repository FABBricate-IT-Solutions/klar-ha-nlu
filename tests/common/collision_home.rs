//! A home with the collisions real homes have and the small test homes hide: the same device
//! name in several rooms, numbered lamps, command words inside names, two lights per room,
//! rooms that share a word, a lock, a garage door, window contacts, an oven on a switch and a
//! thermostat. One English and one German version with the same shape.
use klar_nlu::types::{AreaRec, EntityRec, FloorRec, HomeGraph};

use super::home::{ent, home};

/// (floor_id, name, alias, level)
type Floor = (&'static str, &'static str, &'static str, i32);
/// (area_id, name, alias or "", floor_id)
type Area = (&'static str, &'static str, &'static str, &'static str);
/// (entity_id, name, area_id or "")
type Device = (&'static str, &'static str, &'static str);

const EN_FLOORS: &[Floor] = &[("ground", "Ground Floor", "downstairs", 0), ("upper", "Upstairs", "first floor", 1)];

const EN_AREAS: &[Area] = &[
    ("kitchen", "Kitchen", "", "ground"),
    ("living_room", "Living Room", "lounge", "ground"),
    ("hall", "Hall", "hallway", "ground"),
    ("garage", "Garage", "", "ground"),
    ("office", "Office", "study", "upper"),
    ("bedroom", "Bedroom", "", "upper"),
    ("guest_room", "Guest Room", "", "upper"),
    ("guest_bathroom", "Guest Bathroom", "", "upper"),
];

const EN_DEVICES: &[Device] = &[
    // "Ceiling Light", "Blinds" and "Thermostat" exist in more than one room.
    ("light.kitchen_ceiling", "Ceiling Light", "kitchen"),
    ("light.kitchen_counter", "Counter Light", "kitchen"),
    ("switch.oven", "Oven", "kitchen"),
    ("switch.coffee_machine", "Coffee Machine", "kitchen"),
    ("lock.back_door", "Back Door", "kitchen"),
    ("binary_sensor.kitchen_window", "Kitchen Window", "kitchen"),
    ("light.living_room_ceiling", "Ceiling Light", "living_room"),
    ("light.lamp_1", "Lamp 1", "living_room"),
    ("light.lamp_2", "Lamp 2", "living_room"),
    ("light.open_shelf", "Open Shelf", "living_room"),
    ("cover.living_room_blinds", "Blinds", "living_room"),
    ("climate.living_room", "Thermostat", "living_room"),
    ("media_player.living_room_tv", "TV", "living_room"),
    ("switch.tv_plug", "TV Plug", "living_room"),
    ("binary_sensor.living_room_window", "Living Room Window", "living_room"),
    ("light.hall_light", "Hall Light", "hall"),
    ("light.floor_lamp", "Floor Lamp", "hall"),
    ("lock.front_door", "Front Door", "hall"),
    ("binary_sensor.front_door_contact", "Front Door Contact", "hall"),
    ("cover.garage_door", "Garage Door", "garage"),
    ("light.garage_light", "Garage Light", "garage"),
    ("light.workbench", "Workbench Light", "garage"),
    ("light.office_light", "Office Light", "office"),
    ("light.desk_lamp", "Desk Lamp", "office"),
    ("light.on_air", "On Air", "office"),
    ("light.bedroom_ceiling", "Ceiling Light", "bedroom"),
    ("light.bedside_lamp", "Bedside Lamp", "bedroom"),
    ("light.timer_lamp", "Timer Lamp", "bedroom"),
    ("cover.bedroom_blinds", "Blinds", "bedroom"),
    ("climate.bedroom", "Thermostat", "bedroom"),
    ("fan.bedroom_fan", "Fan", "bedroom"),
    ("binary_sensor.bedroom_window", "Bedroom Window", "bedroom"),
    ("light.guest_light", "Guest Light", "guest_room"),
    ("light.guest_lamp", "Guest Lamp", "guest_room"),
    ("light.guest_bathroom_light", "Guest Bathroom Light", "guest_bathroom"),
    ("light.mirror_light", "Mirror Light", "guest_bathroom"),
    ("scene.movie_night", "Movie Night", ""),
    ("scene.all_off", "All Off", ""),
    ("script.open_garage", "Open Garage", ""),
];

const DE_FLOORS: &[Floor] = &[("eg", "Erdgeschoss", "unten", 0), ("og", "Obergeschoss", "oben", 1)];

const DE_AREAS: &[Area] = &[
    ("kuche", "Küche", "", "eg"),
    ("wohnzimmer", "Wohnzimmer", "", "eg"),
    ("flur", "Flur", "Diele", "eg"),
    ("garage", "Garage", "", "eg"),
    ("buro", "Büro", "Arbeitszimmer", "og"),
    ("schlafzimmer", "Schlafzimmer", "", "og"),
    ("gastezimmer", "Gästezimmer", "", "og"),
    ("gastebad", "Gästebad", "", "og"),
];

const DE_DEVICES: &[Device] = &[
    // "Deckenlicht", "Rollo" und "Heizung" gibt es in mehr als einem Raum.
    ("light.kuche_decke", "Deckenlicht", "kuche"),
    ("light.arbeitsplatte", "Arbeitsplatte", "kuche"),
    ("switch.backofen", "Backofen", "kuche"),
    ("switch.kaffeemaschine", "Kaffeemaschine", "kuche"),
    ("lock.hintertur", "Hintertür", "kuche"),
    ("binary_sensor.kuchenfenster", "Küchenfenster", "kuche"),
    ("light.wohnzimmer_decke", "Deckenlicht", "wohnzimmer"),
    ("light.lampe_1", "Lampe 1", "wohnzimmer"),
    ("light.lampe_2", "Lampe 2", "wohnzimmer"),
    ("light.offenes_regal", "Offenes Regal", "wohnzimmer"),
    ("cover.wohnzimmer_rollo", "Rollo", "wohnzimmer"),
    ("climate.wohnzimmer", "Heizung", "wohnzimmer"),
    ("media_player.wohnzimmer_fernseher", "Fernseher", "wohnzimmer"),
    ("switch.tv_steckdose", "TV Steckdose", "wohnzimmer"),
    ("binary_sensor.wohnzimmerfenster", "Wohnzimmerfenster", "wohnzimmer"),
    ("light.flur_licht", "Flur Licht", "flur"),
    ("light.stehlampe", "Stehlampe", "flur"),
    ("lock.haustur", "Haustür", "flur"),
    ("binary_sensor.haustur_kontakt", "Haustür Kontakt", "flur"),
    ("cover.garagentor", "Garagentor", "garage"),
    ("light.garagenlicht", "Garagenlicht", "garage"),
    ("light.werkbank", "Werkbank Licht", "garage"),
    ("light.buro_licht", "Büro Licht", "buro"),
    ("light.schreibtischlampe", "Schreibtischlampe", "buro"),
    ("light.auf_sendung", "Auf Sendung", "buro"),
    ("light.schlafzimmer_decke", "Deckenlicht", "schlafzimmer"),
    ("light.nachttischlampe", "Nachttischlampe", "schlafzimmer"),
    ("light.timer_lampe", "Timer Lampe", "schlafzimmer"),
    ("cover.schlafzimmer_rollo", "Rollo", "schlafzimmer"),
    ("climate.schlafzimmer", "Heizung", "schlafzimmer"),
    ("fan.schlafzimmer_ventilator", "Ventilator", "schlafzimmer"),
    ("binary_sensor.schlafzimmerfenster", "Schlafzimmerfenster", "schlafzimmer"),
    ("light.gastelicht", "Gästelicht", "gastezimmer"),
    ("light.gastelampe", "Gästelampe", "gastezimmer"),
    ("light.gastebad_licht", "Gästebad Licht", "gastebad"),
    ("light.spiegellicht", "Spiegellicht", "gastebad"),
    ("scene.filmabend", "Filmabend", ""),
    ("scene.alles_aus", "Alles Aus", ""),
    ("script.garage_offnen", "Garage öffnen", ""),
];

fn build(floors: &[Floor], areas: &[Area], devices: &[Device]) -> HomeGraph {
    let alias = |text: &str| if text.is_empty() { vec![] } else { vec![text.to_string()] };
    let areas: Vec<AreaRec> = areas
        .iter()
        .map(|(id, name, other, floor)| AreaRec {
            area_id: (*id).into(),
            name: (*name).into(),
            aliases: alias(other),
            floor_id: Some((*floor).into()),
        })
        .collect();
    let entities: Vec<EntityRec> = devices.iter().map(|(id, name, area)| ent(id, name, area)).collect();
    let mut graph = home(areas, entities, &[]);
    graph.floors = floors
        .iter()
        .map(|(id, name, other, level)| FloorRec {
            floor_id: (*id).into(),
            name: (*name).into(),
            aliases: alias(other),
            level: Some(*level),
        })
        .collect();
    graph
}

pub fn collision_home_en() -> HomeGraph {
    build(EN_FLOORS, EN_AREAS, EN_DEVICES)
}

pub fn collision_home_de() -> HomeGraph {
    build(DE_FLOORS, DE_AREAS, DE_DEVICES)
}

/// The collision home for a language tag; German for `de*`, English otherwise.
pub fn collision_home(lang: &str) -> HomeGraph {
    if lang.starts_with("de") {
        collision_home_de()
    } else {
        collision_home_en()
    }
}
