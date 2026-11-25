//! A device changes only if the sentence asked for it.
//!
//! Every word of a device command is grammar, a value, the kind of device, or part of the
//! name of something the plan aims at. A word that is none of these is foreign: it names a
//! device the home does not have, a second target or a second command. A word for another
//! kind of device ("blinds" when the plan switches lights) is always foreign. A sentence
//! without a foreign word runs as planned. With one, three rules decide:
//!
//! * The target of every step that switches a device must have been said: its kind, the
//!   pack's word for the fixture, a word of its own name that is not the name of its room,
//!   or "it" when the last turn was about it. "Turn off the kitchen" switches the kitchen
//!   lights; "turn off the spaceship in the kitchen" does not.
//! * Every clause and every part of a list ("… and …") that carries a foreign word must
//!   also carry something the plan aims at. "The hall light and the spaceship" changes
//!   nothing.
//! * A part of a list with a verb only another kind of device takes ("lock", "play") and
//!   no kind word of the plan is a command that was dropped.
//!
//! A step that takes dictated text (a list item, a song, a timer name) excuses one such
//! part: its words are content. Scenes and scripts are not judged here; they are matched
//! on their whole name where they are found. A plan the user confirmed is not read again.

use super::context::ParseContext;
use super::draft::{reject, Draft};
use super::mood_plan::changes_state;
use super::reach::reach;
use crate::home::roles::{matches_domain, tv_asked};
use crate::lang::Catalog;
use crate::parse::action::{has_climate_noun, has_light_noun};
use crate::parse::fuzzy::{evidence, Profile};
use crate::parse::infer::{fixture_matches, light_level};
use crate::parse::normalize::{compact, is_script_unit, tokenize};
use crate::parse::resolve::{domain_hint, token_eq};
use crate::parse::respond::speak_unknown;
use crate::types::{EntityRec, HomeGraph, Intent, ParseDecision, RejectReason};
use strsim::levenshtein;

/// Slots that say where a step aims. Every other slot carries a value.
const TARGET_SLOTS: &[&str] = &["entity_id", "area", "floor", "domain"];
/// Domains a kind word can stand for.
const KIND_DOMAINS: &[&str] = &["light", "cover", "lock", "climate", "fan", "media_player", "vacuum", "switch", "scene", "script"];
/// The longest ending a name may carry and still be that name ("kuchni", "salonissa").
const NAME_ENDING: usize = 5;
/// The longest ending of a kind word; packs list most of them in one form only
/// ("isik" for "isiklarini").
const KIND_ENDING: usize = 7;
/// The same for a word of three letters ("luz", "lys").
const SHORT_ENDING: usize = 3;
/// Steps that switch, move or set a device and take no dictated text. Music and the
/// television take free words ("the next song", "louder"), so they are not judged.
const SWITCHING: &[&str] = &[
    "HassTurnOn",
    "HassTurnOff",
    "HassToggle",
    "HassLightSet",
    "HassClimateSetTemperature",
    "HassSetPosition",
    "HassOpenCover",
    "HassCloseCover",
    "HassLock",
    "HassUnlock",
    "HassFanSetSpeed",
    "HassFanSetPercentage",
    "HassVacuumStart",
    "HassVacuumReturnToBase",
];

pub(super) fn guard(context: &ParseContext<'_>, tokens: &[String], clauses: &[Vec<String>], draft: Draft) -> Draft {
    if draft.safety_confirmed {
        return draft;
    }
    let Some(steps) = steps_of(&draft, context.home) else {
        return draft;
    };
    let Some(plan) = Plan::of(&steps, context) else {
        return draft;
    };
    if plan.heard(tokens, clauses) {
        return draft;
    }
    let mut refused = reject(RejectReason::NoTarget, speak_unknown());
    refused.commit.briefing = Some(false);
    refused.commit.clear_pending = draft.commit.clear_pending;
    refused
}

