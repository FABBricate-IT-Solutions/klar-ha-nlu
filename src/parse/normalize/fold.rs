//! NFKC, lowercasing, then fold per script so pack tokens and speech meet.

use unicode_normalization::UnicodeNormalization;

/// Fold speech and lexicon forms the same way. Latin keeps German digraphs for
/// äöüß; every other mark is stripped to its base letter (L-07, L-08, L-09).
pub fn fold_latin(s: &str) -> String {
    let nfkc: String = s.nfkc().collect();
    let mut out = String::with_capacity(nfkc.len());
    for c in nfkc.chars().flat_map(char::to_lowercase) {
        fold_char(c, &mut out);
    }
    out
}

pub fn fold_umlaut(s: &str) -> String {
    fold_latin(s)
}

/// A device or room name folded the way `tokenize` treats speech.
pub fn fold_label(s: &str) -> String {
    fold_latin(s).split(' ').map(strip_possessive).collect::<Vec<_>>().join(" ")
}

/// Home Assistant slugify strips marks (`ü`→`u`). Klar's pack fold uses digraphs (`ü`→`ue`).
pub fn fold_marks(s: &str) -> String {
    let nfkc: String = s.nfkc().collect();
    let mut out = String::with_capacity(nfkc.len());
    for c in nfkc.chars().flat_map(char::to_lowercase) {
        match c {
            'ä' => out.push('a'),
            'ö' => out.push('o'),
            'ü' => out.push('u'),
            'ß' => out.push_str("ss"),
            other => fold_char_markless(other, &mut out),
        }
    }
    out
}

/// Equal when one side used HA's `ü`→`u` slug and the other Klar's `ü`→`ue` fold.
/// Does not treat a bare word as equal to itself plus a trailing „e“ (P-34).
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
                // Only skip when the side without „e“ still has letters after the vowel,
                // so "kueche"≈"kuche" works but "to"≉"toe" (P-34).
                let single_has_more = if a_e { b.get(j + 1).is_some() } else { a.get(i + 1).is_some() };
                if !single_has_more {
                    return false;
                }
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

fn fold_char(c: char, out: &mut String) {
    // Unicode digits (Arabic-Indic, Devanagari, fullwidth, …) → ASCII (L-11).
    // `char::to_digit` is ASCII-only on this toolchain; map the Nd ranges we meet.
    if let Some(digit) = unicode_digit(c) {
        out.push(char::from_digit(digit, 10).unwrap_or(c));
        return;
    }
    match c {
        // Invisible / soft breaks (L-14). Soft hyphen is dropped; ASCII `-` stays for tokenize.
        '\u{00AD}' | '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
        // Apostrophe-like (phones, U+02BC) → ASCII apostrophe for clitic splits (L-14).
        '\u{2019}' | '\u{2018}' | '\u{02BC}' | '\u{2032}' | '`' => out.push('\''),
        // German digraphs (kept for de packs and typed ASCII).
        'ä' => out.push_str("ae"),
        'ö' => out.push_str("oe"),
        'ü' => out.push_str("ue"),
        'ß' => out.push_str("ss"),
        'æ' => out.push_str("ae"),
        'œ' => out.push_str("oe"),
        // Arabic / Hebrew script fold (L-12). Digits already handled above.
        c if is_arabic_letter_or_mark(c) => fold_arabic(c, out),
        c if is_hebrew_letter_or_mark(c) => fold_hebrew(c, out),
        other => fold_char_markless(other, out),
    }
}

/// Decimal value for common Unicode Nd digits. ASCII uses `to_digit`.
pub(super) fn unicode_digit(c: char) -> Option<u32> {
    if let Some(digit) = c.to_digit(10) {
        return Some(digit);
    }
    let u = c as u32;
    const RANGES: &[(u32, u32)] = &[
        (0x0660, 0x0669), // Arabic-Indic
        (0x06F0, 0x06F9), // Extended Arabic-Indic
        (0x0966, 0x096F), // Devanagari
        (0x09E6, 0x09EF), // Bengali
        (0x0A66, 0x0A6F), // Gurmukhi
        (0x0AE6, 0x0AEF), // Gujarati
        (0x0B66, 0x0B6F), // Oriya
        (0x0BE6, 0x0BEF), // Tamil
        (0x0C66, 0x0C6F), // Telugu
        (0x0CE6, 0x0CEF), // Kannada
        (0x0D66, 0x0D6F), // Malayalam
        (0x0E50, 0x0E59), // Thai
        (0x0ED0, 0x0ED9), // Lao
        (0x0F20, 0x0F29), // Tibetan
        (0x1040, 0x1049), // Myanmar
        (0x17E0, 0x17E9), // Khmer
        (0x1810, 0x1819), // Mongolian
        (0xFF10, 0xFF19), // Fullwidth
    ];
    for &(start, end) in RANGES {
        if (start..=end).contains(&u) {
            return Some(u - start);
        }
    }
    None
}

fn fold_char_markless(c: char, out: &mut String) {
    // Full-width Latin already becomes ASCII via NFKC; keep a fallback.
    let c = match c {
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(u32::from(c) - 0xFEE0).unwrap_or(c),
        other => other,
    };
    if is_combining_mark(c) || c == '\u{0640}' {
        // Drop combining marks and tatweel; base letter already emitted (L-09, L-12).
        return;
    }
    if let Some(base) = strip_latin_or_greek(c) {
        out.push(base);
        return;
    }
    match c {
        'ł' => out.push('l'),
        'đ' | 'ð' => out.push('d'),
        'ø' => out.push('o'),
        'ı' => out.push('i'),
        'ſ' => out.push('s'),
        'ŋ' => out.push('n'),
        'ĸ' => out.push('k'),
        other => out.push(other),
    }
}

