//! Language packs. The engine looks up words here; a new language is a new `*.rs` pack.
//!
//! To add for example French later: copy `en.rs` → `fr.rs`, fill the lists, add `LangId::Fr`,
//! register it in `LangId::pack`, and enable `"fr"` in `Settings.languages`.

mod catalog;
mod de;
mod de_pack;
mod en;
mod en_pack;
mod groups;
mod pack;
mod speech;
mod verbs;

use std::cell::Cell;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub use catalog::Catalog;
pub use pack::{GroupClarify, LanguagePack, NumberStyle};
pub use speech::Speech;
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
    let catalog = *map.entry(key).or_insert_with(|| {
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
    let cat = catalog_for(lang_codes);
    let prev = CURRENT.with(|c| {
        let prev = c.get();
        c.set(Some(cat));
        prev
    });
    CatalogBind { prev }
}