/// The steps a draft would run now, after a "yes", or once the user picks an option.
fn steps_of(draft: &Draft, home: &HomeGraph) -> Option<Vec<Intent>> {
    match &draft.decision {
        ParseDecision::Execute => Some(draft.plan.as_ref()?.intents()),
        ParseDecision::Confirm { .. } => Some(draft.commit.confirm.as_ref()?.1.intents()),
        ParseDecision::Clarify { .. } => {
            let (options, template) = draft.commit.clarify.as_ref()?;
            let slot = |option: &String| if home.areas.iter().any(|area| &area.area_id == option) { "area" } else { "entity_id" };
            (!options.is_empty()).then(|| options.iter().map(|option| template.clone().with_set(slot(option), option)).collect())
        }
        ParseDecision::Reject { .. } | ParseDecision::Chat | ParseDecision::Error { .. } => None,
    }
}

/// One step and the devices it reaches.
struct Aim<'a> {
    targets: Vec<&'a EntityRec>,
    /// The domain a room-wide step names.
    domain: Option<String>,
    /// A named device; a room-wide step has none.
    device: Option<&'a EntityRec>,
    /// The step switches a device, so its target must have been said.
    judged: bool,
    /// The last turn was about this target, so "it" and "them" stand for it.
    recalled: bool,
}

struct Plan<'a> {
    cat: &'static Catalog,
    aims: Vec<Aim<'a>>,
    /// Steps whose words are content: a list entry, a song, the weather.
    dictated: usize,
    /// The sentence as it was spoken. "It" is a filler to the parser and is gone from the
    /// words it works on.
    spoken: Vec<String>,
    /// Folded words of every name the plan aims at: devices, their rooms and floors.
    words: Vec<String>,
    /// The words among them that name a room or a floor.
    places: Vec<String>,
    /// The same names as one compact word each ("livingroom").
    wholes: Vec<String>,
    /// Compact values the steps set ("50", "warmwhite").
    values: Vec<String>,
}

impl<'a> Plan<'a> {
    fn of(steps: &[Intent], context: &ParseContext<'a>) -> Option<Self> {
        let (home, cat, session) = (context.home, context.catalog, context.session);
        let mut places: Vec<&str> = Vec::new();
        let mut names: Vec<&str> = Vec::new();
        let mut aims = Vec::new();
        let mut dictated = 0;
        for step in steps {
            let device = step.slot("entity_id").and_then(|id| home.entities.iter().find(|entity| entity.entity_id == id));
            let rooms = device.and_then(|entity| entity.area.as_deref()).into_iter().chain(step.slot("area"));
            for room in rooms {
                if let Some(area) = home.areas.iter().find(|area| area.area_id == room) {
                    places.extend([area.name.as_str(), area.area_id.as_str()]);
                    places.extend(area.aliases.iter().map(String::as_str));
                    places.extend(area.floor_id.as_deref().and_then(|id| home.floor(id)).map(|floor| floor.name.as_str()));
                }
            }
            if let Some(floor) = step.slot("floor").and_then(|id| home.floor(id)) {
                places.extend([floor.name.as_str(), floor.floor_id.as_str()]);
                places.extend(floor.aliases.iter().map(String::as_str));
            }
            names.extend(device.into_iter().flat_map(own_labels));
            let switching = SWITCHING.contains(&step.name.as_str());
            let targets = reach(step, home);
            if !switching && (changes_state(&step.name) || targets.is_empty()) {
                dictated += 1;
            }
            // A scene or a script is found by its whole name; it needs no second look here.
            let routine = device.is_some_and(|entity| matches!(entity.domain.as_str(), "scene" | "script"));
            let recalled = match (device, step.slot("area")) {
                (Some(entity), _) => session.last_entities().any(|id| id == entity.entity_id),
                (None, Some(room)) => session.last_areas().any(|id| id == room),
                (None, None) => false,
            };
            let domain = step.slot("domain").map(str::to_string);
            aims.push(Aim { targets, domain, device, judged: switching && !routine, recalled });
        }
        if !aims.iter().any(|aim| aim.judged) {
            return None;
        }
        let split = |labels: &[&str]| labels.iter().flat_map(|label| tokenize(&label.replace('_', " "))).collect::<Vec<String>>();
        let places_said = split(&places);
        let words = [places_said.clone(), split(&names)].concat();
        let wholes = places.iter().chain(&names).map(|label| compact(label)).filter(|whole| !whole.is_empty()).collect();
        let values = steps
            .iter()
            .flat_map(|step| step.slots.iter())
            .filter(|slot| !TARGET_SLOTS.contains(&slot.name.as_str()))
            .map(|slot| compact(&slot.value))
            .collect();
        Some(Self { cat, aims, dictated, spoken: tokenize(context.text), words, places: places_said, wholes, values })
    }

