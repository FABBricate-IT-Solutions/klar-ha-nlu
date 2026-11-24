//! Last gate before a decision leaves the pipeline: only a positive, present-tense command
//! may change something.
//!
//! The sentence is read clause by clause. A condition or a schedule rejects the sentence.
//! A negated clause is dropped together with everything after it. A question or a report
//! becomes a state query on the same target, or a rejection. A command clause next to
//! them still acts, as long as it produces the same step when it is parsed on its own.

use super::context::ParseContext;
use super::draft::{execute, reject, safety_decision, Draft};
use super::mood_plan::{changes_state, controls_device, same_step, sets_value, state_query, switches, PAYLOAD_SLOTS};
use super::pipeline::probe;
use crate::parse::mood::{question_ending, question_mark, Clause, Reading};
use crate::parse::normalize::tokenize;
use crate::parse::respond::speak_unknown;
use crate::parse::split::split_clauses;
use crate::types::{Intent, ParseDecision, RejectReason};

/// Returns the draft to finish with and whether the guard replaced it.
pub(super) fn guard(context: &ParseContext<'_>, draft: Draft) -> (Draft, bool) {
    // "Do it" answers the confirmation. Its sentence was judged when the question was asked.
    if context.probe || draft.safety_confirmed {
        return (draft, false);
    }
    let Some(pending) = Pending::of(&draft) else {
        return (draft, false);
    };
    let sentence = Sentence::read(context, &pending);
    match sentence.verdict(context, &pending) {
        Verdict::Act => (draft, false),
        Verdict::Refuse(reason) => (reject(reason, speak_unknown()), true),
        Verdict::Ask => (pending.as_query(context, draft), true),
        Verdict::Keep(steps) => (rebuild(context, &draft, steps), true),
    }
}

enum Verdict {
    /// Every clause is a command: leave the draft alone.
    Act,
    Refuse(RejectReason),
    /// Nothing may change; answer with the state of what was named.
    Ask,
    /// Only these steps of the plan stay.
    Keep(Vec<Intent>),
}

/// What the draft would do, and where a clarification would aim.
struct Pending {
    /// Every step of the plan, queries included, in plan order.
    steps: Vec<Intent>,
    /// Entity ids and area ids a clarification offers.
    options: Vec<String>,
    clarify: bool,
}

impl Pending {
    fn of(draft: &Draft) -> Option<Self> {
        let (steps, options, clarify) = match &draft.decision {
            ParseDecision::Execute => (draft.plan.as_ref()?.intents(), Vec::new(), false),
            // A confirmation that is only asked again changes nothing by itself.
            ParseDecision::Confirm { .. } => (draft.commit.confirm.as_ref()?.1.intents(), Vec::new(), false),
            ParseDecision::Clarify { .. } => {
                let (options, template) = draft.commit.clarify.as_ref()?;
                (vec![template.clone()], options.clone(), true)
            }
            _ => return None,
        };
        steps.iter().any(|intent| changes_state(&intent.name)).then_some(Self { steps, options, clarify })
    }

    fn changes(&self) -> impl Iterator<Item = &Intent> {
        self.steps.iter().filter(|intent| changes_state(&intent.name))
    }

    /// Names the household gave to what the draft aims at, and its own phrases.
    /// Words inside them are not grammar.
    fn names(&self, context: &ParseContext<'_>) -> Vec<Vec<String>> {
        let home = context.home;
        let mut names: Vec<String> = Vec::new();
        let targets = self
            .steps
            .iter()
            .flat_map(|intent| ["entity_id", "area", "floor"].into_iter().filter_map(|slot| intent.slot(slot).map(str::to_string)));
        for target in targets.chain(self.options.iter().cloned()) {
            if let Some(entity) = home.entities.iter().find(|entity| entity.entity_id == target) {
                names.push(entity.name.clone());
                names.extend(entity.aliases.iter().cloned());
            }
            if let Some(area) = home.areas.iter().find(|area| area.area_id == target) {
                names.push(area.name.clone());
                names.extend(area.aliases.iter().cloned());
            }
            if let Some(floor) = home.floor(&target) {
                names.push(floor.name.clone());
                names.extend(floor.aliases.iter().cloned());
            }
        }
        names.extend(context.custom.iter().map(|custom| custom.phrase.clone()));
        names.extend(context.policies.iter().filter(|rule| rule.enabled).filter_map(|rule| rule.when.phrase.clone()));
        names.iter().map(|name| tokenize(name)).filter(|words| !words.is_empty()).collect()
    }

    fn payloads(&self) -> Vec<Vec<String>> {
        self.changes()
            .flat_map(|intent| intent.slots.iter())
            .filter(|slot| PAYLOAD_SLOTS.contains(&slot.name.as_str()))
            .map(|slot| tokenize(&slot.value))
            .collect()
    }

    fn as_query(&self, context: &ParseContext<'_>, draft: Draft) -> Draft {
        if self.clarify {
            return clarify_as_query(draft);
        }
        let mut queries: Vec<Intent> = Vec::new();
        for intent in &self.steps {
            let query = if changes_state(&intent.name) {
                match state_query(intent, context.home) {
                    Some(query) => query,
                    None => return reject(RejectReason::NoAction, speak_unknown()),
                }
            } else {
                intent.clone()
            };
            if !queries.iter().any(|known| same_step(known, &query)) {
                queries.push(query);
            }
        }
        rebuild(context, &draft, queries)
    }
}

