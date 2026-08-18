use super::schema::{ExpectedIntent, NluExpectation};
use super::waivers;
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

pub(super) fn record_waiver_or_failure(stats: &mut RunStats, suite: &str, group: &str, label: &str, turns: &[String], error: String) {
    let kind = mismatch_kind(&error);
    let fingerprint = mismatch_fingerprint(&error);
    match waivers::matching(suite, group, label, kind, fingerprint) {
        Some(waiver) => {
            stats.waived += 1;
            stats.used_waivers.insert(waiver.id);
            stats
                .waivers
                .push(format!("{} {group}/{label}: {turns:?} — kind={kind} fingerprint={fingerprint:016x} — {}", waiver.id, waiver.reason));
        }
        None => {
            let expected = waivers::for_case(suite, group, label)
                .map(|waiver| format!("{}={}:{:016x}", waiver.id, waiver.kind, waiver.fingerprint))
                .collect::<Vec<_>>();
            let detail = if expected.is_empty() { "no waiver".into() } else { format!("expected {}", expected.join(",")) };
            record_failure(
                stats,
                group,
                label,
                turns,
                format!("unwaived mismatch kind={kind} fingerprint={fingerprint:016x} ({detail}): {error}"),
            );
        }
    }
}

fn mismatch_kind(error: &str) -> &'static str {
    if error.starts_with("V1/V2 parity mismatch") {
        "parity"
    } else {
        "oracle"
    }
}

fn mismatch_fingerprint(error: &str) -> u64 {
    let normalized = normalize_mismatch(error);
    normalized.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3))
}

fn normalize_mismatch(error: &str) -> String {
    if error.starts_with("V1/V2 parity mismatch") {
        let legacy = parse_result_intents(error, "legacy=ParseResult").unwrap_or("missing");
        let current = parse_result_intents(error, "current=ParseResult").unwrap_or("missing");
        return format!("legacy={legacy}|current={current}");
    }
    let intent_names = error.split("Intent { name: \"").skip(1).filter_map(|tail| tail.split('"').next()).collect::<Vec<_>>().join(",");
    if !intent_names.is_empty() {
        return format!("oracle:{}:{intent_names}", failure_kind(error));
    }
    let mut normalized = String::with_capacity(error.len());
    let mut index = 0;
    let bytes = error.as_bytes();
    while index < bytes.len() {
        if error.get(index..index + 36).is_some_and(uuid_like) {
            normalized.push_str("<conversation-id>");
            index += 36;
        } else {
            let character = error[index..].chars().next().expect("valid string boundary");
            if !character.is_whitespace() || !normalized.ends_with(' ') {
                normalized.push(if character.is_whitespace() { ' ' } else { character });
            }
            index += character.len_utf8();
        }
    }
    normalized
}

fn parse_result_intents<'a>(error: &'a str, marker: &str) -> Option<&'a str> {
    let result = error.split_once(marker)?.1;
    let intents = result.split_once("intents: [")?.1;
    intents.split_once("], speech:").map(|(value, _)| value)
}

fn uuid_like(value: &str) -> bool {
    value.char_indices().all(|(index, character)| {
        matches!(index, 8 | 13 | 18 | 23) && character == '-' || !matches!(index, 8 | 13 | 18 | 23) && character.is_ascii_hexdigit()
    })
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
