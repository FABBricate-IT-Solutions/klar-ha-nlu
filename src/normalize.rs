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
            other => out.extend(other.to_lowercase()),
        }
    }
    out
}

pub fn fold_umlaut(s: &str) -> String {
    fold_latin(s)
}

pub fn tokenize(text: &str) -> Vec<String> {
    fold_latin(text)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

pub fn strip_fillers(tokens: &[String]) -> Vec<String> {
    let cat = catalog();
    tokens
        .iter()
        .enumerate()
        .filter(|(i, t)| {
            if cat
                .strip_pairs
                .iter()
                .any(|(w, nxt)| t.as_str() == *w && tokens.get(i + 1).is_some_and(|n| n == nxt))
            {
                return false;
            }
            if article_one(t) && tokens.get(i + 1).is_some_and(|n| is_time_unit(n)) {
                return true;
            }
            if cat.is_action_keep(t) {
                return true;
            }
            if cat.keep_after.iter().any(|(prev, keep)| {
                t.as_str() == *keep
                    && tokens
                        .get(i.saturating_sub(1))
                        .is_some_and(|p| prev.contains(&p.as_str()))
            }) {
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
    fold_latin(s)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

pub(crate) fn is_time_unit(token: &str) -> bool {
    matches!(
        token,
        "minute"
            | "minutes"
            | "minuten"
            | "hour"
            | "hours"
            | "stunde"
            | "stunden"
            | "second"
            | "seconds"
            | "sekunde"
            | "sekunden"
    )
}

pub(crate) fn article_one(token: &str) -> bool {
    matches!(token, "eine" | "ein" | "einen" | "einer" | "a" | "an")
}
