//! System prompt for the policy trainer. The model never runs on parse.

use super::types::ChatMessage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainerTurn {
    pub role: String,
    pub content: String,
}

fn write_scope(layer: &str, writes: &str) -> String {
    if layer == "all" || layer.is_empty() {
        format!(
            "- You can read and propose writes across house, match, and language in this same chat. Allowed write tools: {writes}. Infer what the operator needs. Offer 2–4 tap replies (LOTSE_CHOICES) instead of asking them to switch lanes. Never tell the operator to open Match, Language, or House."
        )
    } else {
        format!(
            "- Writes this session are layer `{layer}` only. Allowed write tools: {writes}. If the operator wants another lane, say so and wait; do not propose those writes."
        )
    }
}

pub fn system_prompt(layer: &str, context_stub: &str, reply_language: &str) -> String {
    let writes = super::trainer_tools::write_tools_for_layer(layer).join(", ");
    let scope = write_scope(layer, &writes);
    let reply = reply_language.trim();
    let reply = if reply.is_empty() { "en" } else { reply };
    format!(
        "Operator language: {reply}. Write every operator-facing sentence and LOTSE_CHOICES in this language. \
Tool names, entity_ids, and JSON stay unchanged. Do not switch to English unless the operator language is en.\n\n\
{handbook}\n\
You never parse utterances at runtime.\n\n\
Task:\n\
- Answer any question about Klar. Architecture, setup, guides, trade-offs — the whole product.\n\
{scope}\n\
- Cover the household and every Assist language in settings.languages (not only one pinned locale): lexicon slang, match order, house policies, aliases.\n\
- Use tools to read this house. The stub below is compact on purpose. Do not assume a full graph dump.\n\
- Read tools run immediately. Write tools persist only after the operator confirms in chat (Allow once / Allow / YOLO).\n\
- The server validates every write, including under YOLO. Invalid calls come back as errors.\n\n\
Guardrails:\n\
- No new matcher IDs. Only schema.match_ids.\n\
- No verb flips.\n\
- Do not touch particles, fillers, or on/off of the bound locale.\n\
- Effects only from schema.effects.\n\
- Entities, areas, and floors only from the graph (via tools). list_floors for floors; list_areas for rooms.\n\
- compiled_risky floor stays on even if a seed is off.\n\
- Same id as a govern seed replaces that seed. To turn a seed off, post that id with enabled:false.\n\
- Slang belongs in the lexicon overlay, not when.phrase. Household-only wording may use when.phrase plus a template payload (HA Jinja).\n\n\
Output:\n\
- Short prose for the operator plus tool calls. Same language as Operator language. No personality voice. No Apply House detour.\n\
- Tools already attach a `view` the UI renders as Klar cards (path, gaps, guide, architecture). After a tool, do not emit LOTSE_VIEW.\n\
- A panel without a live read is one line `LOTSE_VIEW: kind {{json}}` after the prose, never without JSON, never as operator-facing text. Kinds: guide, architecture, path, gaps, entity, house, matchers, policies, seeds, speech, lexicon, areas, floors, engine, counts, validate, write, languages, phrases, turns.\n\
- If you ask the operator a question, one line `LOTSE_CHOICES: [\"…\",\"…\"]` after the prose: 2–4 short replies they can tap, grounded in this house and the last tool results. No invented entity_ids. Skip the line when you are not asking.\n\
- If the model cannot emit OpenAI tool calls, write one line `TRAINER_TOOL: name {{json}}` per call.\n\n\
Context stub:\n{context_stub}",
        handbook = super::trainer_handbook::HANDBOOK,
        scope = scope
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
    fn prompt_is_lotse_without_voice() {
        let text = system_prompt("all", r#"{"prompt_version":"2","languages":["de","en"],"gap_count":3}"#, "de");
        assert!(text.contains("Operator language: de"));
        assert!(text.contains("Do not switch to English"));
        assert!(text.contains("Lotse"));
        assert!(text.contains("nlu::parse"));
        assert!(text.contains("fallback_agent"));
        assert!(text.contains("compiled_risky"));
        assert!(text.contains("settings.languages"));
        assert!(text.contains("\"languages\":[\"de\",\"en\"]"));
        assert!(text.contains("never parse utterances"));
        assert!(text.contains("TRAINER_TOOL:"));
        assert!(text.contains("LOTSE_VIEW:"));
        assert!(text.contains("do not emit LOTSE_VIEW"));
        assert!(text.contains("LOTSE_CHOICES:"));
        assert!(text.contains("explain_klar") || text.contains("`view`"));
        assert!(text.contains("Allow once"));
        assert!(!text.contains("do not apply yourself"));
        assert!(!text.contains("Jarvis"));
        assert!(!text.contains("Stimme:"));
        assert!(!text.contains("Butler"));
        assert!(text.contains("No Apply House detour"));
        assert!(!text.contains("do not apply yourself"));
        assert!(text.contains("apply_match"));
        assert!(text.contains("apply_house"));
        assert!(text.contains("apply_lexicon"));
        assert!(text.contains("Infer what the operator needs"));
        assert!(!text.contains("another lane"));
        let house = system_prompt("house", r#"{"layer":"house"}"#, "de");
        assert!(house.contains("layer `house`"));
        assert!(house.contains("apply_house"));
        assert!(house.contains("apply_aliases"));
        assert!(house.contains("apply_area"));
        assert!(house.contains("apply_entity"));
        assert!(house.contains("apply_engine"));
        assert!(house.contains("apply_ui"));
        assert!(house.contains("Never say you cannot change the visual theme"));
        assert!(house.contains("list_turns"));
        assert!(house.contains("list_floors"));
        assert!(house.contains("policy payload required"));
        assert!(!house.contains("apply_match"));
    }
}
