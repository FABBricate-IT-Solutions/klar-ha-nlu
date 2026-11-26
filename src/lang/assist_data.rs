//! Built-in assist pack data (package 6.9).
//!
//! Generated locales still compile from `src/lang/packs/`, but phrase corrections
//! live in one YAML file per locale under `packs/assist/`. Those files are embedded
//! and applied as overlays so a pack correction touches a single data file.

use super::assist_embed::ASSIST_YAML;
use super::external::ExternalPack;
use std::sync::OnceLock;

fn loaded() -> &'static Vec<ExternalPack> {
    static PACKS: OnceLock<Vec<ExternalPack>> = OnceLock::new();
    PACKS.get_or_init(|| {
        let mut packs = Vec::with_capacity(ASSIST_YAML.len());
        for (code, raw) in ASSIST_YAML {
            match ExternalPack::from_yaml(raw) {
                Ok(mut pack) => {
                    pack.source = Some(format!("packs/assist/{code}.yaml"));
                    if has_corrections(&pack) {
                        packs.push(pack);
                    }
                }
                Err(err) => tracing::warn!("assist pack {code}: {err}"),
            }
        }
        packs
    })
}

fn has_corrections(pack: &ExternalPack) -> bool {
    !pack.verbs.is_empty()
        || !pack.sets.is_empty()
        || !pack.intents.is_empty()
        || !pack.maps.domain_map.is_empty()
        || !pack.maps.colors.is_empty()
        || !pack.maps.numbers.is_empty()
        || !pack.morphology.room_suffixes.is_empty()
        || !pack.morphology.color_suffixes.is_empty()
        || !pack.morphology.linking.is_empty()
}

/// Assist data overlay for a locale tag, when the YAML carries a real correction.
pub fn overlay_for(tag: &str) -> Option<ExternalPack> {
    let Ok(locale) = super::locale::LocaleId::parse(tag) else {
        return None;
    };
    let chain: Vec<String> = locale.fallback_chain().into_iter().map(|item| item.tag).collect();
    loaded()
        .iter()
        .find(|pack| pack.locales().ok().into_iter().flatten().any(|item| chain.iter().any(|step| step == &item.tag || step == &pack.id)))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::external::PACK_FORMAT;

    #[test]
    fn empty_stub_is_not_an_overlay() {
        let pack = ExternalPack {
            klar_lang_pack: PACK_FORMAT.into(),
            id: "fr".into(),
            bcp47: vec!["fr".into()],
            extends: "fr".into(),
            verbs: Default::default(),
            sets: Default::default(),
            maps: Default::default(),
            morphology: Default::default(),
            intents: vec![],
            source: None,
        };
        assert!(!has_corrections(&pack));
    }

    #[test]
    fn verb_correction_counts() {
        let mut pack = ExternalPack {
            klar_lang_pack: PACK_FORMAT.into(),
            id: "fr".into(),
            bcp47: vec!["fr".into()],
            extends: "fr".into(),
            verbs: Default::default(),
            sets: Default::default(),
            maps: Default::default(),
            morphology: Default::default(),
            intents: vec![],
            source: None,
        };
        pack.verbs.insert("ampoule".into(), "On".into());
        assert!(has_corrections(&pack));
    }
}
