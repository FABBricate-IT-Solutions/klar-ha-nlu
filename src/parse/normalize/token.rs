//! Tokenize folded speech: clitics, Unicode digits, CJK/Thai longest match.

use crate::lang::catalog;
use crate::parse::normalize::fold::{fold_latin, strip_possessive, unicode_digit};

/// Article clitics split before the noun so `l'entrée` is not `lentree` (L-10).
const CLITICS: &[&str] = &["l", "d", "dell", "all", "un", "nell", "sull", "coll", "c", "j", "m", "t", "s", "n"];

/// Letters plus virama / matras so Indic stays one token. Arabic/Hebrew points are
/// stripped in fold, so they are not word chars here.
pub(crate) fn is_word_char(c: char) -> bool {
    c == '\'' || c.is_alphanumeric() || is_indic_mark(c)
}

fn is_indic_mark(c: char) -> bool {
    let u = c as u32;
    matches!(
        u,
        0x0900..=0x0D7F
            | 0x0E31
            | 0x0E34..=0x0E3A
            | 0x0E47..=0x0E4E
            | 0x180B..=0x180E
    ) && !matches!(c, '।' | '॥' | '॰')
}

pub fn tokenize(text: &str) -> Vec<String> {
    tokenize_with(text, &[])
}

/// Like [`tokenize`], with extra CJK/Thai surfaces (home names) for segmentation (L-13).
pub fn tokenize_with(text: &str, extra_surfaces: &[String]) -> Vec<String> {
    let folded = fold_latin(text);
    let mut tokens = if folded.chars().any(is_script_unit) { tokenize_script(&folded, extra_surfaces) } else { tokenize_latin(&folded) };
    repair_asr_off(&mut tokens);
    tokens
}

/// ASR often hears "off" as "of" ("turn of the hall light"). Rewrite when a power verb
/// is present and "off" is not (P-29).
fn repair_asr_off(tokens: &mut [String]) {
    let has_power = tokens.iter().any(|token| matches!(token.as_str(), "turn" | "switch" | "schalte" | "mach"));
    let has_off = tokens.iter().any(|token| token == "off" || token == "aus");
    if !has_power || has_off {
        return;
    }
    if let Some(token) = tokens.iter_mut().find(|token| token.as_str() == "of") {
        *token = "off".into();
    }
}

/// Keep signed numbers, decimals, and `h:mm` clock times as one token so the quantity
/// lexer can read −5, 19.5 / 19,5, and 1:30 instead of dropping the separator.
fn tokenize_latin(folded: &str) -> Vec<String> {
    let chars: Vec<char> = folded.chars().collect();
    let mut out = Vec::new();
    let mut buf = String::new();
    let flush = |buf: &mut String, out: &mut Vec<String>| {
        if buf.is_empty() {
            return;
        }
        let token = strip_possessive(buf);
        buf.clear();
        if !token.is_empty() {
            out.push(token);
        }
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Clitic boundary: l'entrée → entree; possessive Anna's → anna; don't → dont (L-10).
        if c == '\'' {
            let left = buf.to_lowercase();
            if CLITICS.contains(&left.as_str()) {
                buf.clear();
                i += 1;
                continue;
            }
            let possessive = chars.get(i + 1) == Some(&'s') && chars.get(i + 2).is_none_or(|next| !is_word_char(*next) && *next != '\'');
            if possessive {
                i += 2;
                flush(&mut buf, &mut out);
                continue;
            }
            // English contractions and the rest: drop the mark, keep letters in one token.
            i += 1;
            continue;
        }
        let starts_signed = buf.is_empty() && (c == '-' || c == '+') && chars.get(i + 1).and_then(|next| unicode_digit(*next)).is_some();
        let starts_number = buf.is_empty() && unicode_digit(c).is_some();
        if starts_number || starts_signed {
            flush(&mut buf, &mut out);
            let mut num = String::new();
            if starts_signed {
                num.push(c);
                i += 1;
            }
            while i < chars.len() {
                let Some(digit) = unicode_digit(chars[i]) else { break };
                num.push(char::from_digit(digit, 10).unwrap_or(chars[i]));
                i += 1;
            }
            if i < chars.len() && matches!(chars[i], '.' | ',' | ':') && chars.get(i + 1).and_then(|next| unicode_digit(*next)).is_some() {
                let sep = chars[i];
                num.push(if sep == ',' { '.' } else { sep });
                i += 1;
                while i < chars.len() {
                    let Some(digit) = unicode_digit(chars[i]) else { break };
                    num.push(char::from_digit(digit, 10).unwrap_or(chars[i]));
                    i += 1;
                }
            }
            if !num.is_empty() {
                out.push(num);
            }
            continue;
        }
        if is_word_char(c) && c != '\'' {
            buf.push(c);
            i += 1;
            continue;
        }
        flush(&mut buf, &mut out);
        i += 1;
    }
    flush(&mut buf, &mut out);
    out
}

fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{1100}'..='\u{11FF}'
            | '\u{2E80}'..='\u{2EFF}'
            | '\u{3040}'..='\u{30FF}'
            | '\u{3400}'..='\u{4DBF}'
            | '\u{4E00}'..='\u{9FFF}'
            | '\u{A960}'..='\u{A97F}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{F900}'..='\u{FAFF}'
            | '\u{FF65}'..='\u{FF9F}'
    )
}

fn is_thai(c: char) -> bool {
    matches!(c, '\u{0E00}'..='\u{0E7F}')
}

pub(crate) fn is_script_unit(c: char) -> bool {
    is_cjk(c) || is_thai(c)
}

fn tokenize_script(folded: &str, extra_surfaces: &[String]) -> Vec<String> {
    let mut raw = Vec::new();
    let mut latin = String::new();
    let flush_latin = |latin: &mut String, raw: &mut Vec<String>| {
        if !latin.is_empty() {
            // Re-run Latin rules so digits and clitics inside mixed text still split.
            raw.extend(tokenize_latin(latin));
            latin.clear();
        }
    };
    for c in folded.chars() {
        if is_script_unit(c) {
            flush_latin(&mut latin, &mut raw);
            raw.push(c.to_string());
            continue;
        }
        if is_word_char(c) || c == '\'' || unicode_digit(c).is_some() || c == '-' || c == '+' {
            latin.push(c);
            continue;
        }
        flush_latin(&mut latin, &mut raw);
    }
    flush_latin(&mut latin, &mut raw);
    longest_match(&raw, extra_surfaces)
}

fn longest_match(parts: &[String], extra_surfaces: &[String]) -> Vec<String> {
    let cat = &catalog();
    let max_look = cat.max_script_surface_chars().max(extra_max_chars(extra_surfaces));
    let mut out = Vec::new();
    let mut i = 0;
    while i < parts.len() {
        if parts[i].chars().count() != 1 || !parts[i].chars().next().is_some_and(is_script_unit) {
            out.push(parts[i].clone());
            i += 1;
            continue;
        }
        let mut best = 1;
        let mut acc = String::new();
        let limit = parts.len().min(i + max_look);
        for (offset, part) in parts[i..limit].iter().enumerate() {
            if part.chars().count() != 1 || !part.chars().next().is_some_and(is_script_unit) {
                break;
            }
            acc.push_str(part);
            if cat.knows_surface(&acc) || extra_surfaces.iter().any(|surface| surface == &acc) {
                best = offset + 1;
            }
        }
        out.push(parts[i..i + best].join(""));
        i += best;
    }
    out
}

