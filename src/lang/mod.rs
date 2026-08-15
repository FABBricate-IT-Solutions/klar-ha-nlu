//! Language packs. The engine looks up words here; a new language is a new `*.rs` pack.
//!
//! To add for example French later: copy `en.rs` → `fr.rs`, fill the lists, add `LangId::Fr`,
//! register it in `LangId::pack`, and enable `"fr"` in `Settings.languages`.

mod de;
mod en;
mod pack;
mod verbs;

use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

pub use pack::{GroupClarify, LanguagePack, NumberStyle};
pub use verbs::VerbKind;

use pack::LanguagePack as Pack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LangId {
    De,
    En,
}

impl LangId {
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "de" => Some(Self::De),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::De => "de",
            Self::En => "en",
        }
    }

    pub fn pack(self) -> &'static Pack {
        match self {
            Self::De => &de::PACK,
            Self::En => &en::PACK,
        }
    }

    pub const DEFAULT: [LangId; 2] = [LangId::De, LangId::En];
}

pub struct Catalog {
    pub langs: Vec<LangId>,
    verbs: HashMap<&'static str, VerbKind>,
    fillers: HashSet<&'static str>,
    action_keep: HashSet<&'static str>,
    conjunctions: HashSet<&'static str>,
    particles: HashSet<&'static str>,
    affirm: HashSet<&'static str>,
    or_words: HashSet<&'static str>,
    all_words: HashSet<&'static str>,
    query_hint: HashSet<&'static str>,
    question_starts: HashSet<&'static str>,
    question_words: HashSet<&'static str>,
    pub light_nouns: HashSet<&'static str>,
    pub light_singular: HashSet<&'static str>,
    pub light_plural: HashSet<&'static str>,
    pub cover_nouns: HashSet<&'static str>,
    pub curtain_nouns: HashSet<&'static str>,
    pub fan_nouns: HashSet<&'static str>,
    pub climate_nouns: HashSet<&'static str>,
    pub media_nouns: HashSet<&'static str>,
    pub lock_nouns: HashSet<&'static str>,
    pub door_nouns: HashSet<&'static str>,
    pub garage_words: HashSet<&'static str>,
    pub garage_cover: HashSet<&'static str>,
    pub timer_nouns: HashSet<&'static str>,
    pub list_nouns: HashSet<&'static str>,
    pub vacuum_nouns: HashSet<&'static str>,
    pub scene_nouns: HashSet<&'static str>,
    pub script_words: HashSet<&'static str>,
    pub switch_plural: HashSet<&'static str>,
    pub device_side: HashSet<&'static str>,
    pub named_device: HashSet<&'static str>,
    pub power_words: HashSet<&'static str>,
    pub command_hedges: HashSet<&'static str>,
    pub skip_light: HashSet<&'static str>,
    pub laundry_area: HashSet<&'static str>,
    pub laundry_machines: HashSet<&'static str>,
    pub kitchen: HashSet<&'static str>,
    pub open_words: HashSet<&'static str>,
    pub close_words: HashSet<&'static str>,
    pub unlock_follow: HashSet<&'static str>,
    pub cover_open_follow: HashSet<&'static str>,
    pub garage_lock_block: HashSet<&'static str>,
    pub on_words: HashSet<&'static str>,
    pub off_words: HashSet<&'static str>,
    pub scene_named: HashSet<&'static str>,
    pub temp_query: HashSet<&'static str>,
    pub timer_query: HashSet<&'static str>,
    pub sides: HashSet<&'static str>,
    pub island: HashSet<&'static str>,
    pub ceiling: HashSet<&'static str>,
    pub lamp_fixture: HashSet<&'static str>,
    pub pendant: HashSet<&'static str>,
    pub bedside: HashSet<&'static str>,
    pub left: HashSet<&'static str>,
    pub right: HashSet<&'static str>,
    pub brightness: HashSet<&'static str>,
    pub start_words: HashSet<&'static str>,
    pub replay_on_off: HashSet<&'static str>,
    pub replay_off: HashSet<&'static str>,
    pub domain_map: HashMap<&'static str, &'static str>,
    pub sensor_words: HashSet<&'static str>,
    pub lock_verbs: HashSet<&'static str>,
    pub entry_words: HashSet<&'static str>,
    pub oven: HashSet<&'static str>,
    pub laundry_timer: HashSet<&'static str>,
    colors: HashMap<&'static str, &'static str>,
    numbers: HashMap<&'static str, i32>,
    pub number_styles: Vec<NumberStyle>,
    pub room_index_nouns: HashSet<&'static str>,
    fixture_aliases: HashMap<&'static str, &'static [&'static str]>,
    group_clarify: Vec<GroupClarify>,
    pub singular_lamp: HashSet<&'static str>,
    pub singular_lamp_block: HashSet<&'static str>,
    pub strip_pairs: Vec<(&'static str, &'static str)>,
    pub keep_after: Vec<(&'static [&'static str], &'static str)>,
    pub illuminate: HashSet<&'static str>,
    pub list_down: HashSet<&'static str>,
    pub chores: HashSet<&'static str>,
    pub weak_scene: HashSet<&'static str>,
    pub correction: HashSet<&'static str>,
    pub clarify_pick: HashSet<&'static str>,
}

macro_rules! extend_set {
    ($dst:expr, $src:expr) => {
        $dst.extend($src.iter().copied())
    };
}

impl Catalog {
    fn merge(packs: &[&'static Pack]) -> Self {
        let mut c = Self::empty();
        for p in packs {
            c.langs.push(p.id);
            for &(w, k) in p.verbs {
                c.verbs.insert(w, k);
            }
            extend_set!(c.fillers, p.fillers);
            extend_set!(c.action_keep, p.action_keep);
            extend_set!(c.conjunctions, p.conjunctions);
            extend_set!(c.particles, p.particles);
            extend_set!(c.affirm, p.affirm);
            extend_set!(c.or_words, p.or_words);
            extend_set!(c.all_words, p.all_words);
            extend_set!(c.query_hint, p.query_hint);
            extend_set!(c.question_starts, p.question_starts);
            extend_set!(c.question_words, p.question_words);
            extend_set!(c.light_nouns, p.light_nouns);
            extend_set!(c.light_singular, p.light_singular);
            extend_set!(c.light_plural, p.light_plural);
            extend_set!(c.cover_nouns, p.cover_nouns);
            extend_set!(c.curtain_nouns, p.curtain_nouns);
            extend_set!(c.fan_nouns, p.fan_nouns);
            extend_set!(c.climate_nouns, p.climate_nouns);
            extend_set!(c.media_nouns, p.media_nouns);
            extend_set!(c.lock_nouns, p.lock_nouns);
            extend_set!(c.door_nouns, p.door_nouns);
            extend_set!(c.garage_words, p.garage_words);
            extend_set!(c.garage_cover, p.garage_cover);
            extend_set!(c.timer_nouns, p.timer_nouns);
            extend_set!(c.list_nouns, p.list_nouns);
            extend_set!(c.vacuum_nouns, p.vacuum_nouns);
            extend_set!(c.scene_nouns, p.scene_nouns);
            extend_set!(c.script_words, p.script_words);
            extend_set!(c.switch_plural, p.switch_plural);
            extend_set!(c.device_side, p.device_side);
            extend_set!(c.named_device, p.named_device);
            extend_set!(c.power_words, p.power_words);
            extend_set!(c.command_hedges, p.command_hedges);
            extend_set!(c.skip_light, p.skip_light);
            extend_set!(c.laundry_area, p.laundry_area);
            extend_set!(c.laundry_machines, p.laundry_machines);
            extend_set!(c.kitchen, p.kitchen);
            extend_set!(c.open_words, p.open_words);
            extend_set!(c.close_words, p.close_words);
            extend_set!(c.unlock_follow, p.unlock_follow);
            extend_set!(c.cover_open_follow, p.cover_open_follow);
            extend_set!(c.garage_lock_block, p.garage_lock_block);
            extend_set!(c.on_words, p.on_words);
            extend_set!(c.off_words, p.off_words);
            extend_set!(c.scene_named, p.scene_named);
            extend_set!(c.temp_query, p.temp_query);
            extend_set!(c.timer_query, p.timer_query);
            extend_set!(c.sides, p.sides);
            extend_set!(c.island, p.island);
            extend_set!(c.ceiling, p.ceiling);
            extend_set!(c.lamp_fixture, p.lamp_fixture);
            extend_set!(c.pendant, p.pendant);
            extend_set!(c.bedside, p.bedside);
            extend_set!(c.left, p.left);
            extend_set!(c.right, p.right);
            extend_set!(c.brightness, p.brightness);
            extend_set!(c.start_words, p.start_words);
            extend_set!(c.replay_on_off, p.replay_on_off);
            extend_set!(c.replay_off, p.replay_off);
            for &(w, d) in p.domain_map {
                c.domain_map.insert(w, d);
            }
            extend_set!(c.sensor_words, p.sensor_words);
            extend_set!(c.lock_verbs, p.lock_verbs);
            extend_set!(c.entry_words, p.entry_words);
            extend_set!(c.oven, p.oven);
            extend_set!(c.laundry_timer, p.laundry_timer);
            for &(w, color) in p.colors {
                c.colors.insert(w, color);
            }
            for &(w, n) in p.numbers {
                c.numbers.insert(w, n);
            }
            c.number_styles.push(p.number_style);
            extend_set!(c.room_index_nouns, p.room_index_nouns);
            for &(w, aliases) in p.fixture_aliases {
                c.fixture_aliases.insert(w, aliases);
            }
            if let Some(g) = &p.group_clarify {
                c.group_clarify.push(GroupClarify { trigger: g.trigger, pairs: g.pairs, triples: g.triples });
            }
            extend_set!(c.singular_lamp, p.singular_lamp);
            extend_set!(c.singular_lamp_block, p.singular_lamp_block);
            c.strip_pairs.extend(p.strip_pairs.iter().copied());
            c.keep_after.extend(p.keep_after.iter().copied());
            extend_set!(c.illuminate, p.illuminate);
            extend_set!(c.list_down, p.list_down);
            extend_set!(c.chores, p.chores);
            extend_set!(c.weak_scene, p.weak_scene);
            extend_set!(c.correction, p.correction);
            extend_set!(c.clarify_pick, p.clarify_pick);
        }
        c
    }

    fn empty() -> Self {
        Self {
            langs: Vec::new(),
            verbs: HashMap::new(),
            fillers: HashSet::new(),
            action_keep: HashSet::new(),
            conjunctions: HashSet::new(),
            particles: HashSet::new(),
            affirm: HashSet::new(),
            or_words: HashSet::new(),
            all_words: HashSet::new(),
            query_hint: HashSet::new(),
            question_starts: HashSet::new(),
            question_words: HashSet::new(),
            light_nouns: HashSet::new(),
            light_singular: HashSet::new(),
            light_plural: HashSet::new(),
            cover_nouns: HashSet::new(),
            curtain_nouns: HashSet::new(),
            fan_nouns: HashSet::new(),
            climate_nouns: HashSet::new(),
            media_nouns: HashSet::new(),
            lock_nouns: HashSet::new(),
            door_nouns: HashSet::new(),
            garage_words: HashSet::new(),
            garage_cover: HashSet::new(),
            timer_nouns: HashSet::new(),
            list_nouns: HashSet::new(),
            vacuum_nouns: HashSet::new(),
            scene_nouns: HashSet::new(),
            script_words: HashSet::new(),
            switch_plural: HashSet::new(),
            device_side: HashSet::new(),
            named_device: HashSet::new(),
            power_words: HashSet::new(),
            command_hedges: HashSet::new(),
            skip_light: HashSet::new(),
            laundry_area: HashSet::new(),
            laundry_machines: HashSet::new(),
            kitchen: HashSet::new(),
            open_words: HashSet::new(),
            close_words: HashSet::new(),
            unlock_follow: HashSet::new(),
            cover_open_follow: HashSet::new(),
            garage_lock_block: HashSet::new(),
            on_words: HashSet::new(),
            off_words: HashSet::new(),
            scene_named: HashSet::new(),
            temp_query: HashSet::new(),
            timer_query: HashSet::new(),
            sides: HashSet::new(),
            island: HashSet::new(),
            ceiling: HashSet::new(),
            lamp_fixture: HashSet::new(),
            pendant: HashSet::new(),
            bedside: HashSet::new(),
            left: HashSet::new(),
            right: HashSet::new(),
            brightness: HashSet::new(),
            start_words: HashSet::new(),
            replay_on_off: HashSet::new(),
            replay_off: HashSet::new(),
            domain_map: HashMap::new(),
            sensor_words: HashSet::new(),
            lock_verbs: HashSet::new(),
            entry_words: HashSet::new(),
            oven: HashSet::new(),
            laundry_timer: HashSet::new(),
            colors: HashMap::new(),
            numbers: HashMap::new(),
            number_styles: Vec::new(),
            room_index_nouns: HashSet::new(),
            fixture_aliases: HashMap::new(),
            group_clarify: Vec::new(),
            singular_lamp: HashSet::new(),
            singular_lamp_block: HashSet::new(),
            strip_pairs: Vec::new(),
            keep_after: Vec::new(),
            illuminate: HashSet::new(),
            list_down: HashSet::new(),
            chores: HashSet::new(),
            weak_scene: HashSet::new(),
            correction: HashSet::new(),
            clarify_pick: HashSet::new(),
        }
    }

    pub fn verb(&self, t: &str) -> Option<VerbKind> {
        self.verbs.get(t).copied()
    }

    pub fn is_filler(&self, t: &str) -> bool {
        self.fillers.contains(t)
    }

    pub fn is_action_keep(&self, t: &str) -> bool {
        self.action_keep.contains(t)
    }

    pub fn is_conj(&self, t: &str) -> bool {
        self.conjunctions.contains(t)
    }

    pub fn is_particle(&self, t: &str) -> bool {
        self.particles.contains(t)
    }

    pub fn is_affirm(&self, t: &str) -> bool {
        self.affirm.contains(t)
    }

    pub fn is_or(&self, t: &str) -> bool {
        self.or_words.contains(t)
    }

    pub fn is_all(&self, t: &str) -> bool {
        self.all_words.contains(t)
    }

    pub fn is_query_hint(&self, t: &str) -> bool {
        self.query_hint.contains(t)
    }

    pub fn any(&self, tokens: &[String], set: &HashSet<&'static str>) -> bool {
        tokens.iter().any(|t| set.contains(t.as_str()))
    }

    pub fn color(&self, t: &str) -> Option<&'static str> {
        self.colors.get(t).copied()
    }

