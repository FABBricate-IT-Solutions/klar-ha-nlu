//! Reply chips for Lotse follow-up questions. Prompt may emit `LOTSE_CHOICES`; if not, the engine fills them.

use super::extract::json_array;
use super::types::{ChatMessage, ChatRequest};
use super::{chat, LlmEndpoint};
use serde_json::Value;

const MARK: &str = "LOTSE_CHOICES:";
const MAX_CHOICES: usize = 4;
const MAX_CHARS: usize = 120;

pub fn parse_lotse_choices(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.replace('\r', "").lines() {
        let Some(rest) = line.trim().strip_prefix(MARK).map(str::trim) else {
            continue;
        };
        out.extend(decode_choices(rest));
    }
    out.truncate(MAX_CHOICES);
    out
}

pub fn asks_operator(text: &str) -> bool {
    let prose = strip_marks(text);
    if prose.contains('?') || prose.contains('？') {
        return true;
    }
    let lower = prose.to_lowercase();
    if lower.contains("möchtest du")
        || lower.contains("möchten sie")
        || lower.contains("soll ich")
        || lower.contains("willst du")
        || lower.contains("kann ich")
        || lower.contains("darf ich")
        || lower.contains("should i")
        || lower.contains("do you want")
    {
        return true;
    }
    let last = prose.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("").trim().to_lowercase();
    last.starts_with("welche")
        || last.starts_with("welcher")
        || last.starts_with("welches")
        || last.starts_with("soll ")
        || last.starts_with("möchtest")
        || last.starts_with("should ")
        || last.starts_with("which ")
        || last.starts_with("what ")
        || last.starts_with("do you ")
}

pub fn choices_line(rows: &[String]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    format!("\n{MARK} {}", serde_json::to_string(rows).unwrap_or_else(|_| "[]".into()))
}

pub async fn ensure_reply_choices(endpoint: &LlmEndpoint, prose: &str, reply_language: &str) -> String {
    if !parse_lotse_choices(prose).is_empty() || !asks_operator(prose) {
        return String::new();
    }
    match generate(endpoint, prose, reply_language).await {
        Ok(rows) if !rows.is_empty() => choices_line(&rows),
        _ => choices_line(&fallback_choices(prose, reply_language)),
    }
}

fn fallback_choices(prose: &str, reply_language: &str) -> Vec<String> {
    if reply_language.starts_with("de") {
        return vec!["Ja".into(), "Nein".into(), "Nicht jetzt".into()];
    }
    let lower = prose.to_lowercase();
    if lower.contains("möchtest")
        || lower.contains("möchten")
        || lower.contains("soll ich")
        || lower.contains("willst du")
        || lower.contains("kann ich")
        || lower.contains("welche")
        || lower.contains("welcher")
        || lower.contains("welches")
    {
        return vec!["Ja".into(), "Nein".into(), "Nicht jetzt".into()];
    }
    vec!["Yes".into(), "No".into(), "Not now".into()]
}

async fn generate(endpoint: &LlmEndpoint, prose: &str, reply_language: &str) -> Result<Vec<String>, super::types::LlmError> {
    let request = ChatRequest {
        messages: vec![
            ChatMessage::new(
                "system",
                format!(
                    "You write tap replies for a Klar operator. Return a JSON array of 2 to 4 short strings only. Language: {reply_language}. Grounded in the question. No markdown."
                ),
            ),
            ChatMessage::new("user", format!("Question:\n{}", strip_marks(prose))),
        ],
        stream: Some(false),
        temperature: Some(0.1),
        max_tokens: Some(160),
        tools: None,
        tool_choice: None,
    };
    let text = chat(endpoint, request).await?;
    Ok(decode_choices(json_array(&text).unwrap_or(text.trim())))
}

