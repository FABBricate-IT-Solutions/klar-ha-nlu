#!/usr/bin/env python3
"""Generate shared room aliases and native parity sentences from DE oracles."""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from conversation import conversation_overlay
from lang_packs.lexicons import ALL_CORES
from lex import DATA, lex_of
from sentences import sentence_for

GROUPS = ["area", "devices", "query_area", "query_devices", "multiple_intents", "assist", "clarifications", "state_persistance", "timers", "lists"]
SUITES = (
    ("wohnung_mittel", GROUPS + ["conversation"]),
    ("familienhaus_de", GROUPS),
    ("familienhaus_de", ["m0_exact"]),
    ("familienhaus_de", ["m2_floors"]),
)


def load_cases(path: Path) -> list[dict]:
    raw = yaml.safe_load(path.read_text(encoding="utf-8"))
    return [] if raw is None else raw if isinstance(raw, list) else [raw]


def write_rooms(lexes: list[dict]) -> None:
    out = {lex["code"]: {"areas": lex["rooms"], "floors": lex["floors"]} for lex in lexes}
    path = DATA / "parity" / "rooms.yaml"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(out, allow_unicode=True, sort_keys=True), encoding="utf-8")
    print("wrote", path.relative_to(ROOT))


def write_sentences(lex: dict) -> None:
    for suite, groups in SUITES:
        dest_suite = {"m0_exact": "m0_exact", "m2_floors": "m2_floors"}.get(groups[0] if groups[0] in {"m0_exact", "m2_floors"} else "", suite)
        for group in groups:
            src = DATA / suite / group
            if not src.is_dir():
                continue
            overlay = (
                conversation_overlay(lex)
                if group == "conversation"
                else {
                    f"{group}/{path.stem}::{case.get('name') or path.stem}": sentence_for(case, lex, suite)
                    for path in sorted(src.glob("*.yaml"))
                    for case in load_cases(path)
                }
            )
            if overlay:
                dest = DATA / "parity" / lex["code"] / dest_suite / f"{group}.yaml"
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text(yaml.safe_dump(overlay, allow_unicode=True, sort_keys=True), encoding="utf-8")


def _fold_token(word: str) -> str:
    return word.casefold().replace("ä", "ae").replace("ö", "oe").replace("ü", "ue").replace("ß", "ss")


# English colour canons that appear in the German oracle; a pinned de-* pack has no en pack
# to recognise them, so the gate sentences must use the German colour word.
_DE_COLOR_FROM_CANON = {
    "red": "rot",
    "blue": "blau",
    "green": "gruen",
    "yellow": "gelb",
    "orange": "orange",
    "pink": "pink",
    "black": "schwarz",
    "white": "weiss",
    "purple": "lila",
    "warmwhite": "warmweiss",
}


def _apply_like(text: str, like: dict[str, str]) -> str:
    """Swap standard parent words for the regional forms that mean the same (`an` → `aa`)."""
    if not text:
        return text
    # like maps regional → standard; invert so a parent sentence can wear regional clothes.
    standard_to_regional = {_fold_token(standard): regional for regional, standard in (like or {}).items()}
    parts = []
    for token in text.split():
        folded = _fold_token(token)
        if folded in standard_to_regional:
            parts.append(standard_to_regional[folded])
        elif folded in _DE_COLOR_FROM_CANON:
            parts.append(_DE_COLOR_FROM_CANON[folded])
        else:
            parts.append(token)
    return " ".join(parts)


def _source_overlay(case: dict) -> list | list[list]:
    """Overlay payload from the oracle: one flat sentence, or one multi-turn conversation."""
    sentences = case.get("sentences") or []
    if not sentences:
        return []
    if isinstance(sentences[0], list):
        # Multi-turn alternatives → keep the first conversation as a Turns overlay.
        return [list(sentences[0])]
    return [sentences[0]]


def regional_gate_sentence(case: dict, lex: dict, suite: str, core: dict) -> list | list[list]:
    """A regional gate keeps the parent's meaning: use the oracle wording, then regional `like` words.

    Pidgin sentence_for drops German particles and invents room stems the thin pack does not own.
    The parent oracle already matches the engine under inheritance; `like` only renames words.
    """
    parent = core.get("parent")
    if parent in {"de", "en"} and suite.startswith("familienhaus"):
        like = core.get("like") or {}
        payload = _source_overlay(case)
        if isinstance(payload, list) and payload and isinstance(payload[0], list):
            return [[_apply_like(turn, like) for turn in row] for row in payload]
        return [_apply_like(turn, like) for turn in payload]
    return sentence_for(case, lex, suite)


def write_regional_gates(lex: dict, core: dict) -> None:
    """Keep familienhaus/m0/m2 overlays as gates for regional packs; apartment suites inherit."""
    parent = core.get("parent")
    # Regionals whose parent already has parity overlays reuse those files as gates.
    if parent and parent not in {"de", "en"}:
        for suite in ("familienhaus_de", "m0_exact", "m2_floors"):
            src_dir = DATA / "parity" / parent / suite
            dest_dir = DATA / "parity" / lex["code"] / suite
            if not src_dir.is_dir():
                continue
            if dest_dir.exists():
                shutil.rmtree(dest_dir)
            shutil.copytree(src_dir, dest_dir)
        return
    # de-* and en-GB: write the German oracle (plus like-words) so the derived files stay gates.
    # en-GB's runner uses family_home_en; the files still satisfy the suite-presence checks.
    for suite, groups in SUITES:
        dest_suite = {"m0_exact": "m0_exact", "m2_floors": "m2_floors"}.get(
            groups[0] if groups[0] in {"m0_exact", "m2_floors"} else "", suite
        )
        if dest_suite == "wohnung_mittel":
            continue
        for group in groups:
            src = DATA / suite / group
            if not src.is_dir():
                continue
            overlay = {
                f"{group}/{path.stem}::{case.get('name') or path.stem}": regional_gate_sentence(case, lex, suite, core)
                for path in sorted(src.glob("*.yaml"))
                for case in load_cases(path)
            }
            if overlay:
                dest = DATA / "parity" / lex["code"] / dest_suite / f"{group}.yaml"
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text(yaml.safe_dump(overlay, allow_unicode=True, sort_keys=True), encoding="utf-8")


def main() -> None:
    lexes = [lex_of(core) for core in ALL_CORES]
    write_rooms(lexes)
    regionals = []
    for core, lex in zip(ALL_CORES, lexes):
        if core.get("parent"):
            regionals.append((core, lex))
            continue
        write_sentences(lex)
        print("wrote sentences", lex["code"])
    for core, lex in regionals:
        # Apartment suites inherit from the parent; familienhaus overlays stay as gates.
        write_regional_gates(lex, core)
        leftover = DATA / "parity" / lex["code"] / "wohnung_mittel"
        if leftover.exists():
            shutil.rmtree(leftover)
        print("wrote regional gates", lex["code"])


if __name__ == "__main__":
    main()
