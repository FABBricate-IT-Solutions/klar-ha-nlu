//! Shared Lotse tool argument helpers.

use serde_json::Value;

pub fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key).and_then(Value::as_str).filter(|item| !item.is_empty()).ok_or_else(|| format!("{key} required"))
}

pub fn string_list(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|rows| rows.iter().filter_map(Value::as_str).map(str::trim).filter(|item| !item.is_empty()).map(str::to_string).collect())
        .unwrap_or_default()
}

pub fn arg_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(Value::as_bool)
}
