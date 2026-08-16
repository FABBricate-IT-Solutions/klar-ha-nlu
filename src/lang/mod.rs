//! Language packs. Built-in DE/EN stay compiled; extra locales load from YAML.

mod catalog;
mod cli;
mod de;
mod de_pack;
mod de_speech;
mod en;
mod en_pack;
mod external;
mod fuzzy;
mod groups;
mod hassil;
mod locale;
mod morphology;
mod pack;
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
pub use pack::{GroupClarify, LanguagePack, NumberStyle};
pub use resolver::{
    bind_preview_user, install_runtime_packs, install_user_overlay, installed_user_overlay, is_known, load_runtime_dir, pin_language,
    reset_runtime_packs,
};
pub use speech::Speech;
pub use user::{
    push_revision, revision_hash, select_revision, validate_custom, validate_language, LanguageOverlay, LanguageRevision, OverlayIssue,
    SetDelta, MAX_HISTORY, MAX_USER_INTENTS,
};
pub use validate::{validate_pack, ValidationReport};
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

    pub fn from_tag(tag: &str) -> Option<Self> {
        resolver::builtin_for(tag)
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

fn default_catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(|| Catalog::merge(&[LangId::De.pack(), LangId::En.pack()]))
}

fn cache() -> &'static Mutex<HashMap<String, &'static Catalog>> {
    static C: OnceLock<Mutex<HashMap<String, &'static Catalog>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
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
        return default_catalog();
    }
    ids.sort_by_key(|lang| lang.code());
    ids.dedup();
    let user = resolver::effective_user_overlay();
    if overlays.is_empty() && user.is_none() {
        if ids.as_slice() == LangId::DEFAULT {
            return default_catalog();
        }
        let builtin_key = ids.iter().map(|lang| lang.code()).collect::<Vec<_>>().join(",");
        return cached_builtins(&ids, &builtin_key);
    }
    let mut applied: Vec<&str> = overlays.iter().map(|pack| pack.id.as_str()).collect();
    applied.sort_unstable();
    let user_key = user.as_ref().map(resolver::user_overlay_key).unwrap_or_default();
    let key = format!("{}+{}+u{}", ids.iter().map(|lang| lang.code()).collect::<Vec<_>>().join(","), applied.join(","), user_key);
    let mut map = cache().lock().expect("lang catalog cache");
    let catalog = *map.entry(key).or_insert_with(|| {
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
        Box::leak(Box::new(catalog))
    });
    catalog
}

fn cached_builtins(ids: &[LangId], key: &str) -> &'static Catalog {
    let mut map = cache().lock().expect("lang catalog cache");
    let catalog = *map.entry(key.to_string()).or_insert_with(|| {
        let packs: Vec<&'static Pack> = ids.iter().map(|id| id.pack()).collect();
        Box::leak(Box::new(Catalog::merge(&packs)))
    });
    catalog
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
