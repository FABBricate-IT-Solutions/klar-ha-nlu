//! Build the Assist refine system prompt. No model call here.

use super::refine_shots::locale_shots;
use super::refine_voices::{normalize_personality, rules, voice, Lane};
use serde::Serialize;

const GERMAN_EXTRA_MARKERS: &[&str] = &["Stimme:", "Schalt-Bestätigungen", "Antworte nur auf Deutsch"];

pub fn refine_prompt(pack: &str, personality: &str) -> String {
    refine_prompt_for(pack, personality, "")
}

pub fn refine_prompt_for(pack: &str, personality: &str, custom_voice: &str) -> String {
    let rules_text = rules(lane_for_rules(pack));
    let lock = language_lock(pack);
    let voice_text = if is_custom(personality) && !custom_voice.trim().is_empty() {
        custom_voice.trim().to_string()
    } else {
        voice_block(pack, normalize_personality(personality))
    };
    format!("{lock}\n\n{rules_text}\n\n{voice_text}\n\n{lock}")
}

fn is_custom(personality: &str) -> bool {
    personality.trim().eq_ignore_ascii_case("custom")
}

pub fn refine_input(speech: &str) -> String {
    speech.trim().to_string()
}

pub fn language_lock(pack: &str) -> String {
    let name = native_name(pack);
    if is_de(pack) {
        format!("Antworte nur auf {name}. Übersetze nicht ins Englische oder in eine andere Sprache.")
    } else if is_en(pack) {
        format!("Answer only in {name}. Do not translate into German or any other language.")
    } else {
        format!("Answer only in {name} (Klar NLU pack {pack}). Do not translate into German, English, or any other language.")
    }
}

pub fn usable_extra(custom: &str, pack: &str) -> bool {
    if custom.is_empty() {
        return false;
    }
    if !is_de(pack) && GERMAN_EXTRA_MARKERS.iter().any(|marker| custom.contains(marker)) {
        return false;
    }
    true
}

fn stock_voice_pack(pack: &str) -> &str {
    let tag = pack.trim();
    if tag.eq_ignore_ascii_case("de") || tag.eq_ignore_ascii_case("de-de") {
        "de"
    } else if tag.eq_ignore_ascii_case("en") || tag.eq_ignore_ascii_case("en-us") {
        "en"
    } else {
        pack
    }
}

fn voice_block(pack: &str, personality: &str) -> String {
    let pack = stock_voice_pack(pack);
    if pack == "en" {
        let block = voice(personality, Lane::En);
        return format!(
            "Voice: {}.\nSound like this character. Vary the wording. Do not stamp the same opening every time.\nExamples:\n{}",
            trim_dot(block.flavor),
            block.shots
        );
    }
    if pack == "de" {
        let block = voice(personality, Lane::De);
        return format!(
            "Stimme: {}.\nKlinge wie diese Figur. Variiere die Formulierung. Klebe nicht jedes Mal dieselbe Eröffnung davor.\nBeispiele:\n{}",
            trim_dot(block.flavor),
            block.shots
        );
    }
    let block = voice(personality, Lane::Meta);
    let shots = {
        let generated = locale_shots(pack, personality);
        if generated.is_empty() {
            block.shots
        } else {
            generated
        }
    };
    let mut text = format!(
        "Voice: {}.\nKeep the Klar NLU output language. Do not translate into German unless that is the pack. \
Sound like this character. Vary the wording. Do not stamp the same opening every time.\n",
        trim_dot(block.flavor)
    );
    if !shots.is_empty() {
        text.push_str("Examples:\n");
        text.push_str(shots);
        text.push('\n');
    }
    text
}

fn lane_for_rules(pack: &str) -> Lane {
    if is_en(pack) {
        Lane::En
    } else if is_de(pack) {
        Lane::De
    } else {
        Lane::Meta
    }
}

fn is_de(pack: &str) -> bool {
    pack == "de" || pack.starts_with("de-")
}

fn is_en(pack: &str) -> bool {
    pack == "en" || pack.starts_with("en-")
}

fn trim_dot(text: &str) -> &str {
    text.trim_end_matches('.')
}

