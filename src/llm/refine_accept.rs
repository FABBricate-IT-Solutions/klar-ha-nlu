//! Accept rules for Assist refine. No model, no HA I/O.

const LIGHT_CLAIM: &[&str] = &["licht", "light", "lampe", "lamp"];
const FAIL_CLAIM: &[&str] = &["nicht geklappt", "did not work", "nicht erreichbar", "not available"];
const DONE_CLAIM: &[&str] = &["ist an", "is on", "läuft", "playing", "eingeschaltet"];
const STAMP_BAN: &[&str] = &[
    "zur kenntnis genommen",
    "notiert",
    "vermerkt",
    "besorgt",
    "soweit gemeldet",
    "duly noted",
    "taken into account",
    "noted.",
    "enregistré",
    "enregistre",
    "pris en note",
    "fehlinterpretation",
    "genoteerd",
];

const WEATHER_WORDS: &[&str] = &[
    "weather",
    "forecast",
    "degrees",
    "celsius",
    "fahrenheit",
    "humidity",
    "precipitation",
    "sunny",
    "cloudy",
    "rain",
    "rainy",
    "raining",
    "wetter",
    "vorhersage",
    "regen",
    "sonnig",
    "regnerisch",
    "bewölkt",
    "bewolkt",
];
const WEATHER_STEMS: &[&str] = &["°c", "°f", "luftfeucht"];

const SIMPLE_NUM_WORDS: &[&str] = &[
    "null",
    "eins",
    "zwei",
    "drei",
    "vier",
    "fünf",
    "sechs",
    "sieben",
    "acht",
    "neun",
    "zehn",
    "elf",
    "zwölf",
    "dreizehn",
    "vierzehn",
    "fünfzehn",
    "sechzehn",
    "siebzehn",
    "achtzehn",
    "neunzehn",
    "zwanzig",
    "dreissig",
    "dreißig",
    "vierzig",
    "fünfzig",
    "sechzig",
    "siebzig",
    "achtzig",
    "neunzig",
    "hundert",
    "tausend",
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
    "thirty",
    "forty",
    "fifty",
    "sixty",
    "seventy",
    "eighty",
    "ninety",
    "hundred",
    "thousand",
];

