"""Per-locale number tables 0..=100 (and hundred stems) from sparse pack lists.

Package 4.2: expand the 0–20 / tens / 100 stems into a reference table that
round-trips through the parser. Join style is a small grammar per language;
CLDR-quality spellings are used where the sparse list already has them.
"""

from __future__ import annotations

from lang_packs.fold import fold_latin

# Tens + conj + ones ("treinta y cinco", "vinte e cinco").
SPACE_AND = frozenset(
    {
        "es",
        "pt",
        "pt-BR",
        "ca",
        "gl",
        "it",
        "ro",
        "fr",  # also gets et-specials below
    }
)

# Ones + (conj) + tens ("vijfentwintig", "femogtyve") — Germanic spoken order.
ONES_AND_TENS = frozenset({"nl", "af", "da", "nb", "sv", "is", "lb", "de-CH"})

# Tens and ones written as one token ("kaksikymmentayksi", "yirmibir").
CONCAT = frozenset(
    {
        "bg",
        "bn",
        "cs",
        "cy",
        "el",
        "et",
        "eu",
        "fa",
        "fi",
        "ga",
        "gu",
        "he",
        "hi",
        "hr",
        "hu",
        "hy",
        "id",
        "ja",
        "ka",
        "kn",
        "ko",
        "kw",
        "lt",
        "lv",
        "ml",
        "mn",
        "mr",
        "ms",
        "ne",
        "pa",
        "pl",
        "sk",
        "sl",
        "sr",
        "sr-Latn",
        "sw",
        "ta",
        "te",
        "th",
        "tr",
        "uk",
        "ur",
        "vi",
        "zh-CN",
        "zh-HK",
        "zh-TW",
        "ar",
    }
)

# French / Romance: 21, 31, … use "et" between tens and one.
ET_ONES = frozenset({"fr"})

# Conjunction used when expanding SPACE_AND / ONES_AND_TENS forms (folded).
_CONJ = {
    "es": "y",
    "pt": "e",
    "pt-BR": "e",
    "ca": "i",
    "gl": "e",
    "it": "e",
    "ro": "si",
    "fr": "et",
    "nl": "en",
    "af": "en",
    "da": "og",
    "nb": "og",
    "sv": "och",
    "is": "og",
    "lb": "an",
    "de-CH": "und",
}


def _by_value(rows: list[tuple[str, int]]) -> dict[int, str]:
    out: dict[int, str] = {}
    for word, value in rows:
        out.setdefault(int(value), fold_latin(word))
    return out


def _add(dst: dict[str, int], word: str, value: int) -> None:
    key = fold_latin(word)
    if not key:
        return
    # First spelling wins; do not overwrite a stem with a compound.
    dst.setdefault(key, value)


def _space_and(tens: str, ones: str, conj: str) -> list[str]:
    """Spoken multi-word form and a concatenated alias for hyphen-stripped input."""
    spaced = f"{tens} {conj} {ones}".strip()
    compact = f"{tens}{ones}"
    return [spaced, compact]


def _french_compound(tens: str, ones: str, ones_n: int, conj: str) -> list[str]:
    if ones_n == 1:
        return _space_and(tens, ones, conj)
    return [f"{tens}{ones}"]


def number_style_for(code: str) -> str:
    """Rust `NumberStyle` variant for a generated pack."""
    if code in SPACE_AND:
        return "TensAndOnes"
    if code in ONES_AND_TENS:
        return "OnesAndTens"
    return "ListedOnly"


def expand_number_rows(code: str, rows: list[tuple[str, int]], conj: str | None = None) -> list[tuple[str, int]]:
    """Expand sparse stems into a 0..=100 reference table (plus listed hundreds).

    de/en are hand-written and never pass through here. Regional packs only keep
    rows the parent lacks; expansion still runs on their own sparse list first.
    """
    by_val = _by_value(rows)
    out: dict[str, int] = {}
    for word, value in rows:
        _add(out, word, int(value))

    join_conj = fold_latin(conj or _CONJ.get(code, "e"))
    use_space_and = code in SPACE_AND
    use_ones_and_tens = code in ONES_AND_TENS
    use_concat = code in CONCAT

    for n in range(0, 101):
        if n in by_val:
            _add(out, by_val[n], n)
            continue
        tens = (n // 10) * 10
        ones = n % 10
        if ones == 0 or tens < 20 or tens not in by_val or ones not in by_val:
            continue
        tens_w, ones_w = by_val[tens], by_val[ones]
        if code in ET_ONES:
            # French 70/80/90 are irregular (soixante-dix, quatre-vingts, …); only expand
            # the regular decades so we do not invent "soixantedixquatre".
            if tens not in (20, 30, 40, 50, 60):
                continue
            forms = _french_compound(tens_w, ones_w, ones, join_conj)
        elif use_space_and:
            forms = _space_and(tens_w, ones_w, join_conj)
        elif use_ones_and_tens:
            # "vijf en twintig" / "vijfentwintig"
            forms = [f"{ones_w}{join_conj}{tens_w}", f"{ones_w} {join_conj} {tens_w}"]
        elif use_concat:
            forms = [f"{tens_w}{ones_w}"]
        else:
            forms = _space_and(tens_w, ones_w, join_conj)
        for form in forms:
            _add(out, form, n)

    # Hundreds stems 200..=900 when the language writes digit+hundred as one word.
    hundred = by_val.get(100)
    if hundred:
        for count in range(2, 10):
            value = count * 100
            if value in by_val or count not in by_val:
                continue
            if use_space_and and code == "fr":
                forms = [f"{by_val[count]} {hundred}", f"{by_val[count]}{hundred}"]
            elif use_space_and:
                forms = [f"{by_val[count]} {hundred}", f"{by_val[count]}{hundred}"]
            else:
                forms = [f"{by_val[count]}{hundred}"]
            for form in forms:
                _add(out, form, value)

    return sorted(out.items(), key=lambda item: (item[1], item[0]))
