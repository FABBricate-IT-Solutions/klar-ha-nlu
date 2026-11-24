"""Emit the news.rs of a generated pack. de/en keep their hand-written news.rs."""

from __future__ import annotations

from lang_packs.emit_rust import RUST_BANNER, rust_list, rust_pairs, rust_str
from lang_packs.news_lex import news_glue


def news_rs(lang: dict) -> str:
    news = lang["chat"]["news"]
    glue = news_glue(news, lang["cues"]["status_words"])
    return f"""{RUST_BANNER}use crate::lang::groups::News;

pub(super) const NEWS: News = News {{
    words: {rust_list(news["words"], "    ")},
    dismiss: {rust_list(news["dismiss"], "    ")},
    intro: {rust_str(news["intro"])},
    nudge: {rust_str(news["nudge"])},
    done: {rust_str(news["done"])},
    follow: {rust_list(news["follow"], "    ")},
    story: {rust_list(news["story"], "    ")},
    about: {rust_list(news["about"], "    ")},
    about_after: {rust_list(news["about_after"], "    ")},
    topics: {rust_pairs(news["topics"], "    ")},
    glue: {rust_list(glue, "    ")},
    open: {"true" if news["open"] else "false"},
}};
"""
