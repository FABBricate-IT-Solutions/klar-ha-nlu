use crate::lang::catalog;
use crate::parse::normalize::compact;
use crate::types::{EntityRec, HomeGraph};

pub(crate) fn is_tv_switch(domain: &str, entity: &EntityRec) -> bool {
    domain == "media_player" && entity.domain == "switch" && {
        let blob = format!("{} {}", entity.entity_id, compact(&format!("{} {}", entity.name, entity.aliases.join(" "))));
        catalog().tv_words.iter().any(|word| blob.contains(word))
    }
}

pub(crate) fn is_generic_room_light(entity: &EntityRec, home: &HomeGraph) -> bool {
    if !is_light_like(entity) {
        return false;
    }
    let name = compact(&entity.name);
    if matches!(name.as_str(), "licht" | "light" | "lampe" | "lamp" | "leuchte") {
        return true;
    }
    home.areas.iter().any(|area| generic_name(&name, &compact(&area.name)) || generic_name(&name, &compact(&area.area_id)))
}

fn is_light_like(entity: &EntityRec) -> bool {
    entity.domain == "light" || entity.tags.iter().any(|tag| catalog().role_light.contains(compact(tag).as_str()))
}

fn generic_name(name: &str, room: &str) -> bool {
    if room.is_empty() {
        return false;
    }
    let light = name.ends_with("licht") || name.ends_with("light") || name.ends_with("lampe");
    light && (name == format!("{room}licht") || name == format!("{room}light") || name == format!("{room}lampe") || name.starts_with(room))
}
