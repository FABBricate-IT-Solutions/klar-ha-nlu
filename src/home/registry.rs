use crate::parse::normalize::{compact, fold_umlaut};
use crate::types::{AreaRec, EntityRec, HomeGraph};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct EntityStorage {
    data: EntityData,
}

#[derive(Deserialize)]
struct EntityData {
    entities: Vec<RawEntity>,
}

#[derive(Deserialize)]
struct RawEntity {
    entity_id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    original_name: Option<String>,
    #[serde(default)]
    has_entity_name: bool,
    #[serde(default)]
    area_id: Option<String>,
    #[serde(default)]
    platform: Option<String>,
    #[serde(default, deserialize_with = "strings_skip_null")]
    aliases: Vec<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    disabled_by: Option<String>,
    #[serde(default)]
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct DeviceStorage {
    data: DeviceData,
}

#[derive(Deserialize)]
struct DeviceData {
    devices: Vec<RawDevice>,
}

#[derive(Deserialize)]
struct RawDevice {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    name_by_user: Option<String>,
    #[serde(default)]
    area_id: Option<String>,
}

struct DeviceInfo {
    name: String,
    area_id: Option<String>,
}

#[derive(Deserialize)]
struct AreaStorage {
    data: AreaData,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AreaData {
    Wrapped { areas: Vec<RawArea> },
    Flat(Vec<RawArea>),
}

#[derive(Deserialize)]
struct RawArea {
    id: Option<String>,
    area_id: Option<String>,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

pub fn load_home(config_dir: &Path, fallback: HomeGraph) -> HomeGraph {
    let entity_path = config_dir.join(".storage/core.entity_registry");
    let area_path = config_dir.join(".storage/core.area_registry");
    if !entity_path.exists() {
        tracing::info!("kein Entity-Registry unter {}, nutze Default-Wohnung", entity_path.display());
        return fallback;
    }

    let devices = read_devices(&config_dir.join(".storage/core.device_registry"));
    let label_names = crate::home::roles::load_label_names(&config_dir.join(".storage/core.label_registry"));
    let mut entities = match read_entities(&entity_path, &devices, &label_names) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!("Entity-Registry unlesbar: {err}");
            return fallback;
        }
    };
    let areas = read_areas(&area_path).unwrap_or_else(|_| fallback.areas.clone());
    if entities.is_empty() {
        return fallback;
    }
    for ent in &mut entities {
        if ent.area.is_none() {
            ent.area = infer_area(&ent.entity_id, &areas);
        }
    }
    tracing::info!("{} Entitäten, {} Räume aus Home Assistant geladen", entities.len(), areas.len());
    HomeGraph { entities, areas, assist: crate::home::expose::load_assist(config_dir), ..fallback }
}

fn read_devices(path: &Path) -> HashMap<String, DeviceInfo> {
    let Ok(raw) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    let Ok(parsed) = serde_json::from_str::<DeviceStorage>(&raw) else {
        return HashMap::new();
    };
    parsed
        .data
        .devices
        .into_iter()
        .filter_map(|d| {
            let name = clean_label(d.name_by_user).or_else(|| clean_label(d.name))?;
            Some((d.id, DeviceInfo { name, area_id: d.area_id }))
        })
        .collect()
}

fn strings_skip_null<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let values: Vec<Option<String>> = Deserialize::deserialize(deserializer)?;
    Ok(values.into_iter().flatten().filter_map(|value| clean_label(Some(value))).collect())
}

fn clean_label(value: Option<String>) -> Option<String> {
    let cleaned = value?.split_whitespace().collect::<Vec<_>>().join(" ");
    (!cleaned.is_empty()).then_some(cleaned)
}

fn looks_like_entity_id(value: &str) -> bool {
    value.contains('.') && !value.contains(' ') && value.split('.').count() == 2
}

fn humanize_entity_id(entity_id: &str) -> String {
    entity_id.rsplit('.').next().unwrap_or(entity_id).replace('_', " ")
}

fn compose_device_name(device: Option<&str>, original: Option<&str>) -> Option<String> {
    match (device, original) {
        (Some(device), Some(original)) => {
            let device_fold = compact(device);
            let original_fold = compact(original);
            if device_fold == original_fold || device_fold.contains(&original_fold) {
                Some(device.to_string())
            } else if original_fold.contains(&device_fold) {
                Some(original.to_string())
            } else {
                Some(format!("{device} {original}"))
            }
        }
        (Some(device), None) => Some(device.to_string()),
        (None, Some(original)) => Some(original.to_string()),
        (None, None) => None,
    }
}

fn entity_display_name(entity: &RawEntity, device: Option<&DeviceInfo>) -> String {
    if let Some(name) = clean_label(entity.name.clone()).filter(|name| !looks_like_entity_id(name)) {
        return name;
    }
    let original = clean_label(entity.original_name.clone()).filter(|name| !looks_like_entity_id(name));
    let device_name = device.map(|info| info.name.as_str());
    if entity.has_entity_name || original.is_none() {
        if let Some(composed) = compose_device_name(device_name, original.as_deref()) {
            return composed;
        }
    }
    if let Some(original) = original {
        return original;
    }
    entity.aliases.iter().find(|alias| !looks_like_entity_id(alias)).cloned().unwrap_or_else(|| humanize_entity_id(&entity.entity_id))
}

fn infer_area(entity_id: &str, areas: &[AreaRec]) -> Option<String> {
    let slug = entity_id.split_once('.')?.1;
    let mut best: Option<&str> = None;
    for area in areas {
        if (slug == area.area_id || slug.starts_with(&format!("{}_", area.area_id)))
            && best.is_none_or(|cur| area.area_id.len() > cur.len())
        {
            best = Some(area.area_id.as_str());
        }
    }
    best.map(str::to_string)
}