    pub fn number(&self, t: &str) -> Option<i32> {
        self.numbers.get(t).copied()
    }

    pub fn fixture_alias(&self, t: &str) -> &[&str] {
        self.fixture_aliases.get(t).copied().unwrap_or(&[])
    }

    pub fn wants_group_clarify(&self, raw: &[String]) -> bool {
        self.group_clarify.iter().any(|g| g.matches(raw))
    }

    pub fn wants_singular_lamp(&self, tokens: &[String]) -> bool {
        self.any(tokens, &self.singular_lamp) && !self.any(tokens, &self.singular_lamp_block)
    }

    pub fn is_question_start(&self, t: &str) -> bool {
        self.question_starts.contains(t)
    }

    pub fn is_question_word(&self, t: &str) -> bool {
        self.question_words.contains(t)
    }

    pub fn codes(&self) -> Vec<&'static str> {
        self.langs.iter().map(|l| l.code()).collect()
    }

    pub fn has_german_und(&self) -> bool {
        self.number_styles.contains(&NumberStyle::GermanUnd)
    }

    pub fn has_english_tens(&self) -> bool {
        self.number_styles.contains(&NumberStyle::EnglishTens)
    }
}

fn default_catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(|| Catalog::merge(&[LangId::De.pack(), LangId::En.pack()]))
}