fn strip_latin_or_greek(c: char) -> Option<char> {
    // NFKD then drop marks covers ą ě ř, Vietnamese tones, Greek accents (L-08, L-09).
    let mut chars = c.nfkd().filter(|ch| !is_combining_mark(*ch));
    let first = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    if first == c {
        None
    } else {
        Some(first)
    }
}

fn fold_arabic(c: char, out: &mut String) {
    if c == '\u{0640}' || is_arabic_mark(c) {
        return;
    }
    // Alef variants and hamza carriers → bare alef; teh marbuta → heh (L-12).
    let mapped = match c {
        'أ' | 'إ' | 'آ' | 'ٱ' | 'ٲ' | 'ٳ' | 'ٵ' => 'ا',
        'ة' => 'ه',
        'ى' => 'ي',
        'ؤ' => 'و',
        'ئ' => 'ي',
        other => other,
    };
    out.push(mapped);
}

fn fold_hebrew(c: char, out: &mut String) {
    if is_hebrew_mark(c) {
        return;
    }
    out.push(c);
}

fn is_combining_mark(c: char) -> bool {
    let u = c as u32;
    matches!(
        u,
        0x0300..=0x036F
            | 0x0483..=0x0489
            | 0x1AB0..=0x1AFF
            | 0x1DC0..=0x1DFF
            | 0x20D0..=0x20FF
            | 0xFE20..=0xFE2F
    )
}

fn is_arabic_mark(c: char) -> bool {
    let u = c as u32;
    matches!(u, 0x0610..=0x061A | 0x064B..=0x065F | 0x0670 | 0x06D6..=0x06ED | 0x08D3..=0x08FF)
}

fn is_hebrew_mark(c: char) -> bool {
    let u = c as u32;
    matches!(u, 0x0591..=0x05C7)
}

fn is_arabic_letter_or_mark(c: char) -> bool {
    let u = c as u32;
    matches!(u, 0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF)
}

fn is_hebrew_letter_or_mark(c: char) -> bool {
    let u = c as u32;
    matches!(u, 0x0590..=0x05FF | 0xFB1D..=0xFB4F)
}

/// "Anna's" → "anna"; clitics are split in the tokenizer, not here.
pub(super) fn strip_possessive(word: &str) -> String {
    let word = if word.len() > 2 && (word.ends_with("'s") || word.ends_with("'S")) { &word[..word.len() - 2] } else { word };
    word.replace('\'', "")
}

#[cfg(test)]
mod tests {
    use super::{fold_latin, fold_marks, umlaut_eq};

    #[test]
    fn fold_latin_keeps_turkish_and_romanian() {
        assert_eq!(fold_latin("ışık"), "isik");
        assert_eq!(fold_latin("și"), "si");
    }

    #[test]
    fn german_umlauts_become_digraphs_other_marks_strip() {
        assert_eq!(fold_latin("Küche"), "kueche");
        assert_eq!(fold_latin("kök"), "koek");
        assert_eq!(fold_latin("światło"), "swiatlo");
        assert_eq!(fold_latin("đèn"), "den");
        assert_eq!(fold_latin("φῶς"), "φως");
        assert_eq!(fold_latin("ÁNO"), "ano");
    }

    #[test]
    fn nfkc_and_nfd_inputs_fold_alike() {
        assert_eq!(fold_latin("café"), "cafe");
        assert_eq!(fold_latin("cafe\u{0301}"), "cafe");
        assert_eq!(fold_latin("Café"), "cafe");
    }

    #[test]
    fn invisible_and_halfwidth_fold() {
        assert_eq!(fold_latin("a\u{200B}b"), "ab");
        assert_eq!(fold_latin("a\u{00AD}b"), "ab");
        assert_eq!(fold_latin("ｶﾀｶﾅ"), fold_latin("カタカナ"));
        assert!(fold_latin("l\u{02BC}entree").contains('\''));
    }

    #[test]
    fn arabic_hebrew_marks_strip() {
        assert_eq!(fold_latin("مَرْحَبًا"), fold_latin("مرحبا"));
        assert_eq!(fold_latin("ـبابـ"), "باب");
        assert_eq!(fold_latin("שָׁלוֹם"), fold_latin("שלום"));
        assert_eq!(fold_latin("أحمد"), "احمد");
    }

    #[test]
    fn umlaut_eq_matches_ha_slug_not_trailing_e() {
        assert_eq!(fold_marks("Küche"), "kuche");
        assert!(umlaut_eq("kueche", "kuche"));
        assert!(umlaut_eq("buero", "buro"));
        assert!(umlaut_eq("gaestezimmer", "gastezimmer"));
        assert!(!umlaut_eq("kueche", "wohnzimmer"));
        assert!(!umlaut_eq("quelle", "kuche"));
        // P-34: a word is not equal to itself plus „e“.
        assert!(!umlaut_eq("to", "toe"));
        assert!(!umlaut_eq("hallo", "halloe"));
        assert!(!umlaut_eq("raum", "raume"));
    }
}
