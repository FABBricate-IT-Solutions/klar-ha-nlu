#!/usr/bin/env python3
"""Map changed paths to locale-scoped dataset tests for PR CI.

PR CI still skips the full 65-locale Wohn+Family matrix. When a pack or
dataset path changes, run that locale's parity suite (and de/en voice
suites). Generated locales are report-only until they are fail==0.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(Path(__file__).resolve().parent))

from check_lang_packs import MOD_TO_TAG

MAX_LOCALES = 8
HARD_LOCALES = frozenset({"de", "en"})
ORACLE_PREFIXES = (
    ("tests/datasets/wohnung_mittel", "de"),
    ("tests/datasets/wohnung_live", "de"),
    ("tests/datasets/familienhaus_de", "de"),
    ("tests/datasets/wohnung_en", "en"),
    ("tests/datasets/family_home_en", "en"),
)
BUILTIN_FILES = {
    "src/lang/de_pack.rs": "de",
    "src/lang/de.rs": "de",
    "src/lang/en_pack.rs": "en",
    "src/lang/en.rs": "en",
}
VOICE_DE = (
    "suite_deutsch",
    "suite_deutsch_familienhaus",
    "suite_m0_exact_german",
    "suite_m2_floors_german",
    "suite_wohnung_live_assist",
)
VOICE_EN = (
    "suite_english_smoke",
    "suite_english_family_home",
    "suite_m0_exact_english",
    "suite_m2_floors_english",
)


def known_locales(root: Path = ROOT) -> dict[str, str]:
    mapping = {"de": "de", "en": "en"}
    for mod, tag in MOD_TO_TAG.items():
        mapping[mod] = tag
        mapping[tag] = tag
    packs = root / "src" / "lang" / "packs"
    if packs.is_dir():
        for path in packs.iterdir():
            if path.is_dir():
                tag = MOD_TO_TAG.get(path.name, path.name)
                mapping[path.name] = tag
                mapping[tag] = tag
    return mapping


def locales_from_paths(paths: list[str], known: dict[str, str] | None = None) -> set[str]:
    known = known or known_locales()
    found: set[str] = set()
    for raw in paths:
        path = raw.replace("\\", "/").strip()
        if not path or path.endswith("/"):
            path = path.rstrip("/")
        if path in BUILTIN_FILES:
            found.add(BUILTIN_FILES[path])
            continue
        hit = False
        for prefix, tag in ORACLE_PREFIXES:
            if path == prefix or path.startswith(prefix + "/"):
                found.add(tag)
                hit = True
                break
        if hit:
            continue
        parts = path.split("/")
        if len(parts) >= 4 and parts[:3] == ["src", "lang", "packs"] and not parts[3].endswith(".rs"):
            tag = known.get(parts[3])
            if tag:
                found.add(tag)
            continue
        if len(parts) >= 4 and parts[:3] in (["tests", "datasets", "parity"], ["tests", "datasets", "assist"]):
            tag = known.get(parts[3])
            if tag:
                found.add(tag)
    return found


def test_name(tag: str) -> str:
    return "parity_" + tag.replace("-", "_").lower()


def nextest_expr(names: list[str]) -> str:
    return " | ".join(f"test({name})" for name in names)


def plan_for(locales: set[str]) -> tuple[str, list[tuple[list[str], dict[str, str]]]]:
    """Return a human summary and nextest invocations (args, env)."""
    if not locales:
        return "no language-scoped paths; skip dataset suites", []
    if len(locales) > MAX_LOCALES:
        codes = ", ".join(sorted(locales))
        return (
            f"{len(locales)} locales changed (cap {MAX_LOCALES}): {codes}. "
            "skip path-scoped datasets; run language-parity.yml or cargo nextest run --test parity_langs",
            [],
        )
    hard = sorted(locales & HARD_LOCALES)
    report = sorted(locales - HARD_LOCALES)
    runs: list[tuple[list[str], dict[str, str]]] = []
    if hard:
        names = [test_name(tag) for tag in hard]
        voice = []
        if "de" in hard:
            voice.extend(VOICE_DE)
        if "en" in hard:
            voice.extend(VOICE_EN)
        expr = f"({nextest_expr(names)})"
        if voice:
            expr = f"(binary(parity_langs) & {expr}) | (binary(voice_suite) & ({nextest_expr(voice)}))"
        else:
            expr = f"binary(parity_langs) & {expr}"
        runs.append((["cargo", "nextest", "run", "--locked", "--profile", "ci-lang", "-E", expr], {}))
    if report:
        names = [test_name(tag) for tag in report]
        expr = f"binary(parity_langs) & ({nextest_expr(names)})"
        runs.append(
            (
                ["cargo", "nextest", "run", "--locked", "--profile", "ci-lang", "-E", expr],
                {"KLAR_PARITY_REPORT": "1"},
            )
        )
    summary = "path-scoped language tests: " + ", ".join(sorted(locales))
    if report:
        summary += f" (report-only: {', '.join(report)})"
    return summary, runs


def changed_paths(diff_from: str) -> list[str]:
    out = subprocess.check_output(["git", "diff", "--name-only", diff_from, "HEAD"], cwd=ROOT, text=True)
    return [line for line in out.splitlines() if line.strip()]


def run_plan(runs: list[tuple[list[str], dict[str, str]]]) -> int:
    env_base = dict(os.environ)
    for args, extra in runs:
        print("+", " ".join(args), extra or "", flush=True)
        env = dict(env_base)
        env.update(extra)
        proc = subprocess.run(args, cwd=ROOT, env=env)
        if proc.returncode != 0:
            return proc.returncode
    return 0


def self_test() -> None:
    known = {
        "cs": "cs",
        "de_ch": "de-CH",
        "de-CH": "de-CH",
        "zh_cn": "zh-CN",
        "zh-CN": "zh-CN",
        "sr_latn": "sr-Latn",
        "sr-Latn": "sr-Latn",
        "de": "de",
        "en": "en",
        "fr": "fr",
    }
    assert locales_from_paths(["src/lang/packs/cs/pack.rs"], known) == {"cs"}
    assert locales_from_paths(["src/lang/packs/de_ch/speech.rs"], known) == {"de-CH"}
    assert locales_from_paths(["src/lang/de_pack.rs", "src/lang/de.rs"], known) == {"de"}
    assert locales_from_paths(["src/lang/en_pack.rs"], known) == {"en"}
    assert locales_from_paths(["tests/datasets/parity/zh-CN/wohnung_mittel/area.yaml"], known) == {"zh-CN"}
    assert locales_from_paths(["tests/datasets/assist/fr/representative.yaml"], known) == {"fr"}
    assert locales_from_paths(["tests/datasets/wohnung_mittel/area/x.yaml"], known) == {"de"}
    assert locales_from_paths(["tests/datasets/family_home_en/devices/x.yaml"], known) == {"en"}
    assert locales_from_paths(["src/parse/slots.rs", "src/lang/registry.rs", "src/lang/packs/mod.rs"], known) == set()
    assert locales_from_paths(["tests/datasets/parity/rooms.yaml"], known) == set()
    extra = ["af", "ar", "bg", "bn", "ca", "cs", "cy", "da", "el"]
    known.update({code: code for code in extra})
    many = [f"src/lang/packs/{code}/pack.rs" for code in extra]
    summary, runs = plan_for(locales_from_paths(many, known))
    assert runs == []
    assert "cap" in summary
    summary, runs = plan_for({"cs"})
    assert runs and runs[0][1] == {"KLAR_PARITY_REPORT": "1"}
    assert "parity_cs" in runs[0][0][-1]
    summary, runs = plan_for({"de"})
    assert runs and runs[0][1] == {}
    assert "suite_deutsch" in runs[0][0][-1]
    print("ci_lang_tests self-test ok")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--diff-from", help="git ref to diff against HEAD (two-dot, works on shallow clones)")
    parser.add_argument("--run", action="store_true", help="execute nextest for the matched locales")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if not args.diff_from:
        parser.error("need --diff-from or --self-test")
    try:
        paths = changed_paths(args.diff_from)
    except subprocess.CalledProcessError as exc:
        print(f"git diff failed ({exc.returncode}); skip path-scoped datasets", file=sys.stderr)
        return 0
    locales = locales_from_paths(paths)
    summary, runs = plan_for(locales)
    print(summary)
    if args.run and runs:
        return run_plan(runs)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
