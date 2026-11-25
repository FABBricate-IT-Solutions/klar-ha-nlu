use crate::lang::catalog;
use crate::types::AreaRec;

/// Fold German umlauts and common European accents so packs can list ASCII tokens.
/// Full-width Latin (ASR / mobile keyboards) is folded to ASCII first (P-29).
pub fn fold_latin(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    // Lower case first: "Éteins" at the start of a sentence folds like "éteins".
    for c in s.chars().flat_map(char::to_lowercase) {
        let c = match c {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(u32::from(c) - 0xFEE0).unwrap_or(c),
            other => other,
        };
        match c {
            // Phones type curly apostrophes.
            '\u{2019}' | '\u{2018}' => out.push('\''),
            'ä' | 'Ä' => out.push_str("ae"),
            'ö' | 'Ö' => out.push_str("oe"),
            'ü' | 'Ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            'à' | 'á' | 'â' | 'ã' | 'å' | 'ā' | 'ă' => out.push('a'),
            'ç' | 'č' | 'ć' => out.push('c'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' => out.push('i'),
            'ñ' | 'ń' | 'ň' => out.push('n'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ø' | 'ō' | 'ő' => out.push('o'),
            'ù' | 'ú' | 'û' | 'ū' | 'ű' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            'ž' | 'ź' | 'ż' => out.push('z'),
            'š' | 'ś' => out.push('s'),
            'ł' => out.push('l'),
            'đ' => out.push('d'),
            'æ' => out.push_str("ae"),
            'œ' => out.push_str("oe"),
            'ı' => out.push('i'),
            'ş' | 'Ş' => out.push('s'),
            'ğ' | 'Ğ' => out.push('g'),
            'ț' | 'Ț' => out.push('t'),
            'ș' | 'Ș' => out.push('s'),
            other => out.extend(other.to_lowercase()),
        }
    }
    out
}

pub fn fold_umlaut(s: &str) -> String {
    fold_latin(s)
}

/// "Anna's" → "anna", the same for speech and for names, so "Anna's Office" matches.
fn drop_apostrophes(word: &str) -> String {
    let word = if word.len() > 2 && (word.ends_with("'s") || word.ends_with("'S")) { &word[..word.len() - 2] } else { word };
    word.replace('\'', "")
}

/// A device or room name folded the way `tokenize` treats speech.
pub fn fold_label(s: &str) -> String {
    fold_latin(s).split(' ').map(drop_apostrophes).collect::<Vec<_>>().join(" ")
}

/// Home Assistant slugify strips marks (`ü`→`u`). Klar's pack fold uses digraphs (`ü`→`ue`).
pub fn fold_marks(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'ä' | 'Ä' | 'à' | 'á' | 'â' | 'ã' | 'å' | 'ā' | 'ă' => out.push('a'),
            'ö' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ø' | 'ō' | 'ő' => out.push('o'),
            'ü' | 'Ü' | 'ù' | 'ú' | 'û' | 'ū' | 'ű' => out.push('u'),
            'ß' => out.push_str("ss"),
            'ç' | 'č' | 'ć' => out.push('c'),
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' | 'ı' => out.push('i'),
            'ñ' | 'ń' | 'ň' => out.push('n'),
            'ý' | 'ÿ' => out.push('y'),
            'ž' | 'ź' | 'ż' => out.push('z'),
            'š' | 'ś' | 'ş' | 'Ş' => out.push('s'),
            'ł' => out.push('l'),
            'đ' => out.push('d'),
            'æ' => out.push('a'),
            'œ' => out.push('o'),
            'ğ' | 'Ğ' => out.push('g'),
            'ț' | 'Ț' => out.push('t'),
            'ș' | 'Ș' => out.push('s'),
            other => out.extend(other.to_lowercase()),
        }
    }
    out
}

/// Equal when one side used HA's `ü`→`u` slug and the other Klar's `ü`→`ue` fold.
pub fn umlaut_eq(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (mut i, mut j) = (0usize, 0usize);
    while i < a.len() && j < b.len() {
        if a[i] != b[j] {
            return false;
        }
        if matches!(a[i], 'a' | 'o' | 'u') {
            let a_e = a.get(i + 1) == Some(&'e');
            let b_e = b.get(j + 1) == Some(&'e');
            if a_e != b_e {
                if a_e {
                    i += 1;
                } else {
                    j += 1;
                }
            }
        }
        i += 1;
        j += 1;
    }
    i == a.len() && j == b.len()
}

