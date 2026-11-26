//! Language packs. Built-in locales are compiled; assist phrase fixes are YAML data.

mod assist_data;
#[allow(dead_code)]
mod assist_embed;
mod catalog;
mod cli;
mod external;
mod fuzzy;
mod groups;
mod hassil;
mod locale;
mod mood;
mod morphology;
mod pack;
#[allow(clippy::pedantic, clippy::nursery)]
mod packs;
mod registry;
mod resolver;
mod speech;
mod tier;
mod user;
mod validate;
mod verbs;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock};

use parking_lot::Mutex;

pub use catalog::Catalog;
use catalog::Layer;
pub use cli::{import_hassil, new_case, preview, validate_path};
pub use external::{ExternalPack, PackRegistry};
pub use hassil::{import_hassil as parse_hassil, HassilImport};
pub use locale::{LocaleError, LocaleId};
pub use mood::MoodWords;
pub use morphology::Morphology;
pub use pack::{Answers, GroupClarify, Household, LanguagePack, NumberStyle};
pub use registry::{languages, LangMeta};
pub use resolver::{
    bind_preview_user, install_runtime_packs, install_user_overlay, installed_user_overlay, is_known, load_runtime_dir, pin_language,
    reset_runtime_packs,
};
pub use speech::Speech;
pub use tier::{check_selection, check_settings, in_use, offered, offered_as, tier_of, Tier};
pub use user::{
    push_revision, revision_hash, select_revision, validate_custom, validate_language, LanguageOverlay, LanguageRevision, OverlayIssue,
    SetDelta, MAX_HISTORY, MAX_USER_INTENTS,
};
pub use validate::{is_lexicon_path, lexicon_set_paths, validate_pack, ValidationReport};
pub use verbs::VerbKind;

use pack::LanguagePack as Pack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LangId {
    code: &'static str,
}

impl LangId {
    #[allow(non_upper_case_globals)]
    pub const De: Self = Self { code: "de" };
    #[allow(non_upper_case_globals)]
    pub const En: Self = Self { code: "en" };

    pub const fn new(code: &'static str) -> Self {
        Self { code }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        registry::lookup(code)
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        resolver::builtin_for(tag)
    }

    pub fn code(self) -> &'static str {
        self.code
    }

    pub fn pack(self) -> &'static Pack {
        registry::pack(self)
    }

    pub fn all() -> &'static [Self] {
        registry::all_ids()
    }

    pub fn meta(self) -> Option<&'static registry::LangMeta> {
        registry::meta(self)
    }

    /// The pack a regional pack inherits from ("de" for "de-AT").
    pub fn parent(self) -> Option<Self> {
        registry::parent(self)
    }

    pub fn compiled_codes() -> Vec<String> {
        Self::all().iter().map(|id| id.code().to_string()).collect()
    }
}

fn empty_catalog() -> Arc<Catalog> {
    static C: OnceLock<Arc<Catalog>> = OnceLock::new();
    C.get_or_init(|| Arc::new(Catalog::merge(&[]))).clone()
}

const MAX_CACHED_CATALOGS: usize = 32;

/// Bounded LRU of shared catalogs. Evicted entries drop when no request still holds the Arc.
struct CatalogLru {
    map: HashMap<String, Arc<Catalog>>,
    order: VecDeque<String>,
}

impl CatalogLru {
    fn new() -> Self {
        Self { map: HashMap::new(), order: VecDeque::new() }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn len(&self) -> usize {
        self.map.len()
    }

    fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    fn get(&mut self, key: &str) -> Option<Arc<Catalog>> {
        if !self.map.contains_key(key) {
            return None;
        }
        self.touch(key);
        self.map.get(key).cloned()
    }

    fn insert(&mut self, key: String, catalog: Arc<Catalog>) -> Arc<Catalog> {
        if self.map.contains_key(&key) {
            self.touch(&key);
            return self.map.get(&key).cloned().expect("key present");
        }
        while self.map.len() >= MAX_CACHED_CATALOGS {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            } else {
                break;
            }
        }
        self.order.push_back(key.clone());
        self.map.insert(key, catalog.clone());
        catalog
    }

    fn touch(&mut self, key: &str) {
        if let Some(pos) = self.order.iter().position(|entry| entry == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key.to_string());
    }
}

fn cache() -> &'static Mutex<CatalogLru> {
    static C: OnceLock<Mutex<CatalogLru>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(CatalogLru::new()))
}