const UNITS: &[&str] = &["ein", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun"];
const TENS: &[&str] = &["zwanzig", "dreissig", "dreißig", "vierzig", "fünfzig", "sechzig", "siebzig", "achtzig", "neunzig"];

const STRIP_QUOTES: &[char] = &['"', '\'', '`', '“', '”', '«', '»'];

/// True when speech reports a forecast. `training` must not count as `rain`.
pub fn weather_claim(text: &str) -> bool {
    let fold = text.to_lowercase();
    if WEATHER_STEMS.iter().any(|stem| fold.contains(stem)) {
        return true;
    }
    WEATHER_WORDS.iter().any(|word| contains_word(&fold, word))
}

pub fn invents_weather(original: &str, refined: &str) -> bool {
    weather_claim(refined) && !weather_claim(original)
}

pub fn strip_clock_seconds(speech: &str) -> String {
    let chars: Vec<char> = speech.chars().collect();
    let mut out = String::with_capacity(speech.len());
    let mut i = 0;
    while i < chars.len() {
        if let Some((end, hh, mm)) = clock_at(&chars, i) {
            out.push_str(&format!("{hh:02}:{mm}"));
            i = end;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn clean_refined(text: &str) -> String {
    let mut speech = text.trim().trim_matches(STRIP_QUOTES).to_string();
    if speech.contains('\n') {
        speech = speech.lines().map(str::trim).filter(|line| !line.is_empty()).collect::<Vec<_>>().join(" ");
    }
    strip_clock_seconds(speech.trim())
}

pub fn accept_refined(original: &str, refined: &str) -> Option<String> {
    let speech = clean_refined(refined);
    if speech.is_empty() || speech.ends_with("...") || speech.ends_with('…') {
        return None;
    }
    if speech.ends_with('?') && !original.trim_end().ends_with('?') {
        return None;
    }
    if has_intent_name(&speech) {
        return None;
    }
    let source_nums = digit_set(original);
    let result_nums = digit_set(&speech);
    if source_nums != result_nums {
        return None;
    }
    if source_nums.is_empty() && has_number_word(&speech) {
        return None;
    }
    let max_len = (original.chars().count() * 6).max(280);
    if speech.chars().count() > max_len {
        return None;
    }
    let folded = speech.to_lowercase();
    let original_fold = original.to_lowercase();
    if STAMP_BAN.iter().any(|ban| folded.contains(ban)) {
        return None;
    }
    if LIGHT_CLAIM.iter().any(|word| folded.contains(word)) && !LIGHT_CLAIM.iter().any(|word| original_fold.contains(word)) {
        return None;
    }
    if invents_weather(original, &speech) {
        return None;
    }
    if FAIL_CLAIM.iter().any(|word| original_fold.contains(word)) && DONE_CLAIM.iter().any(|word| folded.contains(word)) {
        return None;
    }
    Some(speech)
}

fn has_intent_name(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i + 5 <= chars.len() {
        if chars[i..].starts_with(&['H', 'a', 's', 's']) {
            let before_ok = i == 0 || !is_word(chars[i - 1]);
            let rest = &chars[i + 4..];
            if before_ok && rest.first().copied().is_some_and(|c| c.is_ascii_uppercase()) {
                let mut j = 1;
                while j < rest.len() && rest[j].is_ascii_alphabetic() {
                    j += 1;
                }
                if j >= 2 {
                    let after_ok = i + 4 + j == chars.len() || !is_word(rest[j]);
                    if after_ok {
                        return true;
                    }
                }
            }
        }
        i += 1;
    }
    false
}

fn digit_set(text: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let mut buf = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            buf.push(ch);
        } else if !buf.is_empty() {
            out.insert(std::mem::take(&mut buf));
        }
    }
    if !buf.is_empty() {
        out.insert(buf);
    }
    out
}

fn has_number_word(text: &str) -> bool {
    let fold = text.to_lowercase();
    for token in tokens(&fold) {
        if is_number_token(token) {
            return true;
        }
    }
    false
}

fn is_number_token(token: &str) -> bool {
    if SIMPLE_NUM_WORDS.iter().any(|word| *word == token) {
        return true;
    }
    for unit in UNITS {
        for ten in TENS {
            let mut compound = String::with_capacity(unit.len() + 3 + ten.len());
            compound.push_str(unit);
            compound.push_str("und");
            compound.push_str(ten);
            if token == compound {
                return true;
            }
        }
    }
    false
}

fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| !is_word(ch)).filter(|part| !part.is_empty())
}

