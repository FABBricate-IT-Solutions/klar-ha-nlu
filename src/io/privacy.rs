//! Privacy-safe support-bundle export: hashed IDs, pseudonyms, opt-in raw text.

use super::bundle::{BundleEntry, BundleRequest, BundleResponse};
use crate::parse::normalize::tokenize;
use crate::types::Intent;
use std::collections::BTreeMap;

pub fn hash_conversation_id(id: &str) -> String {
    format!("cid_{:016x}", fnv1a64(id.as_bytes()))
}

pub fn replay_tokens(text: &str) -> Vec<String> {
    tokenize(text)
}

pub fn redact_entries(entries: &[BundleEntry], include_raw_text: bool) -> Vec<BundleEntry> {
    let mut names = NameMap::default();
    entries.iter().map(|entry| redact_entry(entry, include_raw_text, &mut names)).collect()
}

pub fn redact_entry(entry: &BundleEntry, include_raw_text: bool, names: &mut NameMap) -> BundleEntry {
    let tokens = if entry.tokens.is_empty() { replay_tokens(&entry.request.text) } else { entry.tokens.clone() };
    let conversation_id = entry.request.conversation_id.as_deref().filter(|id| !id.is_empty()).map(hash_conversation_id);
    let text = if include_raw_text { entry.request.text.clone() } else { tokens.join(" ") };
    let speech = if include_raw_text { entry.response.speech.clone() } else { String::new() };
    BundleEntry {
        id: entry.id.clone(),
        ts_ms: entry.ts_ms,
        source: entry.source.clone(),
        language: entry.language.clone(),
        tokens: tokens.clone(),
        request: BundleRequest { text, conversation_id },
        response: BundleResponse {
            intents: entry.response.intents.iter().map(|intent| names.intent(intent)).collect(),
            speech,
            clarify: entry.response.clarify,
            chat: entry.response.chat,
            briefing: entry.response.briefing,
        },
    }
}

#[derive(Debug, Default)]
pub struct NameMap {
    entities: BTreeMap<String, String>,
    areas: BTreeMap<String, String>,
}

impl NameMap {
    pub fn entity(&mut self, entity_id: &str) -> String {
        if let Some(name) = self.entities.get(entity_id) {
            return name.clone();
        }
        let domain = entity_id.split_once('.').map(|(domain, _)| domain).unwrap_or("entity");
        let name = format!("{domain}.e{:02}", self.entities.len() + 1);
        self.entities.insert(entity_id.into(), name.clone());
        name
    }

    pub fn area(&mut self, area: &str) -> String {
        if let Some(name) = self.areas.get(area) {
            return name.clone();
        }
        let name = format!("a{:02}", self.areas.len() + 1);
        self.areas.insert(area.into(), name.clone());
        name
    }

    fn intent(&mut self, intent: &Intent) -> Intent {
        let mut out = Intent::new(&intent.name);
        for slot in &intent.slots {
            let value = match slot.name.as_str() {
                "entity_id" => self.entity(&slot.value),
                "area" => self.area(&slot.value),
                _ => slot.value.clone(),
            };
            out = out.with(&slot.name, value);
        }
        out
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
