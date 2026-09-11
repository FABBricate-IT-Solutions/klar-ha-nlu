//! OpenAI tool schemas and the Gemma text fallback `TRAINER_TOOL: name {json}`.

use super::trainer_tools_def;
use super::types::ToolCall;
use serde_json::Value;

pub const READ_TOOLS: &[&str] = &[
    "list_languages",
    "search_house",
    "get_entity",
    "list_lexicon_paths",
    "get_lexicon",
    "list_matchers",
    "list_policies",
    "list_seeds",
    "list_speech",
    "list_gaps",
    "validate_proposal",
    "explain_klar",
    "try_sentence",
    "list_areas",
    "list_floors",
    "count_house",
    "list_engine",
    "list_phrases",
    "list_turns",
];

pub const WRITE_TOOLS: &[&str] = &[
    "apply_lexicon",
    "apply_phrases",
    "apply_match",
    "apply_house",
    "apply_aliases",
    "apply_entity",
    "apply_area",
    "apply_speech",
    "apply_engine",
    "apply_ui",
];

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
    "apply_phrases",
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
    "list_floors",
    "count_house",
    "list_policies",
    "list_seeds",
    "list_speech",
    "list_gaps",
    "apply_house",
    "apply_aliases",
    "apply_entity",
    "apply_area",
    "apply_speech",
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
                "list_floors",
                "count_house",
                "list_lexicon_paths",
                "get_lexicon",
                "list_phrases",
                "list_matchers",
                "list_policies",
                "list_seeds",
                "list_speech",
                "list_gaps",
                "validate_proposal",
                "apply_lexicon",
                "apply_phrases",
                "apply_match",
                "apply_house",
                "apply_aliases",
                "apply_entity",
                "apply_area",
                "apply_speech",
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
    trainer_tools_def::openai_tools()
}

pub fn parse_text_tools(text: &str) -> (String, Vec<ToolCall>) {
    let mut kept = String::new();
    let mut calls = Vec::new();
    let mut index = 0usize;
    for line in text.lines() {
        let (prose, found) = take_tools(line, &mut index);
        calls.extend(found);
        let prose = cut_lotse_mark(&prose);
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
    trim.is_empty()
        || (trim.chars().count() == 1 && trim.chars().all(|ch| ch.is_ascii_alphabetic()))
        || trim.starts_with("LOTSE_VIEW")
        || trim.starts_with("LOTSE_CHOICES")
}

fn cut_lotse_mark(prose: &str) -> String {
    let cut = ["LOTSE_VIEW", "LOTSE_CHOICES"].iter().filter_map(|mark| prose.find(mark)).min().unwrap_or(prose.len());
    prose[..cut].trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_are_named() {
        assert!(is_write_tool("apply_aliases"));
        assert!(is_write_tool("apply_area"));
        assert!(is_write_tool("apply_entity"));
        assert!(is_write_tool("apply_phrases"));
        assert!(is_write_tool("apply_speech"));
        assert!(!is_write_tool("get_entity"));
        assert!(!is_write_tool("list_seeds"));
        let tools = openai_tools();
        assert!(tools.iter().any(|tool| tool["function"]["name"] == "list_floors"));
        let house = tools.iter().find(|tool| tool["function"]["name"] == "apply_house").unwrap();
        let payload = house["function"]["parameters"]["properties"]["policies"]["items"]["properties"]["payload"]["description"]
            .as_str()
            .unwrap_or("");
        assert!(payload.contains("Jinja"), "{payload}");
        let search = tools.iter().find(|tool| tool["function"]["name"] == "search_house").unwrap();
        assert!(search["function"]["description"].as_str().unwrap_or("").contains("floor"), "{search}");
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
        assert!(tool_allowed_for_layer("language", "apply_phrases"));
        assert!(!tool_allowed_for_layer("language", "apply_match"));
        assert!(tool_allowed_for_layer("house", "list_floors"));
        assert!(tool_allowed_for_layer("house", "apply_house"));
        assert!(tool_allowed_for_layer("house", "apply_aliases"));
        assert!(tool_allowed_for_layer("house", "apply_entity"));
        assert!(tool_allowed_for_layer("house", "apply_area"));
        assert!(tool_allowed_for_layer("house", "list_seeds"));
        assert!(tool_allowed_for_layer("house", "apply_speech"));
        assert!(!tool_allowed_for_layer("house", "apply_match"));
        let names: Vec<_> = openai_tools_for("match").iter().map(|tool| tool["function"]["name"].as_str().unwrap().to_string()).collect();
        assert!(names.contains(&"apply_match".into()));
        assert!(!names.iter().any(|name| name == "apply_house"));
        assert_eq!(
            write_tools_for_layer("house"),
            vec!["apply_house", "apply_aliases", "apply_entity", "apply_area", "apply_speech", "apply_engine", "apply_ui"]
        );
        assert!(tool_allowed_for_layer("all", "apply_match"));
        assert!(tool_allowed_for_layer("all", "apply_house"));
        assert!(tool_allowed_for_layer("all", "apply_lexicon"));
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
        let (view, _) = parse_text_tools("Kurz.\nLOTSE_VIEW: architecture\nLOTSE_VIEW: gaps");
        assert_eq!(view, "Kurz.");
        let (inline, _) = parse_text_tools("Siehe LOTSE_VIEW: architecture");
        assert_eq!(inline, "Siehe");
    }
}
