//! Table-driven read-only questions for contact and value sensors (P-17 / #329).
//! Window, door, motion, smoke, humidity, temperature and battery share one route.
//! Cover commands stay on the cover domain; a garage door is never the answer.

use super::context::ParseContext;
use super::draft::{execute, reject, Draft};
use crate::lang::Catalog;
use crate::parse::infer::looks_like_question;
use crate::parse::resolve::resolve;
use crate::parse::respond::speak_unknown;
use crate::types::{EntityRec, HomeGraph, Intent, IntentPlan, RejectReason};

/// One row of the sensor-ask table.
struct Kind {
    device_class: &'static str,
    domain: &'static str,
    /// Multilingual cue tokens (plus pack window/door lists where they exist).
    cues: &'static [&'static str],
    /// Open/closed maps to on/off for binary contacts; value sensors stay unfiltered.
    binary: bool,
}

const KINDS: &[Kind] = &[
    Kind { device_class: "window", domain: "binary_sensor", cues: WINDOW, binary: true },
    Kind { device_class: "door", domain: "binary_sensor", cues: DOOR, binary: true },
    Kind { device_class: "motion", domain: "binary_sensor", cues: MOTION, binary: true },
    Kind { device_class: "smoke", domain: "binary_sensor", cues: SMOKE, binary: true },
    Kind { device_class: "humidity", domain: "sensor", cues: HUMIDITY, binary: false },
    Kind { device_class: "temperature", domain: "sensor", cues: TEMPERATURE, binary: false },
    Kind { device_class: "battery", domain: "sensor", cues: BATTERY, binary: false },
];

#[rustfmt::skip]
const WINDOW: &[&str] = &[
    "window", "windows", "fenster", "raam", "ramen", "fenetre", "fenêtre", "ventanas", "ventana", "finestra", "finestre",
];
#[rustfmt::skip]
const DOOR: &[&str] = &[
    "door", "doors", "tuer", "tür", "tuere", "türe", "tueren", "türen", "deur", "deuren", "porte", "portes", "puerta", "puertas",
];
#[rustfmt::skip]
const MOTION: &[&str] = &[
    "motion", "bewegung", "bewegungsmelder", "beweging", "mouvement", "movimiento", "movimento",
];
#[rustfmt::skip]
const SMOKE: &[&str] = &[
    "smoke", "rauch", "rauchmelder", "rook", "fumee", "fumée", "humo", "fumo",
];
#[rustfmt::skip]
const HUMIDITY: &[&str] = &[
    "humidity", "humid", "feuchte", "luftfeuchtigkeit", "feuchtigkeit", "vochtigheid", "humidite", "humidité", "humedad",
];
#[rustfmt::skip]
const TEMPERATURE: &[&str] = &[
    "temperature", "temperatures", "temp", "temps", "temperatur", "temperaturen", "temperatuur",
];
#[rustfmt::skip]
const BATTERY: &[&str] = &[
    "battery", "batteries", "batterie", "batterien", "akku", "akkus", "batterij",
];

/// Whole-home / indefinite place words that must not fall through to a cover.
#[rustfmt::skip]
const PLACE_PAD: &[&str] = &[
    "house", "home", "somewhere", "anywhere", "everywhere", "haus", "zuhause", "irgendwo", "irgendwohin",
];
#[rustfmt::skip]
const TIME_PAD: &[&str] = &[
    "tonight", "today", "now", "currently", "heute", "jetzt", "gerade", "momentan", "heuteabend", "heutnacht",
];
#[rustfmt::skip]
const QUANTITY_PAD: &[&str] = &["how", "many", "every", "each", "viel", "viele", "jedes", "jede", "jeder", "hoeveel"];
#[rustfmt::skip]
const CLOSED_SYN: &[&str] = &["shut", "dicht"];