fn cache() -> &'static Mutex<HashMap<String, &'static Catalog>> {
    static C: OnceLock<Mutex<HashMap<String, &'static Catalog>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn catalog_for(codes: &[String]) -> &'static Catalog {
    let mut ids: Vec<LangId> = codes.iter().filter_map(|c| LangId::from_code(c)).collect();
    if ids.is_empty() {
        return default_catalog();
    }
    ids.sort_by_key(|l| l.code());
    ids.dedup();
    if ids.as_slice() == LangId::DEFAULT {
        return default_catalog();
    }
    let key = ids.iter().map(|l| l.code()).collect::<Vec<_>>().join(",");
    let mut map = cache().lock().expect("lang catalog cache");
    *map.entry(key).or_insert_with(|| {
        let packs: Vec<&'static Pack> = ids.iter().map(|id| id.pack()).collect();
        Box::leak(Box::new(Catalog::merge(&packs)))
    })
}

thread_local! {
    static CURRENT: Cell<Option<&'static Catalog>> = const { Cell::new(None) };
}

pub fn catalog() -> &'static Catalog {
    CURRENT.with(|c| c.get()).unwrap_or_else(default_catalog)
}

/// Language of spoken replies: the first bound pack (`en` when Assist pins English).
pub fn speech_lang() -> LangId {
    catalog().langs.first().copied().unwrap_or(LangId::De)
}

pub struct CatalogBind {
    prev: Option<&'static Catalog>,
}

impl Drop for CatalogBind {
    fn drop(&mut self) {
        CURRENT.with(|c| c.set(self.prev));
    }
}

/// Bind packs for this parse. Helpers read `catalog()` while the guard lives.
pub fn bind(lang_codes: &[String]) -> CatalogBind {
    let cat = catalog_for(lang_codes);
    let prev = CURRENT.with(|c| {
        let prev = c.get();
        c.set(Some(cat));
        prev
    });
    CatalogBind { prev }
}
