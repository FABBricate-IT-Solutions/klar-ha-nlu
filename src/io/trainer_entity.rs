//! Lotse entity reads and writes: flags, alias replace, gaps with suggested rooms.

use crate::home::assignment::suggested_area;
use crate::home::expose::assist_visible;
use crate::home::gaps::leftover;
use crate::home::overlay::{apply_overlay, load_overlay, save_overlay};
use crate::io::state::AppState;
use crate::io::trainer_args::{arg_bool, arg_str, string_list};
use crate::io::trainer_reads::with_view;
use crate::lang::catalog_for;
use crate::types::EntityRec;
use serde_json::{json, Value};

pub async fn search_house(state: &AppState, args: &Value) -> Result<Value, String> {
    let query = arg_str(args, "q")?.to_lowercase();
    let home = state.home.snapshot().await;
    let entities: Vec<Value> = home
        .entities
        .iter()
        .filter(|entity| {
            entity.entity_id.to_lowercase().contains(&query)
                || entity.name.to_lowercase().contains(&query)
                || entity.aliases.iter().any(|alias| alias.to_lowercase().contains(&query))
        })
        .take(24)
        .map(|entity| entity_card(entity, &home))
        .collect();
    let areas: Vec<Value> = home
        .areas
        .iter()
        .filter(|area| area.area_id.to_lowercase().contains(&query) || area.name.to_lowercase().contains(&query))
        .take(12)
        .map(|area| json!({"area_id": area.area_id, "name": area.name, "floor": area.floor_id}))
        .collect();
    let floors: Vec<Value> = home
        .floors
        .iter()
        .filter(|floor| {
            floor.floor_id.to_lowercase().contains(&query)
                || floor.name.to_lowercase().contains(&query)
                || floor.aliases.iter().any(|alias| alias.to_lowercase().contains(&query))
        })
        .take(8)
        .map(|floor| json!({"floor_id": floor.floor_id, "name": floor.name, "aliases": floor.aliases}))
        .collect();
    Ok(with_view("house", json!({ "entities": entities, "areas": areas, "floors": floors })))
}

pub async fn get_entity(state: &AppState, args: &Value) -> Result<Value, String> {
    let entity_id = arg_str(args, "entity_id")?;
    let home = state.home.snapshot().await;
    home.entities
        .iter()
        .find(|entity| entity.entity_id == entity_id)
        .map(|entity| with_view("entity", entity_card(entity, &home)))
        .ok_or_else(|| "entity is not on the graph".into())
}

pub async fn list_gaps(state: &AppState) -> Result<Value, String> {
    let settings = state.settings.lock().await.clone();
    let home = state.home.snapshot().await;
    let catalog = catalog_for(&settings.languages);
    let gaps: Vec<Value> = leftover(&home, catalog)
        .into_iter()
        .map(|entity| {
            let reason = if entity.area.is_none() { "missing_area" } else { "weak_name" };
            let suggested = suggested_area(&entity, &home);
            json!({
                "entity_id": entity.entity_id,
                "name": entity.name,
                "area": entity.area,
                "reason": reason,
                "exposed": assist_visible(&entity, &home),
                "suggested_area": suggested.as_ref().map(|row| row.area_id.clone()),
                "suggested_name": suggested.as_ref().map(|row| row.name.clone()),
                "suggested_score": suggested.as_ref().map(|row| row.score),
            })
        })
        .collect();
    Ok(with_view("gaps", json!({ "gaps": gaps })))
}

pub async fn preview_entity(state: &AppState, args: &Value) -> Result<Value, String> {
    let change = entity_change(args)?;
    let home = state.home.snapshot().await;
    if !home.entities.iter().any(|entity| entity.entity_id == change.entity_id) {
        return Err("entity is not on the graph".into());
    }
    Ok(json!({"ok": true, "errors": [], "warnings": [], "dry_run": [], "entity_id": change.entity_id}))
}

pub async fn apply_entity(state: &AppState, args: &Value) -> Result<Value, String> {
    let preview = preview_entity(state, args).await?;
    if preview.get("ok") != Some(&json!(true)) {
        return Err(preview.to_string());
    }
    let change = entity_change(args)?;
    persist_entity(state, &change).await?;
    Ok(with_view(
        "write",
        json!({
            "ok": true,
            "entity_id": change.entity_id,
            "aliases": change.aliases,
            "remove_aliases": change.remove_aliases,
            "preferred": change.preferred,
            "nlu_ignore": change.nlu_ignore,
        }),
    ))
}

fn entity_card(entity: &EntityRec, home: &crate::types::HomeGraph) -> Value {
    let suggested = suggested_area(entity, home);
    json!({
        "entity_id": entity.entity_id,
        "name": entity.name,
        "area": entity.area,
        "aliases": entity.aliases,
        "tags": entity.tags,
        "domain": entity.domain,
        "exposed": assist_visible(entity, home),
        "preferred": entity.tags.iter().any(|tag| tag == "preferred"),
        "nlu_ignore": entity.tags.iter().any(|tag| tag == "nlu_ignore"),
        "suggested_area": suggested.as_ref().map(|row| row.area_id.clone()),
        "suggested_name": suggested.as_ref().map(|row| row.name.clone()),
    })
}

