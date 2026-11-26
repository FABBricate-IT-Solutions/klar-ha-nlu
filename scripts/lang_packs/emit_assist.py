"""Emit assist pack data files under packs/assist/ (package 6.9).

Generated locales keep compiled Rust for the engine, but phrase corrections live
in one YAML file per locale. The generator creates a stub when missing and never
overwrites an existing assist file — a pack correction touches that one file.
"""

from __future__ import annotations

from pathlib import Path

from lang_packs.emit_rust import RUST_BANNER

ROOT = Path(__file__).resolve().parents[2]
ASSIST = ROOT / "packs" / "assist"
EMBED = ROOT / "src" / "lang" / "assist_embed.rs"
HANDWRITTEN = {"de", "en"}


def assist_path(code: str) -> Path:
    return ASSIST / f"{code}.yaml"


def stub_yaml(lang: dict) -> str:
    code = lang["code"]
    variants = list(lang.get("variants") or [code])
    bcp = "\n".join(f"  - {tag}" for tag in variants)
    extends = lang.get("parent") or code
    return (
        f"# Assist pack data for {code} (package 6.9).\n"
        f"# Fix a phrase here — do not edit generated Rust under src/lang/packs/{lang['mod']}/.\n"
        f"# The generator creates this stub once and never overwrites it.\n"
        f"klar_lang_pack: \"2.0\"\n"
        f"id: {code}\n"
        f"bcp47:\n{bcp}\n"
        f"extends: {extends}\n"
        f"verbs: {{}}\n"
        f"sets: {{}}\n"
        f"intents: []\n"
    )


def ensure_assist_file(lang: dict) -> None:
    if lang["code"] in HANDWRITTEN:
        return
    ASSIST.mkdir(parents=True, exist_ok=True)
    path = assist_path(lang["code"])
    if path.is_file():
        return
    path.write_text(stub_yaml(lang), encoding="utf-8")


def write_assist_embed(langs: list[dict]) -> None:
    """Compile-time table of assist YAML files included in the binary."""
    rows: list[str] = []
    for lang in sorted(langs, key=lambda item: item["code"]):
        if lang["code"] in HANDWRITTEN:
            continue
        ensure_assist_file(lang)
        # assist_embed.rs lives in src/lang/ → two levels up to the repo root.
        rel = f"../../packs/assist/{lang['code']}.yaml"
        rows.append(f'    ("{lang["code"]}", include_str!("{rel}")),')
    body = "\n".join(rows)
    EMBED.write_text(
        f"""{RUST_BANNER}//! Embedded assist pack data files (package 6.9).
//! Phrase corrections live in `packs/assist/{{code}}.yaml` — one file per locale.

/// `(locale code, raw YAML)` for every generated Assist locale.
pub static ASSIST_YAML: &[(&str, &str)] = &[
{body}
];
""",
        encoding="utf-8",
    )
