use crate::expose::assist_visible;
use crate::lang::catalog;
use crate::normalize::compact;
use crate::types::{AreaRec, EntityRec, HomeGraph};

const INFRA_SWITCH_ID: &[&str] = &[
    "r2d2_",
    "adaptive_lighting",
    "adaptiv_",
    "cloud_alexa",
    "cloud_google",
    "adguard",
    "bitte_nicht_storen",
    "durchsagen",
    "kommunikation",
    "child_lock",
    "wake_sound",
];
const INFRA_SWITCH_NAME: &[&str] = &["klimaanlage", "adaptive"];
const INFRA_LIGHT_ID: &[&str] = &["led_ring", "voice_led", "u7_pro"];
const INFRA_LIGHT_NAME: &[&str] = &["ledring", "u7pro"];
const DURATION_TIMER: &[(i32, &str)] = &[(90, "laundry")];
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

pub fn fallback_climate<'a>(home: &'a HomeGraph) -> Option<&'a str> {
    let climates: Vec<&EntityRec> =
        home.entities.iter().filter(|entity| assist_visible(entity, home) && entity.domain == "climate" && !is_infra(entity)).collect();
    climates
        .iter()
        .find(|entity| entity.entity_id.contains("upper") || compact(&entity.name).contains("upper"))
        .or_else(|| (climates.len() == 1).then(|| &climates[0]))
        .map(|entity| entity.entity_id.as_str())
}

pub fn fallback_cover_area(home: &HomeGraph) -> Option<String> {
    let area = primary_bedroom(home)?;
    home.entities.iter().any(|entity| entity.domain == "cover" && entity.area.as_deref() == Some(area.as_str())).then_some(area)
}

pub fn timer_hint(number: Option<i32>) -> Option<&'static str> {
    number.and_then(|n| DURATION_TIMER.iter().find(|(mins, _)| *mins == n).map(|(_, name)| *name))
}

pub fn is_infra(entity: &EntityRec) -> bool {
    is_infra_light(entity) || is_infra_switch(entity)
}

pub fn is_infra_light(entity: &EntityRec) -> bool {
    entity.domain == "light" && infra_hit(entity, INFRA_LIGHT_ID, INFRA_LIGHT_NAME)
}

fn is_infra_switch(entity: &EntityRec) -> bool {
    entity.domain == "switch" && infra_hit(entity, INFRA_SWITCH_ID, INFRA_SWITCH_NAME)
}

fn infra_hit(entity: &EntityRec, ids: &[&str], names: &[&str]) -> bool {
    let id = entity.entity_id.to_ascii_lowercase();
    let name = compact(&entity.name);
    ids.iter().any(|needle| id.contains(needle)) || names.iter().any(|needle| name.contains(needle))
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
