//! Assist journal for Lotse: last N, date/time window, query, or all (capped).

use crate::io::conversations::ConversationTurn;
use crate::io::state::AppState;
use crate::io::trainer_reads::with_view;
use serde_json::{json, Value};

const MAX_OUT: usize = 80;
const DEFAULT_LAST: usize = 12;

pub async fn list_turns(state: &AppState, args: &Value) -> Result<Value, String> {
    let mut turns = state.journal.list();
    if let Some(id) = args.get("conversation_id").and_then(Value::as_str).filter(|item| !item.is_empty()) {
        turns.retain(|turn| turn.conversation_id == id);
    }
    if let Some(decision) = args.get("decision").and_then(Value::as_str).filter(|item| !item.is_empty()) {
        turns.retain(|turn| turn.decision.eq_ignore_ascii_case(decision));
    }
    if let Some(query) = args.get("query").and_then(Value::as_str).filter(|item| !item.is_empty()) {
        let needle = query.to_lowercase();
        turns.retain(|turn| matches_query(turn, &needle));
    }
    let (since, until) = window(args)?;
    if let Some(since) = since {
        turns.retain(|turn| turn.ts_ms >= since);
    }
    if let Some(until) = until {
        turns.retain(|turn| turn.ts_ms <= until);
    }
    let all = args.get("all").and_then(Value::as_bool).unwrap_or(false);
    if !all {
        let last = args.get("last").and_then(Value::as_u64).unwrap_or(DEFAULT_LAST as u64).clamp(1, MAX_OUT as u64) as usize;
        if turns.len() > last {
            turns = turns.split_off(turns.len() - last);
        }
    } else if turns.len() > MAX_OUT {
        turns = turns.split_off(turns.len() - MAX_OUT);
    }
    let rows: Vec<Value> = turns.iter().map(turn_row).collect();
    Ok(with_view(
        "turns",
        json!({
            "window": "last 24h, max 200 stored",
            "count": rows.len(),
            "turns": rows
        }),
    ))
}

fn matches_query(turn: &ConversationTurn, needle: &str) -> bool {
    turn.decision.to_lowercase().contains(needle)
        || turn.speech.to_lowercase().contains(needle)
        || turn.text.as_deref().unwrap_or("").to_lowercase().contains(needle)
        || turn.tokens.iter().any(|token| token.to_lowercase().contains(needle))
        || turn.last_names.iter().any(|name| name.to_lowercase().contains(needle))
        || turn.evidence_kinds.iter().any(|kind| kind.to_lowercase().contains(needle))
}

fn turn_row(turn: &ConversationTurn) -> Value {
    json!({
        "when": format_when(turn.ts_ms),
        "ts_ms": turn.ts_ms,
        "label": turn.text.as_deref().filter(|item| !item.is_empty()).unwrap_or(turn.speech.as_str()),
        "text": turn.text,
        "speech": turn.speech,
        "decision": turn.decision,
        "confidence": turn.confidence,
        "names": turn.last_names,
        "evidence": turn.evidence_kinds,
        "conversation_id": turn.conversation_id
    })
}

fn window(args: &Value) -> Result<(Option<u64>, Option<u64>), String> {
    if let Some(since) = args.get("since").and_then(Value::as_str).filter(|item| !item.is_empty()) {
        let start = parse_when(since, false)?;
        let end = match args.get("until").and_then(Value::as_str).filter(|item| !item.is_empty()) {
            Some(until) => Some(parse_when(until, true)?),
            None => None,
        };
        return Ok((Some(start), end));
    }
    let date = args.get("date").and_then(Value::as_str).filter(|item| !item.is_empty());
    let time = args.get("time").and_then(Value::as_str).filter(|item| !item.is_empty());
    match (date, time) {
        (Some(date), Some(clock)) => {
            let start = parse_when(&format!("{date}T{clock}"), false)?;
            Ok((Some(start), Some(start.saturating_add(60 * 60 * 1000))))
        }
        (Some(date), None) => {
            let start = parse_when(date, false)?;
            Ok((Some(start), Some(start.saturating_add(24 * 60 * 60 * 1000 - 1))))
        }
        (None, Some(clock)) => {
            let today = format_day(now_ms());
            let start = parse_when(&format!("{today}T{clock}"), false)?;
            Ok((Some(start), Some(start.saturating_add(60 * 60 * 1000))))
        }
        (None, None) => Ok((None, None)),
    }
}

