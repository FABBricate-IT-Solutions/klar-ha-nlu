use super::schema::StateRecord;
use klar_nlu::types::{HomeGraph, Intent};
use std::collections::BTreeMap;

const ENTITY_FIELDS: &[&str] = &[
    "state",
    "brightness",
    "color",
    "percentage",
    "position",
    "temperature",
    "volume_level",
    "volume_step",
    "is_volume_muted",
    "hours",
    "minutes",
    "seconds",
];

#[derive(Debug, Default)]
pub(super) struct TestWorld {
    entities: BTreeMap<String, BTreeMap<String, String>>,
    shopping_list: BTreeMap<String, bool>,
    todo_lists: BTreeMap<String, BTreeMap<String, bool>>,
}

impl TestWorld {
    pub(super) fn from_setup(records: &[StateRecord]) -> Result<Self, String> {
        let mut world = Self::default();
        for (index, record) in records.iter().enumerate() {
            world.apply_record(record).map_err(|error| format!("setup[{index}]: {error}"))?;
        }
        Ok(world)
    }

    pub(super) fn validate_expectations(records: &[StateRecord]) -> Result<(), String> {
        for (index, record) in records.iter().enumerate() {
            validate_record(record).map_err(|error| format!("world_expect[{index}]: {error}"))?;
        }
        Ok(())
    }

