//! OpenAI function schemas for Lotse tools.

use serde_json::{json, Value};

pub fn openai_tools() -> Vec<Value> {
    vec![
        tool("list_languages", "Assist languages from settings.languages.", json!({"type": "object", "properties": {}})),
        tool(
            "search_house",
            "Search entities, areas, and floors on the graph. Wohnung is often a floor_id, not an area. Entity rows include exposed (Assist), not a write.",
            object(&[("q", str_prop("Name, id, or alias fragment."))], &["q"]),
        ),
        tool(
            "get_entity",
            "One graph entity: aliases, area, tags, exposed, suggested_area. Read-only; rooms use apply_area.",
            object(&[("entity_id", str_prop("entity_id"))], &["entity_id"]),
        ),
        tool("list_lexicon_paths", "Known lexicon set paths (SET_KEYS).", json!({"type": "object", "properties": {}})),
        tool(
            "get_lexicon",
            "Current lexicon overlay for a path. Slang only — not custom household sentences.",
            object(&[("language", str_prop("Assist pack")), ("path", str_prop("SET_KEYS path"))], &[]),
        ),
        tool("list_matchers", "Compiled matcher ids with overlay enable/precedence. Not govern seeds.", json!({"type": "object", "properties": {}})),
        tool("list_policies", "House policy rules on the overlay.", json!({"type": "object", "properties": {}})),
        tool(
            "list_seeds",
            "Govern safety seeds (lock/cover). House overlay may replace or set enabled:false. Not matchers — do not use apply_match.",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "list_speech",
            "Speech-bank variants per house rule_id (language + personality + text).",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "list_gaps",
            "Entities that still need mapping. reason is missing_area or weak_name. suggested_area is a hint. Assign rooms with apply_area, not apply_aliases.",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "explain_klar",
            "Klar architecture, setup path, trade-offs, or engine LLM. Returns a view the UI renders.",
            object(&[("topic", str_prop("architecture, setup, tradeoffs, or llm"))], &[]),
        ),
        tool(
            "try_sentence",
            "Parse one utterance on this house like Lab. preferred_area and conversation_id follow the satellite path. nlu_rag overrides settings for this call only.",
            object(
                &[
                    ("text", str_prop("Utterance as spoken at home")),
                    ("language", str_prop("Assist pack")),
                    ("preferred_area", str_prop("area_id from list_areas; empty clears for this turn")),
                    ("conversation_id", str_prop("Assist conversation id, max 128")),
                    ("nlu_rag", json!({"type": "boolean", "description": "Override settings.nlu_rag for this parse only"})),
                ],
                &["text"],
            ),
        ),
        tool("list_areas", "Rooms on the home graph. Not floors — Wohnung is often a floor_id.", json!({"type": "object", "properties": {}})),
        tool("list_floors", "Floors on the home graph (floor_id, name, aliases).", json!({"type": "object", "properties": {}})),
        tool("count_house", "Entity, area, floor, and leftover counts.", json!({"type": "object", "properties": {}})),
        tool("list_engine", "Public engine and operator-chrome settings. No tokens or URLs.", json!({"type": "object", "properties": {}})),
        tool("list_phrases", "Custom sentence overlays (household phrases → intent). Not lexicon slang.", json!({"type": "object", "properties": {}})),
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
            "Dry-run a house/match/language proposal without writing. custom[] is household sentences (phrase + intent), not lexicon.",
            object(
                &[
                    ("layer", str_prop("match, language, house, or all")),
                    ("language", str_prop("Assist pack")),
                    ("policies", json!({"type": "array"})),
                    ("match_controls", json!({"type": "array"})),
                    ("language_overlay", json!({"type": "object"})),
                    ("custom", json!({"type": "array", "items": {"type": "object", "required": ["phrase", "intent"], "properties": {"phrase": {"type": "string"}, "intent": {"type": "string"}, "slots": {"type": "object"}}}})),
                    ("utterances", json!({"type": "array", "items": {"type": "string"}})),
                ],
                &[],
            ),
        ),
        tool(
            "apply_lexicon",
            "Merge add/remove on a known lexicon path (slang tokens). Not custom household sentences — those are apply_phrases. Needs consent.",
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
            "apply_phrases",
            "Add or remove custom household sentences (phrase → known intent). Not lexicon slang. Needs consent.",
            object(
                &[
                    (
                        "add",
                        json!({"type": "array", "items": {"type": "object", "required": ["phrase", "intent"], "properties": {"phrase": {"type": "string"}, "intent": {"type": "string"}, "slots": {"type": "object"}}}}),
                    ),
                    ("remove", json!({"type": "array", "items": {"type": "string"}, "description": "Exact phrases to drop"})),
                ],
                &[],
            ),
        ),
        tool(
            "apply_match",
            "Merge enable/precedence for known matcher ids. Not govern seeds — those are list_seeds / apply_house. Needs consent.",
            object(&[("match_controls", json!({"type": "array"}))], &["match_controls"]),
        ),
        apply_house_tool(),
        tool(
            "apply_aliases",
            "Merge spoken overlay aliases for a graph entity. Does not replace the list or assign rooms. Replace/remove aliases with apply_entity. Rooms: apply_area. Needs consent.",
            object(
                &[("entity_id", str_prop("entity_id")), ("aliases", json!({"type": "array", "items": {"type": "string"}}))],
                &["entity_id", "aliases"],
            ),
        ),
        tool(
            "apply_entity",
            "Set preferred / nlu_ignore, replace aliases, or remove aliases for one graph entity. Same overlay as Mapping. Rooms stay apply_area. Needs consent.",
            object(
                &[
                    ("entity_id", str_prop("entity_id")),
                    ("aliases", json!({"type": "array", "items": {"type": "string"}, "description": "Replace overlay alias list"})),
                    ("remove_aliases", json!({"type": "array", "items": {"type": "string"}})),
                    ("preferred", json!({"type": "boolean"})),
                    ("nlu_ignore", json!({"type": "boolean"})),
                ],
                &["entity_id"],
            ),
        ),
        tool(
            "apply_area",
            "Assign graph entities to a room. area is area_id from list_areas or a unique room name/alias. Empty clears. Batch with assignments. Not apply_aliases or apply_house. Needs consent.",
            object(
                &[
                    ("entity_id", str_prop("One entity_id")),
                    ("area", str_prop("area_id, unique room name, or empty to clear")),
                    ("assignments", json!({"type":"array","items":{"type":"object","required":["entity_id","area"],"properties":{"entity_id":{"type":"string"},"area":{"type":"string"}}}})),
                ],
                &[],
            ),
        ),
        tool(
            "apply_speech",
            "Write speech-bank variants for a house rule_id (language + personality + text). replace defaults true. Needs consent.",
            object(
                &[
                    ("rule_id", str_prop("House or seed rule id")),
                    (
                        "variants",
                        json!({"type": "array", "items": {"type": "object", "required": ["language", "personality", "text"], "properties": {"language": {"type": "string"}, "personality": {"type": "string"}, "text": {"type": "string"}}}}),
                    ),
                    ("replace", json!({"type": "boolean", "description": "Replace that rule's variants. Default true."})),
                ],
                &["rule_id"],
            ),
        ),
        tool(
            "apply_engine",
            "Patch engine settings (refine, calendar_llm, personality, languages, quiet_ack, nlu_rag, extra_prompt, unit_system, …). languages is Assist packs, not apply_ui.locale. Never URL, token, or model. Needs consent.",
            object(
                &[
                    ("personality", str_prop("default, butler, locker, fuersorglich, party, grantig, sarkastisch, pirat, hippie, gollum, jarvis, custom")),
                    ("mode", str_prop("full or context_only")),
                    ("languages", json!({"type": "array", "items": {"type": "string"}})),
                    ("refine_speech", json!({"type": "boolean"})),
                    ("refine_bands", json!({"type": "array", "items": {"type": "string", "enum": ["status", "command", "prompt", "reject"]}})),
                    ("calendar_llm", json!({"type": "boolean"})),
                    ("quiet_ack", json!({"type": "boolean"})),
                    ("nlu_rag", json!({"type": "boolean"})),
                    ("allow_llm_tools", json!({"type": "boolean"})),
                    ("confirm_risky_actions", json!({"type": "boolean"})),
                    ("semantic_adapters", json!({"type": "boolean"})),
                    ("support_bundle", json!({"type": "boolean"})),
                    ("support_bundle_raw_text", json!({"type": "boolean"})),
                    ("extra_prompt", str_prop("House rule user line. Empty keeps pack voice.")),
                    ("unit_system", str_prop("metric or imperial")),
                    ("custom_voice", str_prop("Voice block when personality is custom.")),
                    ("custom_voice_name", str_prop("Label for the custom voice.")),
                    ("custom_voice_seed", str_prop("Character seed. Traits only refine delivery.")),
                    ("custom_voice_traits", json!({"type": "object"})),
                ],
                &[],
            ),
        ),
        tool(
            "apply_ui",
            "Set operator chrome theme to light or dark, or UI locale. Not Assist language (that is apply_engine.languages). Needs consent.",
            object(&[("theme", str_prop("dark or light")), ("locale", str_prop("Operator chrome locale"))], &[]),
        ),
    ]
}

