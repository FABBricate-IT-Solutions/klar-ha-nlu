#!/usr/bin/env python3
"""Aggregate gate for the CI job `release-gates`.

`release-gates` is a required check. It lists every other job of ci.yml under
`needs`, runs with `if: always()` and passes the `needs` context as JSON in the
environment variable NEEDS. This script fails unless every needed job ended
with `success`, so a failed, cancelled or skipped job blocks the merge.
"""

from __future__ import annotations

import json
import os
import sys


def parse_needs(raw: str) -> dict[str, str]:
    """Return job id -> result. Raises ValueError on anything but a `needs` object."""
    needs = json.loads(raw)
    if not isinstance(needs, dict) or not needs:
        raise ValueError("NEEDS lists no jobs")
    results: dict[str, str] = {}
    for job, info in needs.items():
        if not isinstance(info, dict) or not isinstance(info.get("result"), str):
            raise ValueError(f"NEEDS entry {job!r} has no result")
        results[job] = info["result"]
    return results


def unsuccessful(results: dict[str, str]) -> list[str]:
    return sorted(job for job, result in results.items() if result != "success")


def main() -> int:
    try:
        results = parse_needs(os.environ.get("NEEDS", ""))
    except ValueError as error:
        print(f"::error title=release-gates::cannot read the needs context: {error}", file=sys.stderr)
        return 1
    for job in sorted(results):
        print(f"{job}: {results[job]}")
    failed = unsuccessful(results)
    if failed:
        print(f"::error title=release-gates::not successful: {', '.join(failed)}", file=sys.stderr)
        return 1
    print(f"all {len(results)} needed jobs succeeded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
