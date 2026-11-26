//! Pack-backed empty-place lines, colors, and HA state words.

use crate::lang::{LangId, Speech};
use crate::types::SpeechSnapshot;

pub(super) fn empty_place(pack: &str) -> String {
    speech_for(pack).empty_place.to_string()
}

pub(super) fn color_word(raw: Option<&str>, language: &str) -> Option<String> {
    let color = raw?;
    let pack = LangId::from_tag(language).or_else(|| LangId::from_code(language)).map(|id| id.pack());
    if let Some(pack) = pack {
        // Prefer a native spelling that is not identical to the English canon key.
        let mut fallback = None;
        for (native, canon) in pack.maps.colors {
            if *canon != color {
                continue;
            }
            if *native != color {
                return Some((*native).to_string());
            }
            fallback = Some(*native);
        }
        if let Some(native) = fallback {
            return Some(native.to_string());
        }
    }
    Some(color.replace('_', " "))
}

pub(super) fn slot<'a>(snap: &'a SpeechSnapshot, name: &str) -> Option<&'a str> {
    snap.intent.slots.iter().find(|slot| slot.name == name).map(|slot| slot.value.as_str()).filter(|value| !value.is_empty())
}

pub(super) fn speech_for(language: &str) -> Speech {
    LangId::from_tag(language).or_else(|| LangId::from_code(language)).unwrap_or(LangId::En).pack().speech
}
