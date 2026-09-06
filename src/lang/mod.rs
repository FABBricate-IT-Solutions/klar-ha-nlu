//! Language packs. Built-in locales are compiled; YAML stays for user overlays.

mod catalog;
mod cli;
mod external;
mod fuzzy;
mod groups;
mod hassil;
mod locale;
mod morphology;
mod pack;
#[allow(clippy::pedantic, clippy::nursery)]
mod packs;
mod registry;
mod resolver;
mod speech;
mod user;
mod validate;
mod verbs;

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub use catalog::Catalog;
pub use cli::{import_hassil, preview, validate_path};
pub use external::{ExternalPack, PackRegistry};
pub use hassil::{import_hassil as parse_hassil, HassilImport};
pub use locale::{LocaleError, LocaleId};
pub use morphology::Morphology;
pub use pack::{GroupClarify, Household, LanguagePack, NumberStyle};
pub use registry::{languages, LangMeta};
pub use resolver::{
    bind_preview_user, install_runtime_packs, install_user_overlay, installed_user_overlay, is_known, load_runtime_dir, pin_language,
    reset_runtime_packs,
};
pub use speech::Speech;
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

    pub fn compiled_codes() -> Vec<String> {
        Self::all().iter().map(|id| id.code().to_string()).collect()
    }
}

fn empty_catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(|| Catalog::merge(&[]))
}

const MAX_CACHED_CATALOGS: usize = 32;

fn cache() -> &'static Mutex<HashMap<String, &'static Catalog>> {
    static C: OnceLock<Mutex<HashMap<String, &'static Catalog>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Leak at most one Catalog per unique language-set key. The map clears at
/// `MAX_CACHED_CATALOGS` so the leak stays bounded.
fn insert_cached(map: &mut HashMap<String, &'static Catalog>, key: String, catalog: Catalog) -> &'static Catalog {
    if map.len() >= MAX_CACHED_CATALOGS {
        map.clear();
    }
    map.entry(key).or_insert_with(|| Box::leak(Box::new(catalog)))
}

pub(super) fn clear_catalog_cache() {
    cache().lock().expect("lang catalog cache").clear();
}

pub fn catalog_for(codes: &[String]) -> &'static Catalog {
    let mut ids: Vec<LangId> = codes.iter().filter_map(|code| LangId::from_tag(code)).collect();
    let overlays: Vec<ExternalPack> = codes.iter().filter_map(|code| resolver::overlay_for(code)).collect();
    if ids.is_empty() {
        if let Some(base) = overlays.first().and_then(|pack| pack.base_lang().ok()) {
            ids.push(base);
        }
    }
    if ids.is_empty() {
        return empty_catalog();
    }
    ids.sort_by_key(|lang| lang.code());
    ids.dedup();
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
    let mut map = cache().lock().expect("lang catalog cache");
    if let Some(catalog) = map.get(&key) {
        return catalog;
    }
    let packs: Vec<&'static Pack> = ids.iter().map(|id| id.pack()).collect();
    let mut catalog = Catalog::merge(&packs);
    for overlay in &overlays {
        if let Err(err) = resolver::apply_overlay(&mut catalog, overlay) {
            tracing::warn!("language overlay {}: {err}", overlay.id);
        }
    }
    if let Some(user) = &user {
        resolver::apply_user_overlay(&mut catalog, user);
    }
    insert_cached(&mut map, key, catalog)
}

fn cached_builtins(ids: &[LangId], key: &str) -> &'static Catalog {
    let mut map = cache().lock().expect("lang catalog cache");
    if let Some(catalog) = map.get(key) {
        return catalog;
    }
    let packs: Vec<&'static Pack> = ids.iter().map(|id| id.pack()).collect();
    insert_cached(&mut map, key.to_string(), Catalog::merge(&packs))
}

thread_local! {
    static CURRENT: Cell<Option<&'static Catalog>> = const { Cell::new(None) };
}

/// Bound catalog for the current parse. Thread-local so leaf helpers can still
/// call `catalog()` without threading `&Catalog` through every home/parse site.
pub fn catalog() -> &'static Catalog {
    CURRENT.with(|c| c.get()).unwrap_or_else(unbound_catalog)
}

fn unbound_catalog() -> &'static Catalog {
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

/// Language of spoken replies: the first bound pack. Last resort is English, never German.
pub fn speech_lang() -> LangId {
    catalog().langs.first().copied().unwrap_or(LangId::En)
}

pub struct CatalogBind {
    prev: Option<&'static Catalog>,
}

impl Drop for CatalogBind {
    fn drop(&mut self) {
        CURRENT.with(|c| c.set(self.prev));
    }
}

/// Temporary adapter: helpers still read `catalog()` while the guard lives.
/// New parse/home code should take `&Catalog` instead.
pub fn bind(lang_codes: &[String]) -> CatalogBind {
    bind_catalog(catalog_for(lang_codes))
}

pub(crate) fn bind_catalog(cat: &'static Catalog) -> CatalogBind {
    let prev = CURRENT.with(|c| {
        let prev = c.get();
        c.set(Some(cat));
        prev
    });
    CatalogBind { prev }
}
