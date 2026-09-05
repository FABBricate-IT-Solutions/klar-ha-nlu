//! System prompt for the policy trainer. The model never runs on parse.

use super::types::ChatMessage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainerTurn {
    pub role: String,
    pub content: String,
}

pub fn system_prompt(context_json: &str) -> String {
    format!(
        "You are the Klar policy trainer. Klar is a deterministic, local, rule-based NLU. \
You never parse utterances at runtime. You propose overlay JSON for one operator lane.\n\n\
Rules:\n\
- Reply with a single JSON object. No markdown unless fenced json.\n\
- `layer` is `match`, `language`, `house`, or `all`.\n\
- Match: only `match_controls` with ids from schema.match_ids. No new PolicyId matchers.\n\
- Language: only `language_overlay.sets` add/remove on known paths. No verb flips, no fillers/particles/on/off of the bound locale.\n\
- House: `policies` as PolicyRule. Effects only from schema.effects. Ground entity/area/floor on graph.\n\
- Same id as a govern seed replaces that seed. To turn a seed off, post that id with enabled:false.\n\
- Slang belongs in the lexicon overlay, not when.phrase.\n\
- compiled_risky floor stays on even if a seed is off.\n\
- Pin language to the bound Assist locale in the context.\n\
- origin of applied rows is trainer after a human Apply. Do not apply yourself.\n\n\
Context JSON:\n{context_json}"
    )
}

pub fn history_messages(turns: &[TrainerTurn]) -> Result<Vec<ChatMessage>, super::types::LlmError> {
    let mut out = Vec::new();
    for turn in turns.iter().take(8) {
        let role = turn.role.trim();
        if !matches!(role, "user" | "assistant") {
            return Err(super::types::LlmError::InvalidRequest("role"));
        }
        if turn.content.is_empty() || turn.content.chars().count() > 4000 {
            return Err(super::types::LlmError::InvalidRequest("content"));
        }
        out.push(ChatMessage { role: role.to_string(), content: turn.content.clone() });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_pins_schema_and_floor() {
        let text = system_prompt(r#"{"language":"de","schema":{"seed_ids":["seed:confirm-lock"]}}"#);
        assert!(text.contains("compiled_risky"));
        assert!(text.contains("seed:confirm-lock"));
        assert!(text.contains("\"language\":\"de\""));
        assert!(text.contains("never parse utterances"));
    }
}
