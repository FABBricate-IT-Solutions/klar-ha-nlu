#!/usr/bin/env python3
"""Run the repository guardrails (package 6.11).

  boundary        ADR 0003: no utterance-matching regexes in custom_components/klar_nlu/
  responsibility  every long module names its owner and its one job (replaces the 500-line gate)
  focus           the frozen non-core areas are still frozen and still off by default
  commits         fix/feat commits that mostly touch frozen paths carry a Frozen-area trailer
  share           print the share of fixes that went to news/LLM paths; --max-share fails above it

`commits` and `share` read git history: pass `--base origin/staging` for a pull request, or
`--since DATE` for a window (default: the baseline in scripts/guardrails.toml).
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from guardrails import boundary, focus, responsibility  # noqa: E402
from guardrails.config import GuardrailError, load  # noqa: E402

STATIC = {
    "boundary": boundary.check,
    "responsibility": responsibility.check,
    "focus": focus.check_static,
}


def history(args: argparse.Namespace, config: dict) -> list[focus.Commit]:
    if args.base:
        return focus.git_commits([f"{args.base}..HEAD"])
    return focus.git_commits([f"--since={args.since or config['focus']['baseline']}"])


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("checks", nargs="*", help="any of: " + ", ".join([*STATIC, "commits", "share"]) + " (default: the static ones)")
    parser.add_argument("--base", help="git revision a pull request is based on (commits in base..HEAD)")
    parser.add_argument("--since", help="start of the window for `share` (git date)")
    parser.add_argument("--max-share", type=float, help="fail when the non-core fix share is above this percent")
    args = parser.parse_intermixed_args(argv)
    config = load()
    checks = args.checks or [*STATIC]
    unknown = [name for name in checks if name not in (*STATIC, "commits", "share")]
    if unknown:
        parser.error(f"unknown check: {', '.join(unknown)}")
    failures: list[str] = []
    try:
        for name in checks:
            if name in STATIC:
                failures.extend(STATIC[name](config))
            elif name == "commits":
                failures.extend(focus.check_commits(history(args, config), config))
            elif name == "share":
                percent, text = focus.share_report(history(args, config), config)
                print(text)
                if args.max_share is not None and percent > args.max_share:
                    failures.append(f"{text}; the limit for this run is {args.max_share} %")
            else:
                raise AssertionError(name)
    except GuardrailError as error:
        failures.append(str(error))
    for failure in failures:
        print(failure, file=sys.stderr)
    if not failures:
        print("guardrails ok: " + ", ".join(checks))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
