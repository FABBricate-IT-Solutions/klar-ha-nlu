//! Pack-configurable inflection and compound hooks. Empty lists mean no suffixes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkingMorpheme {
    pub morpheme: &'static str,
    pub min_rest_len: usize,
    pub require_noun: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Morphology {
    pub room_suffixes: Vec<&'static str>,
    pub color_suffixes: Vec<&'static str>,
    pub linking: Vec<LinkingMorpheme>,
}

/// Const morphology on a compiled pack. Merged into `Catalog.morphology`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackMorphology {
    pub room_suffixes: &'static [&'static str],
    pub color_suffixes: &'static [&'static str],
    pub linking: &'static [LinkingMorpheme],
}

impl PackMorphology {
    pub const EMPTY: Self = Self { room_suffixes: &[], color_suffixes: &[], linking: &[] };
    pub const GERMAN: Self =
        Self { room_suffixes: DEFAULT_ROOM_SUFFIXES, color_suffixes: DEFAULT_COLOR_SUFFIXES, linking: DEFAULT_LINKING };
}

pub const DEFAULT_ROOM_SUFFIXES: &[&str] = &["en", "n", "s"];
pub const DEFAULT_COLOR_SUFFIXES: &[&str] = &["en", "em", "er", "es", "e"];
pub const DEFAULT_LINKING: &[LinkingMorpheme] = &[
    LinkingMorpheme { morpheme: "en", min_rest_len: 5, require_noun: false },
    LinkingMorpheme { morpheme: "n", min_rest_len: 4, require_noun: true },
    LinkingMorpheme { morpheme: "s", min_rest_len: 4, require_noun: true },
];

impl Morphology {
    pub fn effective_room_suffixes(&self) -> &[&str] {
        &self.room_suffixes
    }

    pub fn effective_color_suffixes(&self) -> &[&str] {
        &self.color_suffixes
    }

    pub fn effective_linking(&self) -> Vec<LinkingMorpheme> {
        self.linking.clone()
    }
}
