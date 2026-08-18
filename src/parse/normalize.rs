use crate::lang::catalog;

/// Fold German umlauts and common European accents so packs can list ASCII tokens.
pub fn fold_latin(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
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

pub fn tokenize(text: &str) -> Vec<String> {
    let folded = fold_latin(text);
    if folded.chars().any(is_script_unit) {
        tokenize_script(&folded)
    } else {
        folded.split(|c: char| !c.is_alphanumeric()).filter(|t| !t.is_empty()).map(|t| t.to_string()).collect()
    }
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

fn is_script_unit(c: char) -> bool {
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
        if c.is_alphanumeric() {
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
        for (offset, part) in parts[i..].iter().enumerate() {
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
    fold_latin(s).chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// "Schlafzimmern" / "Wohnzimmers" / "bedrooms" match the room stem.
pub fn inflected_eq(token: &str, label: &str) -> bool {
    label.len() >= 6
        && catalog().morphology.effective_room_suffixes().iter().any(|suffix| token.strip_suffix(suffix).is_some_and(|stem| stem == label))
}

pub(crate) fn is_time_unit(token: &str) -> bool {
    let cat = catalog();
    cat.hours.contains(token) || cat.minutes.contains(token) || cat.seconds.contains(token)
}

pub(crate) fn article_one(token: &str) -> bool {
    catalog().article_one.contains(token)
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
    fn fold_latin_keeps_turkish_and_romanian() {
        assert_eq!(fold_latin("ışık"), "isik");
        assert_eq!(fold_latin("și"), "si");
    }

    #[test]
    fn latin_tokenize_is_unchanged_when_text_has_no_cjk() {
        assert_eq!(tokenize("allume la lumiere salon"), vec!["allume", "la", "lumiere", "salon"]);
    }
}
