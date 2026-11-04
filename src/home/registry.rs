use crate::parse::normalize::{compact, fold_umlaut};
use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct EntityStorage {
    #[serde(default)]
    minor_version: u32,
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
    /// Home Assistant 2026.11 stores the spoken aliases here. Null means the computed full name.
    #[serde(default, deserialize_with = "optional_strings")]
    aliases_v2: Option<Vec<String>>,
    #[serde(default)]
    compat_name: Option<String>,
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
    #[serde(default)]
    via_device_id: Option<String>,
}

struct DeviceInfo {
    name: Option<String>,
    area_id: Option<String>,
    via_device_id: Option<String>,
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
    #[serde(default)]
    floor_id: Option<String>,
}

#[derive(Deserialize)]
struct FloorStorage {
    data: FloorData,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FloorData {
    Wrapped { floors: Vec<RawFloor> },
    Flat(Vec<RawFloor>),
}

#[derive(Deserialize)]
struct RawFloor {
    #[serde(default)]
    floor_id: Option<String>,
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    level: Option<i32>,
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
    let floors = read_floors(&config_dir.join(".storage/core.floor_registry")).unwrap_or_else(|_| fallback.floors.clone());
    if entities.is_empty() {
        return fallback;
    }
    for ent in &mut entities {
        if ent.area.is_none() {
            ent.area = infer_area(&ent.entity_id, &areas);
        }
    }
    tracing::info!("{} Entitäten, {} Räume, {} Etagen aus Home Assistant geladen", entities.len(), areas.len(), floors.len());
    let mut graph = HomeGraph { entities, areas, floors, assist: crate::home::expose::load_assist(config_dir), ..fallback };
    crate::home::labels::rebuild_label_index(&mut graph);
    graph
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
            if d.id.is_empty() {
                return None;
            }
            let name = clean_label(d.name_by_user).or_else(|| clean_label(d.name));
            let area_id = d.area_id.filter(|id| !id.is_empty());
            let via_device_id = d.via_device_id.filter(|id| !id.is_empty());
            Some((d.id, DeviceInfo { name, area_id, via_device_id }))
        })
        .collect()
}

/// Child first, then the device it is attached to. Home Assistant 2026.11 child devices inherit the parent area.
fn device_lineage<'a>(devices: &'a HashMap<String, DeviceInfo>, start: &str) -> Vec<&'a DeviceInfo> {
    let mut out = Vec::new();
    let mut id = start.to_string();
    let mut seen = Vec::new();
    while out.len() < 4 && !seen.contains(&id) {
        seen.push(id.clone());
        let Some(device) = devices.get(&id) else { break };
        let parent = device.via_device_id.clone();
        out.push(device);
        let Some(parent) = parent else { break };
        id = parent;
    }
    out
}

fn device_area(devices: &HashMap<String, DeviceInfo>, start: &str) -> Option<String> {
    device_lineage(devices, start).into_iter().find_map(|device| device.area_id.clone())
}

fn device_name_chain(devices: &HashMap<String, DeviceInfo>, start: &str) -> Vec<String> {
    let mut names: Vec<String> = device_lineage(devices, start).into_iter().filter_map(|device| device.name.clone()).collect();
    names.reverse();
    names
}

fn strings_skip_null<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    let values: Vec<Option<String>> = Deserialize::deserialize(deserializer)?;
    Ok(values.into_iter().flatten().filter_map(|value| clean_label(Some(value))).collect())
}

