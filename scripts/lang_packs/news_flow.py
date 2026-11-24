"""Follow-up, topic and request words of the news briefing, one row per generated pack.

de and en are hand-written in src/lang/packs. Their regional packs read the
same word lists from those Rust files, so the two never drift apart.
"""

from __future__ import annotations

import re
from pathlib import Path

from lang_packs.news_flow_asia import ASIA
from lang_packs.news_flow_indic import INDIC
from lang_packs.news_flow_mena import MENA
from lang_packs.news_flow_north import NORTH
from lang_packs.news_flow_slavic import SLAVIC
from lang_packs.news_flow_west import WEST
from lang_packs.news_row import words

_PACKS = Path(__file__).resolve().parents[2] / "src" / "lang" / "packs"
_GERMAN = (
    "Was gibt es heute für Nachrichten?",
    "Erzähl mir mehr über die Schulen",
    "Was ist an den Schulen passiert?",
    "Gib mir dazu alle aktuellen Nachrichten",
    "Sportnachrichten",
    "Nachrichten über Tesla",
)
_ENGLISH = (
    "What's in the news today?",
    "Tell me more about the schools",
    "What happened at the schools?",
    "Give me all the current news about that",
    "sports news",
    "news about Tesla",
)
_SWISS = (
    "Was git es hüt für Nachrichte?",
    "Verzell mer meh über d Schuele",
    "Was isch a de Schuele passiert?",
    "Gib mer dezue alli aktuelle Nachrichte",
    "Sportnachrichte",
    "Nachrichte über Tesla",
)

# Hand-written packs: only the sentences of the generated regression test live here.
HAND: dict[str, dict] = {
    "de": {"say": _GERMAN, "want": ("sport", "tesla")},
    "en": {"say": _ENGLISH, "want": ("sports", "tesla")},
}


def _strings(block: str) -> list[str]:
    return re.findall(r'"((?:[^"\\]|\\.)*)"', block)


def _hand(code: str) -> dict:
    """The follow-up lists of a hand-written pack, read from its news.rs."""
    source = (_PACKS / code / "news.rs").read_text(encoding="utf-8")

    def field(name: str) -> list[str]:
        found = re.search(rf"\b{name}: &\[(.*?)\]", source, re.S)
        return _strings(found.group(1)) if found else []

    glue = re.search(r"const GLUE: &\[&str\] = &\[(.*?)\];", source, re.S)
    topics = re.search(r"\btopics: &\[(.*?)\n    \]", source, re.S)
    return {
        "follow": field("follow"),
        "story": field("story"),
        "about": field("about"),
        "about_after": field("about_after"),
        "topics": re.findall(r'\("([^"]+)", "([^"]+)"\)', topics.group(1)) if topics else [],
        "glue": _strings(glue.group(1)) if glue else [],
    }


def hand_nudge(code: str) -> str:
    """The spoken "want more on one of them?" line of a hand-written pack."""
    source = (_PACKS / code / "news.rs").read_text(encoding="utf-8")
    return _strings(re.search(r"\bnudge: (\".*?\"),\n", source).group(1))[0]


def _regional(code: str, say: tuple[str, ...], want: tuple[str, str], open_topic: bool, **extra: str) -> dict:
    base = _hand(code)
    for key, text in extra.items():
        base[key] = base.get(key, []) + words(text)
    return {**base, "news": base.get("news", []), "say": say, "want": want, "open": open_topic}


FLOW: dict[str, dict] = {
    **WEST,
    **NORTH,
    **SLAVIC,
    **ASIA,
    **INDIC,
    **MENA,
    "de-AT": _regional("de", _GERMAN, ("sport", "tesla"), True),
    "de-CH": _regional(
        "de",
        _SWISS,
        ("sport", "tesla"),
        False,
        news="nachrichte neuigkeite nöiigkeite meldige schlagzile",
        follow="meh dezue dadezue erschti zweiti dritti letschti wiiter",
        story="meldig",
        glue="git gits isch hüt verzell mer alli aktuelli wott möcht lose ghöre säg a d",
    ),
    "en-GB": _regional("en", _ENGLISH, ("sports", "tesla"), True),
}


def weak_status(status: list[str]) -> list[str]:
    """Generated packs list "what" as their status word. Beside the news it is a plain question word."""
    return [word for word in status if word not in {"status", "state", "etat", "zustand"}]
