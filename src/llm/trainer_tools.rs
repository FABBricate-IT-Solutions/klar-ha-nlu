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
];

pub const WRITE_TOOLS: &[&str] = &["apply_lexicon", "apply_match", "apply_house", "apply_aliases"];

pub fn is_write_tool(name: &str) -> bool {
    WRITE_TOOLS.contains(&name)
}

pub fn known_tool(name: &str) -> bool {
    READ_TOOLS.contains(&name) || WRITE_TOOLS.contains(&name)
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
    ]
}

pub fn parse_text_tools(text: &str) -> (String, Vec<ToolCall>) {
    let mut kept = String::new();
    let mut calls = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if let Some(call) = parse_tool_line(line, index) {
            calls.push(call);
            continue;
        }
        if !kept.is_empty() {
            kept.push('\n');
        }
        kept.push_str(line);
    }
    (kept, calls)
}

pub fn parse_tool_line(line: &str, index: usize) -> Option<ToolCall> {
    let rest = line.trim().strip_prefix("TRAINER_TOOL:")?.trim();
    let (name, json_text) = split_name_json(rest)?;
    if !known_tool(name) {
        return None;
    }
    let _: Value = serde_json::from_str(json_text).ok()?;
    Some(ToolCall::function(format!("text_{index}_{name}"), name, json_text))
}

fn split_name_json(rest: &str) -> Option<(&str, &str)> {
    let rest = rest.trim();
    let space = rest.find(|ch: char| ch.is_whitespace())?;
    let name = rest[..space].trim();
    let json_text = rest[space..].trim();
    if name.is_empty() || !json_text.starts_with('{') {
        return None;
    }
    Some((name, json_text))
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
    fn parses_text_fallback_and_keeps_prose() {
        let (text, calls) = parse_text_tools(
            "Adding the slang.\nTRAINER_TOOL: apply_aliases {\"entity_id\":\"light.wohnzimmer\",\"aliases\":[\"decke\"]}\nDone.",
        );
        assert_eq!(text, "Adding the slang.\nDone.");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].function.name, "apply_aliases");
        assert!(calls[0].function.arguments.contains("light.wohnzimmer"));
        assert!(parse_tool_line("TRAINER_TOOL: nope {}", 0).is_none());
    }
}