fn native_name(pack: &str) -> String {
    crate::lang::LangId::from_tag(pack)
        .and_then(|id| id.meta())
        .map(|meta| meta.native_name.to_string())
        .unwrap_or_else(|| pack.to_string())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PersonalityPreview {
    pub personality: String,
    pub flavor: String,
    pub prompt: String,
}

pub fn personality_preview(pack: &str, personality: &str) -> PersonalityPreview {
    personality_preview_for(pack, personality, "")
}

pub fn personality_preview_for(pack: &str, personality: &str, custom_voice: &str) -> PersonalityPreview {
    let prompt = refine_prompt_for(pack, personality, custom_voice);
    if is_custom(personality) {
        let flavor = custom_voice.lines().next().unwrap_or("custom").trim().to_string();
        return PersonalityPreview { personality: "custom".into(), flavor, prompt };
    }
    let personality = normalize_personality(personality).to_string();
    let flavor = voice(&personality, lane_for_rules(pack)).flavor.to_string();
    PersonalityPreview { personality, flavor, prompt }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_butler_keeps_safety_and_stock_voice() {
        let prompt = refine_prompt("de", "butler");
        assert!(prompt.contains("Keine Home-Assistant-Werkzeuge"));
        assert!(prompt.contains("Ziffern bleiben Ziffern"));
        assert!(prompt.contains("2 bleibt 2"));
        assert!(prompt.contains("2 Lichter sind an, 3 Lichter sind aus."));
        assert!(prompt.contains("21,5 °C"));
        assert!(prompt.contains("Keine neuen Zahlen"));
        assert!(prompt.contains("Länge folgt der NLU-Vorlage"));
        assert!(!prompt.contains("ein Satz"));
        assert!(prompt.contains("Uhrzeiten ohne Sekunden"));
        assert!(prompt.contains("14:44 nicht 14:44:55"));
        assert!(prompt.contains("In der Küche ist das Licht aus"));
        assert!(prompt.contains("Offene Fragen"));
        assert!(prompt.contains("Ist die Vorlage eine Frage, bleibt die Antwort eine Frage."));
        assert!(prompt.contains("Butler"));
        assert!(!prompt.contains("Ein oder zwei Sätze."));
        assert!(!prompt.contains("Formel: Sehr wohl."));
        assert!(!prompt.contains("Hänge immer an"));
        assert!(usable_extra("Ein oder zwei Sätze.", "de"));
        assert!(!usable_extra("", "de"));
    }

    #[test]
    fn empty_extra_uses_builtin_voice() {
        let prompt = refine_prompt("de", "butler");
        assert!(prompt.contains("Butler"));
        assert!(prompt.contains("Klebe nicht jedes Mal dieselbe Eröffnung davor."));
        assert!(prompt.contains("Status"));
    }

    #[test]
    fn english_prompt_locks_language() {
        let prompt = refine_prompt("en", "locker");
        assert!(prompt.contains("Do not call Home Assistant tools"));
        assert!(prompt.contains("casual"));
        assert!(prompt.contains("Voice:"));
        assert!(prompt.to_lowercase().contains("open questions"));
        assert!(prompt.contains("Do not stamp the same opening every time."));
        assert!(prompt.contains("all set"));
        assert!(prompt.contains("Clock times without seconds"));
        assert!(prompt.contains("Length follows the NLU source"));
        assert!(!prompt.contains("one spoken sentence"));
        assert!(prompt.contains("Do not translate into German"));
        assert!(!prompt.contains("Additional style instruction"));
    }

    #[test]
    fn german_stored_extra_ignored_for_english() {
        let prompt = refine_prompt("en", "butler");
        assert!(prompt.contains("Do not translate into German"));
        assert!(prompt.contains("Voice:"));
        assert!(!prompt.contains("Stimme:"));
        assert!(!usable_extra("Stimme: Jarvis.\nSchalt-Bestätigungen", "en"));
        assert!(usable_extra("Keep replies to one sentence.", "en"));
    }

    #[test]
    fn other_packs_use_meta_rules() {
        for pack in ["fr", "nl", "ja"] {
            let prompt = refine_prompt(pack, "butler");
            assert!(!prompt.contains("Stimme:"), "{pack}");
            assert!(!prompt.contains("Klebe nicht jedes Mal"), "{pack}");
            assert!(prompt.to_lowercase().contains("same language"), "{pack}");
            assert!(prompt.to_lowercase().contains("input line"), "{pack}");
        }
        let french = refine_prompt("fr", "butler");
        assert!(french.contains("Examples:"));
        assert!(french.contains('→'));
        let swiss = refine_prompt("de-CH", "butler");
        assert!(!swiss.contains("Stimme:"));
        assert!(swiss.contains("Schwyzerdütsch"));
        assert_eq!(refine_prompt("de", "not-a-voice"), refine_prompt("de", "default"));
        let de_de = refine_prompt("de-DE", "jarvis");
        assert!(de_de.contains("Stimme:"));
        assert!(de_de.contains("Jarvis"));
    }

    #[test]
    fn preview_matches_selected_personality() {
        let preview = personality_preview("de", "jarvis");
        assert_eq!(preview.personality, "jarvis");
        assert!(preview.flavor.contains("Jarvis"));
        assert_eq!(preview.prompt, refine_prompt("de", "jarvis"));
    }

    #[test]
    fn each_personality_is_distinct() {
        let mut seen = std::collections::BTreeSet::new();
        for name in
            ["default", "butler", "locker", "fuersorglich", "party", "grantig", "sarkastisch", "pirat", "hippie", "gollum", "jarvis"]
        {
            let prompt = refine_prompt("de", name);
            assert!(!prompt.contains("Hänge immer an"), "{name}");
            assert!(prompt.contains('→'), "{name}");
            assert!(seen.insert(prompt), "{name} duplicate");
        }
    }

    #[test]
    fn custom_voice_keeps_lock_and_rules() {
        let prompt = refine_prompt_for("de", "custom", "Stimme: knochentrocken.\nKeine Floskeln.");
        assert!(prompt.contains("Antworte nur auf"));
        assert!(prompt.contains("Keine Home-Assistant-Werkzeuge"));
        assert!(prompt.contains("Stimme: knochentrocken."));
        assert!(!prompt.contains("Butler"));
        assert_eq!(refine_prompt_for("de", "custom", ""), refine_prompt("de", "default"));
        assert_eq!(refine_prompt("de", "custom"), refine_prompt("de", "default"));
    }
}
