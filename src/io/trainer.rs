//! Trainer context and validate. Chat completions live in `crate::llm`.

use crate::home::gaps::leftover;
use crate::home::overlay::load_overlay;
use crate::io::auth::reads_allowed;
use crate::io::state::AppState;
use crate::lang::{catalog_for, validate_language, LanguageOverlay};
use crate::nlu::{parse_with_controls, safety_decide_policies};
use crate::parse::{match_catalog, match_control_warnings, sanitize_match_controls};
use crate::session::Session;
use crate::types::{
    govern_safety_seeds, sanitize_rules, AreaRec, EntityRec, FloorRec, HomeGraph, Intent, IntentPlan, MatchCatalogRow, MatchControl,
    ParseDecision, PolicyEffect, PolicyRule, Settings, SpeechBank, MAX_POLICY_RULES,
};
use axum::extract::{ConnectInfo, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

const PROMPT_VERSION: &str = "2";
const SMOKES: &[(&str, &str)] = &[
    ("de", "licht wohnzimmer an"),
    ("en", "turn on the living room light"),
    ("fr", "allume la lumiere salon"),
    ("ja", "つけて 電気 リビング"),
    ("ar", "شغل ضوء صالون"),
    ("pt-BR", "liga luz sala"),
    ("de-CH", "mach liecht wohnzimmer"),
    ("zh-CN", "打开 灯 客厅"),
];

#[derive(Debug, Deserialize)]
pub struct TrainerQuery {
    pub layer: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TrainerContext {
    pub language: String,
    pub layer: String,
    pub prompt_version: String,
    pub graph: GraphOut,
    pub gaps: Vec<String>,
    pub matches: Vec<MatchCatalogRow>,
    pub seeds: Vec<PolicyRule>,
    pub overlays: OverlaysOut,
    pub schema: SchemaOut,
}

#[derive(Debug, Serialize)]
pub struct GraphOut {
    pub areas: Vec<AreaRec>,
    pub floors: Vec<FloorRec>,
    pub entities: Vec<EntityRec>,
}

#[derive(Debug, Serialize)]
pub struct OverlaysOut {
    pub policies: Vec<PolicyRule>,
    pub match_controls: Vec<MatchControl>,
    pub language: LanguageOverlay,
}

#[derive(Debug, Serialize)]
pub struct SchemaOut {
    pub effects: Vec<&'static str>,
    pub when_fields: Vec<&'static str>,
    pub max_rules: usize,
    pub seed_ids: Vec<String>,
    pub match_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProposalIn {
    pub layer: Option<String>,
    pub language: Option<String>,
    pub policies: Option<Vec<PolicyRule>>,
    pub match_controls: Option<Vec<MatchControl>>,
    pub language_overlay: Option<LanguageOverlay>,
    pub utterances: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct Issue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DryRunRow {
    pub text: String,
    pub decision: String,
    pub seed: Option<String>,
    pub house: Option<String>,
    pub compiled_risky: bool,
}

#[derive(Debug, Serialize)]
pub struct ValidateOut {
    pub ok: bool,
    pub errors: Vec<Issue>,
    pub warnings: Vec<Issue>,
    pub dry_run: Vec<DryRunRow>,
}

pub async fn trainer_context(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<TrainerQuery>,
) -> Result<Json<TrainerContext>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(load_context(&state, &query).await?))
}

pub async fn load_context(state: &AppState, query: &TrainerQuery) -> Result<TrainerContext, StatusCode> {
    let settings = state.settings.lock().await.clone();
    let language = pin_language(query.language.as_deref(), &settings).map_err(|_| StatusCode::BAD_REQUEST)?;
    let home = state.home.snapshot().await;
    let catalog = catalog_for(&[language.clone()]);
    let overlay = load_overlay(&state.data_dir);
    Ok(TrainerContext {
        language,
        layer: query.layer.clone().unwrap_or_else(|| "all".into()),
        prompt_version: PROMPT_VERSION.into(),
        graph: GraphOut { areas: home.areas.clone(), floors: home.floors.clone(), entities: home.entities.clone() },
        gaps: leftover(&home, catalog).into_iter().map(|entity| entity.entity_id).collect(),
        matches: match_catalog(),
        seeds: govern_safety_seeds().to_vec(),
        overlays: OverlaysOut {
            policies: state.policies.lock().await.clone(),
            match_controls: state.match_controls.lock().await.clone(),
            language: overlay.language,
        },
        schema: schema_out(),
    })
}

pub async fn validate_proposal(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ProposalIn>,
) -> Result<Json<ValidateOut>, StatusCode> {
    if !reads_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let settings = state.settings.lock().await.clone();
    let language = pin_language(body.language.as_deref(), &settings).map_err(|_| StatusCode::BAD_REQUEST)?;
    let home = state.home.snapshot().await;
    let house = match body.policies.clone() {
        Some(rules) => rules,
        None => state.policies.lock().await.clone(),
    };
    let match_controls = match body.match_controls.clone() {
        Some(rows) => rows,
        None => state.match_controls.lock().await.clone(),
    };
    let speech_bank = state.speech_bank.lock().await.clone();
    let overlay = match body.language_overlay.clone() {
        Some(language) => language,
        None => load_overlay(&state.data_dir).language,
    };
    Ok(Json(validate(
        &home,
        &settings,
        &language,
        body.layer.as_deref().unwrap_or("all"),
        house,
        match_controls,
        overlay,
        &speech_bank,
        body.utterances.as_deref().unwrap_or(&[]),
    )))
}

#[allow(clippy::too_many_arguments)]
pub fn validate(
    home: &HomeGraph,
    settings: &Settings,
    language: &str,
    layer: &str,
    house: Vec<PolicyRule>,
    match_controls: Vec<MatchControl>,
    overlay: LanguageOverlay,
    speech_bank: &SpeechBank,
    extra: &[String],
) -> ValidateOut {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let catalog = catalog_for(&[language.to_string()]);
    if layer_includes(layer, "house") {
        match sanitize_rules(house.clone()) {
            Ok(rules) => errors.extend(ground_rules(home, &rules)),
            Err(message) => errors.push(issue("policies", message)),
        }
    }
    if layer_includes(layer, "match") {
        match sanitize_match_controls(match_controls.clone()) {
            Ok(rows) => warnings.extend(match_control_warnings(&rows).into_iter().map(|message| issue("match_controls", message))),
            Err(message) => errors.push(issue("match_controls", message)),
        }
    }
    if layer_includes(layer, "language") {
        for row in validate_language(&overlay) {
            errors.push(Issue { path: row.path, message: row.message });
        }
        errors.extend(blocked_lexicon_tokens(catalog, &overlay));
    }
    let mut pinned = settings.clone();
    pinned.languages = vec![language.to_string()];
    let dry_run =
        if errors.is_empty() { dry_run_rows(home, &pinned, &house, &match_controls, speech_bank, language, extra) } else { Vec::new() };
    ValidateOut { ok: errors.is_empty(), errors, warnings, dry_run }
}

pub fn context_stub(ctx: &TrainerContext, languages: &[String]) -> String {
    serde_json::json!({
        "prompt_version": ctx.prompt_version,
        "languages": languages,
        "layer": ctx.layer,
        "schema": ctx.schema,
        "gap_count": ctx.gaps.len(),
        "policy_count": ctx.overlays.policies.len(),
        "match_overlay_count": ctx.overlays.match_controls.len(),
    })
    .to_string()
}

fn schema_out() -> SchemaOut {
    SchemaOut {
        effects: vec!["confirm", "block", "allow", "prefer_entity", "prefer_area", "reply", "script", "template", "llm"],
        when_fields: vec!["intent", "domain", "area", "entity_id", "floor", "name", "phrase", "area_wide"],
        max_rules: MAX_POLICY_RULES,
        seed_ids: govern_safety_seeds().iter().map(|rule| rule.id.clone()).collect(),
        match_ids: match_catalog().into_iter().map(|row| row.id).collect(),
    }
}

fn pin_language(requested: Option<&str>, settings: &Settings) -> Result<String, &'static str> {
    let code = requested
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .or_else(|| settings.languages.first().cloned())
        .unwrap_or_else(|| "en".into());
    if crate::lang::LangId::from_code(&code).is_none() && crate::lang::LangId::from_tag(&code).is_none() {
        return Err("unknown language");
    }
    Ok(code)
}

fn layer_includes(layer: &str, name: &str) -> bool {
    layer == "all" || layer == name
}

fn issue(path: &str, message: impl Into<String>) -> Issue {
    Issue { path: path.into(), message: message.into() }
}

fn ground_rules(home: &HomeGraph, rules: &[PolicyRule]) -> Vec<Issue> {
    let mut errors = Vec::new();
    for rule in rules {
        if let Some(entity_id) = rule.when.entity_id.as_deref() {
            if !home.entities.iter().any(|entity| entity.entity_id == entity_id) {
                errors.push(issue(&format!("policies.{}.when.entity_id", rule.id), "entity is not on the graph"));
            }
        }
        if let Some(area) = rule.when.area.as_deref() {
            if !home.areas.iter().any(|record| record.area_id == area) {
                errors.push(issue(&format!("policies.{}.when.area", rule.id), "area is not on the graph"));
            }
        }
        if let Some(floor) = rule.when.floor.as_deref() {
            if home.floor(floor).is_none() {
                errors.push(issue(&format!("policies.{}.when.floor", rule.id), "floor is not on the graph"));
            }
        }
        match rule.effect {
            PolicyEffect::PreferEntity => {
                if rule.prefer.as_ref().is_some_and(|id| !home.entities.iter().any(|entity| entity.entity_id == *id)) {
                    errors.push(issue(&format!("policies.{}.prefer", rule.id), "prefer entity is not on the graph"));
                }
            }
            PolicyEffect::PreferArea => {
                if rule.prefer.as_ref().is_some_and(|id| !home.areas.iter().any(|record| record.area_id == *id)) {
                    errors.push(issue(&format!("policies.{}.prefer", rule.id), "prefer area is not on the graph"));
                }
            }
            PolicyEffect::Confirm
            | PolicyEffect::Block
            | PolicyEffect::Allow
            | PolicyEffect::Reply
            | PolicyEffect::Script
            | PolicyEffect::Template
            | PolicyEffect::Llm => {}
        }
    }
    errors
}

fn blocked_lexicon_tokens(catalog: &crate::lang::Catalog, overlay: &LanguageOverlay) -> Vec<Issue> {
    let mut errors = Vec::new();
    for (path, delta) in &overlay.sets {
        for token in delta.add.iter().chain(delta.remove.iter()) {
            let word = token.trim();
            if catalog.is_particle(word)
                || catalog.is_filler(word)
                || catalog.on_words().contains(word)
                || catalog.off_words().contains(word)
            {
                errors.push(issue(&format!("language.sets.{path}"), format!("token {word} is a particle or filler of the bound locale")));
            }
        }
    }
    errors
}

fn dry_run_rows(
    home: &HomeGraph,
    settings: &Settings,
    house: &[PolicyRule],
    match_controls: &[MatchControl],
    speech_bank: &SpeechBank,
    language: &str,
    extra: &[String],
) -> Vec<DryRunRow> {
    let mut rows = Vec::new();
    for text in smokes(language).into_iter().chain(extra.iter().map(String::as_str)) {
        let mut session = Session::new();
        let outcome = parse_with_controls(text, home, &mut session, &[], settings, house, speech_bank, match_controls);
        let trace = outcome.policy_trace.clone().unwrap_or_default();
        rows.push(DryRunRow {
            text: text.to_string(),
            decision: outcome.decision.type_name().into(),
            seed: trace.seed.map(|layer| layer.id),
            house: trace.house.map(|layer| layer.id),
            compiled_risky: trace.compiled_risky,
        });
    }
    rows.extend(plan_rows(home, settings, house, speech_bank));
    rows
}

fn smokes(language: &str) -> Vec<&'static str> {
    SMOKES.iter().filter(|(code, _)| *code == language).map(|(_, text)| *text).collect()
}

