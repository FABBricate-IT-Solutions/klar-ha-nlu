use crate::home::classify::is_generic_room_light;
use crate::lang::catalog;
use crate::parse::normalize::compact;
use crate::types::{EntityRec, HomeGraph};

const WEAK: &[&str] = &[
    "hue",
    "color",
    "lamp",
    "spot",
    "play",
    "group",
    "helper",
    "entity",
    "device",
    "switch",
    "light",
    "licht",
    "led",
    "ring",
    "voice",
    "home",
    "assistant",
    "satellite",
    "pro",
];

use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;

pub fn leftover(home: &HomeGraph) -> Vec<EntityRec> {
    home.entities.iter().filter(|e| assist_visible(e, home) && needs_mapping(e, home)).cloned().collect()
}

pub fn needs_mapping(entity: &EntityRec, home: &HomeGraph) -> bool {
    if is_infra(entity) {
        return false;
    }
    if entity.area.is_none() {
        return true;
    }
    if is_generic_room_light(entity, home) {
        return false;
    }
    if entity.area.as_ref().is_some_and(|area| entity.entity_id == format!("light.{area}")) {
        return false;
    }
    distinctive(entity, home).is_empty()
}

fn distinctive(entity: &EntityRec, home: &HomeGraph) -> Vec<String> {
    let rooms = room_words(home);
    std::iter::once(entity.name.as_str())
        .chain(entity.aliases.iter().map(String::as_str))
        .flat_map(|label| label.split(|c: char| !c.is_ascii_alphanumeric()).filter(|p| !p.is_empty()).map(compact).collect::<Vec<_>>())
        .filter(|part| {
            part.len() > 2
                && !part.chars().all(|c| c.is_ascii_digit())
                && !catalog().generic.contains(&part.as_str())
                && !WEAK.contains(&part.as_str())
                && !rooms.contains(part)
        })
        .collect()
}

fn room_words(home: &HomeGraph) -> Vec<String> {
    home.areas
        .iter()
        .flat_map(|area| {
            std::iter::once(compact(&area.name))
                .chain(std::iter::once(compact(&area.area_id)))
                .chain(area.aliases.iter().map(|a| compact(a)))
        })
        .chain(home.entities.iter().filter_map(|e| e.area.as_ref()).map(|a| compact(a)))
        .filter(|w| w.len() > 2)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AreaRec, EntityRec, HomeGraph};

    fn home() -> HomeGraph {
        HomeGraph {
            areas: vec![AreaRec { area_id: "schlafzimmer".into(), name: "Schlafzimmer".into(), aliases: vec!["bedroom".into()] }],
            entities: vec![
                ent("light.schlafzimmer", "Kugel", Some("schlafzimmer")),
                ent("light.schlafzimmer_licht", "Schlafzimmer Licht", Some("schlafzimmer")),
                ent("light.hue_play_1", "Hue play 1", Some("wohnzimmer")),
                ent("light.hue_color_lamp_1", "Arbeitszimmer", Some("arbeitszimmer")),
                ent("light.orphan", "light.orphan", None),
            ],
            ..Default::default()
        }
    }

    fn ent(id: &str, name: &str, area: Option<&str>) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: "light".into(),
            area: area.map(str::to_string),
            aliases: Vec::new(),
            tags: Vec::new(),
        }
    }

    #[test]
    fn kugel_and_room_group_are_automatic() {
        let home = home();
        assert!(!needs_mapping(&home.entities[0], &home));
        assert!(!needs_mapping(&home.entities[1], &home));
    }

    #[test]
    fn hue_play_and_room_named_lamp_need_a_human() {
        let home = home();
        assert!(needs_mapping(&home.entities[2], &home));
        assert!(needs_mapping(&home.entities[3], &home));
        assert!(needs_mapping(&home.entities[4], &home));
        let leftover = leftover(&home);
        let ids: Vec<_> = leftover.iter().map(|e| e.entity_id.as_str()).collect();
        assert_eq!(ids, ["light.hue_play_1", "light.hue_color_lamp_1", "light.orphan"]);
    }

    #[test]
    fn assist_hides_entities_voice_cannot_see() {
        let mut home = home();
        home.assist = Some(["light.hue_play_1".into()].into());
        let leftover = leftover(&home);
        let ids: Vec<_> = leftover.iter().map(|e| e.entity_id.as_str()).collect();
        assert_eq!(ids, ["light.hue_play_1"]);
    }
}
