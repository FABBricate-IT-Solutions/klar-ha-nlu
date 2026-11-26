//! Text normalizer: NFKC → lower → script fold → tokenize.

mod fillers;
mod fold;
mod token;

pub use fillers::{compact, inflected_eq, join_tokens, strip_fillers, strip_fillers_for_areas};
pub use fold::{fold_label, fold_latin, fold_marks, fold_umlaut, umlaut_eq};
pub use token::{tokenize, tokenize_with};

pub(crate) use fillers::{article_one, is_time_unit};
pub(crate) use token::{is_script_unit, is_word_char};

use crate::types::HomeGraph;

/// Surfaces the CJK/Thai segmenter may join, from every pack set plus home names (L-13).
pub fn home_script_surfaces(home: &HomeGraph) -> Vec<String> {
    let mut out = Vec::new();
    for area in &home.areas {
        out.push(fold_latin(&area.name));
        out.extend(area.aliases.iter().map(|alias| fold_latin(alias)));
    }
    for floor in &home.floors {
        out.push(fold_latin(&floor.name));
        out.extend(floor.aliases.iter().map(|alias| fold_latin(alias)));
    }
    for entity in &home.entities {
        out.push(fold_latin(&entity.name));
        out.extend(entity.aliases.iter().map(|alias| fold_latin(alias)));
    }
    out.sort();
    out.dedup();
    out
}

/// Tokenize with the active packs and this home's names in the segmenter dictionary.
pub fn tokenize_home(text: &str, home: &HomeGraph) -> Vec<String> {
    tokenize_with(text, &home_script_surfaces(home))
}
