use crate::types::{known_intent, Intent, IntentPlan, Personality};
use serde::{Deserialize, Serialize};

pub const MAX_POLICY_RULES: usize = 64;
pub const MAX_SPEECH_VARIANTS: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchControl {
    pub id: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precedence: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchCatalogRow {
    pub id: String,
    pub precedence: u16,
    pub summary_key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Confirm,
    Block,
    Allow,
    PreferEntity,
    PreferArea,
    Reply,
    Script,
    Template,
    Llm,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyMatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phrase: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub area_wide: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRule {
    pub id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub when: PolicyMatch,
    pub effect: PolicyEffect,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

const fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechVariant {
    pub language: String,
    pub personality: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechBankEntry {
    pub rule_id: String,
    #[serde(default)]
    pub variants: Vec<SpeechVariant>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechBank {
    #[serde(default)]
    pub entries: Vec<SpeechBankEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyHit {
    Confirm,
    Block,
    Allow,
    PreferEntity,
    PreferArea,
    Reply,
    Script,
    Template,
    Llm,
}

impl PolicyHit {
    pub fn from_effect(effect: PolicyEffect) -> Self {
        match effect {
            PolicyEffect::Confirm => Self::Confirm,
            PolicyEffect::Block => Self::Block,
            PolicyEffect::Allow => Self::Allow,
            PolicyEffect::PreferEntity => Self::PreferEntity,
            PolicyEffect::PreferArea => Self::PreferArea,
            PolicyEffect::Reply => Self::Reply,
            PolicyEffect::Script => Self::Script,
            PolicyEffect::Template => Self::Template,
            PolicyEffect::Llm => Self::Llm,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Confirm => "confirm",
            Self::Block => "block",
            Self::Allow => "allow",
            Self::PreferEntity => "prefer_entity",
            Self::PreferArea => "prefer_area",
            Self::Reply => "reply",
            Self::Script => "script",
            Self::Template => "template",
            Self::Llm => "llm",
        }
    }

    pub fn is_action(self) -> bool {
        matches!(self, Self::Reply | Self::Script | Self::Template | Self::Llm)
    }
}

pub fn sanitize_rules(rules: Vec<PolicyRule>) -> Result<Vec<PolicyRule>, &'static str> {
    if rules.len() > MAX_POLICY_RULES {
        return Err("too many policy rules");
    }
    let mut seen = std::collections::BTreeSet::new();
    for rule in &rules {
        if rule.id.is_empty() || rule.id.chars().count() > 64 || !seen.insert(rule.id.as_str()) {
            return Err("invalid policy id");
        }
        if rule.label.chars().count() > 80 {
            return Err("policy label too long");
        }
        if let Some(intent) = rule.when.intent.as_deref() {
            if !known_intent(intent) && !intent.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
                return Err("invalid policy intent");
            }
        }
        if matches!(rule.effect, PolicyEffect::PreferEntity | PolicyEffect::PreferArea) && rule.prefer.as_ref().is_none_or(|v| v.is_empty())
        {
            return Err("prefer value required");
        }
        if let Some(phrase) = rule.when.phrase.as_deref() {
            if !(4..=200).contains(&phrase.chars().count()) {
                return Err("policy phrase must be 4–200 characters");
            }
        }
        match rule.effect {
            PolicyEffect::Reply => require_payload(rule.payload.as_deref(), 1, 200)?,
            PolicyEffect::Script => {
                require_payload(rule.payload.as_deref(), 1, 128)?;
                if script_entity_id(rule.payload.as_deref().unwrap_or("")).is_none() {
                    return Err("invalid script id");
                }
            }
            PolicyEffect::Template | PolicyEffect::Llm => require_payload(rule.payload.as_deref(), 1, 500)?,
            PolicyEffect::Confirm | PolicyEffect::Block | PolicyEffect::Allow | PolicyEffect::PreferEntity | PolicyEffect::PreferArea => {}
        }
    }
    Ok(rules)
}

fn require_payload(value: Option<&str>, min: usize, max: usize) -> Result<(), &'static str> {
    match value.map(str::trim) {
        Some(text) if (min..=max).contains(&text.chars().count()) => Ok(()),
        _ => Err("policy payload required"),
    }
}

pub fn script_entity_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let id = if trimmed.contains('.') { trimmed.to_string() } else { format!("script.{trimmed}") };
    let (domain, name) = id.split_once('.')?;
    if domain != "script" || name.is_empty() || name.len() > 64 {
        return None;
    }
    name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_').then_some(id)
}

pub fn sanitize_speech_bank(bank: SpeechBank) -> Result<SpeechBank, &'static str> {
    if bank.entries.len() > MAX_POLICY_RULES {
        return Err("too many speech entries");
    }
    for entry in &bank.entries {
        if entry.rule_id.is_empty() || entry.variants.len() > MAX_SPEECH_VARIANTS {
            return Err("invalid speech bank entry");
        }
        for variant in &entry.variants {
            if variant.language.len() > 8 || variant.personality.len() > 32 || variant.text.chars().count() > 200 {
                return Err("invalid speech variant");
            }
        }
    }
    Ok(bank)
}

pub fn first_matching_rule<'a>(rules: &'a [PolicyRule], plan: &IntentPlan) -> Option<(&'a PolicyRule, PolicyHit)> {
    rules.iter().filter(|rule| rule.enabled).find_map(|rule| {
        plan.steps.iter().any(|step| matches_when(&rule.when, &step.intent)).then_some((rule, PolicyHit::from_effect(rule.effect)))
    })
}

