//! OpenAI tool schemas and the Gemma text fallback `TRAINER_TOOL: name {json}`.

use super::types::ToolCall;
use serde_json::{json, Value};

pub const READ_TOOLS: &[&str] = &[
    "list_languages",
    "search_house",
    "get_entity",
    "list_lexicon_paths",
    "get_lexicon",
    "list_matchers",
    "list_policies",
    "list_gaps",
    "validate_proposal",
    "explain_klar",
    "try_sentence",
    "list_areas",
    "count_house",
    "list_engine",
    "list_phrases",
    "list_turns",
];

pub const WRITE_TOOLS: &[&str] = &["apply_lexicon", "apply_match", "apply_house", "apply_aliases", "apply_engine", "apply_ui"];

pub fn is_write_tool(name: &str) -> bool {
    WRITE_TOOLS.contains(&name)
}

pub fn known_tool(name: &str) -> bool {
    READ_TOOLS.contains(&name) || WRITE_TOOLS.contains(&name)
}

const MATCH_TOOLS: &[&str] = &[
    "list_languages",
    "explain_klar",
    "try_sentence",
    "list_engine",
    "list_turns",
    "list_matchers",
    "validate_proposal",
    "apply_match",
    "apply_engine",
    "apply_ui",
];
const LANGUAGE_TOOLS: &[&str] = &[
    "list_languages",
    "explain_klar",
    "try_sentence",
    "list_engine",
    "list_turns",
    "list_lexicon_paths",
    "get_lexicon",
    "list_phrases",
    "apply_lexicon",
    "apply_engine",
    "apply_ui",
];
const HOUSE_TOOLS: &[&str] = &[
    "list_languages",
    "explain_klar",
    "try_sentence",
    "list_engine",
    "list_turns",
    "search_house",
    "get_entity",
    "list_areas",
    "count_house",
    "list_policies",
    "list_gaps",
    "apply_house",
    "apply_aliases",
    "apply_engine",
    "apply_ui",
];

pub fn tools_for_layer(layer: &str) -> &'static [&'static str] {
    match layer {
        "match" => MATCH_TOOLS,
        "language" => LANGUAGE_TOOLS,
        "house" => HOUSE_TOOLS,
        _ => {
            const ALL: &[&str] = &[
                "list_languages",
                "explain_klar",
                "try_sentence",
                "list_engine",
                "list_turns",
                "search_house",
                "get_entity",
                "list_areas",
                "count_house",
                "list_lexicon_paths",
                "get_lexicon",
                "list_phrases",
                "list_matchers",
                "list_policies",
                "list_gaps",
                "validate_proposal",
                "apply_lexicon",
                "apply_match",
                "apply_house",
                "apply_aliases",
                "apply_engine",
                "apply_ui",
            ];
            ALL
        }
    }
}

pub fn write_tools_for_layer(layer: &str) -> Vec<&'static str> {
    tools_for_layer(layer).iter().copied().filter(|name| is_write_tool(name)).collect()
}

pub fn tool_allowed_for_layer(layer: &str, name: &str) -> bool {
    tools_for_layer(layer).contains(&name)
}

pub fn openai_tools_for(layer: &str) -> Vec<Value> {
    let allowed = tools_for_layer(layer);
    openai_tools().into_iter().filter(|tool| allowed.contains(&tool["function"]["name"].as_str().unwrap_or(""))).collect()
}

