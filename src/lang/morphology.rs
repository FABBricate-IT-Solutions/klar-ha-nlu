//! Pack-configurable inflection and compound hooks. Defaults match the historic hardcoded lists.

#[derive(Debug, Clone, PartialEq, Eq)]
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

pub const DEFAULT_ROOM_SUFFIXES: &[&str] = &["en", "n", "s"];
pub const DEFAULT_COLOR_SUFFIXES: &[&str] = &["en", "em", "er", "es", "e"];

pub fn default_linking() -> Vec<LinkingMorpheme> {
    vec![
        LinkingMorpheme { morpheme: "en", min_rest_len: 5, require_noun: false },
        LinkingMorpheme { morpheme: "n", min_rest_len: 4, require_noun: true },
        LinkingMorpheme { morpheme: "s", min_rest_len: 4, require_noun: true },
    ]
}

impl Morphology {
    pub fn effective_room_suffixes(&self) -> &[&str] {
        if self.room_suffixes.is_empty() {
            DEFAULT_ROOM_SUFFIXES
        } else {
            &self.room_suffixes
        }
    }

    pub fn effective_color_suffixes(&self) -> &[&str] {
        if self.color_suffixes.is_empty() {
            DEFAULT_COLOR_SUFFIXES
        } else {
            &self.color_suffixes
        }
    }

    pub fn effective_linking(&self) -> Vec<LinkingMorpheme> {
        if self.linking.is_empty() {
            default_linking()
        } else {
            self.linking.clone()
        }
    }
}
