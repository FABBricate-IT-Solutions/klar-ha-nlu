//! Build the Assist refine system prompt. No model call here.

use super::refine_shots::locale_shots;
use super::refine_voices::{normalize_personality, rules, voice, Lane};

const GERMAN_EXTRA_MARKERS: &[&str] = &["Stimme:", "Schalt-Bestätigungen", "Antworte nur auf Deutsch"];

pub fn refine_prompt(pack: &str, personality: &str, extra: Option<&str>) -> String {
    let personality = normalize_personality(personality);
    let custom = extra.unwrap_or("").trim();
    let stock = voice_block(pack, personality);
    let voice_text = if usable_extra(custom, pack) { custom.to_string() } else { stock };
    let rules_text = rules(lane_for_rules(pack));
    let lock = language_lock(pack);
    format!("{lock}\n\n{rules_text}\n\n{voice_text}\n\n{lock}")
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

fn usable_extra(custom: &str, pack: &str) -> bool {
    if custom.is_empty() {
        return false;
    }
    if !is_de(pack) && GERMAN_EXTRA_MARKERS.iter().any(|marker| custom.contains(marker)) {
        return false;
    }
    true
}

fn voice_block(pack: &str, personality: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_butler_keeps_safety_and_extra() {
        let prompt = refine_prompt("de", "butler", Some("Ein oder zwei Sätze."));
        assert!(prompt.contains("Keine Home-Assistant-Werkzeuge"));
        assert!(prompt.contains("Ziffern bleiben Ziffern"));
        assert!(prompt.contains("2 bleibt 2"));
        assert!(prompt.contains("2 Lichter sind an, 3 Lichter sind aus."));
        assert!(prompt.contains("21,5 °C"));
        assert!(prompt.contains("Keine neuen Zahlen"));
        assert!(prompt.contains("ein Satz"));
        assert!(prompt.contains("Uhrzeiten ohne Sekunden"));
        assert!(prompt.contains("14:44 nicht 14:44:55"));
        assert!(prompt.contains("Ein oder zwei Sätze."));
        assert!(prompt.contains("Offene Fragen"));
        assert!(prompt.contains("Ist die Vorlage eine Frage, bleibt die Antwort eine Frage."));
        assert!(!prompt.contains("Formel: Sehr wohl."));
        assert!(!prompt.contains("Hänge immer an"));
        assert!(!prompt.contains("Butler"));
    }

    #[test]
    fn empty_extra_uses_builtin_voice() {
        let prompt = refine_prompt("de", "butler", None);
        assert!(prompt.contains("Butler"));
        assert!(prompt.contains("Klebe nicht jedes Mal dieselbe Eröffnung davor."));
        assert!(prompt.contains("Status"));
    }

    #[test]
    fn english_prompt_locks_language() {
        let prompt = refine_prompt("en", "locker", None);
        assert!(prompt.contains("Do not call Home Assistant tools"));
        assert!(prompt.contains("casual"));
        assert!(prompt.contains("Voice:"));
        assert!(prompt.to_lowercase().contains("open questions"));
        assert!(prompt.contains("Do not stamp the same opening every time."));
        assert!(prompt.contains("all set"));
        assert!(prompt.contains("Clock times without seconds"));
        assert!(prompt.contains("Do not translate into German"));
        assert!(!prompt.contains("Additional style instruction"));
    }

    #[test]
    fn german_stored_extra_ignored_for_english() {
        let prompt = refine_prompt("en", "butler", Some("Stimme: Jarvis.\nSchalt-Bestätigungen"));
        assert!(prompt.contains("Do not translate into German"));
        assert!(prompt.contains("Voice:"));
        assert!(!prompt.contains("Stimme:"));
    }

    #[test]
    fn other_packs_use_meta_rules() {
        for pack in ["fr", "nl", "ja"] {
            let prompt = refine_prompt(pack, "butler", None);
            assert!(!prompt.contains("Stimme:"), "{pack}");
            assert!(!prompt.contains("Klebe nicht jedes Mal"), "{pack}");
            assert!(prompt.to_lowercase().contains("same language"), "{pack}");
            assert!(prompt.to_lowercase().contains("input line"), "{pack}");
        }
        let french = refine_prompt("fr", "butler", None);
        assert!(french.contains("Examples:"));
        assert!(french.contains('→'));
        let swiss = refine_prompt("de-CH", "butler", None);
        assert!(!swiss.contains("Stimme:"));
        assert!(swiss.contains("Schwyzerdütsch"));
        assert_eq!(refine_prompt("de", "not-a-voice", None), refine_prompt("de", "default", None));
    }

    #[test]
    fn each_personality_is_distinct() {
        let mut seen = std::collections::BTreeSet::new();
        for name in
            ["default", "butler", "locker", "fuersorglich", "party", "grantig", "sarkastisch", "pirat", "hippie", "gollum", "jarvis"]
        {
            let prompt = refine_prompt("de", name, None);
            assert!(!prompt.contains("Hänge immer an"), "{name}");
            assert!(prompt.contains('→'), "{name}");
            assert!(seen.insert(prompt), "{name} duplicate");
        }
    }
}
