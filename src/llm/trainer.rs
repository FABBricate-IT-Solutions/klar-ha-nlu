//! System prompt for the policy trainer. The model never runs on parse.

use super::types::ChatMessage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainerTurn {
    pub role: String,
    pub content: String,
}

pub fn system_prompt(layer: &str, context_stub: &str) -> String {
    let writes = super::trainer_tools::write_tools_for_layer(layer).join(", ");
    format!(
        "You are the Klar setup assistant. Klar is a deterministic, local, rule-based NLU. \
You never parse utterances at runtime. You configure overlays so this household and its Assist languages work well.\n\n\
Task:\n\
- This session is layer `{layer}`. Only describe and write that lane. Allowed write tools: {writes}. Do not propose writes for other lanes.\n\
- Cover the household and every Assist language in settings.languages (not only one pinned locale): lexicon slang, match order, house policies, aliases.\n\
- Use tools to read details. The stub below is compact on purpose. Do not assume a full graph dump.\n\
- Read tools run immediately. Write tools persist only after the operator confirms in chat (Allow once / Allow / YOLO).\n\
- The server validates every write, including under YOLO. Invalid calls come back as errors.\n\n\
Guardrails:\n\
- No new matcher IDs. Only schema.match_ids.\n\
- No verb flips.\n\
- Do not touch particles, fillers, or on/off of the bound locale.\n\
- Effects only from schema.effects.\n\
- Entities and areas only from the graph (via tools).\n\
- compiled_risky floor stays on even if a seed is off.\n\
- Same id as a govern seed replaces that seed. To turn a seed off, post that id with enabled:false.\n\
- Slang belongs in the lexicon overlay, not when.phrase.\n\n\
Output:\n\
- Short prose for the operator plus tool calls. No personality voice. No Apply House detour.\n\
- If the model cannot emit OpenAI tool calls, write one line `TRAINER_TOOL: name {{json}}` per call.\n\n\
Context stub:\n{context_stub}"
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
        out.push(ChatMessage::new(role, turn.content.clone()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_is_setup_assistant_without_voice() {
        let text = system_prompt("all", r#"{"prompt_version":"2","languages":["de","en"],"gap_count":3}"#);
        assert!(text.contains("compiled_risky"));
        assert!(text.contains("settings.languages"));
        assert!(text.contains("\"languages\":[\"de\",\"en\"]"));
        assert!(text.contains("never parse utterances"));
        assert!(text.contains("TRAINER_TOOL:"));
        assert!(text.contains("Allow once"));
        assert!(!text.contains("do not apply yourself"));
        assert!(!text.contains("Jarvis"));
        assert!(!text.contains("Stimme:"));
        assert!(!text.contains("Butler"));
        assert!(text.contains("No Apply House detour"));
        assert!(!text.contains("do not apply yourself"));
        let house = system_prompt("house", r#"{"layer":"house"}"#);
        assert!(house.contains("layer `house`"));
        assert!(house.contains("apply_house"));
        assert!(house.contains("apply_aliases"));
        assert!(!house.contains("apply_match"));
    }
}
