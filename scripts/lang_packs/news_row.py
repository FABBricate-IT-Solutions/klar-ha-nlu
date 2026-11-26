"""Row helpers for the news follow-up lexicon.

Words are written the way people spell them. `cut` folds them the way
src/parse/normalize.rs tokenizes speech, so "l'actualité" is stored as
"lactualite" and "i dag" stays a two-word phrase.
"""

from __future__ import annotations

from lang_packs.fold import fold_latin

_MARKS = (
    (0x0300, 0x036F),
    (0x0483, 0x0489),
    (0x0591, 0x05C7),
    (0x0610, 0x061A),
    (0x064B, 0x065F),
    (0x0670, 0x0670),
    (0x06D6, 0x06ED),
    (0x0900, 0x0D7F),
    (0x0E31, 0x0E31),
    (0x0E34, 0x0E3A),
    (0x0E47, 0x0E4E),
    (0x180B, 0x180E),
    (0xFE20, 0xFE2F),
)
_SCRIPT = (
    (0x1100, 0x11FF),
    (0x2E80, 0x2EFF),
    (0x3040, 0x30FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0x9FFF),
    (0xA960, 0xA97F),
    (0xAC00, 0xD7AF),
    (0xF900, 0xFAFF),
    (0xFF65, 0xFF9F),
    (0x0E00, 0x0E7F),
)


def _in(char: str, ranges: tuple[tuple[int, int], ...]) -> bool:
    return any(low <= ord(char) <= high for low, high in ranges)


def _word_char(char: str) -> bool:
    if char in "'\u200c\u200d":
        return True
    if char in "।॥॰":
        return False
    return char.isalnum() or _in(char, _MARKS)


_CLITICS = {"l", "d", "dell", "all", "un", "nell", "sull", "coll", "c", "j", "m", "t", "s", "n"}


def _bare(word: str) -> str:
    if len(word) > 2 and word.endswith("'s"):
        word = word[:-2]
    return word.replace("'", "")


def _split_clitics(folded: str) -> list[str]:
    """Mirror src/parse/normalize/token.rs: l'entrée → entree; don't → dont."""
    out: list[str] = []
    buf: list[str] = []
    chars = list(folded)
    i = 0
    while i < len(chars):
        char = chars[i]
        if char == "'":
            left = "".join(buf).lower()
            if left in _CLITICS:
                buf.clear()
                i += 1
                continue
            if i + 1 < len(chars) and chars[i + 1] == "s" and (i + 2 >= len(chars) or not _word_char(chars[i + 2])):
                token = _bare("".join(buf))
                buf.clear()
                if token:
                    out.append(token)
                i += 2
                continue
            i += 1
            continue
        if _word_char(char):
            buf.append(char)
            i += 1
            continue
        token = _bare("".join(buf))
        buf.clear()
        if token:
            out.append(token)
        i += 1
    token = _bare("".join(buf))
    if token:
        out.append(token)
    return out


def cut(text: str) -> str:
    """One entry as the tokens speech would give, joined by single spaces."""
    folded = fold_latin(text.strip())
    if any(_in(char, _SCRIPT) for char in folded):
        return "".join(folded.split())
    return " ".join(_split_clitics(folded))


def words(text: str) -> list[str]:
    """Space-separated entries. A plus sign joins the words of one phrase."""
    out: list[str] = []
    for item in text.split():
        entry = cut(item.replace("+", " "))
        if entry and entry not in out:
            out.append(entry)
    return out


def pairs(text: str) -> list[tuple[str, str]]:
    """`word=query` entries. The query keeps its spelling: it is what gets searched."""
    out: list[tuple[str, str]] = []
    for item in text.split():
        key, _, query = item.partition("=")
        entry = cut(key.replace("+", " "))
        if entry:
            out.append((entry, (query or key).replace("+", " ")))
    return out


def row(
    news: str = "",
    follow: str = "",
    story: str = "",
    about: str = "",
    after: str = "",
    topics: str = "",
    glue: str = "",
    say: tuple[str, ...] = (),
    want: tuple[str, str] = ("", ""),
) -> dict:
    """`say`: ask, more about one story, a what-question, "all the news about that", a section, a named topic."""
    if len(say) != 6:
        raise ValueError("six sentences are needed")
    return {
        "news": words(news),
        "follow": words(follow),
        "story": words(story),
        "about": words(about),
        "about_after": words(after),
        "topics": pairs(topics),
        "glue": words(glue),
        "say": say,
        "want": want,
    }