pub fn openai_tools() -> Vec<Value> {
    vec![
        tool("list_languages", "Assist languages from settings.languages.", json!({"type": "object", "properties": {}})),
        tool("search_house", "Search entities and areas on the graph.", object(&[("q", str_prop("Name, id, or alias fragment."))], &["q"])),
        tool("get_entity", "One graph entity with aliases and area.", object(&[("entity_id", str_prop("entity_id"))], &["entity_id"])),
        tool("list_lexicon_paths", "Known lexicon set paths (SET_KEYS).", json!({"type": "object", "properties": {}})),
        tool(
            "get_lexicon",
            "Current lexicon overlay for a path.",
            object(&[("language", str_prop("Assist pack")), ("path", str_prop("SET_KEYS path"))], &[]),
        ),
        tool("list_matchers", "Compiled matcher ids with overlay enable/precedence.", json!({"type": "object", "properties": {}})),
        tool("list_policies", "House policy rules.", json!({"type": "object", "properties": {}})),
        tool("list_gaps", "Unmapped entities with name and area.", json!({"type": "object", "properties": {}})),
        tool(
            "explain_klar",
            "Klar architecture, setup path, trade-offs, or engine LLM. Returns a view the UI renders.",
            object(&[("topic", str_prop("architecture, setup, tradeoffs, or llm"))], &[]),
        ),
        tool(
            "try_sentence",
            "Parse one utterance on this house. Returns the live policy path view.",
            object(&[("text", str_prop("Utterance as spoken at home")), ("language", str_prop("Assist pack"))], &["text"]),
        ),
        tool("list_areas", "Rooms on the home graph.", json!({"type": "object", "properties": {}})),
        tool("count_house", "Entity, area, and leftover counts.", json!({"type": "object", "properties": {}})),
        tool("list_engine", "Public engine and operator-chrome settings. No tokens or URLs.", json!({"type": "object", "properties": {}})),
        tool("list_phrases", "Custom sentence overlays.", json!({"type": "object", "properties": {}})),
        tool(
            "list_turns",
            "Assist journal. Last 24h / 200 turns. Filter by last N, date, time, since/until, query, decision, or all.",
            object(
                &[
                    ("last", json!({"type": "integer", "description": "Newest N turns. Default 12, max 80."})),
                    ("all", json!({"type": "boolean", "description": "Up to 80 newest matching turns."})),
                    ("date", str_prop("YYYY-MM-DD")),
                    ("time", str_prop("HH:MM, with date or today")),
                    ("since", str_prop("YYYY-MM-DDTHH:MM")),
                    ("until", str_prop("YYYY-MM-DDTHH:MM")),
                    ("query", str_prop("Text, speech, device name, or evidence fragment")),
                    ("decision", str_prop("execute, reject, clarify, confirm, chat")),
                    ("conversation_id", str_prop("One Assist conversation")),
                ],
                &[],
            ),
        ),
        tool(
            "validate_proposal",
            "Dry-run a house/match/language proposal without writing.",
            object(
                &[
                    ("layer", str_prop("match, language, house, or all")),
                    ("language", str_prop("Assist pack")),
                    ("policies", json!({"type": "array"})),
                    ("match_controls", json!({"type": "array"})),
                    ("language_overlay", json!({"type": "object"})),
                    ("utterances", json!({"type": "array", "items": {"type": "string"}})),
                ],
                &[],
            ),
        ),
        tool(
            "apply_lexicon",
            "Merge add/remove on a known lexicon path. Needs operator consent.",
            object(
                &[
                    ("language", str_prop("Assist pack from settings.languages")),
                    ("path", str_prop("SET_KEYS path")),
                    ("add", json!({"type": "array", "items": {"type": "string"}})),
                    ("remove", json!({"type": "array", "items": {"type": "string"}})),
                ],
                &["language", "path"],
            ),
        ),
        tool(
            "apply_match",
            "Merge enable/precedence for known matcher ids. Needs operator consent.",
            object(&[("match_controls", json!({"type": "array"}))], &["match_controls"]),
        ),
        tool(
            "apply_house",
            "Upsert house PolicyRule rows by id. Seed enabled:false is allowed. Needs operator consent.",
            object(&[("policies", json!({"type": "array"}))], &["policies"]),
        ),
        tool(
            "apply_aliases",
            "Merge overlay aliases for a graph entity. Needs operator consent.",
            object(
                &[("entity_id", str_prop("entity_id")), ("aliases", json!({"type": "array", "items": {"type": "string"}}))],
                &["entity_id", "aliases"],
            ),
        ),
        tool(
            "apply_engine",
            "Patch engine settings (refine, calendar_llm, personality, languages, quiet_ack, nlu_rag, extra_prompt, …). Never URL, token, or model. Needs consent.",
            object(
                &[
                    ("personality", str_prop("default, butler, locker, fuersorglich, party, grantig, sarkastisch, pirat, hippie, gollum, jarvis")),
                    ("mode", str_prop("full or context_only")),
                    ("languages", json!({"type": "array", "items": {"type": "string"}})),
                    ("refine_speech", json!({"type": "boolean"})),
                    ("calendar_llm", json!({"type": "boolean"})),
                    ("quiet_ack", json!({"type": "boolean"})),
                    ("nlu_rag", json!({"type": "boolean"})),
                    ("allow_llm_tools", json!({"type": "boolean"})),
                    ("confirm_risky_actions", json!({"type": "boolean"})),
                    ("semantic_adapters", json!({"type": "boolean"})),
                    ("support_bundle", json!({"type": "boolean"})),
                    ("support_bundle_raw_text", json!({"type": "boolean"})),
                    ("extra_prompt", str_prop("House rule user line. Empty keeps pack voice.")),
                ],
                &[],
            ),
        ),
        tool(
            "apply_ui",
            "Set operator chrome theme to light or dark, or UI locale. Use this when the operator asks for light mode, helles Design, or appearance. Not Assist language. Needs consent.",
            object(&[("theme", str_prop("dark or light")), ("locale", str_prop("Operator chrome locale"))], &[]),
        ),
    ]
}

pub fn parse_text_tools(text: &str) -> (String, Vec<ToolCall>) {
    let mut kept = String::new();
    let mut calls = Vec::new();
    let mut index = 0usize;
    for line in text.lines() {
        let (prose, found) = take_tools(line, &mut index);
        calls.extend(found);
        if leftover_after_tools(&prose) {
            continue;
        }
        if !kept.is_empty() {
            kept.push('\n');
        }
        kept.push_str(&prose);
    }
    (kept, calls)
}