    fn heard(&self, tokens: &[String], clauses: &[Vec<String>]) -> bool {
        if !(0..tokens.len()).any(|at| self.foreign(tokens, at)) {
            return true;
        }
        if self.aims.iter().any(|aim| aim.judged && !self.said(aim, tokens)) {
            return false;
        }
        // Each command of the sentence, and each part of a list in it, stands for
        // something the plan does. Dictated steps account for one part each.
        let joins = |token: &String| (self.cat.is_conj(token) || self.cat.is_or(token)) && !self.named(token);
        let parts: Vec<&[String]> = clauses.iter().flat_map(|clause| clause.split(joins)).filter(|part| !part.is_empty()).collect();
        parts.iter().filter(|part| !self.part_heard(part, parts.len() > 1)).count() <= self.dictated
    }

    fn part_heard(&self, part: &[String], listed: bool) -> bool {
        let kinded = part.iter().any(|token| self.kind(token, None));
        let anchored =
            kinded || part.iter().any(|token| self.named(token)) || (!listed && self.aims.iter().any(|aim| self.referred(aim, part)));
        let foreign = (0..part.len()).any(|at| self.foreign(part, at));
        // "… and lock the door" with no lock in the plan is a command that was dropped.
        let other = listed && part.iter().any(|token| !self.named(token) && !self.kind(token, None) && self.cat.acts_on_kind(token));
        (anchored || !foreign) && (kinded || !other)
    }

    /// The sentence names the device itself or its kind, or points back at it, so the
    /// target was not guessed.
    fn said(&self, aim: &Aim<'_>, tokens: &[String]) -> bool {
        tokens.iter().any(|token| self.kind(token, Some(aim)) || self.fixture(token, aim) || self.own(token, aim))
            || self.referred(aim, tokens)
    }

    /// The last turn was about this target and the sentence points back at it: every
    /// foreign word stands next to "it" or "them", as the verb of "nudge it up" does. A
    /// word for another kind of device is not such a verb.
    fn referred(&self, aim: &Aim<'_>, tokens: &[String]) -> bool {
        let points = |pair: &[String], word: &String| {
            (pair[0] == *word && self.cat.refers_back(&pair[1])) || (pair[1] == *word && self.cat.refers_back(&pair[0]))
        };
        let beside = |word: &String| !self.cat.names_any_kind(word) && self.spoken.windows(2).any(|pair| points(pair, word));
        aim.judged && aim.recalled && (0..tokens.len()).filter(|at| self.foreign(tokens, *at)).all(|at| beside(&tokens[at]))
    }

    /// A word of the device's own name. The name of its room does not count: the room
    /// alone picks no device.
    fn own(&self, token: &str, aim: &Aim<'_>) -> bool {
        let Some(entity) = aim.device else {
            return false;
        };
        let place = self.places.iter().any(|word| same(token, word));
        let word = own_labels(entity)
            .any(|label| compact(label) == token || tokenize(&label.replace('_', " ")).iter().any(|word| same(token, word)));
        word && !place && !self.cat.is_grammar(token)
    }