fn parse_when(raw: &str, end_of_minute: bool) -> Result<u64, String> {
    let raw = raw.trim().replace(' ', "T");
    let (date, clock) = raw.split_once('T').unwrap_or((raw.as_str(), "00:00"));
    let parts: Vec<u32> = date.split('-').filter_map(|part| part.parse().ok()).collect();
    if parts.len() != 3 {
        return Err("use YYYY-MM-DD or YYYY-MM-DDTHH:MM".into());
    }
    let hm: Vec<u32> = clock.split(':').filter_map(|part| part.parse().ok()).collect();
    let hour = *hm.first().unwrap_or(&0);
    let minute = *hm.get(1).unwrap_or(&0);
    civil_to_unix_ms(parts[0] as i32, parts[1], parts[2], hour, minute)
        .map(|ms| if end_of_minute { ms.saturating_add(59_999) } else { ms })
        .ok_or_else(|| "invalid date".into())
}

fn civil_to_unix_ms(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Option<u64> {
    if !(1..=12).contains(&month) || day == 0 || day > 31 || hour > 23 || minute > 59 {
        return None;
    }
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = u32::try_from(year - era * 400).ok()?;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + i32::try_from(doe).ok()? - 719_468;
    let secs = i64::from(days) * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60;
    u64::try_from(secs.checked_mul(1000)?).ok()
}

fn format_when(ts_ms: u64) -> String {
    let secs = ts_ms / 1000;
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rem = secs % 86_400;
    let hour = rem / 3600;
    let minute = (rem % 3600) / 60;
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}")
}

fn format_day(ts_ms: u64) -> String {
    format_when(ts_ms).chars().take(10).collect()
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = u32::try_from(z - era * 146_097).unwrap_or(0);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::{default_home, LoadedHome};
    use crate::io::conversations::turn_from_outcome;
    use crate::io::state::AppState;
    use crate::types::{ParseDecision, ParseOutcome, RejectReason, Settings};

    fn state() -> AppState {
        let dir = std::env::temp_dir().join(format!("klar-lotse-turns-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
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

    fn outcome(text: &str, speech: &str, decision: ParseDecision) -> ParseOutcome {
        ParseOutcome {
            schema_version: "2.0".into(),
            text: text.into(),
            conversation_id: "c1".into(),
            decision,
            speech: speech.into(),
            confidence: 0.9,
            margin: 1.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            retrieval: None,
            policy_trace: None,
            quiet_ack_eligible: false,
        }
    }

    #[test]
    fn date_roundtrip() {
        let ms = parse_when("2026-09-06T04:20", false).unwrap();
        assert_eq!(format_when(ms), "2026-09-06T04:20");
        assert!(parse_when("nope", false).is_err());
    }

    #[tokio::test]
    async fn filters_last_and_query() {
        let state = state();
        state.journal.append(turn_from_outcome(
            &outcome("licht an", "Geht.", ParseDecision::Execute),
            true,
            vec!["Wohnzimmer".into()],
            None,
        ));
        state.journal.append(turn_from_outcome(
            &outcome("kalender", "Nichts.", ParseDecision::Reject { reason: RejectReason::NoAction }),
            true,
            Vec::new(),
            None,
        ));
        let last = list_turns(&state, &json!({"last":1})).await.unwrap();
        assert_eq!(last["count"], 1);
        assert_eq!(last["turns"][0]["text"], "kalender");
        let hit = list_turns(&state, &json!({"query":"licht","all":true})).await.unwrap();
        assert_eq!(hit["count"], 1);
        assert_eq!(hit["turns"][0]["decision"], "execute");
    }
}