struct EntityChange {
    entity_id: String,
    aliases: Option<Vec<String>>,
    remove_aliases: Vec<String>,
    preferred: Option<bool>,
    nlu_ignore: Option<bool>,
}

fn entity_change(args: &Value) -> Result<EntityChange, String> {
    let entity_id = arg_str(args, "entity_id")?.to_string();
    let aliases = args
        .get("aliases")
        .and_then(Value::as_array)
        .map(|_| string_list(args, "aliases").into_iter().filter(|alias| valid_alias(alias)).collect::<Vec<_>>());
    let remove_aliases = string_list(args, "remove_aliases");
    let preferred = arg_bool(args, "preferred");
    let nlu_ignore = arg_bool(args, "nlu_ignore");
    if aliases.is_none() && remove_aliases.is_empty() && preferred.is_none() && nlu_ignore.is_none() {
        return Err("aliases, remove_aliases, preferred, or nlu_ignore required".into());
    }
    Ok(EntityChange { entity_id, aliases, remove_aliases, preferred, nlu_ignore })
}

fn valid_alias(alias: &str) -> bool {
    let chars = alias.chars().count();
    (2..=40).contains(&chars) && !alias.chars().any(char::is_control)
}

async fn persist_entity(state: &AppState, change: &EntityChange) -> Result<(), String> {
    let mut overlay = load_overlay(&state.data_dir);
    if let Some(aliases) = &change.aliases {
        overlay.aliases.insert(change.entity_id.clone(), aliases.clone());
    }
    if !change.remove_aliases.is_empty() {
        let list = overlay.aliases.entry(change.entity_id.clone()).or_default();
        list.retain(|alias| !change.remove_aliases.iter().any(|drop| drop == alias));
    }
    if let Some(preferred) = change.preferred {
        overlay.preferred.retain(|id| id != &change.entity_id);
        if preferred {
            overlay.preferred.push(change.entity_id.clone());
        }
    }
    if let Some(ignore) = change.nlu_ignore {
        overlay.nlu_ignore.retain(|id| id != &change.entity_id);
        if ignore {
            overlay.nlu_ignore.push(change.entity_id.clone());
        }
    }
    save_overlay(&state.data_dir, &overlay).map_err(|_| "save overlay")?;
    state
        .home
        .edit(|next| {
            apply_overlay(next, &overlay);
            let ent = next.entities.iter_mut().find(|entity| entity.entity_id == change.entity_id)?;
            if let Some(aliases) = &change.aliases {
                if !aliases.is_empty() {
                    ent.aliases = aliases.clone();
                }
            }
            for drop in &change.remove_aliases {
                ent.aliases.retain(|alias| alias != drop);
            }
            let mut tags = ent.tags.clone();
            tags.retain(|tag| tag != "preferred" && tag != "nlu_ignore");
            if overlay.preferred.iter().any(|id| id == &change.entity_id) {
                tags.push("preferred".into());
            }
            if overlay.nlu_ignore.iter().any(|id| id == &change.entity_id) {
                tags.push("nlu_ignore".into());
            }
            ent.tags = tags;
            Some(())
        })
        .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::types::Settings;

    fn state(tag: &str) -> AppState {
        let dir = std::env::temp_dir().join(format!("klar-trainer-entity-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::pinned("de"),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir,
            None,
        )
    }

    #[tokio::test]
    async fn apply_entity_sets_flags_and_replaces_aliases() {
        let state = state("flags");
        apply_entity(&state, &json!({"entity_id":"light.wohnzimmer","aliases":["kugel"],"preferred":true,"nlu_ignore":true}))
            .await
            .unwrap();
        let home = state.home.snapshot().await;
        let entity = home.entities.iter().find(|item| item.entity_id == "light.wohnzimmer").unwrap();
        assert_eq!(entity.aliases, vec!["kugel"]);
        assert!(entity.tags.iter().any(|tag| tag == "preferred"));
        assert!(entity.tags.iter().any(|tag| tag == "nlu_ignore"));
        apply_entity(&state, &json!({"entity_id":"light.wohnzimmer","remove_aliases":["kugel"],"preferred":false})).await.unwrap();
        let home = state.home.snapshot().await;
        let entity = home.entities.iter().find(|item| item.entity_id == "light.wohnzimmer").unwrap();
        assert!(!entity.aliases.iter().any(|alias| alias == "kugel"));
        assert!(!entity.tags.iter().any(|tag| tag == "preferred"));
        assert!(preview_entity(&state, &json!({"entity_id":"light.missing","preferred":true})).await.is_err());
    }

    #[tokio::test]
    async fn list_gaps_includes_suggested_area() {
        let state = state("gaps");
        state.apply_areas(&[("light.wohnzimmer".into(), String::new())]).await;
        let out = list_gaps(&state).await.unwrap();
        let gaps = out["gaps"].as_array().unwrap();
        let row = gaps.iter().find(|item| item["entity_id"] == "light.wohnzimmer").unwrap();
        assert_eq!(row["reason"], "missing_area");
        assert_eq!(row["suggested_area"], "wohnzimmer");
    }
}
