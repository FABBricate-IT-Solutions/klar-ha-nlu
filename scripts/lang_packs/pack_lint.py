"""Pack quality gates for CI (package 4.4): script, placeholders, numbers,
EN/DE leftovers, identical open/close answers, household not verb-built.
"""

from __future__ import annotations

import re
import unicodedata
from collections import Counter
from pathlib import Path

from lang_packs.native_colors import EN_COLOR_WORDS

HANDWRITTEN = frozenset({"de", "en"})
REGIONAL_OK_UNDE = frozenset({"de", "de_at", "de_ch"})
EN_CHAT_STUBS = frozenset({"mood", "tell", "idea", "why", "weather", "stop", "news"})
EN_NATIVE_NAMES = frozenset(
    {
        "Hindi",
        "Bangla",
        "Gujarati",
        "Kannada",
        "Malayalam",
        "Marathi",
        "Tamil",
        "Telugu",
        "Punjabi",
        "Nepali",
        "Armenian",
        "Georgian",
        "Mongolian",
        "Galician",
    }
)
ACTION_FIELDS = (
    "lock",
    "unlock",
    "cover_open",
    "cover_close",
    "turn_on",
    "turn_off",
    "done",
)
OPPOSITE_PAIRS = (
    ("cover_open", "cover_close"),
    ("lock", "unlock"),
)
SCRIPT_DIRS = {
    "bg": "CYRILLIC",
    "sr": "CYRILLIC",
    "uk": "CYRILLIC",
    "mn": "CYRILLIC",
    "el": "GREEK",
    "zh_cn": "CJK",
    "zh_tw": "CJK",
    "zh_hk": "CJK",
    "ja": "CJK",
    "ko": "HANGUL",
    "th": "THAI",
    "ar": "ARABIC",
    "fa": "ARABIC",
    "ur": "ARABIC",
    "he": "HEBREW",
    "hi": "DEVANAGARI",
    "mr": "DEVANAGARI",
    "ne": "DEVANAGARI",
    "bn": "BENGALI",
    "gu": "GUJARATI",
    "kn": "KANNADA",
    "ml": "MALAYALAM",
    "ta": "TAMIL",
    "te": "TELUGU",
    "pa": "GURMUKHI",
    "hy": "ARMENIAN",
    "ka": "GEORGIAN",
}
FIELD_RE = re.compile(r'(\w+):\s*"((?:\\.|[^"\\])*)"')
LIST_RE = re.compile(r"(\w+):\s*&\[(.*?)\]", re.S)
NUM_RE = re.compile(r'\("([^"]+)",\s*(\d+)\)')
COLOR_RE = re.compile(r'\("([^"]+)",\s*"([^"]+)"\)')
VERB_RE = re.compile(r'\("([^"]+)",\s*VerbKind::(\w+)\)')
NATIVE_RE = re.compile(r'native_name:\s*"([^"]+)"')
CODE_RE = re.compile(r'code:\s*"([^"]+)"')


def script_of(word: str) -> str:
    counts: Counter[str] = Counter()
    for ch in word:
        if not ch.isalpha():
            continue
        name = unicodedata.name(ch, "")
        if name.startswith("LATIN") or name.startswith("FULLWIDTH LATIN"):
            counts["LATIN"] += 1
        elif "CJK" in name or "HIRAGANA" in name or "KATAKANA" in name:
            counts["CJK"] += 1
        elif name:
            counts[name.split()[0]] += 1
    return counts.most_common(1)[0][0] if counts else "NONE"


def _pack_text(pack: Path) -> str:
    return "\n".join(path.read_text(encoding="utf-8") for path in sorted(pack.glob("*.rs")))


def _speech_map(text: str) -> dict[str, str]:
    block = text
    if "Speech {" in text:
        start = text.find("Speech {")
        block = text[start : start + 12000]
    return {key: value for key, value in FIELD_RE.findall(block) if key in ACTION_FIELDS or key in {a for pair in OPPOSITE_PAIRS for a in pair}}


def _list_field(text: str, name: str) -> list[str]:
    for key, body in LIST_RE.findall(text):
        if key == name:
            return re.findall(r'"([^"]+)"', body)
    return []


def _alpha_outside_placeholders(template: str) -> bool:
    stripped = re.sub(r"\{\w+\}", "", template)
    return any(ch.isalpha() for ch in stripped)


def lint_packs(packs_dir: Path, registry: Path) -> list[str]:
    """Return human-readable defect lines; empty means green."""
    errors: list[str] = []
    en_speech = _speech_map(_pack_text(packs_dir / "en")) if (packs_dir / "en").is_dir() else {}
    en_placeholders = {key: set(re.findall(r"\{(\w+)\}", value)) for key, value in en_speech.items()}
    native_by_code = _registry_natives(registry)

    for pack in sorted(path for path in packs_dir.iterdir() if path.is_dir()):
        name = pack.name
        text = _pack_text(pack)
        speech = _speech_map(text)
        errors.extend(_lint_speech(name, speech, en_placeholders))
        errors.extend(_lint_household_chat(name, text))
        errors.extend(_lint_numbers(name, text, pack))
        errors.extend(_lint_script_colors(name, text))
        if name not in REGIONAL_OK_UNDE and "und" in _list_field(text, "fillers"):
            errors.append(f"{name}: filler still has German und")
        tag = name.replace("_", "-") if name in {"de_at", "de_ch", "en_gb", "pt_br", "sr_latn", "zh_cn", "zh_tw", "zh_hk"} else name
        native = native_by_code.get(tag) or native_by_code.get(name)
        if native in EN_NATIVE_NAMES:
            errors.append(f"{name}: native_name is English {native!r}")
    return errors