fn extra_max_chars(extra: &[String]) -> usize {
    extra
        .iter()
        .filter(|word| !word.is_empty() && word.chars().all(is_script_unit))
        .map(|word| word.chars().count())
        .max()
        .unwrap_or(1)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::{tokenize, tokenize_with};
    use crate::parse::normalize::fold::fold_latin;

    #[test]
    fn latin_tokenize_stays_space_split() {
        assert_eq!(tokenize("Licht im Wohnzimmer an"), vec!["licht", "im", "wohnzimmer", "an"]);
        assert_eq!(tokenize("Turn on the kitchen light"), vec!["turn", "on", "the", "kitchen", "light"]);
    }

    #[test]
    fn fullwidth_latin_and_asr_of_become_off() {
        assert_eq!(tokenize("ｔｕｒｎ ｏｆｆ ｔｈｅ ｈａｌｌ ｌｉｇｈｔ"), vec!["turn", "off", "the", "hall", "light"]);
        assert_eq!(tokenize("turn of the hall light"), vec!["turn", "off", "the", "hall", "light"]);
    }

    #[test]
    fn latin_tokenize_keeps_signed_decimal_and_clock() {
        assert_eq!(tokenize("-5 degrees"), vec!["-5", "degrees"]);
        assert_eq!(tokenize("19.5"), vec!["19.5"]);
        assert_eq!(tokenize("19,5 Grad"), vec!["19.5", "grad"]);
        assert_eq!(tokenize("Timer auf 1:30"), vec!["timer", "auf", "1:30"]);
        assert_eq!(tokenize("R2D2 soll saugen"), vec!["r2d2", "soll", "saugen"]);
        assert_eq!(tokenize("lamp1"), vec!["lamp1"]);
    }

    #[test]
    fn elision_splits_clitic_from_noun() {
        assert_eq!(tokenize("l'entrée"), vec!["entree"]);
        assert_eq!(tokenize("Allume la lumière de l'entrée"), vec!["allume", "la", "lumiere", "de", "entree"]);
        assert_eq!(tokenize("dell'ingresso"), vec!["ingresso"]);
        assert_eq!(tokenize("d'acqua"), vec!["acqua"]);
        assert_eq!(tokenize("Anna's Office"), vec!["anna", "office"]);
        assert_eq!(tokenize("don't open the garage"), vec!["dont", "open", "the", "garage"]);
    }

    #[test]
    fn unicode_digits_become_ascii() {
        assert_eq!(tokenize("٥٠"), vec!["50"]);
        assert_eq!(tokenize("۵۰"), vec!["50"]);
        assert_eq!(tokenize("५०"), vec!["50"]);
        assert_eq!(tokenize("５０"), vec!["50"]);
    }

    #[test]
    fn script_yes_words_rejoin_from_known_surfaces() {
        let _ja = crate::lang::bind(&["ja".into()]);
        assert_eq!(tokenize("はい"), vec!["はい"]);
        drop(_ja);
        let _th = crate::lang::bind(&["th".into()]);
        assert_eq!(tokenize("ใช่"), vec!["ใช่"]);
    }

    #[test]
    fn home_name_joins_outside_pack_catalog() {
        let _ja = crate::lang::bind(&["ja".into()]);
        let room = fold_latin("洗面所");
        assert_ne!(tokenize("洗面所"), vec![room.clone()]);
        assert_eq!(tokenize_with("洗面所", &[room.clone()]), vec![room]);
    }

    #[test]
    fn long_unknown_cjk_stays_under_a_fixed_budget() {
        let _zh = crate::lang::bind(&["zh-CN".into()]);
        let text: String = std::iter::repeat_n('灯', 2000).collect();
        let started = std::time::Instant::now();
        let tokens = tokenize(&text);
        let elapsed = started.elapsed();
        assert_eq!(tokens.len(), 2000);
        assert!(elapsed.as_millis() < 200, "R-07: 2000 CJK chars took {} ms", elapsed.as_millis());
    }

    #[test]
    fn indic_virama_stays_inside_the_word() {
        assert_eq!(tokenize("জ্বালাও আলো বসার"), vec!["জ্বালাও", "আলো", "বসার"]);
        assert_eq!(tokenize("जलाओ बत्ती बैठक"), vec!["जलाओ", "बत्ती", "बैठक"]);
    }
}
