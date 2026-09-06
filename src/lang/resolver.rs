//! Scoped catalog resolution: request tag → overlay → builtin. No silent last-wins on verbs.

use super::catalog::Catalog;
use super::external::{leak, ExternalPack, PackRegistry};
use super::locale::{LocaleError, LocaleId};
use super::user::LanguageOverlay;
use super::validate::{set_field, validate_pack};
use super::LangId;
use std::cell::RefCell;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

thread_local! {
    static PREVIEW_USER: RefCell<Option<LanguageOverlay>> = const { RefCell::new(None) };
}

fn runtime_packs() -> &'static Mutex<Vec<ExternalPack>> {
    static PACKS: OnceLock<Mutex<Vec<ExternalPack>>> = OnceLock::new();
    PACKS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn install_runtime_packs(packs: Vec<ExternalPack>) {
    *runtime_packs().lock().expect("lang overlay registry") = packs;
    super::clear_catalog_cache();
}

pub fn installed_packs() -> Vec<ExternalPack> {
    runtime_packs().lock().expect("lang overlay registry").clone()
}

pub fn load_runtime_dir(dir: &Path) -> Result<usize, String> {
    let packs = PackRegistry::load_dir(dir)?;
    let count = packs.len();
    install_runtime_packs(packs);
    Ok(count)
}

pub fn reset_runtime_packs() {
    install_user_overlay(None);
    install_runtime_packs(Vec::new());
}

pub fn pin_language(raw: &str) -> Result<String, LocaleError> {
    let locale = LocaleId::parse(raw)?;
    if !is_known(&locale) {
        return Err(LocaleError::Unknown(locale.tag));
    }
    Ok(locale.tag)
}

pub fn is_known(locale: &LocaleId) -> bool {
    locale.fallback_chain().iter().any(|step| builtin_for(&step.tag).is_some() || overlay_for(&step.tag).is_some())
}

pub fn builtin_for(tag: &str) -> Option<LangId> {
    if let Some(id) = LangId::from_code(tag) {
        return Some(id);
    }
    let locale = LocaleId::parse(tag).ok()?;
    for step in locale.fallback_chain() {
        if let Some(id) = LangId::from_code(&step.tag) {
            return Some(id);
        }
    }
    None
}

pub fn overlay_for(tag: &str) -> Option<ExternalPack> {
    let Ok(locale) = LocaleId::parse(tag) else {
        return None;
    };
    let chain: Vec<String> = locale.fallback_chain().into_iter().map(|item| item.tag).collect();
    installed_packs()
        .into_iter()
        .find(|pack| pack.locales().ok().into_iter().flatten().any(|item| chain.iter().any(|step| step == &item.tag || step == &pack.id)))
}

pub fn apply_overlay(catalog: &mut Catalog, pack: &ExternalPack) -> Result<(), String> {
    let report = validate_pack(pack, Some(catalog));
    if !report.ok() {
        let first =
            report.errors.first().map(|issue| format!("{}: {}", issue.path, issue.message)).unwrap_or_else(|| "invalid pack".into());
        return Err(first);
    }
    for (token, kind) in pack.verb_entries()? {
        catalog.verbs.insert(leak(&token), kind);
    }
    for (token, domain) in &pack.maps.domain_map {
        catalog.domain_map.insert(leak(token), leak(domain));
    }
    for (token, color) in &pack.maps.colors {
        catalog.colors.insert(leak(token), leak(color));
    }
    for (token, number) in &pack.maps.numbers {
        catalog.numbers.insert(leak(token), *number);
    }
    for (path, words) in &pack.sets {
        let Some(key) = set_field(path) else {
            continue;
        };
        for word in words {
            catalog.words_mut(key).insert(leak(word));
        }
    }
    let extra = pack.morphology();
    catalog.morphology.room_suffixes.extend(extra.room_suffixes);
    catalog.morphology.color_suffixes.extend(extra.color_suffixes);
    catalog.morphology.linking.extend(extra.linking);
    catalog.pack_intents.extend(pack.custom_intents());
    Ok(())
}

fn user_overlay() -> &'static Mutex<Option<LanguageOverlay>> {
    static USER: OnceLock<Mutex<Option<LanguageOverlay>>> = OnceLock::new();
    USER.get_or_init(|| Mutex::new(None))
}

pub fn install_user_overlay(overlay: Option<LanguageOverlay>) {
    *user_overlay().lock().expect("user language overlay") = overlay;
    super::clear_catalog_cache();
}

pub fn installed_user_overlay() -> Option<LanguageOverlay> {
    user_overlay().lock().expect("user language overlay").clone()
}

pub fn effective_user_overlay() -> Option<LanguageOverlay> {
    PREVIEW_USER.with(|slot| slot.borrow().clone()).or_else(installed_user_overlay)
}

pub struct PreviewUserBind;

impl Drop for PreviewUserBind {
    fn drop(&mut self) {
        PREVIEW_USER.with(|slot| slot.borrow_mut().take());
    }
}

pub fn bind_preview_user(overlay: Option<LanguageOverlay>) -> PreviewUserBind {
    PREVIEW_USER.with(|slot| *slot.borrow_mut() = overlay);
    PreviewUserBind
}

pub fn user_overlay_key(overlay: &LanguageOverlay) -> String {
    super::user::user_overlay_key(overlay)
}

pub fn apply_user_overlay(catalog: &mut Catalog, overlay: &LanguageOverlay) {
    for (path, delta) in &overlay.sets {
        let Some(key) = set_field(path) else {
            continue;
        };
        for word in &delta.add {
            catalog.words_mut(key).insert(leak(word));
        }
        for word in &delta.remove {
            catalog.words_mut(key).retain(|existing| *existing != word.as_str());
        }
    }
}
