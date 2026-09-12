//! Outdoor weather lines. Never names a room.

use crate::types::{SpeechEntity, SpeechForecast, SpeechSnapshot};
use crate::units::{entity_temperature, speak_converted};

use super::render_place::slot;
use super::weather_i18n::{frames_for, locale_temp, speak_condition, unit_word, weekday_label};

const RAINY: &[&str] = &["rainy", "pouring", "lightning-rainy", "snowy-rainy", "hail"];

pub(super) fn weather_query(snap: &SpeechSnapshot) -> Option<String> {
    if !is_weather(snap) {
        return None;
    }
    let ask = slot(snap, "weather_ask").unwrap_or("now");
    let frames = frames_for(&snap.language);
    let wet = will_rain(snap);
    let line = match ask {
        "rain" => frames[if wet { 3 } else { 4 }].to_string(),
        "umbrella" => frames[if wet { 5 } else { 6 }].to_string(),
        _ => {
            let (condition, temp, used_forecast) = facts(snap)?;
            let unit = unit_word(&snap.language, snap.unit_system);
            let template = if used_forecast { day_template(snap, frames) } else { frames[0] };
            let mut spoken = fill(template, &condition, &temp, unit);
            if used_forecast {
                if let Some(label) = weekday_label(&snap.language, day_key(snap)) {
                    spoken = format!("{label} {spoken}");
                }
            }
            spoken
        }
    };
    Some(line)
}

fn is_weather(snap: &SpeechSnapshot) -> bool {
    slot(snap, "weather_ask").is_some()
        || slot(snap, "domain") == Some("weather")
        || snap.entities.iter().any(|entity| entity.domain == "weather")
        || slot(snap, "entity_id").is_some_and(|id| id.starts_with("weather."))
}

fn facts(snap: &SpeechSnapshot) -> Option<(String, String, bool)> {
    if let Some(day) = pick_day(snap) {
        let condition = speak_condition(&day.condition, &snap.language);
        let temp = day
            .temperature
            .map(|raw| locale_temp(speak_converted(raw, crate::units::TempScale::Celsius, snap.unit_system), &snap.language))
            .unwrap_or_default();
        if !condition.is_empty() || !temp.is_empty() {
            return Some((condition, temp, true));
        }
    }
    current_facts(snap).map(|(condition, temp)| (condition, temp, false))
}

fn current_facts(snap: &SpeechSnapshot) -> Option<(String, String)> {
    let entity = weather_entity(snap)?;
    let condition = speak_condition(&entity.state, &snap.language);
    let temp = entity_temperature(entity)
        .map(|(raw, ha)| locale_temp(speak_converted(raw, ha, snap.unit_system), &snap.language))
        .unwrap_or_default();
    if condition.is_empty() && temp.is_empty() {
        return None;
    }
    Some((condition, temp))
}