fn apply_house_tool() -> Value {
    tool(
        "apply_house",
        "Upsert house PolicyRule rows by id, or remove overlay ids. Does not assign rooms — use apply_area. Seed enabled:false turns a seed off; remove lets the compiled seed return. Not apply_match. Needs consent. when.floor is a floor_id. template/reply/script/llm require payload.",
        object(
            &[
                (
                    "policies",
                    json!({
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["id", "effect"],
                            "properties": {
                                "id": {"type": "string"},
                                "enabled": {"type": "boolean"},
                                "label": {"type": "string"},
                                "when": {
                                    "type": "object",
                                    "properties": {
                                        "intent": {"type": "string"},
                                        "domain": {"type": "string"},
                                        "area": {"type": "string", "description": "area_id from list_areas"},
                                        "entity_id": {"type": "string"},
                                        "floor": {"type": "string", "description": "floor_id from list_floors"},
                                        "name": {"type": "string"},
                                        "phrase": {"type": "string", "description": "Household wording, 4–200 characters"},
                                        "area_wide": {"type": "boolean"}
                                    }
                                },
                                "effect": {"type": "string", "enum": ["confirm", "block", "allow", "prefer_entity", "prefer_area", "reply", "script", "template", "llm"]},
                                "prefer": {"type": "string"},
                                "payload": {"type": "string", "description": "Required for reply/script/template/llm. template: HA Jinja."}
                            }
                        }
                    }),
                ),
                ("remove", json!({"type": "array", "items": {"type": "string"}, "description": "Overlay policy ids to drop"})),
            ],
            &[],
        ),
    )
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