def _registry_natives(registry: Path) -> dict[str, str]:
    if not registry.is_file():
        return {}
    text = registry.read_text(encoding="utf-8")
    out: dict[str, str] = {}
    for block in re.finditer(r"LangMeta\s*\{([^}]+)\}", text):
        body = block.group(1)
        code_m = CODE_RE.search(body)
        native_m = NATIVE_RE.search(body)
        if code_m and native_m:
            out[code_m.group(1)] = native_m.group(1)
    return out


def _lint_speech(name: str, speech: dict[str, str], en_placeholders: dict[str, set[str]]) -> list[str]:
    errors: list[str] = []
    if name in HANDWRITTEN:
        return errors
    for field in ACTION_FIELDS:
        value = speech.get(field)
        if value is None:
            continue
        if not _alpha_outside_placeholders(value):
            errors.append(f"{name}: speech.{field} is placeholder-only ({value!r})")
        want = en_placeholders.get(field)
        if want is not None:
            got = set(re.findall(r"\{(\w+)\}", value))
            if got != want and field in {"lock", "unlock", "cover_open", "cover_close", "done"}:
                # done may use {name}; lock/cover use {target} like en
                if field == "done" and "name" not in got:
                    errors.append(f"{name}: speech.done missing {{name}}")
                if field != "done" and "target" not in got and "name" not in got:
                    errors.append(f"{name}: speech.{field} missing expected placeholders")
    for left, right in OPPOSITE_PAIRS:
        if speech.get(left) and speech.get(left) == speech.get(right):
            errors.append(f"{name}: speech.{left} identical to {right}")
    return errors


def _lint_household_chat(name: str, text: str) -> list[str]:
    errors: list[str] = []
    if name in HANDWRITTEN:
        return errors
    weekday = _list_field(text, "weekday")
    if not weekday:
        errors.append(f"{name}: household.weekday is empty")
    clock = _list_field(text, "clock")
    weather = _list_field(text, "weather")
    on_verbs = {word for word, kind in VERB_RE.findall(text) if kind == "On"}
    greet = _list_field(text, "greet")
    if greet and set(greet) <= on_verbs:
        errors.append(f"{name}: chat.greet is only On verbs {greet}")
    for field, words in (("feeling", _list_field(text, "feeling")), ("tell", _list_field(text, "tell")), ("advice", _list_field(text, "advice"))):
        stubs = [word for word in words if word in EN_CHAT_STUBS]
        if stubs and name not in {"en", "en_gb"}:
            errors.append(f"{name}: chat.{field} still has EN stubs {stubs}")
    # Verb-built household: single token that is also an action verb, or "query off set" style.
    verbs = {word for word, _kind in VERB_RE.findall(text)}
    for field, words in (("clock", clock), ("weather", weather), ("explain", _list_field(text, "explain"))):
        for phrase in words:
            parts = phrase.split()
            if len(parts) >= 2 and all(part in verbs for part in parts):
                errors.append(f"{name}: household.{field} looks verb-built ({phrase!r})")
    return errors


def _lint_numbers(name: str, text: str, pack: Path) -> list[str]:
    errors: list[str] = []
    chunk = text
    numbers_file = pack / "numbers.rs"
    if numbers_file.is_file():
        chunk = numbers_file.read_text(encoding="utf-8")
    rows = NUM_RE.findall(chunk)
    words = [word for word, _value in rows]
    dup = sorted({word for word, count in Counter(words).items() if count > 1})
    if dup:
        errors.append(f"{name}: duplicate number words {dup[:8]}")
    # Spaced warm-white color aliases are OK; action verbs must stay single tokens.
    action = {"On", "Off", "Open", "Close", "Set", "Query", "Lock", "Unlock"}
    multi_verbs = sorted(
        {word for word, kind in VERB_RE.findall(text) if " " in word and kind in action}
    )
    if multi_verbs:
        errors.append(f"{name}: multi-word verb keys {multi_verbs[:8]}")
    return errors


def _lint_script_colors(name: str, text: str) -> list[str]:
    errors: list[str] = []
    if name not in SCRIPT_DIRS:
        return errors
    colors = COLOR_RE.findall(text)
    match = re.search(r"colors:\s*&\[(.*?)\]", text, re.S)
    if match:
        colors = COLOR_RE.findall(match.group(1))
    english = [word for word, _canon in colors if word.casefold() in EN_COLOR_WORDS]
    if english:
        errors.append(f"{name}: English color words {english[:8]}")
    # Verbs are emitted sorted; measure the whole action lexicon, not the first rows.
    verbs = [word for word, _kind in VERB_RE.findall(text)]
    lights = _list_field(text, "light_nouns")
    sample = verbs + lights + [word for word, _canon in colors]
    if sample:
        latin = sum(1 for word in sample if script_of(word) == "LATIN")
        if latin / len(sample) > 0.6:
            errors.append(f"{name}: core lexicon {100 * latin / len(sample):.0f}% Latin (want native script)")
    return errors