fn plan_rows(home: &HomeGraph, settings: &Settings, house: &[PolicyRule], speech_bank: &SpeechBank) -> Vec<DryRunRow> {
    let lock =
        IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("entity_id", "lock.wohnungstuer").with("domain", "lock")], 0.9, &[]);
    let cover = IntentPlan::from_intents(
        vec![Intent::new("HassTurnOff").with("entity_id", "cover.wohnzimmer_rollo").with("domain", "cover")],
        0.9,
        &[],
    );
    [("lock.entity", lock), ("cover.close", cover)]
        .into_iter()
        .filter(|(_, plan)| {
            home.entities
                .iter()
                .any(|entity| plan.steps.iter().any(|step| step.intent.slot("entity_id") == Some(entity.entity_id.as_str())))
        })
        .map(|(label, plan)| {
            let (decision, _) = safety_decide_policies(home, settings, plan, 0.9, 1.0, false, (house, speech_bank));
            DryRunRow {
                text: label.into(),
                decision: decision_label(&decision),
                seed: None,
                house: None,
                compiled_risky: matches!(decision, ParseDecision::Confirm { .. }),
            }
        })
        .collect()
}

fn decision_label(decision: &ParseDecision) -> String {
    decision.type_name().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::lang::{LanguageOverlay, SetDelta};
    use std::collections::HashMap;

    fn settings() -> Settings {
        Settings::pinned("de")
    }

    #[test]
    fn unknown_match_id_is_rejected() {
        let home = default_home();
        let controls = vec![MatchControl { id: "media_new_matcher".into(), enabled: true, precedence: None }];
        let out =
            validate(&home, &settings(), "de", "match", Vec::new(), controls, LanguageOverlay::default(), &SpeechBank::default(), &[]);
        assert!(!out.ok);
        assert!(out.errors.iter().any(|row| row.path == "match_controls"));
    }

    #[test]
    fn bound_locale_particle_add_is_rejected() {
        let home = default_home();
        let mut sets = HashMap::new();
        sets.insert("nouns.light_nouns".into(), SetDelta { add: vec!["an".into()], remove: Vec::new() });
        let de = validate(
            &home,
            &settings(),
            "de",
            "language",
            Vec::new(),
            Vec::new(),
            LanguageOverlay { sets: sets.clone() },
            &SpeechBank::default(),
            &[],
        );
        assert!(!de.ok, "{de:?}");
        let mut ja_sets = HashMap::new();
        ja_sets.insert("nouns.light_nouns".into(), SetDelta { add: vec!["つけて".into()], remove: Vec::new() });
        let ja = validate(
            &home,
            &Settings::pinned("ja"),
            "ja",
            "language",
            Vec::new(),
            Vec::new(),
            LanguageOverlay { sets: ja_sets },
            &SpeechBank::default(),
            &[],
        );
        assert!(!ja.ok, "{ja:?}");
    }

    #[test]
    fn missing_entity_is_not_grounded() {
        let home = default_home();
        let rules = vec![PolicyRule {
            id: "ghost".into(),
            enabled: true,
            label: "x".into(),
            when: crate::types::PolicyMatch { entity_id: Some("light.missing".into()), ..crate::types::PolicyMatch::default() },
            effect: PolicyEffect::Block,
            prefer: None,
            payload: None,
        }];
        let out = validate(&home, &settings(), "de", "house", rules, Vec::new(), LanguageOverlay::default(), &SpeechBank::default(), &[]);
        assert!(!out.ok);
        assert!(out.errors.iter().any(|row| row.path.contains("entity_id")));
    }

    #[test]
    fn context_stub_is_compact() {
        let ctx = TrainerContext {
            language: "de".into(),
            layer: "all".into(),
            prompt_version: "2".into(),
            graph: GraphOut { areas: Vec::new(), floors: Vec::new(), entities: Vec::new() },
            gaps: vec!["light.a".into(), "light.b".into()],
            matches: Vec::new(),
            seeds: Vec::new(),
            overlays: OverlaysOut { policies: Vec::new(), match_controls: Vec::new(), language: LanguageOverlay::default() },
            schema: schema_out(),
        };
        let stub = context_stub(&ctx, &["de".into(), "en".into()]);
        assert!(stub.contains("\"gap_count\":2"));
        assert!(stub.contains("\"languages\":[\"de\",\"en\"]"));
        assert!(!stub.contains("light.a"));
        assert!(!stub.contains("graph"));
    }
}
