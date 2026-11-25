//! Which devices are a way into the home.

use crate::lang::Catalog;
use crate::parse::normalize::tokenize;
use crate::types::{EntityRec, HomeGraph};

/// Home Assistant's cover classes that let a person or a car in.
const ENTRANCE_CLASSES: &[&str] = &["garage", "gate", "door"];
/// A pack word this long also counts at the end of a compound ("Garagentür", "Hoftüre").
const COMPOUND_WORD: usize = 4;

/// A lock, or a cover that is a garage door, a gate or a door. Moving one in either
/// direction is a safety matter, so the direction does not decide the risk.
///
/// The device class decides where Home Assistant knows one. A cover without a class is
/// judged by its name, with the door and garage words of the active language packs.
pub fn is_entrance(entity: &EntityRec, home: &HomeGraph, cat: &Catalog) -> bool {
    match entity.domain.as_str() {
        "lock" => true,
        "cover" => match home.device_class(&entity.entity_id) {
            Some(class) => ENTRANCE_CLASSES.contains(&class),
            None => named_like_entrance(entity, cat),
        },
        _ => false,
    }
}

fn named_like_entrance(entity: &EntityRec, cat: &Catalog) -> bool {
    let object = entity.entity_id.split_once('.').map_or("", |(_, object)| object).replace('_', " ");
    let labels = std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str)).chain(std::iter::once(&*object));
    let words: Vec<String> = labels.flat_map(tokenize).collect();
    let entrance = |word: &str| {
        [cat.garage_words(), cat.garage_cover(), cat.door_nouns()]
            .into_iter()
            .flatten()
            .any(|known| word == *known || (known.chars().count() >= COMPOUND_WORD && (word.ends_with(known) || word.starts_with(known))))
    };
    words.iter().any(|word| entrance(word))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::catalog_for;

    fn cover(id: &str, name: &str) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: id.split('.').next().unwrap_or("").into(),
            platform: None,
            area: None,
            aliases: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[test]
    fn the_device_class_decides_where_it_is_known() {
        let cat = catalog_for(&["en".into()]);
        let mut home = HomeGraph::default();
        home.device_classes.insert("cover.north".into(), "garage".into());
        home.device_classes.insert("cover.drive".into(), "gate".into());
        home.device_classes.insert("cover.garage_window_blind".into(), "blind".into());
        assert!(is_entrance(&cover("cover.north", "North"), &home, &cat));
        assert!(is_entrance(&cover("cover.drive", "Driveway"), &home, &cat));
        assert!(!is_entrance(&cover("cover.garage_window_blind", "Garage Window Blind"), &home, &cat), "a blind in the garage is no door");
    }

    #[test]
    fn a_cover_without_a_class_is_judged_by_its_name() {
        let home = HomeGraph::default();
        let en = catalog_for(&["en".into()]);
        assert!(is_entrance(&cover("cover.garage_door", "Garage Door"), &home, &en));
        assert!(!is_entrance(&cover("cover.bedroom_blinds", "Blinds"), &home, &en));
        let de = catalog_for(&["de".into()]);
        assert!(is_entrance(&cover("cover.garagentor", "Garagentor"), &home, &de));
        assert!(is_entrance(&cover("cover.tor", "Tor"), &home, &de));
        assert!(!is_entrance(&cover("cover.rollo", "Rollo Motor"), &home, &de), "a motor is no gate");
    }

    #[test]
    fn locks_are_entrances_and_lights_are_not() {
        let home = HomeGraph::default();
        let cat = catalog_for(&["en".into()]);
        assert!(is_entrance(&cover("lock.front_door", "Front Door"), &home, &cat));
        assert!(!is_entrance(&cover("light.door_light", "Door Light"), &home, &cat));
    }
}
