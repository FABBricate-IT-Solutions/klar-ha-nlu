//! What a spoken word is to a device command: grammar that names nothing, or the kind of
//! device it asks for. Every other word has to be the name of something in the home.

use super::keys::WordKey;
use super::Catalog;
use crate::lang::verbs::VerbKind;

/// Lists whose words steer a command and name no device.
const GRAMMAR: &[WordKey] = &[
    WordKey::Fillers,
    WordKey::ActionKeep,
    WordKey::Conjunctions,
    WordKey::Particles,
    WordKey::Affirm,
    WordKey::OrWords,
    WordKey::AllWords,
    WordKey::QueryHint,
    WordKey::QuestionStarts,
    WordKey::QuestionWords,
    WordKey::Correction,
    WordKey::ClarifyPick,
    WordKey::PowerWords,
    WordKey::CommandHedges,
    WordKey::OpenWords,
    WordKey::CloseWords,
    WordKey::RollClose,
    WordKey::UnlockFollow,
    WordKey::CoverOpenFollow,
    WordKey::OnWords,
    WordKey::OffWords,
    WordKey::StartWords,
    WordKey::ReplayOnOff,
    WordKey::ReplayOff,
    WordKey::StatusWords,
    WordKey::OpenClose,
    WordKey::ArticleOne,
    WordKey::Illuminate,
    WordKey::Hours,
    WordKey::Minutes,
    WordKey::Seconds,
    WordKey::Left,
    WordKey::Right,
    WordKey::Sides,
];

/// Domains a pack has kind words for.
const KINDS: &[&str] = &["light", "cover", "lock", "climate", "fan", "media_player", "vacuum", "switch", "scene", "script", "timer"];

/// The lists that name a kind of device of `domain`.
fn kind_lists(domain: &str) -> &'static [WordKey] {
    match domain {
        "light" => &[
            WordKey::LightNouns,
            WordKey::LightSingular,
            WordKey::LightPlural,
            WordKey::RoleLight,
            WordKey::LampFixture,
            WordKey::Ceiling,
            WordKey::Pendant,
            WordKey::Bedside,
            WordKey::Island,
            WordKey::Brightness,
        ],
        "cover" => &[WordKey::CoverNouns, WordKey::CurtainNouns, WordKey::WindowWords, WordKey::GarageCover, WordKey::GarageWords],
        "lock" => &[WordKey::LockNouns, WordKey::DoorNouns, WordKey::LockVerbs],
        "climate" => &[WordKey::ClimateNouns, WordKey::RoleClimate, WordKey::ClimateCool, WordKey::ClimateHeat, WordKey::TempQuery],
        "fan" => &[WordKey::FanNouns, WordKey::RoleFan],
        "media_player" => &[WordKey::MediaNouns, WordKey::TvWords, WordKey::RoleMedia],
        "vacuum" => &[WordKey::VacuumNouns],
        "switch" => &[WordKey::SwitchPlural, WordKey::OutletWords, WordKey::BareSwitch, WordKey::LaundryMachines],
        "scene" | "script" => &[WordKey::SceneNouns, WordKey::ScriptWords],
        "timer" => &[WordKey::TimerNouns],
        _ => &[],
    }
}

/// Verb classes that are really the name of a kind of device.
fn kind_verb(kind: VerbKind, domain: &str) -> bool {
    match domain {
        "light" => matches!(kind, VerbKind::Brightness),
        "cover" => matches!(kind, VerbKind::Position | VerbKind::Roll),
        "lock" => matches!(kind, VerbKind::Lock | VerbKind::LockNoun | VerbKind::Unlock | VerbKind::CloseLock),
        "climate" => matches!(kind, VerbKind::Climate | VerbKind::Temperature),
        "fan" => matches!(kind, VerbKind::FanNoun | VerbKind::Speed),
        "media_player" => matches!(kind, VerbKind::Mute | VerbKind::Play | VerbKind::Pause | VerbKind::Playback | VerbKind::Next),
        "vacuum" => matches!(kind, VerbKind::Vacuum | VerbKind::VacuumNoun | VerbKind::Dock),
        "scene" | "script" => matches!(kind, VerbKind::Scene),
        "timer" => matches!(kind, VerbKind::Timer),
        _ => false,
    }
}

impl Catalog {
    /// Every word the packs list for a kind of device of this domain.
    pub fn kind_words<'a>(&'a self, domain: &'a str) -> impl Iterator<Item = &'static str> + 'a {
        let mapped = self.domain_map.iter().filter(move |(_, mapped)| **mapped == domain).map(|(word, _)| *word);
        let listed = kind_lists(domain).iter().flat_map(|key| self.words(*key).iter().copied());
        let verbs = self.verbs.iter().filter(move |(_, kind)| kind_verb(**kind, domain)).map(|(word, _)| *word);
        mapped.chain(listed).chain(verbs)
    }

    /// The word names a kind of device of this domain: "light", "blinds", "heating".
    pub fn names_kind(&self, token: &str, domain: &str) -> bool {
        self.domain_map.get(token).is_some_and(|mapped| *mapped == domain)
            || kind_lists(domain).iter().any(|key| self.words(*key).contains(token))
            || self.verb(token).is_some_and(|kind| kind_verb(kind, domain))
    }

    /// The word is a verb only one kind of device takes: "lock", "play", "dock".
    pub fn acts_on_kind(&self, token: &str) -> bool {
        self.verb(token).is_some_and(|kind| KINDS.iter().any(|domain| kind_verb(kind, domain)))
    }

    /// The word names a kind of device, whatever the domain.
    pub fn names_any_kind(&self, token: &str) -> bool {
        KINDS.iter().any(|domain| self.names_kind(token, domain))
    }

    /// The word stands for what the last turn was about: "it", "them".
    pub fn refers_back(&self, token: &str) -> bool {
        self.moods().any(|mood| mood.refers.contains(&token))
    }

    /// The word steers a command or sets a value and names no device: a verb, an article,
    /// "all", "please", a number, a colour.
    pub fn is_grammar(&self, token: &str) -> bool {
        if GRAMMAR.iter().any(|key| self.words(*key).contains(token)) || self.is_except(token) {
            return true;
        }
        if self.colors.contains_key(token) || self.numbers.contains_key(token) {
            return true;
        }
        if self.verb(token).is_some() && !self.names_any_kind(token) {
            return true;
        }
        if self.strip_pairs.iter().any(|(first, second)| *first == token || *second == token) {
            return true;
        }
        let worded = |list: &[&str]| list.iter().any(|entry| entry.split(' ').any(|word| word == token));
        self.moods().any(|mood| {
            [
                mood.negation,
                mood.exception_leads,
                mood.exception_places,
                mood.question_starts,
                mood.ask_frames,
                mood.request_frames,
                mood.courtesy,
                mood.padding,
                mood.refers,
                mood.subjects,
                mood.state_verbs,
                mood.participles,
                mood.state_padding,
                mood.conditions,
                mood.time_words,
                mood.clock_leads,
                mood.clock_marks,
                mood.measure_units,
                mood.question_openers,
                mood.question_particles,
                mood.question_endings,
            ]
            .iter()
            .any(|list| worded(list))
        }) || self.answers().any(|answers| worded(answers.yes) || worded(answers.no) || worded(answers.cancel))
    }
}