/// Letters plus virama / matras / harakat so Indic and pointed Arabic stay one token.
pub(crate) fn is_word_char(c: char) -> bool {
    c == '\'' || c == '\u{200C}' || c == '\u{200D}' || c.is_alphanumeric() || is_word_mark(c)
}

fn is_word_mark(c: char) -> bool {
    let u = c as u32;
    matches!(
        u,
        0x0300..=0x036F
            | 0x0483..=0x0489
            | 0x0591..=0x05C7
            | 0x0610..=0x061A
            | 0x064B..=0x065F
            | 0x0670
            | 0x06D6..=0x06ED
            | 0x0900..=0x0D7F
            | 0x0E31
            | 0x0E34..=0x0E3A
            | 0x0E47..=0x0E4E
            | 0x180B..=0x180E
            | 0xFE20..=0xFE2F
    ) && !matches!(c, '।' | '॥' | '॰')
}

pub fn tokenize(text: &str) -> Vec<String> {
    let folded = fold_latin(text);
    let mut tokens = if folded.chars().any(is_script_unit) { tokenize_script(&folded) } else { tokenize_latin(&folded) };
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
        let token = drop_apostrophes(buf);
        buf.clear();
        if !token.is_empty() {
            out.push(token);
        }
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // Only open a number token at a boundary. Digits inside names ("R2D2", "lamp1") stay
        // in the word; a leading '-'/'+' before a digit still starts a signed number.
        let starts_signed = buf.is_empty() && (c == '-' || c == '+') && chars.get(i + 1).is_some_and(|next| next.is_ascii_digit());
        let starts_number = buf.is_empty() && c.is_ascii_digit();
        if starts_number || starts_signed {
            flush(&mut buf, &mut out);
            let mut num = String::new();
            if starts_signed {
                num.push(c);
                i += 1;
            }
            while i < chars.len() && chars[i].is_ascii_digit() {
                num.push(chars[i]);
                i += 1;
            }
            if i < chars.len() && matches!(chars[i], '.' | ',' | ':') && chars.get(i + 1).is_some_and(|next| next.is_ascii_digit()) {
                let sep = chars[i];
                num.push(if sep == ',' { '.' } else { sep });
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    num.push(chars[i]);
                    i += 1;
                }
            }
            if !num.is_empty() {
                out.push(num);
            }
            continue;
        }
        if is_word_char(c) {
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

fn tokenize_script(folded: &str) -> Vec<String> {
    let mut raw = Vec::new();
    let mut latin = String::new();
    let flush_latin = |latin: &mut String, raw: &mut Vec<String>| {
        if !latin.is_empty() {
            raw.push(std::mem::take(latin));
        }
    };
    for c in folded.chars() {
        if is_script_unit(c) {
            flush_latin(&mut latin, &mut raw);
            raw.push(c.to_string());
            continue;
        }
        if is_word_char(c) {
            latin.push(c);
            continue;
        }
        flush_latin(&mut latin, &mut raw);
    }
    flush_latin(&mut latin, &mut raw);
    longest_match(&raw)
}

fn longest_match(parts: &[String]) -> Vec<String> {
    let cat = catalog();
    // R-07: only look ahead as far as the longest CJK/Thai catalog form.
    let max_look = cat.max_script_surface_chars();
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
            if cat.knows_surface(&acc) {
                best = offset + 1;
            }
        }
        out.push(parts[i..i + best].join(""));
        i += best;
    }
    out
}

/// Like [`strip_fillers`], but a multi-word area name that contains a filler ("living room",
/// "tv room") is first collapsed to one compact `area_id` token ("livingroom"), the form area
/// matching already compares against. Otherwise "room" is stripped and the name is lost, and
/// its other words ("tv", "play", "front") read as commands or devices.
pub fn strip_fillers_for_areas(tokens: &[String], areas: &[AreaRec]) -> Vec<String> {
    strip_fillers(&collapse_area_names(tokens, areas))
}