fn take_tools(line: &str, index: &mut usize) -> (String, Vec<ToolCall>) {
    let mut prose = String::new();
    let mut rest = line;
    let mut calls = Vec::new();
    while let Some(at) = rest.find("TRAINER_TOOL:") {
        prose.push_str(&rest[..at]);
        rest = rest[at + "TRAINER_TOOL:".len()..].trim_start();
        if let Some((name, json, after)) = take_name_json(rest) {
            if known_tool(name) {
                calls.push(ToolCall::function(format!("text_{index}_{name}"), name, json));
                *index += 1;
                rest = after;
                continue;
            }
        }
        prose.push_str("TRAINER_TOOL:");
        prose.push_str(rest);
        rest = "";
        break;
    }
    prose.push_str(rest);
    (prose, calls)
}

fn take_name_json(rest: &str) -> Option<(&str, String, &str)> {
    let rest = rest.trim_start();
    let space = rest.find(char::is_whitespace)?;
    let name = rest[..space].trim();
    let after_name = rest[space..].trim_start();
    if name.is_empty() || !after_name.starts_with('{') {
        return None;
    }
    let end = brace_end(after_name)?;
    let json = after_name[..=end].to_string();
    serde_json::from_str::<Value>(&json).ok()?;
    Some((name, json, after_name[end + 1..].trim_start()))
}

fn brace_end(text: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escape = false;
    for (index, ch) in text.char_indices() {
        if in_str {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => in_str = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn leftover_after_tools(text: &str) -> bool {
    let trim = text.trim();
    trim.is_empty() || (trim.chars().count() == 1 && trim.chars().all(|ch| ch.is_ascii_alphabetic()))
}

fn tool(name: &str, description: &str, parameters: Value) -> Value {
    json!({"type": "function", "function": {"name": name, "description": description, "parameters": parameters}})
}

fn str_prop(description: &str) -> Value {
    json!({"type": "string", "description": description})
}

fn object(fields: &[(&str, Value)], required: &[&str]) -> Value {
    let mut properties = serde_json::Map::new();
    for (name, schema) in fields {
        properties.insert((*name).into(), schema.clone());
    }
    json!({"type": "object", "properties": properties, "required": required})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_are_named() {
        assert!(is_write_tool("apply_aliases"));
        assert!(!is_write_tool("get_entity"));
        assert!(openai_tools().iter().any(|tool| tool["function"]["name"] == "apply_house"));
    }

    #[test]
    fn lane_filter_keeps_writes_on_their_spur() {
        assert!(tool_allowed_for_layer("match", "apply_match"));
        assert!(tool_allowed_for_layer("match", "list_matchers"));
        assert!(tool_allowed_for_layer("match", "list_languages"));
        assert!(tool_allowed_for_layer("match", "explain_klar"));
        assert!(tool_allowed_for_layer("house", "try_sentence"));
        assert!(tool_allowed_for_layer("house", "count_house"));
        assert!(tool_allowed_for_layer("language", "list_phrases"));
        assert!(!tool_allowed_for_layer("match", "apply_house"));
        assert!(!tool_allowed_for_layer("match", "apply_lexicon"));
        assert!(tool_allowed_for_layer("language", "apply_lexicon"));
        assert!(!tool_allowed_for_layer("language", "apply_match"));
        assert!(tool_allowed_for_layer("house", "apply_house"));
        assert!(tool_allowed_for_layer("house", "apply_aliases"));
        assert!(!tool_allowed_for_layer("house", "apply_match"));
        let names: Vec<_> = openai_tools_for("match").iter().map(|tool| tool["function"]["name"].as_str().unwrap().to_string()).collect();
        assert!(names.contains(&"apply_match".into()));
        assert!(!names.iter().any(|name| name == "apply_house"));
        assert_eq!(write_tools_for_layer("house"), vec!["apply_house", "apply_aliases", "apply_engine", "apply_ui"]);
    }

    #[test]
    fn parses_text_fallback_and_keeps_prose() {
        let (text, calls) = parse_text_tools(
            "Adding the slang.\nTRAINER_TOOL: apply_aliases {\"entity_id\":\"light.wohnzimmer\",\"aliases\":[\"decke\"]}\nDone.",
        );
        assert_eq!(text, "Adding the slang.\nDone.");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].function.name, "apply_aliases");
        assert!(calls[0].function.arguments.contains("light.wohnzimmer"));
        assert!(parse_text_tools("TRAINER_TOOL: nope {}").1.is_empty());
        let (glued, many) = parse_text_tools("TRAINER_TOOL: list_matchers {} TRAINER_TOOL: list_policies {}I");
        assert!(glued.is_empty(), "{glued}");
        assert_eq!(many.iter().map(|call| call.function.name.as_str()).collect::<Vec<_>>(), ["list_matchers", "list_policies"]);
    }
}
