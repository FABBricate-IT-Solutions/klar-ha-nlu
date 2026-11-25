use crate::lang::catalog;
use crate::parse::action::Action;
use crate::session::Session;

pub(crate) fn is_also_token(token: &str) -> bool {
    matches!(
        token,
        "auch"
            | "too"
            | "also"
            | "well"
            | "aussi"
            | "ook"
            | "tambien"
            | "anche"
            | "tambem"
            | "tambe"
            | "ogsaa"
            | "ocksa"
            | "myos"
            | "taky"
            | "tiez"
            | "tez"
            | "takodjer"
            | "tudi"
            | "takodje"
            | "също"
            | "επισης"
            | "такође"
            | "також"
            | "也"
            | "都"
            | "ايضا"
            | "גם"
            | "هم"
            | "بھی"
            | "dahi"
            | "deasemenea"
            | "duay"
            | "도"
            | "も"
            | "hefyd"
            | "ka"
            | "ere"
            | "freisin"
            | "tamén"
            | "lika"
            | "genausou"
            | "ynwedh"
            | "irgi"
            | "ari"
            | "juga"
            | "pia"
            | "nua"
            | "bhi"
            | "pan"
            | "kooda"
            | "koode"
            | "suddha"
            | "kuda"
            | "vi"
            | "pani"
            | "el"
            | "ch"
    )
}

pub(crate) fn guess_action(tokens: &[String], session: &Session, number: Option<i32>) -> Action {
    if number.is_some() {
        return crate::parse::numbers::guess_numbered_action(
            tokens,
            session.last_entities().any(|entity| entity.starts_with("climate."))
                || session.last_names().any(|name| name.contains("Climate"))
                || session.last_domains().any(|domain| domain == "climate"),
            session.last_entities().any(|entity| entity.starts_with("cover.")) || session.last_domains().any(|domain| domain == "cover"),
            session.last_entities().any(|entity| entity.starts_with("fan.")) || session.last_domains().any(|domain| domain == "fan"),
        );
    }
    if crate::parse::quantity::is_relative_temp(tokens) {
        return Action::SetTemp;
    }
    // "and the hall", "no, the bedroom", "auch das Büro": carry the last action (P-14).
    if is_followup_target(tokens) {
        return followup_action(tokens, session);
    }
    let cat = catalog();
    if cat.any(tokens, cat.tv_words()) {
        Action::On
    } else if cat.any(tokens, cat.media_nouns()) {
        Action::MediaPlay
    } else {
        Action::GetState
    }
}

fn is_followup_target(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.iter().any(|token| is_also_token(token))
        || tokens.first().is_some_and(|token| {
            cat.is_conj(token) || matches!(token.as_str(), "no" | "nein" | "nope" | "nicht" | "stattdessen" | "instead")
        })
}

fn followup_action(tokens: &[String], session: &Session) -> Action {
    if catalog().any(tokens, catalog().off_words()) {
        return Action::Off;
    }
    // Bare "and the kitchen too" stays a positive command — do not inherit a prior
    // TurnOff ("Alle Lichter aus" / "turn off all the lights").
    match session.last.first().map(|turn| turn.name.as_str()) {
        Some("HassLightSet") => Action::SetLight,
        Some("HassClimateSetTemperature") => Action::SetTemp,
        Some("HassFanSetSpeed") => Action::FanSpeed,
        Some("HassTurnOn") | Some("HassToggle") => Action::On,
        _ => Action::On,
    }
}