    pub(super) fn apply_intents(&mut self, intents: &[Intent], home: &HomeGraph, strict: bool) -> Result<(), String> {
        for intent in intents {
            if let Err(error) = self.apply_intent(intent, home, strict) {
                if strict {
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    pub(super) fn assert_records(&self, records: &[StateRecord]) -> Result<(), String> {
        for (index, record) in records.iter().enumerate() {
            self.assert_record(record).map_err(|error| format!("world_expect[{index}]: {error}"))?;
        }
        Ok(())
    }

    fn apply_record(&mut self, record: &StateRecord) -> Result<(), String> {
        match validate_record(record)? {
            Record::Entity { entity_id, values } => {
                self.entities.insert(entity_id, values);
            }
            Record::ShoppingItem(item) => {
                self.shopping_list.insert(item, false);
            }
            Record::ShoppingCompletedItem(item) => {
                self.shopping_list.insert(item, true);
            }
            Record::TodoItem { list_name, item } => {
                self.todo_lists.entry(list_name).or_default().insert(item, false);
            }
            Record::TodoCompletedItem { list_name, item } => {
                self.todo_lists.entry(list_name).or_default().insert(item, true);
            }
        }
        Ok(())
    }

    fn assert_record(&self, record: &StateRecord) -> Result<(), String> {
        match validate_record(record)? {
            Record::Entity { entity_id, values } => {
                let actual = self.entities.get(&entity_id).ok_or_else(|| format!("missing entity {entity_id}"))?;
                for (field, expected) in values {
                    if actual.get(&field) != Some(&expected) {
                        return Err(format!("{entity_id}.{field}: expected {expected:?}, got {:?}", actual.get(&field)));
                    }
                }
            }
            Record::ShoppingItem(item) => {
                if !self.shopping_list.contains_key(&item) {
                    return Err(format!("shopping list is missing {item:?}"));
                }
            }
            Record::ShoppingCompletedItem(item) => {
                if self.shopping_list.get(&item) != Some(&true) {
                    return Err(format!("shopping list item {item:?} is not completed"));
                }
            }
            Record::TodoItem { list_name, item } => {
                if !self.todo_lists.get(&list_name).is_some_and(|items| items.contains_key(&item)) {
                    return Err(format!("todo list {list_name:?} is missing {item:?}"));
                }
            }
            Record::TodoCompletedItem { list_name, item } => {
                if self.todo_lists.get(&list_name).and_then(|items| items.get(&item)) != Some(&true) {
                    return Err(format!("todo list {list_name:?} item {item:?} is not completed"));
                }
            }
        }
        Ok(())
    }

    fn apply_intent(&mut self, intent: &Intent, home: &HomeGraph, strict: bool) -> Result<(), String> {
        match intent.name.as_str() {
            "HassShoppingListAddItem" => {
                let item = required_slot(intent, "item")?;
                self.shopping_list.insert(item.to_string(), false);
                return Ok(());
            }
            "HassShoppingListCompleteItem" => {
                let item = required_slot(intent, "item")?;
                if !self.shopping_list.contains_key(item) {
                    return Err(format!("cannot complete missing shopping list item {item:?}"));
                }
                self.shopping_list.insert(item.to_string(), true);
                return Ok(());
            }
            "HassListAddItem" | "HassListCompleteItem" => {
                let list_name = intent
                    .slot("entity_id")
                    .or_else(|| intent.slot("name"))
                    .ok_or_else(|| format!("{} requires entity_id or name", intent.name))?;
                let item = required_slot(intent, "item")?;
                let completed = intent.name == "HassListCompleteItem";
                if list_name == "shopping_list" {
                    if completed && !self.shopping_list.contains_key(item) {
                        return Err(format!("cannot complete missing shopping list item {item:?}"));
                    }
                    self.shopping_list.insert(item.to_string(), completed);
                } else {
                    let items = self.todo_lists.entry(list_name.to_string()).or_default();
                    if completed && !items.contains_key(item) {
                        return Err(format!("cannot complete missing todo item {item:?} in {list_name:?}"));
                    }
                    items.insert(item.to_string(), completed);
                }
                return Ok(());
            }
            "HassGetState" | "HassClimateGetTemperature" => return Ok(()),
            _ => {}
        }
        if !entity_intent_supported(&intent.name) {
            return if strict { Err(format!("world_expect cannot simulate unsupported intent {}", intent.name)) } else { Ok(()) };
        }
        let targets = target_entities(intent, home);
        if strict && targets.is_empty() {
            return Err(format!("world_expect cannot apply {} without a target", intent.name));
        }
        for entity_id in targets {
            self.apply_entity_intent(&entity_id, intent)?;
        }
        Ok(())
    }

    fn apply_entity_intent(&mut self, entity_id: &str, intent: &Intent) -> Result<(), String> {
        let state = self.entities.entry(entity_id.to_string()).or_default();
        match intent.name.as_str() {
            "HassTurnOn" => {
                state.insert("state".into(), on_state(entity_id).into());
            }
            "HassTurnOff" => {
                state.insert("state".into(), off_state(entity_id).into());
            }
            "HassToggle" => {
                let active =
                    state.get("state").is_some_and(|value| matches!(value.as_str(), "on" | "open" | "locked" | "active" | "playing"));
                state.insert("state".into(), if active { off_state(entity_id) } else { on_state(entity_id) }.into());
            }
            "HassMediaPause" | "HassPauseTimer" => {
                state.insert("state".into(), "paused".into());
            }
            "HassMediaUnpause" => {
                state.insert("state".into(), "playing".into());
            }
            "HassCancelTimer" => {
                state.insert("state".into(), "idle".into());
            }
            "HassStartTimer" => {
                state.insert("state".into(), "active".into());
                let duration = intent_duration(intent);
                if duration > 0 {
                    set_duration(state, duration);
                }
            }
            "HassIncreaseTimer" => {
                let duration = state_duration(state) + intent_duration(intent);
                set_duration(state, duration);
            }
            "HassDecreaseTimer" => {
                let duration = state_duration(state).saturating_sub(intent_duration(intent));
                set_duration(state, duration);
            }
            "HassMediaPlayerMute" => {
                state.insert("is_volume_muted".into(), "true".into());
            }
            "HassMediaPlayerUnmute" => {
                state.insert("is_volume_muted".into(), "false".into());
            }
            "HassSetVolumeRelative" => {
                state.insert("volume_step".into(), required_slot(intent, "volume_step")?.into());
            }
            "HassLightSet" | "HassClimateSetTemperature" | "HassFanSetSpeed" | "HassSetPosition" | "HassSetVolume" => {
                copy_state_slots(state, intent)
            }
            other => return Err(format!("unsupported entity transition {other}")),
        }
        Ok(())
    }
}

enum Record {
    Entity { entity_id: String, values: BTreeMap<String, String> },
    ShoppingItem(String),
    ShoppingCompletedItem(String),
    TodoItem { list_name: String, item: String },
    TodoCompletedItem { list_name: String, item: String },
}

fn validate_record(record: &StateRecord) -> Result<Record, String> {
    let values = record
        .values
        .iter()
        .map(|(name, value)| scalar(value).map(|value| (name.clone(), value)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    if values.contains_key("entity_id") {
        let unknown: Vec<&str> =
            values.keys().map(String::as_str).filter(|field| *field != "entity_id" && !ENTITY_FIELDS.contains(field)).collect();
        if !unknown.is_empty() {
            return Err(format!("unknown entity state fields {unknown:?}"));
        }
        let mut state = values;
        let entity_id = state.remove("entity_id").expect("checked");
        return Ok(Record::Entity { entity_id, values: state });
    }
    if values.len() == 1 {
        if let Some(item) = values.get("shopping_list_item") {
            return Ok(Record::ShoppingItem(item.clone()));
        }
        if let Some(item) = values.get("shopping_list_completed_item") {
            return Ok(Record::ShoppingCompletedItem(item.clone()));
        }
    }
    if values.len() == 2 {
        if let (Some(list_name), Some(item)) = (values.get("list_name"), values.get("todo_item")) {
            return Ok(Record::TodoItem { list_name: list_name.clone(), item: item.clone() });
        }
        if let (Some(list_name), Some(item)) = (values.get("list_name"), values.get("todo_completed_item")) {
            return Ok(Record::TodoCompletedItem { list_name: list_name.clone(), item: item.clone() });
        }
    }
    Err(format!(
        "unsupported state record; expected entity_id, shopping_list item, or list_name + todo item, got {:?}",
        values.keys().collect::<Vec<_>>()
    ))
}

fn required_slot<'a>(intent: &'a Intent, name: &str) -> Result<&'a str, String> {
    intent.slot(name).ok_or_else(|| format!("{} requires slot {name}", intent.name))
}

fn entity_intent_supported(name: &str) -> bool {
    matches!(
        name,
        "HassTurnOn"
            | "HassTurnOff"
            | "HassToggle"
            | "HassMediaPause"
            | "HassMediaUnpause"
            | "HassMediaPlayerMute"
            | "HassMediaPlayerUnmute"
            | "HassSetVolume"
            | "HassSetVolumeRelative"
            | "HassCancelTimer"
            | "HassPauseTimer"
            | "HassStartTimer"
            | "HassIncreaseTimer"
            | "HassDecreaseTimer"
            | "HassLightSet"
            | "HassClimateSetTemperature"
            | "HassFanSetSpeed"
            | "HassSetPosition"
    )
}

fn scalar(value: &serde_yaml::Value) -> Result<String, String> {
    match value {
        serde_yaml::Value::Null => Ok("null".into()),
        serde_yaml::Value::Bool(value) => Ok(value.to_string()),
        serde_yaml::Value::Number(value) => Ok(value.to_string()),
        serde_yaml::Value::String(value) => Ok(value.clone()),
        _ => Err(format!("expected scalar state value, got {value:?}")),
    }
}

fn target_entities(intent: &Intent, home: &HomeGraph) -> Vec<String> {
    if let Some(entity_id) = intent.slot("entity_id") {
        return vec![entity_id.to_string()];
    }
    let Some(area) = intent.slot("area") else {
        return Vec::new();
    };
    let domain = intent.slot("domain");
    home.entities
        .iter()
        .filter(|entity| entity.area.as_deref() == Some(area) && domain.is_none_or(|wanted| entity.domain == wanted))
        .map(|entity| entity.entity_id.clone())
        .collect()
}

fn copy_state_slots(state: &mut BTreeMap<String, String>, intent: &Intent) {
    for field in ENTITY_FIELDS {
        if let Some(value) = intent.slot(field) {
            state.insert((*field).into(), value.into());
        }
    }
}

fn on_state(entity_id: &str) -> &'static str {
    if entity_id.starts_with("cover.") {
        "open"
    } else if entity_id.starts_with("lock.") {
        "locked"
    } else {
        "on"
    }
}

fn off_state(entity_id: &str) -> &'static str {
    if entity_id.starts_with("cover.") {
        "closed"
    } else if entity_id.starts_with("lock.") {
        "unlocked"
    } else {
        "off"
    }
}

fn intent_duration(intent: &Intent) -> u64 {
    duration(intent.slot("hours"), intent.slot("minutes"), intent.slot("seconds"))
}

fn state_duration(state: &BTreeMap<String, String>) -> u64 {
    duration(state.get("hours").map(String::as_str), state.get("minutes").map(String::as_str), state.get("seconds").map(String::as_str))
}

fn duration(hours: Option<&str>, minutes: Option<&str>, seconds: Option<&str>) -> u64 {
    hours.and_then(|value| value.parse::<u64>().ok()).unwrap_or(0) * 3600
        + minutes.and_then(|value| value.parse::<u64>().ok()).unwrap_or(0) * 60
        + seconds.and_then(|value| value.parse::<u64>().ok()).unwrap_or(0)
}

fn set_duration(state: &mut BTreeMap<String, String>, duration: u64) {
    state.remove("hours");
    state.remove("minutes");
    state.remove("seconds");
    let hours = duration / 3600;
    let minutes = (duration % 3600) / 60;
    let seconds = duration % 60;
    if hours > 0 {
        state.insert("hours".into(), hours.to_string());
    }
    if minutes > 0 {
        state.insert("minutes".into(), minutes.to_string());
    }
    if seconds > 0 {
        state.insert("seconds".into(), seconds.to_string());
    }
}