fn strip_marks(text: &str) -> String {
    text.replace('\r', "")
        .lines()
        .filter(|line| {
            let trim = line.trim();
            !trim.starts_with(MARK) && !trim.starts_with("LOTSE_VIEW:")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn decode_choices(raw: &str) -> Vec<String> {
    let value: Value =
        serde_json::from_str(raw).ok().or_else(|| json_array(raw).and_then(|body| serde_json::from_str(body).ok())).unwrap_or(Value::Null);
    let rows = value.as_array().cloned().or_else(|| value.get("choices").and_then(Value::as_array).cloned()).unwrap_or_default();
    rows.into_iter()
        .filter_map(|row| {
            let text = row.as_str().unwrap_or("").trim();
            if text.is_empty() || text.chars().count() > MAX_CHARS {
                None
            } else {
                Some(text.to_string())
            }
        })
        .take(MAX_CHOICES)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::post;
    use axum::{Json, Router};
    use serde_json::json;
    use tokio::net::TcpListener;

    #[test]
    fn parse_reads_array_and_hides_view_lines() {
        let text = "Welche Lücke zuerst?\nLOTSE_VIEW: gaps {\"gaps\":[]}\nLOTSE_CHOICES: [\"nur Licht\",\"alle Lücken\"]";
        assert_eq!(parse_lotse_choices(text), vec!["nur Licht", "alle Lücken"]);
        assert!(asks_operator(text));
        assert!(asks_operator(
            "Möchtest du, dass ich eine solche Regel als PolicyRule im Haus-Layer anlege? Ich kann auch eine Szene definieren.",
        ));
        assert!(!asks_operator("Die Lücken sind todo.zeiterfassung."));
        assert!(!asks_operator("LOTSE_CHOICES: [\"x\"]"));
        assert_eq!(parse_lotse_choices("LOTSE_CHOICES: {\"choices\":[\"ja\",\"nein\"]}"), vec!["ja", "nein"]);
        assert!(parse_lotse_choices("LOTSE_CHOICES: [").is_empty());
    }

    #[test]
    fn choices_line_roundtrip() {
        let line = choices_line(&["nur Licht".into(), "nichts ändern".into()]);
        assert!(line.starts_with('\n'));
        assert_eq!(parse_lotse_choices(&format!("Frage?{line}")), vec!["nur Licht", "nichts ändern"]);
    }

    #[tokio::test]
    async fn fills_missing_choices_from_llm() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let app = Router::new().route(
                "/v1/chat/completions",
                post(|Json(body): Json<Value>| async move {
                    assert_eq!(body["chat_template_kwargs"]["enable_thinking"], false);
                    let reply = json!({"choices":[{"message":{"role":"assistant","content":"[\"nur Licht\",\"alle Lücken\"]"}}]});
                    ([(axum::http::header::CONTENT_TYPE, "application/json")], reply.to_string())
                }),
            );
            axum::serve(listener, app).await.unwrap();
        });
        let endpoint = LlmEndpoint::from_parts(&format!("http://{addr}/v1"), "", "test-model").unwrap();
        let extra = ensure_reply_choices(&endpoint, "Welche Lücke zuerst?", "de").await;
        assert_eq!(parse_lotse_choices(&format!("Welche Lücke zuerst?{extra}")), vec!["nur Licht", "alle Lücken"]);
        assert!(ensure_reply_choices(&endpoint, "Die Lücken sind todo.zeiterfassung.", "de").await.is_empty());
        assert!(ensure_reply_choices(&endpoint, "Welche zuerst?\nLOTSE_CHOICES: [\"schon da\"]", "de").await.is_empty());
        handle.abort();
    }

    #[tokio::test]
    async fn falls_back_to_yes_no_when_llm_empty() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let app = Router::new().route(
                "/v1/chat/completions",
                post(|Json(_body): Json<Value>| async move {
                    let reply = json!({"choices":[{"message":{"role":"assistant","content":""}}]});
                    ([(axum::http::header::CONTENT_TYPE, "application/json")], reply.to_string())
                }),
            );
            axum::serve(listener, app).await.unwrap();
        });
        let endpoint = LlmEndpoint::from_parts(&format!("http://{addr}/v1"), "", "test-model").unwrap();
        let extra =
            ensure_reply_choices(&endpoint, "Möchtest du, dass ich eine solche Regel als PolicyRule im Haus-Layer anlege?", "de").await;
        assert_eq!(parse_lotse_choices(&format!("q{extra}")), vec!["Ja", "Nein", "Nicht jetzt"]);
        handle.abort();
    }
}
