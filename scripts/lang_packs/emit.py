"""Emit compiled Klar language packs. Never overwrite handwritten de/en."""

from __future__ import annotations

from pathlib import Path

from lang_packs.emit_pack import pack_rs, speech_rs
from lang_packs.emit_rust import PY_BANNER, RUST_BANNER, rust_str, unique_verbs

ROOT = Path(__file__).resolve().parents[2]
PACKS = ROOT / "src" / "lang" / "packs"
REG = ROOT / "src" / "lang" / "registry.rs"
MOD = ROOT / "src" / "lang" / "packs" / "mod.rs"
PY_LANGS = ROOT / "custom_components" / "klar_nlu" / "languages.py"
HANDWRITTEN = {"de", "en"}


def pack_dir(lang: dict) -> Path:
    if lang["code"] in HANDWRITTEN:
        raise SystemExit(f"refusing to overwrite hand-written pack {lang['code']}")
    dest = PACKS / lang["mod"]
    dest.mkdir(parents=True, exist_ok=True)
    return dest


def write_pack(lang: dict) -> None:
    dest = pack_dir(lang)
    (dest / "mod.rs").write_text(f"{RUST_BANNER}mod pack;\nmod speech;\nmod verbs;\n\npub use pack::PACK;\n", encoding="utf-8")
    (dest / "verbs.rs").write_text(verbs_rs(lang), encoding="utf-8")
    write_speech(lang)
    (dest / "pack.rs").write_text(pack_rs(lang), encoding="utf-8")


def write_speech(lang: dict) -> None:
    (pack_dir(lang) / "speech.rs").write_text(speech_rs(lang), encoding="utf-8")


def verbs_rs(lang: dict) -> str:
    rows = ",\n".join(f'    ({rust_str(word)}, VerbKind::{kind})' for word, kind in unique_verbs(lang["verbs"]))
    return f"""{RUST_BANNER}use crate::lang::verbs::VerbKind;

pub(super) const VERBS: &[(&str, VerbKind)] = &[
{rows},
];
"""


def write_registry(langs: list[dict]) -> None:
    rows = ",\n    ".join(
        "LangRow {{ id: LangId::new({code}), meta: LangMeta {{ code: {code}, native_name: {native}, script: {script}, variants: &[{vars}] }}, pack: &{path} }}".format(
            code=rust_str(item["code"]),
            native=rust_str(item["native"]),
            script=rust_str(item["script"]),
            vars=", ".join(rust_str(v) for v in item["variants"]),
            path=item["path"],
        )
        for item in langs
    )
    text = f"""{RUST_BANNER}//! Compiled language table. de/en live in packs/ like every other locale.
//! Every compiled locale is first-class. The engine never matches De|En in parse.

use super::pack::LanguagePack;
use super::{{packs, LangId}};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LangMeta {{
    pub code: &'static str,
    pub native_name: &'static str,
    pub script: &'static str,
    pub variants: &'static [&'static str],
}}

struct LangRow {{
    id: LangId,
    meta: LangMeta,
    pack: &'static LanguagePack,
}}

const LANGS: &[LangRow] = &[
    {rows},
];

pub fn all_ids() -> &'static [LangId] {{
    static IDS: OnceLock<Vec<LangId>> = OnceLock::new();
    IDS.get_or_init(|| LANGS.iter().map(|row| row.id).collect()).as_slice()
}}

fn row(code: &str) -> Option<&'static LangRow> {{
    static IDX: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    let map = IDX.get_or_init(|| LANGS.iter().enumerate().map(|(i, row)| (row.id.code(), i)).collect());
    map.get(code).map(|&i| &LANGS[i])
}}

pub fn lookup(code: &str) -> Option<LangId> {{
    row(code).map(|item| item.id)
}}

pub fn pack(id: LangId) -> &'static LanguagePack {{
    row(id.code()).map(|item| item.pack).unwrap_or_else(|| panic!("unregistered LangId `{{}}`", id.code()))
}}

pub fn meta(id: LangId) -> Option<&'static LangMeta> {{
    row(id.code()).map(|item| &item.meta)
}}

pub fn languages() -> &'static [LangMeta] {{
    static METAS: OnceLock<Vec<LangMeta>> = OnceLock::new();
    METAS.get_or_init(|| LANGS.iter().map(|row| row.meta).collect()).as_slice()
}}
"""
    REG.write_text(text, encoding="utf-8")
    mods = "\n".join(f"pub mod {item['mod']};" for item in sorted(langs, key=lambda row: row["mod"]))
    MOD.write_text(f"{RUST_BANNER}//! Compiled Assist language packs.\n\n{mods}\n", encoding="utf-8")
    codes = [item["code"] for item in langs]
    variants = ",\n".join(
        f'    "{item["code"]}": ({", ".join(repr(v) for v in item["variants"])},)' for item in langs
    )
    names = ",\n".join(f'    "{item["code"]}": {item["native"]!r}' for item in langs)
    PY_LANGS.write_text(
        f'''{PY_BANNER}
SUPPORTED_LANGUAGES = {tuple(codes)!r}

LANGUAGE_VARIANTS = {{
{variants}
}}

LANGUAGE_NAMES = {{
{names}
}}
''',
        encoding="utf-8",
    )

