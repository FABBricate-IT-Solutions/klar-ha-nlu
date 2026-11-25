"""Regional packs (de-AT, de-CH, en-GB, pt-BR): a thin overlay on a parent pack.

The engine binds a regional pack on top of its parent, so the regional pack holds only
what the region says differently. Two sources feed it:

- `like`: a regional word and the standard word it stands for ("aa" is said for "an").
  The regional word is written into every parent list that holds the standard word and
  gets the same verb kind, so it can never mean something else than its counterpart.
- `add`: words with no single standard counterpart, by pack path ("cues.synonym_pairs").

That holds for every word class, also the mood words: "ned" is said for "nicht", so a
regional negation forbids the action like the standard one does.

Replies, household texts and the news lists stay the regional pack's own.
"""

from __future__ import annotations

from lang_packs.extras import pack_extras
from lang_packs.fold import fold_latin
from lang_packs.rust_read import read_pack

WORD_GROUPS = ("talk", "nouns", "fixtures", "cues", "chat")
CHAT_LISTS = ("greet", "thanks", "feeling", "identity", "tell", "yarn", "world", "advice", "open")
HOUSEHOLD_LISTS = ("teach", "explain", "undo", "clock", "date", "weekday", "today", "weather", "clock_skip")
ANSWER_LISTS = ("yes", "no", "cancel")
# Every group whose word lists a regional word can be mirrored into.
MIRRORED = (*WORD_GROUPS, "maps", "household", "answers", "mood")
# Hand-written regional tables (calendar_lex, timer_lex): what the parent lacks is kept.
OWN_TABLES = {
    ("nouns", "calendar_nouns"),
    ("cues", "calendar_query"),
    ("cues", "calendar_create"),
    ("cues", "calendar_today"),
    ("cues", "calendar_tomorrow"),
    ("cues", "calendar_when"),
    ("cues", "calendar_delete"),
    ("cues", "calendar_move"),
    ("cues", "hours"),
    ("cues", "minutes"),
    ("cues", "seconds"),
}


def module_of(code: str) -> str:
    return code.replace("-", "_").lower()


def _is_list(value) -> bool:
    return isinstance(value, list)


def _heads(rows: list) -> set[str]:
    heads = (row[0] if isinstance(row, tuple) else row for row in rows)
    return {head for head in heads if isinstance(head, str)}


def _lists(parent: dict):
    """`(group, field, rows)` for every word list of the parent a regional word may join."""
    for group in MIRRORED:
        for field, value in parent[group].items():
            if _is_list(value) and (group != "answers" or field in ANSWER_LISTS):
                yield group, field, value


def _known(parent: dict) -> set[str]:
    """Every single word the parent already gives a meaning."""
    words = {word for word, _kind in parent["verbs"]}
    for _group, _field, rows in _lists(parent):
        words |= _heads(rows)
    return words


def _blank(full: dict, parent: dict) -> dict:
    out = dict(full)
    out["verbs"] = []
    out["fixture_aliases"] = []
    for group in WORD_GROUPS:
        out[group] = dict(full[group])
        for field, value in full[group].items():
            if not _is_list(value) or (group == "chat" and field not in CHAT_LISTS):
                continue
            keep = (group, field) in OWN_TABLES
            have = set(parent[group].get(field) or [])
            out[group][field] = [word for word in value if fold_latin(word) not in have] if keep else []
    maps = dict(full["maps"])
    maps["domain_map"] = []
    maps["room_index_nouns"] = []
    maps["number_style"] = parent["maps"]["number_style"]
    maps["colors"] = []
    # Number words are a hand-written list per region: the ones the parent lacks are regional.
    have = _heads(parent["maps"]["numbers"])
    maps["numbers"] = [row for row in full["maps"]["numbers"] if fold_latin(row[0]) not in have]
    out["maps"] = maps
    out["household"] = {**full["household"], **_own_phrases(full["code"], parent)}
    out["answers"] = {**full["answers"], **{field: [] for field in ANSWER_LISTS}}
    out["mood"] = {}
    # news.rs is built from the full lexicon; its glue list reads the status words.
    out["news_status"] = list(full["cues"]["status_words"])
    return out


def _own_phrases(code: str, parent: dict) -> dict:
    """Hand-written household phrases of the region that the parent does not have ("d uhrzit")."""
    written = pack_extras(code)
    rows = {}
    for field in HOUSEHOLD_LISTS:
        have = {phrase.strip() for phrase in parent["household"].get(field) or []}
        rows[field] = [phrase for phrase in written.get(field, []) if fold_latin(phrase).strip() not in have]
    return rows


def _mirror(out: dict, parent: dict, regional: str, standard: str) -> bool:
    """Write `regional` wherever the parent holds `standard`. False when the parent never does."""
    found = False
    for word, kind in parent["verbs"]:
        if word == standard:
            out["verbs"].append((regional, kind))
            found = True
    for group, field, rows in _lists(parent):
        for item in rows:
            if item == standard:
                out[group].setdefault(field, []).append(regional)
            elif isinstance(item, tuple) and item[0] == standard:
                if field == "fixture_aliases":
                    out["fixture_aliases"].append((regional, list(item[1])))
                else:
                    out[group].setdefault(field, []).append((regional, item[1]))
            else:
                continue
            found = True
    return found


def _add(out: dict, parent: dict, code: str, path: str, rows: list) -> None:
    if path == "verbs":
        kinds = dict(parent["verbs"])
        for word, kind in rows:
            if kinds.get(fold_latin(word), kind) != kind:
                raise SystemExit(f"{code}: verb {word} is {kinds[fold_latin(word)]} in {parent['code']}, not {kind}")
        out["verbs"].extend(rows)
        return
    group, field = path.split(".")
    if not _is_list(out.get(group, {}).get(field)):
        raise SystemExit(f"{code}: unknown pack path {path}")
    out[group][field].extend(rows)


def thin_pack(full: dict, core: dict) -> dict:
    """The regional pack: `full` (speech, household texts, news) with only regional words left."""
    code = core["code"]
    parent = read_pack(module_of(core["parent"]))
    parent["code"] = core["parent"]
    out = _blank(full, parent)
    known = _known(parent)
    for regional, standard in (core.get("like") or {}).items():
        regional, standard = fold_latin(regional), fold_latin(standard)
        if regional in known:
            raise SystemExit(f"{code}: {regional} is already a {core['parent']} word, not a regional one")
        if not _mirror(out, parent, regional, standard):
            raise SystemExit(f"{code}: {standard} (for {regional}) is not a {core['parent']} word")
    for path, rows in (core.get("add") or {}).items():
        _add(out, parent, code, path, rows)
    out["parent"] = core["parent"]
    return out