fn day_template(snap: &SpeechSnapshot, frames: [&'static str; 7]) -> &'static str {
    match day_key(snap) {
        "tomorrow" => frames[2],
        "today" => frames[1],
        "weekend" | "mon" | "tue" | "wed" | "thu" | "fri" | "sat" | "sun" => frames[0],
        _ => frames[0],
    }
}

fn day_key(snap: &SpeechSnapshot) -> &str {
    slot(snap, "weather_day").or_else(|| slot(snap, "weather_ask")).unwrap_or("now")
}

fn weather_entity(snap: &SpeechSnapshot) -> Option<&SpeechEntity> {
    snap.entities.iter().find(|entity| entity.domain == "weather")
}

fn will_rain(snap: &SpeechSnapshot) -> bool {
    if let Some(want) = target_date(snap) {
        if let Some(part) = slot(snap, "weather_part") {
            let hours: Vec<_> =
                snap.hourly.iter().filter(|row| row.datetime.starts_with(&want) && hour_in_part(&row.datetime, part)).collect();
            if !hours.is_empty() {
                return hours.iter().any(|row| wet_row(row));
            }
        }
        if let Some(day) = snap.forecast.iter().find(|row| row.datetime.starts_with(&want)) {
            return wet_row(day);
        }
    }
    weather_entity(snap).is_some_and(|entity| rainy(&entity.state))
}

fn wet_row(row: &SpeechForecast) -> bool {
    rainy(&row.condition) || row.precipitation.unwrap_or(0.0) > 0.0 || row.precipitation_probability.unwrap_or(0.0) >= 40.0
}

fn rainy(condition: &str) -> bool {
    RAINY.iter().any(|name| *name == condition)
}

fn pick_day(snap: &SpeechSnapshot) -> Option<&SpeechForecast> {
    let want = target_date(snap)?;
    snap.forecast.iter().find(|day| day.datetime.starts_with(&want))
}

fn target_date(snap: &SpeechSnapshot) -> Option<String> {
    let (y, m, d) = ymd(&snap.now)?;
    let today = weekday_mon0(y, m, d);
    let (y, m, d) = match day_key(snap) {
        "now" => return None,
        "today" => (y, m, d),
        "tomorrow" => add_one_day(y, m, d),
        "weekend" => first_weekday(y, m, d, today, &[5, 6]),
        "mon" => first_weekday(y, m, d, today, &[0]),
        "tue" => first_weekday(y, m, d, today, &[1]),
        "wed" => first_weekday(y, m, d, today, &[2]),
        "thu" => first_weekday(y, m, d, today, &[3]),
        "fri" => first_weekday(y, m, d, today, &[4]),
        "sat" => first_weekday(y, m, d, today, &[5]),
        "sun" => first_weekday(y, m, d, today, &[6]),
        _ => (y, m, d),
    };
    Some(format!("{y:04}-{m:02}-{d:02}"))
}

fn first_weekday(mut y: i32, mut m: u32, mut d: u32, mut dow: u8, want: &[u8]) -> (i32, u32, u32) {
    for _ in 0..8 {
        if want.contains(&dow) {
            return (y, m, d);
        }
        let next = add_one_day(y, m, d);
        y = next.0;
        m = next.1;
        d = next.2;
        dow = (dow + 1) % 7;
    }
    (y, m, d)
}

fn weekday_mon0(y: i32, m: u32, d: u32) -> u8 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut year = y;
    if m < 3 {
        year -= 1;
    }
    let dow_sun0 = (year + year / 4 - year / 100 + year / 400 + T[m as usize - 1] + d as i32).rem_euclid(7);
    ((dow_sun0 + 6) % 7) as u8
}

fn hour_in_part(iso: &str, part: &str) -> bool {
    let hour: u32 = iso.get(11..13).and_then(|raw| raw.parse().ok()).unwrap_or(99);
    match part {
        "morning" => (6..12).contains(&hour),
        "afternoon" => (12..18).contains(&hour),
        "evening" => (18..22).contains(&hour),
        "night" => !(6..22).contains(&hour),
        _ => true,
    }
}

fn ymd(iso: &str) -> Option<(i32, u32, u32)> {
    let stamp = iso.get(..10)?;
    Some((stamp.get(0..4)?.parse().ok()?, stamp.get(5..7)?.parse().ok()?, stamp.get(8..10)?.parse().ok()?))
}

fn add_one_day(y: i32, m: u32, d: u32) -> (i32, u32, u32) {
    let dim = days_in_month(y, m);
    if d < dim {
        (y, m, d + 1)
    } else if m < 12 {
        (y, m + 1, 1)
    } else {
        (y + 1, 1, 1)
    }
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(y) => 29,
        2 => 28,
        _ => 31,
    }
}

fn leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

fn fill(template: &str, condition: &str, temp: &str, unit: &str) -> String {
    let temp_unit = match (temp.is_empty(), unit.is_empty()) {
        (true, _) => String::new(),
        (false, true) => temp.to_string(),
        (false, false) => format!("{temp} {unit}"),
    };
    let mut line = template.replace("{c}", condition).replace("{t} {u}", &temp_unit).replace("{t}", temp).replace("{u}", unit);
    while line.contains("  ") {
        line = line.replace("  ", " ");
    }
    line.replace(" ,", ",").replace("، .", ".").replace("，。", "。").replace(", .", ".").replace(" .", ".").trim().to_string()
}