pub fn matches_when(when: &PolicyMatch, intent: &Intent) -> bool {
    if when.phrase.is_some() {
        return false;
    }
    field(when.intent.as_deref(), Some(intent.name.as_str()))
        && field(when.domain.as_deref(), intent.slot("domain").or_else(|| intent.slot("entity_id").and_then(domain_of)))
        && field(when.area.as_deref(), intent.slot("area"))
        && field(when.entity_id.as_deref(), intent.slot("entity_id"))
        && field(when.floor.as_deref(), intent.slot("floor"))
        && (!when.area_wide || intent.slot("area").is_some() || intent.slot("floor").is_some())
        && when.name.as_deref().is_none_or(|needle| {
            let hay = format!("{} {}", intent.slot("name").unwrap_or(""), intent.slot("entity_id").unwrap_or(""));
            hay.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
        })
}

fn field(expected: Option<&str>, actual: Option<&str>) -> bool {
    expected.is_none_or(|want| actual.is_some_and(|got| got.eq_ignore_ascii_case(want)))
}

fn domain_of(entity_id: &str) -> Option<&str> {
    entity_id.split_once('.').map(|(domain, _)| domain)
}

pub fn allow_permitted(plan: &IntentPlan) -> bool {
    plan.steps.iter().all(|step| {
        let intent = &step.intent;
        intent.slot("entity_id").is_some()
            && intent.slot("area").is_none()
            && intent.slot("floor").is_none()
            && !area_wide_lock_or_cover(intent)
    })
}

fn area_wide_lock_or_cover(intent: &Intent) -> bool {
    let domain = intent.slot("domain").or_else(|| intent.slot("entity_id").and_then(domain_of)).unwrap_or("");
    let area_wide = intent.slot("area").is_some() || intent.slot("floor").is_some();
    area_wide && matches!(domain, "lock" | "cover")
}

pub fn pick_speech(
    bank: &SpeechBank,
    rule_id: &str,
    language: &str,
    personality: Personality,
    conversation_id: &str,
    turn: u64,
) -> Option<String> {
    let variants: Vec<&SpeechVariant> = bank
        .entries
        .iter()
        .find(|entry| entry.rule_id == rule_id)?
        .variants
        .iter()
        .filter(|variant| variant.language == language && variant.personality == personality_key(personality))
        .collect();
    if variants.is_empty() {
        return None;
    }
    let index = hash_pick(conversation_id, turn, rule_id) % variants.len();
    Some(variants[index].text.clone())
}

pub fn fill_speech(template: &str, plan: &IntentPlan) -> String {
    let intent = plan.steps.first().map(|step| &step.intent);
    let name = intent.and_then(|item| item.slot("name")).unwrap_or("");
    let area = intent.and_then(|item| item.slot("area")).unwrap_or("");
    template.replace("{name}", name).replace("{area}", area)
}

fn personality_key(personality: Personality) -> &'static str {
    match personality {
        Personality::Default => "default",
        Personality::Butler => "butler",
        Personality::Locker => "locker",
        Personality::Fuersorglich => "fuersorglich",
        Personality::Party => "party",
        Personality::Grantig => "grantig",
        Personality::Sarkastisch => "sarkastisch",
        Personality::Pirat => "pirat",
        Personality::Hippie => "hippie",
        Personality::Gollum => "gollum",
        Personality::Jarvis => "jarvis",
        Personality::Custom => "default",
    }
}

fn hash_pick(conversation_id: &str, turn: u64, rule_id: &str) -> usize {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in conversation_id.bytes().chain(rule_id.bytes()).chain(turn.to_le_bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Intent;

    fn plan(intent: Intent) -> IntentPlan {
        IntentPlan::from_intents(vec![intent], 1.0, &[])
    }

    #[test]
    fn allow_named_light_is_permitted() {
        let ready = plan(Intent::new("HassTurnOff").with("entity_id", "light.kugel"));
        assert!(allow_permitted(&ready));
    }

    #[test]
    fn allow_area_lock_is_not_permitted() {
        let blocked = plan(Intent::new("HassTurnOff").with("area", "wohnzimmer").with("domain", "lock"));
        assert!(!allow_permitted(&blocked));
    }

    #[test]
    fn first_enabled_rule_wins() {
        let rules = vec![
            PolicyRule {
                id: "block-ac".into(),
                enabled: true,
                label: "AC".into(),
                when: PolicyMatch { entity_id: Some("climate.schlafzimmer_ac".into()), ..PolicyMatch::default() },
                effect: PolicyEffect::Block,
                prefer: None,
                payload: None,
            },
            PolicyRule {
                id: "later".into(),
                enabled: true,
                label: "later".into(),
                when: PolicyMatch { domain: Some("climate".into()), ..PolicyMatch::default() },
                effect: PolicyEffect::Confirm,
                prefer: None,
                payload: None,
            },
        ];
        let ready = plan(Intent::new("HassTurnOff").with("entity_id", "climate.schlafzimmer_ac").with("domain", "climate"));
        let (rule, hit) = first_matching_rule(&rules, &ready).expect("match");
        assert_eq!(rule.id, "block-ac");
        assert_eq!(hit, PolicyHit::Block);
    }

    #[test]
    fn script_id_normalizes_and_rejects_junk() {
        assert_eq!(script_entity_id("leaving_home").as_deref(), Some("script.leaving_home"));
        assert_eq!(script_entity_id("script.good_night").as_deref(), Some("script.good_night"));
        assert!(script_entity_id("light.kugel").is_none());
        assert!(script_entity_id("../etc").is_none());
    }

    #[test]
    fn reply_without_payload_is_rejected() {
        let rules = vec![PolicyRule {
            id: "empty".into(),
            enabled: true,
            label: "x".into(),
            when: PolicyMatch { phrase: Some("hallo dort".into()), ..PolicyMatch::default() },
            effect: PolicyEffect::Reply,
            prefer: None,
            payload: None,
        }];
        assert!(sanitize_rules(rules).is_err());
    }
}