fn contains_word(hay: &str, needle: &str) -> bool {
    let chars: Vec<char> = hay.chars().collect();
    let needle: Vec<char> = needle.chars().collect();
    if needle.is_empty() || chars.len() < needle.len() {
        return false;
    }
    for i in 0..=chars.len() - needle.len() {
        if chars[i..i + needle.len()] == needle[..] {
            let before_ok = i == 0 || !is_word(chars[i - 1]);
            let after = i + needle.len();
            let after_ok = after == chars.len() || !is_word(chars[after]);
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

fn is_word(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn clock_at(chars: &[char], i: usize) -> Option<(usize, u32, String)> {
    if i > 0 && is_word(chars[i - 1]) {
        return None;
    }
    let mut j = i;
    if j >= chars.len() || !chars[j].is_ascii_digit() {
        return None;
    }
    let mut hh = chars[j].to_digit(10)?;
    j += 1;
    if j < chars.len() && chars[j].is_ascii_digit() {
        hh = hh * 10 + chars[j].to_digit(10)?;
        j += 1;
    }
    if j >= chars.len() || chars[j] != ':' {
        return None;
    }
    j += 1;
    if j + 1 >= chars.len() || !chars[j].is_ascii_digit() || !chars[j + 1].is_ascii_digit() {
        return None;
    }
    let mm = format!("{}{}", chars[j], chars[j + 1]);
    j += 2;
    if j + 2 < chars.len() && chars[j] == ':' && chars[j + 1].is_ascii_digit() && chars[j + 2].is_ascii_digit() {
        j += 3;
    }
    if j < chars.len() && is_word(chars[j]) {
        return None;
    }
    Some((j, hh, mm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_keeps_facts_and_rejects_inventions() {
        assert_eq!(
            accept_refined("Wohnzimmer Licht ist an.", "Das Licht im Wohnzimmer ist an.").as_deref(),
            Some("Das Licht im Wohnzimmer ist an.")
        );
        assert_eq!(
            accept_refined("Heizung Wohnzimmer auf 21 Grad.", "Die Heizung im Wohnzimmer auf 21 Grad.").as_deref(),
            Some("Die Heizung im Wohnzimmer auf 21 Grad.")
        );
        assert_eq!(
            accept_refined("Better Thermostat Wohnzimmer ist 21,5 °C.", "Im Wohnzimmer sind es 21,5 °C.").as_deref(),
            Some("Im Wohnzimmer sind es 21,5 °C.")
        );
        assert_eq!(accept_refined("Temperatur im Schlafzimmer.", "Die Temperatur im Schlafzimmer ist 20 Grad."), None);
        assert_eq!(accept_refined("Temperatur im Schlafzimmer.", "Die Temperatur im Schlafzimmer ist zwanzig Grad."), None);
        assert_eq!(accept_refined("Klimaanlage auf 19 Grad.", "Die Klimaanlage ist auf neunzehn Grad."), None);
        assert_eq!(accept_refined("Erledigt: HassSetPosition.", "HassSetPosition ist erledigt."), None);
        assert_eq!(accept_refined("Licht ist an.", "Licht ist an..."), None);
        assert_eq!(accept_refined("Wohnzimmer TV ist an.", "Das Licht im Wohnzimmer ist an."), None);
        assert_eq!(accept_refined("Nothing tomorrow.", "Tomorrow will be sunny."), None);
        assert_eq!(accept_refined("Team training at 3.", "Team training is at 3.").as_deref(), Some("Team training is at 3."));
        assert_eq!(accept_refined("Der Fernseher ist gerade nicht erreichbar.", "Das Licht im Wohnzimmer ist an."), None);
        assert_eq!(accept_refined("Temperatur im Schlafzimmer.", "Wie ist die Temperatur im Schlafzimmer?"), None);
        assert_eq!(
            accept_refined("Meinst du Küche oder Wohnzimmer?", "Küche oder Wohnzimmer, Sir?").as_deref(),
            Some("Küche oder Wohnzimmer, Sir?")
        );
        assert_eq!(
            accept_refined("Wohnzimmer Licht ist an.", "Das Licht im Wohnzimmer ist an. Ich habe es für Sie eingeschaltet.").as_deref(),
            Some("Das Licht im Wohnzimmer ist an. Ich habe es für Sie eingeschaltet.")
        );
        assert_eq!(clean_refined("Das Licht ist an.\nIch habe es eingeschaltet."), "Das Licht ist an. Ich habe es eingeschaltet.");
        assert_eq!(accept_refined("Licht ist an.", &format!("Licht ist an. {}", "x".repeat(400))), None);
        assert_eq!(clean_refined("Es ist 14:44:55."), "Es ist 14:44.");
        assert_eq!(accept_refined("Es ist 14:44.", "Es ist 14:44:55.").as_deref(), Some("Es ist 14:44."));
    }

    #[test]
    fn rejects_bureaucratic_stamps() {
        assert_eq!(accept_refined("Licht ist an.", "Zur Kenntnis genommen. Licht ist an."), None);
        assert_eq!(accept_refined("Licht ist an.", "Das ist besorgt."), None);
        assert_eq!(accept_refined("Licht ist an.", "soweit gemeldet"), None);
    }

    #[test]
    fn weather_ignores_training() {
        assert!(!weather_claim("Team training at 3."));
        assert!(weather_claim("Tomorrow will be sunny."));
        assert!(weather_claim("21 °C"));
    }
}