fn optional_strings<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<Vec<String>>, D::Error> {
    let values: Option<Vec<Option<String>>> = Option::deserialize(deserializer)?;
    Ok(values.map(|values| values.into_iter().flatten().filter_map(|value| clean_label(Some(value))).collect()))
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

fn compose_parts(parts: &[String]) -> Option<String> {
    let mut kept: Vec<String> = Vec::new();
    for part in parts {
        let fold = compact(part);
        if fold.is_empty() {
            continue;
        }
        if kept.iter().any(|prev| {
            let prev_fold = compact(prev);
            prev_fold == fold || prev_fold.contains(&fold)
        }) {
            continue;
        }
        if let Some(last) = kept.last() {
            if fold.contains(&compact(last)) {
                kept.pop();
            }
        }
        kept.push(part.clone());
    }
    (!kept.is_empty()).then(|| kept.join(" "))
}

fn entity_display_name(entity: &RawEntity, device: Option<&DeviceInfo>, compose_user: bool, chain: &[String]) -> String {
    let user = clean_label(entity.name.clone()).filter(|name| !looks_like_entity_id(name));
    // From registry storage 1.24 the stored name is only the entity part. Home Assistant adds the device.
    if compose_user {
        let mut parts = chain.to_vec();
        if let Some(part) = user.clone().or_else(|| clean_label(entity.original_name.clone()).filter(|name| !looks_like_entity_id(name))) {
            parts.push(part);
        }
        if let Some(name) = compose_parts(&parts) {
            return name;
        }
    } else if let Some(name) = user {
        return name;
    }
    let original = clean_label(entity.original_name.clone()).filter(|name| !looks_like_entity_id(name));
    let device_name = device.and_then(|info| info.name.as_deref());
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
    let compose_user = parsed.minor_version >= 24;
    Ok(parsed
        .data
        .entities
        .into_iter()
        .filter(|e| e.disabled_by.is_none())
        .filter(|e| keep_domain(&e.entity_id))
        .map(|e| {
            let domain = e.entity_id.split('.').next().unwrap_or("").to_string();
            let device_id = e.device_id.as_deref();
            let device = device_id.and_then(|id| devices.get(id));
            let chain = device_id.map(|id| device_name_chain(devices, id)).unwrap_or_default();
            let name = entity_display_name(&e, device, compose_user, &chain);
            let area = e.area_id.clone().filter(|id| !id.is_empty()).or_else(|| device_id.and_then(|id| device_area(devices, id)));
            let mut aliases = e.aliases_v2.unwrap_or(e.aliases);
            if compose_user {
                for extra in [e.compat_name, e.name, e.original_name] {
                    let Some(extra) = clean_label(extra).filter(|value| value != &name && !aliases.contains(value)) else { continue };
                    aliases.push(extra);
                }
            }
            EntityRec {
                entity_id: e.entity_id,
                name,
                domain,
                platform: e.platform,
                area,
                aliases,
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
    let mut areas: Vec<AreaRec> = areas
        .into_iter()
        .map(|a| {
            let area_id = a.id.or(a.area_id).unwrap_or_else(|| fold_umlaut(&a.name));
            AreaRec {
                area_id: area_id.clone(),
                name: a.name.clone(),
                aliases: merge_area_aliases(&area_id, &a.name, a.aliases),
                floor_id: a.floor_id.filter(|id| !id.is_empty()),
            }
        })
        .collect();
    drop_borrowed_area_names(&mut areas);
    Ok(areas)
}

fn read_floors(path: &Path) -> Result<Vec<FloorRec>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: FloorStorage = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let floors = match parsed.data {
        FloorData::Wrapped { floors } => floors,
        FloorData::Flat(floors) => floors,
    };
    Ok(floors
        .into_iter()
        .filter_map(|floor| {
            let floor_id = floor.floor_id.or(floor.id).filter(|id| !id.is_empty())?;
            Some(FloorRec {
                aliases: crate::home::registry_yaml::merge_floor_aliases(&floor_id, &floor.name, floor.level, floor.aliases),
                floor_id,
                name: floor.name,
                level: floor.level,
            })
        })
        .collect())
}

pub(crate) fn keep_domain(entity_id: &str) -> bool {
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
            | Some("calendar")
            | Some("weather")
    )
}

pub use crate::home::registry_yaml::load_home_config;
use crate::home::registry_yaml::{drop_borrowed_area_names, merge_area_aliases};

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
