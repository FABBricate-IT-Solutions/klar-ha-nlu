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
    if tokens.iter().any(|token| is_also_token(token)) {
        followup_action(tokens, session)
    } else {
        Action::GetState
    }
}

fn followup_action(tokens: &[String], session: &Session) -> Action {
    if catalog().any(tokens, catalog().off_words()) {
        return Action::Off;
    }
    match session.last.first().map(|turn| turn.name.as_str()) {
        Some("HassLightSet") => Action::SetLight,
        _ => Action::On,
    }
}
