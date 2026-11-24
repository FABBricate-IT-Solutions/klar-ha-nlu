//! What a parse would change in the home: the steps that run now or after a "yes", and the
//! devices each step reaches once Home Assistant expands rooms, floors and domains.
use klar_nlu::session::Session;
use klar_nlu::types::{HomeGraph, Intent, ParseDecision, ParseOutcome};

/// Intents that only read, plus reminders: a reminder notes something down for later and
/// switches no device.
const HARMLESS: &[&str] = &[
    "HassGetState",
    "HassClimateGetTemperature",
    "HassTimerStatus",
    "MassGetQueue",
    "KlarGetCalendarEvents",
    "KlarNoMusicPlayer",
    "SetReminderRelative",
    "SetReminderAbsolute",
    "SetReminderDaypart",
];

pub fn changes_state(intent: &Intent) -> bool {
    !HARMLESS.contains(&intent.name.as_str())
}

/// The state-changing steps the engine committed to: an executed plan, the plan that is
/// waiting for the user's "yes", or the step a clarifying question would run once the user
/// picks one of its options. A confirmation or a question counts, because one more word
/// then runs it.
pub fn intended_changes(out: &ParseOutcome, session: &Session, home: &HomeGraph) -> Vec<Intent> {
    let steps = match &out.decision {
        ParseDecision::Execute => out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default(),
        ParseDecision::Confirm { .. } => session.pending_confirm().map(|pending| pending.plan.intents()).unwrap_or_default(),
        ParseDecision::Clarify { .. } => {
            session.pending_clarify().map(|pending| answers(&pending.template, &pending.options, home)).unwrap_or_default()
        }
        ParseDecision::Reject { .. } | ParseDecision::Chat | ParseDecision::Error { .. } => Vec::new(),
    };
    steps.into_iter().filter(changes_state).collect()
}

/// One step per option of a clarifying question: the template aimed at that device or room.
fn answers(template: &Intent, options: &[String], home: &HomeGraph) -> Vec<Intent> {
    if options.is_empty() {
        return vec![template.clone()];
    }
    let slot = |option: &String| if home.areas.iter().any(|area| &area.area_id == option) { "area" } else { "entity_id" };
    options.iter().map(|option| template.clone().with_set(slot(option), option)).collect()
}

/// Every exposed device a step reaches. A step that names a device reaches that device. A
/// step that names a room or a floor reaches everything there, narrowed by the domain only
/// if the step carries one; that is how Home Assistant matches a room-wide intent.
pub fn reach(intent: &Intent, home: &HomeGraph) -> Vec<String> {
    if let Some(id) = intent.slot("entity_id") {
        return vec![id.to_string()];
    }
    let (area, floor, domain) = (intent.slot("area"), intent.slot("floor"), intent.slot("domain"));
    if area.is_none() && floor.is_none() && domain.is_none() {
        return Vec::new();
    }
    let on_floor = |area_id: &str, floor_id: &str| home.areas_on_floor(floor_id).any(|rec| rec.area_id == area_id);
    home.entities
        .iter()
        .filter(|e| home.assist.as_ref().is_none_or(|ids| ids.contains(&e.entity_id)))
        .filter(|e| area.is_none_or(|wanted| e.area.as_deref() == Some(wanted)))
        .filter(|e| floor.is_none_or(|wanted| e.area.as_deref().is_some_and(|area_id| on_floor(area_id, wanted))))
        .filter(|e| domain.is_none_or(|wanted| e.domain == wanted))
        .map(|e| e.entity_id.clone())
        .collect()
}

pub fn show(intent: &Intent) -> String {
    let slots: Vec<String> = intent.slots.iter().map(|slot| format!("{}={}", slot.name, slot.value)).collect();
    format!("{}{{{}}}", intent.name, slots.join(","))
}