    /// A kind word for one aim, or for any aim of the plan.
    fn kind(&self, token: &str, aim: Option<&Aim<'_>>) -> bool {
        let fits = |aim: &Aim<'_>| {
            aim.domain.as_deref().is_some_and(|domain| self.kind_of(token, domain))
                || KIND_DOMAINS.iter().any(|domain| {
                    aim.targets.iter().any(|entity| entity.domain == *domain || matches_domain(entity, domain, self.cat))
                        && self.kind_of(token, domain)
                })
        };
        match aim {
            Some(aim) => fits(aim),
            None => self.aims.iter().any(fits),
        }
    }

    /// The word stands for a device of `domain`: a pack word, one the engine reads by
    /// itself ("hue", "tv"), or such a word with an ending or a slip of the tongue.
    fn kind_of(&self, token: &str, domain: &str) -> bool {
        let owned = token.to_string();
        let word = std::slice::from_ref(&owned);
        self.cat.names_kind(token, domain)
            || domain_hint(word) == Some(domain)
            || (domain == "light" && has_light_noun(word))
            || (domain == "climate" && has_climate_noun(word))
            || (domain == "media_player" && tv_asked(word))
            || (!self.cat.is_grammar(token)
                && self.cat.kind_words(domain).any(|kind| same(token, kind) || inflected(token, kind, KIND_ENDING)))
    }

    /// The word is part of a name the plan aims at.
    fn named(&self, token: &str) -> bool {
        self.words.iter().any(|word| same(token, word))
            || self.wholes.iter().any(|whole| whole == token || (token.chars().count() >= 4 && whole.contains(token)))
            || self.aims.iter().any(|aim| self.fixture(token, aim))
    }

    /// The pack's word for a fixture ("bola" for a lamp called "Kugel") that this step reaches.
    fn fixture(&self, token: &str, aim: &Aim<'_>) -> bool {
        !self.cat.fixture_alias(token).is_empty() && aim.targets.iter().any(|entity| fixture_matches(entity, token))
    }

    /// The word, or it and its neighbour, is a value the plan sets.
    fn value(&self, tokens: &[String], at: usize) -> bool {
        let token = &tokens[at];
        if token.chars().any(|c| c.is_ascii_digit()) || self.values.iter().any(|value| value == token) {
            return true;
        }
        // Levels the engine reads by itself: "hell", "full", "medium".
        if light_level(std::slice::from_ref(token)).is_some() {
            return true;
        }
        let pair = |left: usize| tokens.get(left + 1).is_some_and(|next| self.values.contains(&format!("{}{next}", tokens[left])));
        pair(at) || (at > 0 && pair(at - 1))
    }

    /// The word names something the plan does not do: a word for another kind of device,
    /// or a word no list knows.
    fn foreign(&self, tokens: &[String], at: usize) -> bool {
        if self.known(tokens, at) {
            return false;
        }
        let token = &tokens[at];
        if self.cat.names_any_kind(token) {
            return true;
        }
        if !sign(token) {
            return !self.cat.is_grammar(token);
        }
        // A script written without spaces is cut into words by the pack's own lists, and a
        // word the pack does not list falls apart into its signs. One sign alone is a
        // particle or an ending; signs in a row are a word.
        let loose = |at: usize| sign(&tokens[at]) && !self.known(tokens, at) && !self.cat.is_grammar(&tokens[at]);
        loose(at) && ((at > 0 && loose(at - 1)) || (at + 1 < tokens.len() && loose(at + 1)))
    }

    /// The word belongs to what the plan does: a name, a value or the kind of device.
    fn known(&self, tokens: &[String], at: usize) -> bool {
        self.named(&tokens[at]) || self.value(tokens, at) || self.kind(&tokens[at], None)
    }
}

/// One sign of a script that is written without spaces.
fn sign(token: &str) -> bool {
    token.chars().count() == 1 && token.chars().all(is_script_unit)
}

/// The name of a device, its other names and the name in its id.
fn own_labels(entity: &EntityRec) -> impl Iterator<Item = &str> {
    let tail = entity.entity_id.split_once('.').map(|(_, tail)| tail);
    std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str)).chain(tail)
}

/// The spoken word is this word of a name, give or take an ending or a slip of the tongue.
fn same(token: &str, word: &str) -> bool {
    token_eq(token, word) || evidence(token, word, Profile::Target).is_some() || inflected(token, word, NAME_ENDING)
}

/// "lampat" for "lampa", "luces" for "luz", "ligt" for "light": the word with its last
/// letter changed and an ending of at most `ending` letters, or one letter off.
fn inflected(token: &str, word: &str, ending: usize) -> bool {
    let (spoken, known) = (token.chars().count(), word.chars().count());
    if known < 3 || spoken < 3 {
        return false;
    }
    let stem: String = word.chars().take(known - 1).collect();
    let ending = if known == 3 { SHORT_ENDING } else { ending };
    (token.starts_with(&stem) && spoken < known + ending) || (known >= 4 && levenshtein(token, word) <= 1)
}