fn read_entities(
    path: &Path,
    devices: &HashMap<String, DeviceInfo>,
    label_names: &HashMap<String, String>,
) -> Result<Vec<EntityRec>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: EntityStorage = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(parsed
        .data
        .entities
        .into_iter()
        .filter(|e| e.disabled_by.is_none())
        .filter(|e| keep_domain(&e.entity_id))
        .map(|e| {
            let domain = e.entity_id.split('.').next().unwrap_or("").to_string();
            let device = e.device_id.as_ref().and_then(|id| devices.get(id));
            let name = entity_display_name(&e, device);
            let area = e.area_id.or_else(|| device.and_then(|info| info.area_id.clone()));
            EntityRec {
                entity_id: e.entity_id,
                name,
                domain,
                platform: e.platform,
                area,
                aliases: e.aliases,
                tags: crate::home::roles::expand_entity_tags(e.labels, label_names),
            }
        })
        .collect())
}

fn read_areas(path: &Path) -> Result<Vec<AreaRec>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: AreaStorage = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let areas = match parsed.data {
        AreaData::Wrapped { areas } => areas,
        AreaData::Flat(areas) => areas,
    };
    Ok(areas
        .into_iter()
        .map(|a| {
            let area_id = a.id.or(a.area_id).unwrap_or_else(|| fold_umlaut(&a.name));
            AreaRec { area_id: area_id.clone(), name: a.name.clone(), aliases: merge_area_aliases(&area_id, &a.name, a.aliases) }
        })
        .collect())
}

fn keep_domain(entity_id: &str) -> bool {
    matches!(
        entity_id.split('.').next(),
        Some("light")
            | Some("switch")
            | Some("climate")
            | Some("fan")
            | Some("cover")
            | Some("scene")
            | Some("script")
            | Some("vacuum")
            | Some("media_player")
            | Some("lock")
            | Some("timer")
            | Some("todo")
    )
}

pub use crate::home::registry_yaml::load_home_config;
use crate::home::registry_yaml::merge_area_aliases;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;

    #[test]
    fn german_rooms_get_english_aliases() {
        let office = merge_area_aliases("arbeitszimmer", "Arbeitszimmer", vec![]);
        assert!(office.iter().any(|a| a == "office"), "{office:?}");
        assert!(office.iter().any(|a| a == "study"), "{office:?}");
        let living = merge_area_aliases("wohnzimmer", "Wohnzimmer", vec![]);
        assert!(living.iter().any(|a| a == "living"), "{living:?}");
        let bed = merge_area_aliases("schlafzimmer", "Schlafzimmer", vec![]);
        assert!(bed.iter().any(|a| a == "bedroom"), "{bed:?}");
    }

    #[test]
    fn hue_ids_inherit_room_from_entity_id() {
        let areas = default_home().areas;
        assert_eq!(infer_area("light.schlafzimmer", &areas).as_deref(), Some("schlafzimmer"));
        assert_eq!(infer_area("light.schlafzimmer_ambilight", &areas).as_deref(), Some("schlafzimmer"));
        assert_eq!(infer_area("light.hue_color_lamp_2", &areas), None);
    }

    fn raw(id: &str) -> RawEntity {
        RawEntity {
            entity_id: id.into(),
            name: None,
            original_name: None,
            has_entity_name: false,
            area_id: None,
            platform: None,
            aliases: vec![],
            labels: vec![],
            disabled_by: None,
            device_id: None,
        }
    }

    fn device(name: &str) -> DeviceInfo {
        DeviceInfo { name: name.into(), area_id: None }
    }

    #[test]
    fn display_name_uses_device_when_entity_names_are_empty() {
        let mut entity = raw("climate.better_thermostat_wohnzimmer");
        entity.has_entity_name = true;
        assert_eq!(entity_display_name(&entity, Some(&device("Better Thermostat Wohnzimmer"))), "Better Thermostat Wohnzimmer");
    }

    #[test]
    fn display_name_does_not_repeat_device_and_original() {
        let mut entity = raw("vacuum.r2d2");
        entity.has_entity_name = true;
        entity.original_name = Some(" R2D2".into());
        assert_eq!(entity_display_name(&entity, Some(&device("R2D2"))), "R2D2");
    }

    #[test]
    fn display_name_joins_device_and_distinct_original() {
        let mut entity = raw("switch.better_thermostat_wohnzimmer_child_lock");
        entity.has_entity_name = true;
        entity.original_name = Some("Child Lock".into());
        assert_eq!(entity_display_name(&entity, Some(&device("Better Thermostat Wohnzimmer"))), "Better Thermostat Wohnzimmer Child Lock");
    }

    #[test]
    fn display_name_prefers_user_name() {
        let mut entity = raw("light.wohnzimmer");
        entity.name = Some("Decke".into());
        entity.has_entity_name = true;
        assert_eq!(entity_display_name(&entity, Some(&device("Wohnzimmer Licht"))), "Decke");
    }

    #[test]
    fn display_name_never_keeps_entity_id() {
        let mut entity = raw("light.wohnzimmer");
        entity.name = Some("light.wohnzimmer".into());
        entity.aliases = vec!["Wohnzimmer Licht".into()];
        assert_eq!(entity_display_name(&entity, None), "Wohnzimmer Licht");
        assert_eq!(entity_display_name(&raw("climate.better_thermostat_wohnzimmer"), None), "better thermostat wohnzimmer");
    }
}
