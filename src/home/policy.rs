use crate::home::expose::assist_visible;
use crate::lang::catalog;
use crate::parse::normalize::compact;
use crate::types::{AreaRec, EntityRec, HomeGraph};
use std::sync::OnceLock;

/// Platform helpers (Adaptive Lighting, Voice LEDs, cloud). Apartment IDs live on the overlay.
/// Needles are shared with `custom_components/klar_nlu/speech.py`.
const BEDROOM_HINTS: &[&str] = &["bedroom", "bedrooms", "schlafzimmer", "master_bedroom", "master"];
const WHOLE_HOME: &[&str] = &["wohnung", "everywhere", "ueberall", "zuhause", "home", "house", "apartment"];

pub fn is_whole_home(area: &AreaRec) -> bool {
    WHOLE_HOME.contains(&area.area_id.as_str())
        || area.aliases.iter().any(|alias| WHOLE_HOME.contains(&alias.as_str()))
        || WHOLE_HOME.contains(&compact(&area.name).as_str())
}

pub fn mentions_generic_bedroom(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "bedroom" | "bedrooms" | "schlafzimmer"))
        && !tokens.iter().any(|token| matches!(token.as_str(), "2" | "3" | "4" | "two" | "three" | "zwei" | "drei"))
}

pub fn primary_bedroom(home: &HomeGraph) -> Option<String> {
    let beds: Vec<&AreaRec> = home.areas.iter().filter(|area| is_bedroom_area(area)).collect();
    beds.iter()
        .find(|area| area.area_id.contains("master") || area.aliases.iter().any(|alias| alias == "master"))
        .or_else(|| beds.first())
        .map(|area| area.area_id.clone())
}

fn is_bedroom_area(area: &AreaRec) -> bool {
    if area.area_id.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    std::iter::once(area.area_id.as_str()).chain(area.aliases.iter().map(String::as_str)).any(|name| BEDROOM_HINTS.contains(&name))
}

pub fn laundry_areas(home: &HomeGraph) -> Vec<String> {
    let cat = catalog();
    home.areas
        .iter()
        .filter(|area| {
            cat.laundry_area.contains(area.area_id.as_str()) || area.aliases.iter().any(|alias| cat.laundry_area.contains(alias.as_str()))
        })
        .map(|area| area.area_id.clone())
        .collect()
}

pub fn fallback_climate(home: &HomeGraph) -> Option<&str> {
    if let Some(id) = home.policy.preferred_climate.as_deref() {
        if home.entities.iter().any(|entity| entity.entity_id == id && assist_visible(entity, home) && !is_infra(entity)) {
            return Some(id);
        }
    }
    let climates: Vec<&EntityRec> =
        home.entities.iter().filter(|entity| assist_visible(entity, home) && entity.domain == "climate" && !is_infra(entity)).collect();
    (climates.len() == 1).then(|| climates[0].entity_id.as_str())
}

pub fn fallback_cover_area(home: &HomeGraph) -> Option<String> {
    let area = primary_bedroom(home)?;
    home.entities.iter().any(|entity| entity.domain == "cover" && entity.area.as_deref() == Some(area.as_str())).then_some(area)
}

pub fn timer_hint(home: &HomeGraph, number: Option<i32>) -> Option<&str> {
    number.and_then(|n| home.policy.timer_hints.get(&n).map(String::as_str))
}

pub fn is_infra(entity: &EntityRec) -> bool {
    tagged_infra(entity) || is_infra_light(entity) || is_infra_switch(entity) || is_infra_sensor(entity)
}

pub fn is_infra_light(entity: &EntityRec) -> bool {
    entity.domain == "light" && (tagged_infra(entity) || infra_hit(entity))
}

fn is_infra_switch(entity: &EntityRec) -> bool {
    entity.domain == "switch" && (tagged_infra(entity) || infra_hit(entity))
}

fn is_infra_sensor(entity: &EntityRec) -> bool {
    matches!(entity.domain.as_str(), "sensor" | "binary_sensor") && (tagged_infra(entity) || infra_hit(entity))
}

fn tagged_infra(entity: &EntityRec) -> bool {
    entity.tags.iter().any(|tag| tag.eq_ignore_ascii_case("infra"))
}

fn platform_needles() -> &'static [&'static str] {
    static RAW: &str = include_str!("../../custom_components/klar_nlu/infra_needles.txt");
    static NEEDLES: OnceLock<Vec<&'static str>> = OnceLock::new();
    NEEDLES.get_or_init(|| RAW.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#')).collect()).as_slice()
}

fn infra_hit(entity: &EntityRec) -> bool {
    let id = entity.entity_id.to_ascii_lowercase();
    let name = compact(&entity.name);
    platform_needles().iter().any(|needle| id.contains(needle) || name.contains(needle))
}

pub fn preferred_named<'a>(named: &[&'a EntityRec]) -> Option<&'a EntityRec> {
    let cat = catalog();
    named
        .iter()
        .copied()
        .filter(|entity| cat.named_device.contains(compact(&entity.name).as_str()))
        .min_by_key(|entity| compact(&entity.name).len())
        .or_else(|| (named.len() == 1).then(|| named[0]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HomePolicy;

    fn light(id: &str, name: &str, tags: &[&str]) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: "light".into(),
            platform: None,
            area: Some("flur".into()),
            aliases: Vec::new(),
            tags: tags.iter().map(|t| (*t).to_string()).collect(),
        }
    }

    #[test]
    fn apartment_needles_are_tags_not_compiled_ids() {
        assert!(!is_infra(&light("light.u7_pro_led", "U7 Pro LED", &[])));
        assert!(is_infra(&light("light.u7_pro_led", "U7 Pro LED", &["infra"])));
        assert!(is_infra_light(&light("light.satellite_led_ring", "LED Ring", &[])));
        assert!(is_infra(&EntityRec {
            entity_id: "sensor.satellite1_db12c8_temperature".into(),
            name: "Satellite1 db12c8 Temperature".into(),
            domain: "sensor".into(),
            platform: None,
            area: Some("wohnzimmer".into()),
            aliases: Vec::new(),
            tags: Vec::new(),
        }));
    }

    #[test]
    fn timer_hint_reads_home_policy() {
        let home =
            HomeGraph { policy: HomePolicy { timer_hints: [(90, "laundry".into())].into(), ..Default::default() }, ..Default::default() };
        assert_eq!(timer_hint(&home, Some(90)), Some("laundry"));
        assert_eq!(timer_hint(&home, Some(5)), None);
        assert_eq!(timer_hint(&HomeGraph::default(), Some(90)), None);
    }
}