pub(super) fn clear_catalog_cache() {
    cache().lock().clear();
}

pub fn catalog_for(codes: &[String]) -> Arc<Catalog> {
    let mut ids: Vec<LangId> = Vec::new();
    for code in codes {
        if let Some(id) = LangId::from_tag(code) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    let overlays: Vec<ExternalPack> = codes.iter().filter_map(|code| resolver::overlay_for(code)).collect();
    if ids.is_empty() {
        if let Some(base) = overlays.first().and_then(|pack| pack.base_lang().ok()) {
            ids.push(base);
        }
    }
    if ids.is_empty() {
        return empty_catalog();
    }
    // Keep settings / request order (R-21). Do not sort alphabetically: that made
    // bilingual homes always answer in the alphabetically first pack.
    if ids.len() == LangId::all().len() {
        return empty_catalog();
    }
    let user = resolver::effective_user_overlay();
    if overlays.is_empty() && user.is_none() {
        let builtin_key = ids.iter().map(|lang| lang.code()).collect::<Vec<_>>().join(",");
        return cached_builtins(&ids, &builtin_key);
    }
    let mut applied: Vec<&str> = overlays.iter().map(|pack| pack.id.as_str()).collect();
    applied.sort_unstable();
    let user_key = user.as_ref().map(resolver::user_overlay_key).unwrap_or_default();
    let key = format!("{}+{}+u{}", ids.iter().map(|lang| lang.code()).collect::<Vec<_>>().join(","), applied.join(","), user_key);
    let mut map = cache().lock();
    if let Some(catalog) = map.get(&key) {
        crate::ops::ops().record_cache_hit(map.len() as u64);
        return catalog;
    }
    let mut catalog = Catalog::merge_layers(&layers_of(&ids));
    for overlay in &overlays {
        if let Err(err) = resolver::apply_overlay(&mut catalog, overlay) {
            tracing::warn!("language overlay {}: {err}", overlay.id);
        }
    }
    if let Some(user) = &user {
        resolver::apply_user_overlay(&mut catalog, user);
    }
    let out = map.insert(key, Arc::new(catalog));
    crate::ops::ops().record_cache_miss(map.len() as u64);
    out
}

/// The catalog a request binds under these settings. `languages` may be empty (every
/// locale may be pinned) or name nothing a pack answers; a request that carries no pin
/// of its own then binds the configured default pack instead of an empty catalog.
pub fn bound_catalog(settings: &crate::types::Settings) -> Arc<Catalog> {
    let catalog = catalog_for(&settings.languages);
    if catalog.langs.is_empty() {
        return catalog_for(&[settings.default_pack().to_string()]);
    }
    catalog
}

fn cached_builtins(ids: &[LangId], key: &str) -> Arc<Catalog> {
    let mut map = cache().lock();
    if let Some(catalog) = map.get(key) {
        crate::ops::ops().record_cache_hit(map.len() as u64);
        return catalog;
    }
    let out = map.insert(key.to_string(), Arc::new(Catalog::merge_layers(&layers_of(ids))));
    crate::ops::ops().record_cache_miss(map.len() as u64);
    out
}

/// A regional pack is bound on top of its parent. A parent that is bound anyway is not read twice.
fn layers_of(ids: &[LangId]) -> Vec<Layer> {
    ids.iter().map(|id| Layer { pack: id.pack(), base: id.parent().filter(|parent| !ids.contains(parent)).map(LangId::pack) }).collect()
}

thread_local! {
    static CURRENT: RefCell<Option<Arc<Catalog>>> = const { RefCell::new(None) };
}

/// Bound catalog for the current parse. Thread-local so leaf helpers can still
/// call `catalog()` without threading `&Catalog` through every home/parse site.
pub fn catalog() -> Arc<Catalog> {
    CURRENT.with(|c| c.borrow().clone()).unwrap_or_else(unbound_catalog)
}

fn unbound_catalog() -> Arc<Catalog> {
    // Production: empty until Assist/Wyoming pins a locale (do not merge 67 packs).
    // Unit tests still call leaf helpers without bind(); they expect the bilingual
    // de+en catalog those helpers were written against.
    #[cfg(test)]
    {
        catalog_for(&["de".into(), "en".into()])
    }
    #[cfg(not(test))]
    {
        empty_catalog()
    }
}

thread_local! {
    static SPEECH_OVERRIDE: RefCell<Option<LangId>> = const { RefCell::new(None) };
}

/// Prefer this pack for spoken replies while the guard lives (R-21).
pub struct SpeechBind {
    prev: Option<LangId>,
}

impl Drop for SpeechBind {
    fn drop(&mut self) {
        SPEECH_OVERRIDE.with(|slot| *slot.borrow_mut() = self.prev.take());
    }
}

pub fn bind_speech(id: LangId) -> SpeechBind {
    let prev = SPEECH_OVERRIDE.with(|slot| slot.replace(Some(id)));
    SpeechBind { prev }
}

fn speech_override() -> Option<LangId> {
    SPEECH_OVERRIDE.with(|slot| *slot.borrow())
}

/// Language of spoken replies: the detected utterance pack when set, else the first
/// bound pack in settings order. Last resort is English, never German.
pub fn speech_lang() -> LangId {
    speech_override().or_else(|| catalog().langs.first().copied()).unwrap_or(LangId::En)
}

/// Among the bound packs, pick the one whose lexicon best matches the tokens (R-21).
/// Ties keep settings order. No signal keeps the first bound pack.
pub fn detect_speech_lang(tokens: &[String], langs: &[LangId]) -> LangId {
    if langs.is_empty() {
        return LangId::En;
    }
    if langs.len() == 1 {
        return langs[0];
    }
    let mut scores = vec![0i32; langs.len()];
    for token in tokens {
        let owners: Vec<usize> =
            langs.iter().enumerate().filter(|(_, id)| pack_knows_token(id.pack(), token)).map(|(index, _)| index).collect();
        match owners.as_slice() {
            [only] => scores[*only] += 3,
            many if !many.is_empty() => {
                for index in many {
                    scores[*index] += 1;
                }
            }
            _ => {}
        }
    }
    let mut best = 0usize;
    for (index, score) in scores.iter().enumerate().skip(1) {
        if *score > scores[best] {
            best = index;
        }
    }
    langs[best]
}

fn pack_knows_token(pack: &Pack, token: &str) -> bool {
    pack.verbs.iter().any(|(word, _)| *word == token)
        || pack.household.clock.iter().any(|phrase| phrase.split_whitespace().any(|word| word == token))
        || pack.talk.query_hint.contains(&token)
        || pack.talk.question_starts.contains(&token)
}

pub struct CatalogBind {
    prev: Option<Arc<Catalog>>,
}

impl Drop for CatalogBind {
    fn drop(&mut self) {
        CURRENT.with(|c| {
            *c.borrow_mut() = self.prev.take();
        });
    }
}

/// Temporary adapter: helpers still read `catalog()` while the guard lives.
/// New parse/home code should take `&Catalog` instead.
pub fn bind(lang_codes: &[String]) -> CatalogBind {
    bind_catalog(catalog_for(lang_codes))
}

pub(crate) fn bind_catalog(cat: Arc<Catalog>) -> CatalogBind {
    let prev = CURRENT.with(|c| c.replace(Some(cat)));
    CatalogBind { prev }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn catalog_cache_reuses_arc_and_stays_bounded() {
        clear_catalog_cache();
        let first = catalog_for(&["de".into()]);
        let again = catalog_for(&["de".into()]);
        assert!(Arc::ptr_eq(&first, &again));

        let codes: Vec<String> = LangId::all().iter().map(|id| id.code().to_string()).collect();
        let n = codes.len();
        assert!(n > 8, "need enough packs to cycle unique pairs");
        let mut early = None;
        for i in 0..1000 {
            let a = codes[i % n].clone();
            let b = codes[(i.wrapping_mul(7) + 3) % n].clone();
            let cat = catalog_for(&[a, b]);
            if i == 0 {
                early = Some(Arc::downgrade(&cat));
            }
        }
        assert!(cache().lock().len() <= MAX_CACHED_CATALOGS);
        // Unique pair keys exceed the LRU; the first entry is gone once nobody holds it.
        assert!(early.expect("early").upgrade().is_none());
        clear_catalog_cache();
    }

    #[test]
    fn catalog_cache_survives_panic_while_locked() {
        clear_catalog_cache();
        let _ = std::panic::catch_unwind(|| {
            let _guard = cache().lock();
            panic!("poison probe");
        });
        let cat = catalog_for(&["en".into()]);
        assert!(!cat.langs.is_empty());
        clear_catalog_cache();
    }
}