fn collapse_area_names(tokens: &[String], areas: &[AreaRec]) -> Vec<String> {
    let cat = catalog();
    let mut names: Vec<(Vec<String>, String)> = areas
        .iter()
        .flat_map(|area| {
            let labels = std::iter::once(area.name.as_str()).chain(area.aliases.iter().map(String::as_str));
            let token = compact(&area.area_id);
            labels.map(move |label| (tokenize(label), token.clone()))
        })
        .filter(|(words, _)| words.len() > 1 && words.iter().any(|w| cat.is_filler(w)) && !words.iter().all(|w| cat.is_filler(w)))
        .collect();
    // Longest names first, so "guest bed room" wins over "bed room".
    names.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        match names.iter().find(|(words, _)| tokens[i..].starts_with(words)) {
            Some((words, token)) => {
                out.push(token.clone());
                i += words.len();
            }
            None => {
                out.push(tokens[i].clone());
                i += 1;
            }
        }
    }
    out
}

pub fn strip_fillers(tokens: &[String]) -> Vec<String> {
    let cat = catalog();
    tokens
        .iter()
        .enumerate()
        .filter(|(i, t)| {
            if cat.strip_pairs.iter().any(|(w, nxt)| t.as_str() == *w && tokens.get(i + 1).is_some_and(|n| n == nxt)) {
                return false;
            }
            if article_one(t) && tokens.get(i + 1).is_some_and(|n| is_time_unit(n)) {
                return true;
            }
            if cat.is_action_keep(t) {
                return true;
            }
            if cat
                .keep_after
                .iter()
                .any(|(prev, keep)| t.as_str() == *keep && tokens.get(i.saturating_sub(1)).is_some_and(|p| prev.contains(&p.as_str())))
            {
                return true;
            }
            !cat.is_filler(t)
        })
        .map(|(_, t)| t.clone())
        .collect()
}

pub fn join_tokens(tokens: &[String]) -> String {
    tokens.join(" ")
}

pub fn compact(s: &str) -> String {
    fold_latin(s).chars().filter(|c| is_word_char(*c) && *c != '\'').collect()
}

/// "Schlafzimmern" / "Wohnzimmers" / "bedrooms" match the room stem.
pub fn inflected_eq(token: &str, label: &str) -> bool {
    label.len() >= 6
        && catalog().morphology.effective_room_suffixes().iter().any(|suffix| token.strip_suffix(suffix).is_some_and(|stem| stem == label))
}

pub(crate) fn is_time_unit(token: &str) -> bool {
    let cat = catalog();
    cat.hours().contains(token)
        || cat.minutes().contains(token)
        || cat.seconds().contains(token)
        || matches!(token, "hour" | "hours" | "hrs" | "minute" | "minutes" | "min" | "mins" | "second" | "seconds" | "sec" | "secs")
}

pub(crate) fn article_one(token: &str) -> bool {
    catalog().article_one().contains(token)
}

#[cfg(test)]
mod tests {
    use super::{fold_latin, tokenize};

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
    fn fold_latin_keeps_turkish_and_romanian() {
        assert_eq!(fold_latin("ışık"), "isik");
        assert_eq!(fold_latin("și"), "si");
    }

    #[test]
    fn latin_tokenize_is_unchanged_when_text_has_no_cjk() {
        assert_eq!(tokenize("allume la lumiere salon"), vec!["allume", "la", "lumiere", "salon"]);
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
        use super::compact;
        assert_eq!(compact("জ্বালাও"), "জ্বালাও");
        assert_eq!(compact("बत्ती"), "बत्ती");
    }

    #[test]
    fn compact_keeps_letters_outside_ascii() {
        use super::compact;
        assert_eq!(compact("Wohnzimmer!"), "wohnzimmer");
        assert_eq!(compact("филм"), "филм");
        assert_ne!(compact("филм"), compact("антре"));
        assert_eq!(compact("فيلم"), "فيلم");
        assert_eq!(compact("סרט"), "סרט");
    }

    #[test]
    fn umlaut_eq_matches_ha_slug_and_klar_fold() {
        use super::{fold_marks, umlaut_eq};
        assert_eq!(fold_marks("Küche"), "kuche");
        assert_eq!(fold_marks("Büro"), "buro");
        assert_eq!(fold_marks("Gästezimmer"), "gastezimmer");
        assert!(umlaut_eq("kueche", "kuche"));
        assert!(umlaut_eq("kuche", "kueche"));
        assert!(umlaut_eq("buero", "buro"));
        assert!(umlaut_eq("gaestezimmer", "gastezimmer"));
        assert!(!umlaut_eq("kueche", "wohnzimmer"));
        assert!(!umlaut_eq("quelle", "kuche"));
    }
}
