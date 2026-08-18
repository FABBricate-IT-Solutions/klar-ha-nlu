#!/usr/bin/env python3
"""Generate shared room aliases and native parity sentences from DE oracles."""

from __future__ import annotations

import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from lang_packs.lexicons import ALL_CORES
from lex import DATA, lex_of
from sentences import sentence_for

GROUPS = ["area", "devices", "query_area", "query_devices", "multiple_intents", "assist", "clarifications", "state_persistance", "timers", "lists"]
SUITES = (("wohnung_mittel", GROUPS), ("familienhaus_de", GROUPS), ("familienhaus_de", ["m0_exact"]), ("familienhaus_de", ["m2_floors"]))


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
            overlay = {f"{group}/{path.stem}::{case.get('name') or path.stem}": sentence_for(case, lex, suite) for path in sorted(src.glob("*.yaml")) for case in load_cases(path)}
            if overlay:
                dest = DATA / "parity" / lex["code"] / dest_suite / f"{group}.yaml"
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_text(yaml.safe_dump(overlay, allow_unicode=True, sort_keys=True), encoding="utf-8")


def main() -> None:
    lexes = [lex_of(core) for core in ALL_CORES]
    write_rooms(lexes)
    for lex in lexes:
        write_sentences(lex)
        print("wrote sentences", lex["code"])


if __name__ == "__main__":
    main()