pub(super) fn route(context: &ParseContext<'_>, tokens: &[String]) -> Option<Draft> {
    let kind = detect_kind(context.catalog, tokens)?;
    if !is_status_question(context.catalog, tokens, kind) {
        return None;
    }
    // Blinds / position asks keep the cover parser even when the user said "windows".
    if kind.device_class == "window" && wants_cover_not_contact(context.catalog, tokens) {
        return None;
    }
    let resolved = resolve(tokens, context.home, Some(kind.domain));
    // Temperature without a room or "sensor" cue belongs to weather/climate routes.
    if kind.device_class == "temperature" && !context.catalog.any(tokens, context.catalog.sensor_words()) && resolved.areas.is_empty() {
        return None;
    }
    // Multi-room lists fall through to the clause parser (do not reject here).
    if resolved.areas.len() > 1 || resolved.floors.len() > 1 {
        return None;
    }
    // Room whose "windows" are blinds/curtains: let the cover parser answer.
    if kind.device_class == "window" {
        if let Some(area) = resolved.areas.first() {
            if !has_class_sensor(context.catalog, context.home, Some(area), kind) && has_window_cover(context.catalog, context.home, area) {
                return None;
            }
        }
    }
    if has_leftover_content(context.catalog, tokens, kind, &resolved.areas, &resolved.floors, context.home) {
        // Unknown place: do not silently answer the whole home, and never a cover.
        if kind.device_class == "window" {
            return Some(reject(RejectReason::NoTarget, speak_unknown()));
        }
        return None;
    }
    let open = context.catalog.any(tokens, context.catalog.open_words());
    let closed =
        context.catalog.any(tokens, context.catalog.close_words()) || tokens.iter().any(|word| CLOSED_SYN.contains(&word.as_str()));
    let mut intent =
        Intent::new("HassGetState").with("domain", kind.domain).with("device_class", kind.device_class).with("response", "list");
    if kind.binary && (open || closed) {
        intent = intent.with("state", if open { "on" } else { "off" });
    }
    if let Some(area) = resolved.areas.first() {
        intent = intent.with("area", area);
    } else if let Some(floor) = resolved.floors.first() {
        intent = intent.with("floor", floor);
    }
    if super::validation::validate_plan(&IntentPlan::from_intents(vec![intent.clone()], 1.0, &[]), context.home).is_err() {
        if kind.device_class == "window" {
            if let Some(area) = intent.slot("area") {
                if has_window_cover(context.catalog, context.home, area) {
                    return None;
                }
            }
            return Some(reject(RejectReason::NoTarget, speak_unknown()));
        }
        return None;
    }
    if kind.binary && open && closed {
        return Some(reject(RejectReason::Unsupported, speak_unknown()));
    }
    let source = format!("{}_state_query", kind.device_class);
    Some(execute(context, vec![intent], &source, 1.0, 1.0, false, false))
}

fn wants_cover_not_contact(cat: &Catalog, tokens: &[String]) -> bool {
    tokens.iter().any(|word| {
        matches!(
            word.as_str(),
            "position"
                | "blinds"
                | "blind"
                | "curtain"
                | "curtains"
                | "shades"
                | "shade"
                | "rollo"
                | "rollos"
                | "jalousie"
                | "vorhang"
                | "vorhaenge"
                | "markise"
        ) || cat.curtain_nouns().contains(word.as_str())
    })
}

fn detect_kind<'a>(cat: &Catalog, tokens: &[String]) -> Option<&'a Kind> {
    KINDS.iter().find(|kind| tokens.iter().any(|word| cue_match(cat, kind, word)))
}

fn cue_match(cat: &Catalog, kind: &Kind, word: &str) -> bool {
    kind.cues.iter().any(|cue| *cue == word)
        || (kind.device_class == "window" && cat.window_words().contains(word))
        || (kind.device_class == "door" && cat.door_nouns().contains(word))
}

fn is_status_question(cat: &Catalog, tokens: &[String], kind: &Kind) -> bool {
    let Some(cue_at) = tokens.iter().position(|word| cue_match(cat, kind, word)) else {
        return false;
    };
    if kind.device_class == "window" && (cat.any(tokens, cat.door_nouns()) || cat.any(tokens, cat.garage_words())) {
        return false;
    }
    if kind.device_class == "door" && cat.any(tokens, cat.garage_words()) {
        return false;
    }
    let command_before = tokens[..cue_at].iter().any(|word| {
        cat.open_words().contains(word.as_str())
            || cat.close_words().contains(word.as_str())
            || CLOSED_SYN.contains(&word.as_str())
            || matches!(word.as_str(), "turn" | "set")
    });
    let question = looks_like_question(tokens)
        || tokens.iter().any(|word| matches!(word.as_str(), "which" | "any" | "there" | "how" | "viele" | "welche"));
    let state_after = tokens[cue_at + 1..].iter().any(|word| {
        cat.open_words().contains(word.as_str())
            || cat.close_words().contains(word.as_str())
            || CLOSED_SYN.contains(&word.as_str())
            || cat.status_words().contains(word.as_str())
    });
    if command_before && !question {
        return false;
    }
    question || state_after || cat.any(tokens, cat.status_words())
}

