#!/usr/bin/env python3
"""Join the Klar scorecard and the Home Assistant Assist run into one stamped benchmark.

CI runs `klar eval gate --out scorecard.json` and `scripts/bench_assist.py --out assist.json`
on the same commit, then this script. The result names the commit, the Assist package versions
and, per language, both engines on the same held-out sentences. It refuses to join runs that
belong to different commits, or whose commit is not a full SHA, so a number in the artifact
can always be traced to the tree that produced it.

    python3 scripts/benchmark_report.py --scorecard scorecard.json --assist assist.json \\
        --out benchmark.json [--markdown benchmark.md]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from pathlib import Path

SCHEMA_VERSION = "benchmark.1"
SHA = re.compile(r"^[0-9a-f]{40}$")
KLAR_KEYS = (
    "accuracy",
    "in_domain_accuracy",
    "intent_macro_f1",
    "slot_micro_f1",
    "reject_fpr",
    "reject_fnr",
    "reject_unsafe",
    "typo_recovery",
)


class ReportError(ValueError):
    """The two runs cannot be joined into an honest comparison."""


def check_commit(scorecard: dict, assist: dict) -> str:
    klar, ha = scorecard.get("commit"), assist.get("commit")
    for who, sha in (("scorecard", klar), ("assist run", ha)):
        if not isinstance(sha, str) or not SHA.match(sha):
            raise ReportError(f"{who} is not stamped with a full commit SHA: {sha!r}")
    if klar != ha:
        raise ReportError(f"scorecard is for {klar} but the assist run is for {ha}")
    return klar


def percent(value: float) -> float:
    return round(100.0 * value, 2)


def fraction(rate: dict) -> float:
    return rate["accuracy"] / 100.0


def join(scorecard: dict, assist: dict) -> dict:
    commit = check_commit(scorecard, assist)
    languages = {}
    for card in scorecard.get("languages", []):
        code = card["language"]
        ha = (assist.get("languages") or {}).get(code)
        if ha is None:
            raise ReportError(f"the assist run has no results for {code}")
        metrics = card["metrics"]
        languages[code] = {
            "utterances": card["utterances"],
            "in_domain": card["in_domain"],
            "klar": {key: metrics[key] for key in KLAR_KEYS},
            "assist": {
                "accuracy": fraction(ha),
                "hassil_only_accuracy": fraction(ha["hassil_only"]),
                "in_domain_accuracy": fraction(ha["in_domain"]),
                "in_domain_hassil_only_accuracy": fraction(ha["in_domain"]["hassil_only"]),
                "gazetteer": code.split("-", 1)[0] in assist.get("gazetteer_languages", []),
            },
        }
    if not languages:
        raise ReportError("the scorecard has no languages")
    return {
        "schema_version": SCHEMA_VERSION,
        "commit": commit,
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "corpus": "tests/datasets/{family_home_en,familienhaus_de}/m7_heldout",
        "assist_versions": assist.get("versions", {}),
        "klar_scorecard_schema": scorecard.get("schema_version"),
        "languages": languages,
    }


def markdown(report: dict) -> str:
    lines = [
        f"### Klar NLU vs Home Assistant Assist at `{report['commit'][:12]}`",
        "",
        "Assist = hassil + home-assistant-intents, plus the gazetteer matcher for English, as Core's default agent runs them.",
        f"Versions: {', '.join(f'{name} {version}' for name, version in report['assist_versions'].items())}",
        "",
        "| Language | Sentences (in-domain) | Klar all | Klar in-domain | Assist all | Assist in-domain | hassil only, in-domain | Gazetteer |",
        "|---|---:|---:|---:|---:|---:|---:|:--:|",
    ]
    for code, row in sorted(report["languages"].items()):
        klar, ha = row["klar"], row["assist"]
        lines.append(
            f"| {code} | {row['utterances']} ({row['in_domain']}) | {percent(klar['accuracy'])}% | {percent(klar['in_domain_accuracy'])}% "
            f"| {percent(ha['accuracy'])}% | {percent(ha['in_domain_accuracy'])}% | {percent(ha['in_domain_hassil_only_accuracy'])}% "
            f"| {'yes' if ha['gazetteer'] else 'no (en only)'} |"
        )
    lines.append("")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--scorecard", type=Path, required=True)
    parser.add_argument("--assist", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--markdown", type=Path)
    args = parser.parse_args(argv)
    try:
        report = join(json.loads(args.scorecard.read_text(encoding="utf-8")), json.loads(args.assist.read_text(encoding="utf-8")))
    except ReportError as err:
        print(f"benchmark report refused: {err}", file=sys.stderr)
        return 1
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    if args.markdown:
        args.markdown.write_text(markdown(report), encoding="utf-8")
    print(f"wrote {args.out} for {report['commit']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
