//! Temperature only. Operator `unit_system` vs HA climate/weather unit.
//! Integer pairing keeps `70°F → 21°C → 70°F` from drifting.

use crate::types::{Intent, SpeechEntity, UnitSystem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempScale {
    Celsius,
    Fahrenheit,
}

impl TempScale {
    pub fn from_unit_system(system: UnitSystem) -> Self {
        match system {
            UnitSystem::Metric => Self::Celsius,
            UnitSystem::Imperial => Self::Fahrenheit,
        }
    }

    pub fn from_label(raw: &str) -> Self {
        let compact: String = raw.chars().filter(|ch| !ch.is_whitespace()).collect::<String>().to_ascii_lowercase();
        if compact.contains("fahrenheit") || compact.contains("°f") || compact.ends_with('f') && compact.contains('°') {
            return Self::Fahrenheit;
        }
        if matches!(compact.as_str(), "f" | "°f") {
            return Self::Fahrenheit;
        }
        Self::Celsius
    }
}

/// Stable integer map used for spoken whole degrees.
pub fn fahrenheit_to_celsius(f: i32) -> i32 {
    ((f - 32) * 5 + 4) / 9
}

pub fn celsius_to_fahrenheit(c: i32) -> i32 {
    (c * 9 + 2) / 5 + 32
}

pub fn fahrenheit_to_celsius_f(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

pub fn celsius_to_fahrenheit_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

/// Explicit `celsius` / `fahrenheit` / `°C` / `°F` on the utterance.
pub fn uttered_scale(tokens: &[String]) -> Option<TempScale> {
    tokens.iter().find_map(|token| token_scale(token))
}

fn token_scale(token: &str) -> Option<TempScale> {
    let raw = token.to_ascii_lowercase();
    if raw.contains("fahrenheit") || raw.contains("°f") {
        return Some(TempScale::Fahrenheit);
    }
    if raw.contains("celsius")
        || raw.contains("zelsius")
        || raw.contains("centigrade")
        || raw.contains("°c")
        || raw == "grad"
        || raw == "grads"
    {
        return Some(TempScale::Celsius);
    }
    None
}

/// Spoken number → HA Celsius (parse treats HA as metric).
pub fn to_ha_celsius(spoken: i32, tokens: &[String], operator: UnitSystem) -> i32 {
    let scale = uttered_scale(tokens).unwrap_or_else(|| TempScale::from_unit_system(operator));
    match scale {
        TempScale::Celsius => spoken,
        TempScale::Fahrenheit => fahrenheit_to_celsius(spoken),
    }
}

/// Rewrite a bound set-temp slot into the HA Celsius value.
pub fn bind_set_temp(intent: &mut Intent, tokens: &[String], operator: UnitSystem) {
    if intent.name != "HassClimateSetTemperature" {
        return;
    }
    let Some(raw) = intent.slot("temperature") else {
        return;
    };
    let Ok(spoken) = raw.parse::<i32>() else {
        return;
    };
    let ha = to_ha_celsius(spoken, tokens, operator);
    if ha.to_string() == raw {
        return;
    }
    *intent = intent.clone().with_set("temperature", ha.to_string());
}

pub fn convert_value(value: f64, from: TempScale, to: UnitSystem) -> f64 {
    let dest = TempScale::from_unit_system(to);
    if from == dest {
        return value;
    }
    match (from, dest) {
        (TempScale::Celsius, TempScale::Fahrenheit) => {
            if nearly_int(value) {
                f64::from(celsius_to_fahrenheit(value.round() as i32))
            } else {
                celsius_to_fahrenheit_f(value)
            }
        }
        (TempScale::Fahrenheit, TempScale::Celsius) => {
            if nearly_int(value) {
                f64::from(fahrenheit_to_celsius(value.round() as i32))
            } else {
                fahrenheit_to_celsius_f(value)
            }
        }
        (TempScale::Celsius, TempScale::Celsius) | (TempScale::Fahrenheit, TempScale::Fahrenheit) => value,
    }
}

pub fn format_temp(value: f64) -> String {
    if nearly_int(value) {
        return format!("{}", value.round() as i32);
    }
    let text = format!("{value:.1}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn spoken_unit_word(system: UnitSystem, de: bool) -> &'static str {
    match system {
        UnitSystem::Metric if de => "Grad",
        UnitSystem::Metric => "degrees",
        UnitSystem::Imperial => "Fahrenheit",
    }
}

pub fn entity_temperature(entity: &SpeechEntity) -> Option<(f64, TempScale)> {
    let raw = attr_num(entity, "current_temperature").or_else(|| attr_num(entity, "temperature"))?;
    Some((raw, entity_temp_scale(Some(entity))))
}

pub fn entity_temp_scale(entity: Option<&SpeechEntity>) -> TempScale {
    let Some(entity) = entity else {
        return TempScale::Celsius;
    };
    if let Some(label) = attr_str(entity, "temperature_unit").or_else(|| attr_str(entity, "unit_of_measurement")) {
        return TempScale::from_label(&label);
    }
    TempScale::Celsius
}

pub fn speak_temp(raw: &str, ha: TempScale, operator: UnitSystem) -> String {
    let Ok(value) = raw.parse::<f64>() else {
        return raw.to_string();
    };
    format_temp(convert_value(value, ha, operator))
}

pub fn speak_converted(value: f64, ha: TempScale, operator: UnitSystem) -> String {
    format_temp(convert_value(value, ha, operator))
}

fn attr_str(entity: &SpeechEntity, key: &str) -> Option<String> {
    match entity.attributes.get(key)? {
        serde_json::Value::String(text) if !text.is_empty() => Some(text.clone()),
        serde_json::Value::Number(num) => Some(num.to_string()),
        _ => None,
    }
}

fn attr_num(entity: &SpeechEntity, key: &str) -> Option<f64> {
    match entity.attributes.get(key)? {
        serde_json::Value::Number(num) => num.as_f64(),
        serde_json::Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn nearly_int(value: f64) -> bool {
    (value - value.round()).abs() < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seventy_fahrenheit_round_trips() {
        assert_eq!(fahrenheit_to_celsius(70), 21);
        assert_eq!(celsius_to_fahrenheit(21), 70);
        assert_eq!(fahrenheit_to_celsius(32), 0);
        assert_eq!(celsius_to_fahrenheit(0), 32);
        assert_eq!(fahrenheit_to_celsius(212), 100);
        assert_eq!(celsius_to_fahrenheit(100), 212);
    }

    #[test]
    fn parse_uses_setting_unless_word_overrides() {
        let bare = ["set".into(), "to".into(), "70".into()];
        assert_eq!(to_ha_celsius(70, &bare, UnitSystem::Imperial), 21);
        assert_eq!(to_ha_celsius(21, &["21".into(), "grad".into()], UnitSystem::Metric), 21);
        assert_eq!(to_ha_celsius(21, &["21".into(), "grad".into()], UnitSystem::Imperial), 21);
        let explicit = ["70".into(), "fahrenheit".into()];
        assert_eq!(to_ha_celsius(70, &explicit, UnitSystem::Metric), 21);
        let celsius = ["21".into(), "celsius".into()];
        assert_eq!(to_ha_celsius(21, &celsius, UnitSystem::Imperial), 21);
    }

    #[test]
    fn speech_converts_celsius_snapshot_to_fahrenheit() {
        assert_eq!(format_temp(convert_value(21.0, TempScale::Celsius, UnitSystem::Imperial)), "70");
        assert_eq!(format_temp(convert_value(21.5, TempScale::Celsius, UnitSystem::Imperial)), "70.7");
        assert_eq!(format_temp(convert_value(21.5, TempScale::Celsius, UnitSystem::Metric)), "21.5");
        assert_eq!(spoken_unit_word(UnitSystem::Imperial, true), "Fahrenheit");
        assert_eq!(spoken_unit_word(UnitSystem::Metric, true), "Grad");
    }
}