fn has_leftover_content(cat: &Catalog, tokens: &[String], kind: &Kind, areas: &[String], floors: &[String], home: &HomeGraph) -> bool {
    let article =
        |at: usize| cat.article_one().contains(tokens[at].as_str()) && tokens.get(at + 1).is_some_and(|next| cue_match(cat, kind, next));
    tokens.iter().enumerate().any(|(at, word)| {
        if article(at) || cue_match(cat, kind, word) || padding(word, cat) {
            return false;
        }
        if place_label(home, areas, floors).iter().any(|label| crate::parse::normalize::tokenize(label).contains(word)) {
            return false;
        }
        true
    })
}

fn padding(word: &str, cat: &Catalog) -> bool {
    cat.fillers().contains(word)
        || cat.is_question_word(word)
        || cat.is_question_start(word)
        || cat.open_words().contains(word)
        || cat.close_words().contains(word)
        || cat.all_words().contains(word)
        || cat.status_words().contains(word)
        || cat.or_words().contains(word)
        || cat.sensor_words().contains(word)
        || PLACE_PAD.contains(&word)
        || TIME_PAD.contains(&word)
        || QUANTITY_PAD.contains(&word)
        || CLOSED_SYN.contains(&word)
        || matches!(
            word,
            "any"
                | "there"
                | "which"
                | "what"
                | "do"
                | "does"
                | "did"
                | "i"
                | "have"
                | "has"
                | "got"
                | "we"
                | "our"
                | "opened"
                | "tell"
                | "check"
                | "see"
                | "know"
                | "whether"
                | "about"
                | "show"
                | "get"
                | "level"
                | "reading"
                | "value"
                | "is"
                | "are"
                | "zu"
                | "auf"
                | "oder"
        )
        || report_word(word, cat)
}

fn report_word(word: &str, cat: &Catalog) -> bool {
    cat.moods().any(|mood| {
        [mood.question_starts, mood.subjects, mood.state_verbs, mood.participles, mood.ask_frames, mood.state_padding]
            .iter()
            .any(|list| list.iter().any(|entry| entry.split(' ').any(|part| part == word)))
    })
}

fn has_class_sensor(cat: &Catalog, home: &HomeGraph, area: Option<&str>, kind: &Kind) -> bool {
    home.entities.iter().any(|entity| {
        entity.domain == kind.domain
            && area.is_none_or(|wanted| entity.area.as_deref() == Some(wanted))
            && (home.device_class(&entity.entity_id) == Some(kind.device_class) || name_suggests(cat, entity, kind))
    })
}

fn name_suggests(cat: &Catalog, entity: &EntityRec, kind: &Kind) -> bool {
    let blob = format!("{} {}", entity.entity_id, entity.name).to_lowercase();
    kind.cues.iter().any(|cue| cue.len() >= 4 && blob.contains(cue))
        || (kind.device_class == "window" && cat.window_words().iter().any(|word| word.len() >= 4 && blob.contains(word)))
        || (kind.device_class == "door" && cat.door_nouns().iter().any(|word| word.len() >= 4 && blob.contains(word)))
}

fn has_window_cover(cat: &Catalog, home: &HomeGraph, area: &str) -> bool {
    home.entities
        .iter()
        .any(|entity| entity.domain == "cover" && entity.area.as_deref() == Some(area) && !is_garage_door(cat, home, entity))
}

fn is_garage_door(cat: &Catalog, home: &HomeGraph, entity: &EntityRec) -> bool {
    if matches!(home.device_class(&entity.entity_id), Some("garage" | "gate")) {
        return true;
    }
    let blob = format!("{} {}", entity.entity_id, entity.name).to_lowercase();
    cat.garage_words().iter().any(|word| word.len() >= 4 && blob.contains(word)) || blob.contains("garagentor")
}

fn place_label(home: &HomeGraph, areas: &[String], floors: &[String]) -> Vec<String> {
    let mut labels = Vec::new();
    for area in areas {
        if let Some(record) = home.areas.iter().find(|record| record.area_id == *area) {
            labels.push(record.name.clone());
            labels.push(record.area_id.clone());
            labels.extend(record.aliases.iter().cloned());
        }
    }
    for floor in floors {
        if let Some(record) = home.floor(floor) {
            labels.push(record.name.clone());
            labels.push(record.floor_id.clone());
            labels.extend(record.aliases.iter().cloned());
        }
    }
    labels
}
