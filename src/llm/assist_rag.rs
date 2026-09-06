//! RAG protocol parse. Prefer structured `tool` events over spoken `KLAR_PARSE:`.

use std::collections::BTreeMap;

use super::types::ChatEvent;

const PARSE: &str = "klar.parse";
const ACT: &str = "klar.act";

const INSTRUCT_DE: &str = "Wenn der Satz ein Hausbefehl ist, antworte mit genau einer Zeile und sonst nichts: \
KLAR_PARSE: <klarer Befehl>. \
Sonst antworte kurz im Gespräch. \
Nenne niemals Werkzeuge, Intents oder Präfixe.";
const INSTRUCT_EN: &str = "If the sentence is a home command, reply with exactly one line and nothing else: \
KLAR_PARSE: <clear command>. \
Otherwise reply briefly in conversation. \
Never name tools, intents, or prefixes.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KlarTool {
    pub tool: String,
    pub text: Option<String>,
    pub intent: Option<String>,
    pub slots: BTreeMap<String, String>,
}

impl KlarTool {
    pub fn event(&self) -> ChatEvent {
        ChatEvent::Tool {
            tool: self.tool.clone(),
            text: self.text.clone(),
            intent: self.intent.clone(),
            slots: if self.slots.is_empty() { None } else { Some(self.slots.clone()) },
        }
    }

    pub fn spoken_line(&self) -> String {
        if self.tool == PARSE {
            format!("KLAR_PARSE: {}", self.text.as_deref().unwrap_or("").trim())
        } else {
            let mut line = format!("KLAR_ACT: {}", self.intent.as_deref().unwrap_or("").trim());
            for (key, value) in &self.slots {
                line.push(' ');
                line.push_str(key);
                line.push('=');
                line.push_str(value);
            }
            line
        }
    }
}

pub fn rag_instruct(pack: &str) -> &'static str {
    if pack == "de" || pack.starts_with("de-") {
        INSTRUCT_DE
    } else {
        INSTRUCT_EN
    }
}

pub fn retrieval_lines(retrieval: Option<&serde_json::Value>, pack: &str) -> String {
    let Some(serde_json::Value::Object(map)) = retrieval else {
        return String::new();
    };
    let mut names = Vec::new();
    if let Some(serde_json::Value::Array(entities)) = map.get("entities") {
        for item in entities.iter().take(8) {
            if let Some(name) = item.get("name").and_then(|value| value.as_str()).filter(|name| !name.is_empty()) {
                names.push(name.to_string());
            }
        }
    }
    let areas: Vec<String> = map
        .get("areas")
        .and_then(|value| value.as_array())
        .map(|rows| rows.iter().filter_map(|item| item.as_str().map(str::to_string)).take(8).collect())
        .unwrap_or_default();
    let last: Vec<String> = map
        .get("last")
        .and_then(|value| value.as_array())
        .map(|rows| rows.iter().filter_map(|item| item.as_str().map(str::to_string)).take(8).collect())
        .unwrap_or_default();
    let mut bits = Vec::new();
    if !names.is_empty() {
        bits.push(names.join(", "));
    }
    if !areas.is_empty() {
        bits.push(areas.join("/"));
    }
    if !last.is_empty() {
        bits.push(last.join(" · "));
    }
    if bits.is_empty() {
        return String::new();
    }
    let label = if pack == "de" || pack.starts_with("de-") { "Kontext" } else { "Context" };
    format!("{label}: {}", bits.join("; "))
}

pub fn rag_prompt(pack: &str, retrieval: Option<&serde_json::Value>, extra: Option<&str>) -> String {
    let instruct = rag_instruct(pack);
    let context = retrieval_lines(retrieval, pack);
    [extra.unwrap_or("").trim(), context.as_str(), instruct].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join("\n")
}

pub fn parse_tool_reply(speech: &str) -> Option<KlarTool> {
    let text = speech.trim();
    if let Some(rest) = text.strip_prefix("KLAR_PARSE:") {
        return Some(KlarTool { tool: PARSE.into(), text: Some(rest.trim().to_string()), intent: None, slots: BTreeMap::new() });
    }
    let rest = text.strip_prefix("KLAR_ACT:")?;
    let body = rest.trim();
    let (name, rest) = body.split_once(' ').unwrap_or((body, ""));
    let mut slots = BTreeMap::new();
    for part in rest.split_whitespace() {
        if let Some((key, value)) = part.split_once('=') {
            if !key.is_empty() && !value.is_empty() {
                slots.insert(key.to_string(), value.to_string());
            }
        }
    }
    Some(KlarTool { tool: ACT.into(), text: None, intent: Some(name.trim().to_string()), slots })
}

pub fn holds_klar_tool_prefix(speech: &str) -> bool {
    let stripped = speech.trim_start();
    let marker = "KLAR_";
    stripped.is_empty() || stripped.starts_with(marker) || marker.starts_with(stripped)
}

pub fn leaks_klar_tools(speech: &str) -> bool {
    if parse_tool_reply(speech).is_some() {
        return false;
    }
    let text = speech.to_lowercase().replace('`', "");
    text.contains("klar.parse") || text.contains("klar.act") || text.contains("klar_parse") || text.contains("klar_act")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_protocol_and_holds_prefix() {
        let parse = parse_tool_reply("KLAR_PARSE: mach die kugel an").unwrap();
        assert_eq!(parse.tool, "klar.parse");
        assert_eq!(parse.text.as_deref(), Some("mach die kugel an"));
        let event = serde_json::to_value(parse.event()).unwrap();
        assert_eq!(event["type"], "tool");
        assert_eq!(event["tool"], "klar.parse");
        assert_eq!(event["text"], "mach die kugel an");
        let act = parse_tool_reply("KLAR_ACT: HassTurnOn entity_id=light.kugel").unwrap();
        assert_eq!(act.intent.as_deref(), Some("HassTurnOn"));
        assert_eq!(act.slots.get("entity_id").map(String::as_str), Some("light.kugel"));
        assert!(holds_klar_tool_prefix(""));
        assert!(holds_klar_tool_prefix("KL"));
        assert!(holds_klar_tool_prefix("KLAR_PARSE: Licht"));
        assert!(!holds_klar_tool_prefix("Natürlich"));
        assert!(!holds_klar_tool_prefix("Klar, gerne."));
        assert!(leaks_klar_tools("Please use `klar.parse`"));
        assert!(!leaks_klar_tools("KLAR_PARSE: licht an"));
        let prompt = rag_prompt("de", Some(&json!({"entities":[{"name":"Kugel"}]})), None);
        assert!(prompt.contains("KLAR_PARSE:"));
        assert!(!prompt.contains("klar.parse"));
        assert!(prompt.contains("Kugel"));
        let lines =
            retrieval_lines(Some(&json!({"entities": (0..12).map(|i| json!({"name": format!("n{i}")})).collect::<Vec<_>>()})), "en");
        assert!(lines.contains("n0"));
        assert!(!lines.contains("n8"));
    }
}
