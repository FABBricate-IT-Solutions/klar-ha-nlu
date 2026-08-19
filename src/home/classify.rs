use crate::lang::Catalog;
use crate::parse::normalize::compact;
use crate::types::{EntityRec, HomeGraph};

pub(crate) fn is_tv_switch(domain: &str, entity: &EntityRec, cat: &Catalog) -> bool {
    domain == "media_player" && entity.domain == "switch" && {
        let blob = format!("{} {}", entity.entity_id, compact(&format!("{} {}", entity.name, entity.aliases.join(" "))));
        cat.tv_words().iter().any(|word| blob.contains(word))
    }
}

pub(crate) fn is_generic_room_light(entity: &EntityRec, home: &HomeGraph, cat: &Catalog) -> bool {
    if !is_light_like(entity, cat) {
        return false;
    }
    let name = compact(&entity.name);
    if cat.light_nouns().contains(name.as_str()) || cat.light_singular().contains(name.as_str()) {
        return true;
    }
    home.areas.iter().any(|area| generic_name(&name, &compact(&area.name), cat) || generic_name(&name, &compact(&area.area_id), cat))
}

fn is_light_like(entity: &EntityRec, cat: &Catalog) -> bool {
    entity.domain == "light" || entity.tags.iter().any(|tag| cat.role_light().contains(compact(tag).as_str()))
}

fn generic_name(name: &str, room: &str, cat: &Catalog) -> bool {
    if room.is_empty() {
        return false;
    }
    let light = cat.light_nouns().iter().any(|noun| name.ends_with(noun)) || cat.light_singular().iter().any(|noun| name.ends_with(noun));
    light
        && (cat.light_nouns().iter().any(|noun| *name == format!("{room}{noun}"))
            || cat.light_singular().iter().any(|noun| *name == format!("{room}{noun}"))
            || name.starts_with(room))
}