/// The plan again with fewer or other steps, through the same safety decision as any plan.
fn rebuild(context: &ParseContext<'_>, draft: &Draft, steps: Vec<Intent>) -> Draft {
    let clear_pending = draft.commit.clear_pending;
    let rebuilt = safety_decision(execute(context, steps, "mood_guard", draft.confidence, draft.margin, clear_pending, false), context);
    match rebuilt.decision {
        ParseDecision::Execute | ParseDecision::Confirm { .. } => rebuilt,
        _ => reject(RejectReason::NoAction, speak_unknown()),
    }
}

/// The question stays open, but the answer will read the state instead of changing it.
fn clarify_as_query(mut draft: Draft) -> Draft {
    if let Some((_, template)) = draft.commit.clarify.as_mut() {
        let mut query = Intent::new("HassGetState");
        if let Some(domain) = template.slot("domain") {
            query = query.with("domain", domain);
        }
        *template = query;
    }
    draft.output_candidate = None;
    draft
}

/// A clause as a sentence of its own: without the conjunction that tied it to its neighbour.
fn spoken(tokens: &[String], context: &ParseContext<'_>) -> String {
    let joins = |token: &String| context.catalog.conjunctions().contains(token.as_str());
    let start = tokens.iter().position(|token| !joins(token)).unwrap_or(tokens.len());
    let end = tokens.iter().rposition(|token| !joins(token)).map_or(start, |last| last + 1);
    tokens[start..end].join(" ")
}

struct Sentence {
    clauses: Vec<(Vec<String>, Reading)>,
    /// A question mark or a question particle that no clause accounts for.
    asked: bool,
}

impl Sentence {
    fn read(context: &ParseContext<'_>, pending: &Pending) -> Self {
        let names = pending.names(context);
        let payloads = pending.payloads();
        let clauses: Vec<(Vec<String>, Reading)> = split_clauses(&tokenize(context.text), context.home)
            .into_iter()
            .map(|tokens| {
                let mut clause = Clause::new(&tokens);
                names.iter().for_each(|name| clause.mask_name(name));
                payloads.iter().for_each(|words| clause.mask_span(words));
                let reading = clause.read(context.catalog);
                (tokens, reading)
            })
            .collect();
        // "Can you turn on the light?" is a request. A mark behind a clause that already
        // reads as a question belongs to that clause.
        let explained = clauses.iter().any(|(_, reading)| reading.asked || reading.requested);
        let asked = (question_mark(context.text) && !explained) || question_ending(context.text, context.catalog);
        Self { clauses, asked }
    }

    fn verdict(&self, context: &ParseContext<'_>, pending: &Pending) -> Verdict {
        let readings = || self.clauses.iter().map(|(_, reading)| reading);
        if readings().any(|reading| reading.conditional) {
            return Verdict::Refuse(RejectReason::Unsupported);
        }
        if readings().any(|reading| reading.bare_negation) {
            return Verdict::Refuse(RejectReason::NoAction);
        }
        if self.scheduled(context, pending) {
            return Verdict::Refuse(RejectReason::Unsupported);
        }
        let negated_from = readings().position(|reading| reading.negated).unwrap_or(self.clauses.len());
        let closed: Vec<bool> = self.clauses.iter().enumerate().map(|(at, (_, reading))| at >= negated_from || reading.asked).collect();
        if !self.asked && !closed.contains(&true) {
            return Verdict::Act;
        }
        let asks_only = negated_from == self.clauses.len();
        if self.asked || !closed.contains(&false) || pending.clarify {
            return if asks_only { Verdict::Ask } else { Verdict::Refuse(RejectReason::NoAction) };
        }
        // Command clauses stand next to closed ones. A step stays if the command clauses
        // alone produce it; a query stays anyway.
        let open: Vec<String> =
            self.clauses.iter().zip(&closed).filter(|(_, closed)| !**closed).map(|((tokens, _), _)| spoken(tokens, context)).collect();
        let alone = probe(context, &open.join(" "));
        let kept: Vec<Intent> = pending
            .steps
            .iter()
            .filter(|step| !changes_state(&step.name) || alone.intents.iter().any(|found| same_step(found, step)))
            .cloned()
            .collect();
        if kept.len() == pending.steps.len() {
            Verdict::Act
        } else if kept.is_empty() {
            Verdict::Refuse(RejectReason::NoAction)
        } else {
            Verdict::Keep(kept)
        }
    }

    /// A time word puts a device command off to another time, and nothing here can keep
    /// that promise. A clause that only sets a timer or a calendar entry owns its time.
    fn scheduled(&self, context: &ParseContext<'_>, pending: &Pending) -> bool {
        if !pending.changes().any(|intent| controls_device(&intent.name)) {
            return false;
        }
        let cat = context.catalog;
        // "set the heating at 21": the number is the value. "turn on the light at 6": the
        // command is complete without it, so the number is an hour.
        let value = |number: i32, tokens: &[String]| pending.changes().any(|intent| sets_value(intent, number)) && !switches(tokens, cat);
        // A question about a time is a question; it is closed as one.
        let timed = |(tokens, reading): &&(Vec<String>, Reading)| {
            !reading.asked && (reading.timed || reading.clock.is_some_and(|number| !value(number, tokens)))
        };
        if self.clauses.len() == 1 {
            return self.clauses.iter().any(|clause| timed(&clause));
        }
        self.clauses.iter().filter(timed).any(|(tokens, _)| {
            let alone = probe(context, &spoken(tokens, context)).intents;
            alone.is_empty() || alone.iter().any(|intent| controls_device(&intent.name))
        })
    }
}
