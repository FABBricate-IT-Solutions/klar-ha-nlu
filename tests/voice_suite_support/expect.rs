use super::schema::{ExpectedIntent, NluExpectation};
use super::RunStats;
use klar_nlu::types::{Intent, ParseResult};
use std::collections::BTreeMap;

pub(super) fn exact_result_ok(expected: &NluExpectation, result: &ParseResult) -> Result<(), String> {
    if let Some(intents) = &expected.intents {
        exact_intents_ok(intents, &result.intents)?;
    }
    if let Some(reject) = expected.reject {
        let actual_reject = result.intents.is_empty() && !result.clarify && !result.chat;
        if actual_reject != reject {
            return Err(format!(
                "expected reject={reject}, got intents={:?} clarify={} chat={}",
                result.intents, result.clarify, result.chat
            ));
        }
    }
    if let Some(clarify) = expected.clarify {
        if result.clarify != clarify {
            return Err(format!("expected clarify={clarify}, got clarify={} intents={:?}", result.clarify, result.intents));
        }
    }
    Ok(())
}

pub(super) fn exact_intents_ok(expected: &[ExpectedIntent], actual: &[Intent]) -> Result<(), String> {
    if expected.len() != actual.len() {
        return Err(format!("exact intents: expected {} in declared order, got {}: {actual:?}", expected.len(), actual.len()));
    }
    for (index, (wanted, got)) in expected.iter().zip(actual).enumerate() {
        if wanted.intent != got.name {
            return Err(format!("exact intent[{index}]: expected {}, got {}", wanted.intent, got.name));
        }
        exact_slots_ok(index, wanted, got)?;
    }
    Ok(())
}

fn exact_slots_ok(index: usize, expected: &ExpectedIntent, actual: &Intent) -> Result<(), String> {
    let expected_slots = expected
        .slots
        .iter()
        .map(|(name, value)| scalar(value).map(|value| (name.clone(), value)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut actual_slots = BTreeMap::new();
    for slot in &actual.slots {
        if actual_slots.insert(slot.name.clone(), slot.value.clone()).is_some() {
            return Err(format!("exact intent[{index}] {} has duplicate slot {}", actual.name, slot.name));
        }
    }
    if expected_slots != actual_slots {
        return Err(format!("exact intent[{index}] {} slots: expected {expected_slots:?}, got {actual_slots:?}", actual.name));
    }
    Ok(())
}

fn scalar(value: &serde_yaml::Value) -> Result<String, String> {
    match value {
        serde_yaml::Value::Null => Ok("null".into()),
        serde_yaml::Value::Bool(value) => Ok(value.to_string()),
        serde_yaml::Value::Number(value) => Ok(value.to_string()),
        serde_yaml::Value::String(value) => Ok(value.clone()),
        _ => Err(format!("expected a scalar value, got {value:?}")),
    }
}

pub(super) fn record_failure(stats: &mut RunStats, group: &str, label: &str, turns: &[String], error: String) {
    stats.fail += 1;
    stats.fails.push(format!("{group}/{label}: {turns:?} → {error}"));
}

pub(super) fn failure_kind(line: &str) -> &'static str {
    for (needle, kind) in [
        ("exact intent", "exact"),
        ("world_expect", "world"),
        ("reject=", "reject"),
        ("clarify", "clarify"),
        ("HassSetPosition", "position"),
        ("HassFanSetSpeed", "fan"),
        ("HassLightSet", "lightset"),
        ("HassClimate", "climate"),
        ("HassGetState", "query"),
        ("HassTurnOff", "off"),
        ("HassTurnOn", "on"),
        ("Timer", "timer"),
        ("Shopping", "list"),
        ("todo", "list"),
    ] {
        if line.contains(needle) {
            return kind;
        }
    }
    "other"
}
