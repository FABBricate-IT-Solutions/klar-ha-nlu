//! Read `klar_nlu.json` so that one value this build cannot use does not void the file.
//!
//! The strict parse runs first. When it fails, the value it failed on is dropped (an object
//! key, or a list item) and the parse runs again, so the field falls back to its default and
//! everything else is kept. The caller keeps a copy of the original bytes.

use super::overlay::Overlay;
use serde_json::Value;
use serde_path_to_error::Segment;

/// Schema version written into the file. Files without the field are version 0.
pub const OVERLAY_VERSION: u32 = 1;

/// Give up after this many dropped values; the file is then treated as unreadable.
const MAX_DROPS: usize = 256;

#[derive(Debug)]
pub struct Parsed {
    pub overlay: Overlay,
    /// One line per value that was dropped, as `path: reason`. Empty for a clean file.
    pub dropped: Vec<String>,
}

/// Parse exactly, without dropping anything.
pub fn parse_strict(raw: &[u8]) -> Result<Overlay, String> {
    serde_json::from_slice(raw).map_err(|err| err.to_string())
}

/// Parse, dropping single values that do not fit. `Err` means the file is not a JSON object
/// or cannot be read even after dropping.
pub fn parse_tolerant(raw: &[u8]) -> Result<Parsed, String> {
    let mut dropped = Vec::new();
    let overlay = match parse_strict(raw) {
        Ok(overlay) => overlay,
        Err(_) => {
            let mut value: Value = serde_json::from_slice(raw).map_err(|err| err.to_string())?;
            if !value.is_object() {
                return Err("top level is not a JSON object".into());
            }
            prune_until_readable(&mut value, &mut dropped)?
        }
    };
    if overlay.version > OVERLAY_VERSION {
        dropped.push(format!("version: {} is newer than this engine reads ({OVERLAY_VERSION})", overlay.version));
    }
    Ok(Parsed { overlay, dropped })
}

fn prune_until_readable(value: &mut Value, dropped: &mut Vec<String>) -> Result<Overlay, String> {
    loop {
        let err = match serde_path_to_error::deserialize::<_, Overlay>(&*value) {
            Ok(overlay) => return Ok(overlay),
            Err(err) => err,
        };
        let line = format!("{}: {}", err.path(), err.inner());
        if dropped.len() >= MAX_DROPS || !drop_at(value, err.path()) {
            return Err(line);
        }
        dropped.push(line);
    }
}

enum Step {
    Key(String),
    Index(usize),
}

/// Remove the value the path points at. When that exact spot cannot be removed, remove the
/// nearest enclosing key or list item instead. `false` when only the top level is left.
fn drop_at(root: &mut Value, path: &serde_path_to_error::Path) -> bool {
    let mut steps = Vec::new();
    for segment in path.iter() {
        match segment {
            Segment::Map { key } => steps.push(Step::Key(key.clone())),
            Segment::Seq { index } => steps.push(Step::Index(*index)),
            Segment::Enum { .. } | Segment::Unknown => break,
        }
    }
    while let Some(last) = steps.pop() {
        let Some(parent) = resolve(root, &steps) else {
            continue;
        };
        match (parent, last) {
            (Value::Object(fields), Step::Key(key)) => {
                if fields.remove(&key).is_some() {
                    return true;
                }
            }
            (Value::Array(items), Step::Index(index)) => {
                if index < items.len() {
                    items.remove(index);
                    return true;
                }
            }
            (_, Step::Key(_) | Step::Index(_)) => {}
        }
    }
    false
}

fn resolve<'a>(root: &'a mut Value, steps: &[Step]) -> Option<&'a mut Value> {
    let mut at = root;
    for step in steps {
        at = match step {
            Step::Key(key) => at.get_mut(key.as_str())?,
            Step::Index(index) => at.get_mut(*index)?,
        };
    }
    Some(at)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_file_drops_nothing() {
        let parsed = parse_tolerant(br#"{"aliases": {"light.flur": ["gang"]}, "version": 1}"#).unwrap();
        assert!(parsed.dropped.is_empty());
        assert_eq!(parsed.overlay.aliases["light.flur"], ["gang"]);
    }

    #[test]
    fn a_broken_list_item_is_dropped_and_its_neighbours_stay() {
        let raw = br#"{"policies": [
            {"id": "tor", "effect": "confirm", "when": {"domain": "cover"}},
            {"id": "kaputt", "effect": "teleport"},
            {"id": "schloss", "effect": "block", "when": {"domain": "lock"}}]}"#;
        let parsed = parse_tolerant(raw).unwrap();
        let ids: Vec<_> = parsed.overlay.policies.iter().map(|rule| rule.id.as_str()).collect();
        assert_eq!(ids, ["tor", "schloss"]);
        assert_eq!(parsed.dropped.len(), 2, "{:?}", parsed.dropped);
        assert!(parsed.dropped[0].starts_with("policies[1].effect"), "{:?}", parsed.dropped);
    }

    #[test]
    fn a_section_of_the_wrong_type_falls_back_to_its_default() {
        let parsed =
            parse_tolerant(br#"{"aliases": {"light.flur": ["gang"]}, "custom": "nein", "timer_hints": {"neunzig": "waesche"}}"#).unwrap();
        assert_eq!(parsed.overlay.aliases["light.flur"], ["gang"]);
        assert!(parsed.overlay.custom.is_empty());
        assert!(parsed.overlay.timer_hints.is_empty());
    }

    #[test]
    fn syntax_errors_and_non_objects_are_unreadable() {
        assert!(parse_tolerant(b"{\"aliases\": {").is_err());
        assert!(parse_tolerant(b"[1, 2]").is_err());
        assert!(parse_tolerant(b"null").is_err());
    }

    #[test]
    fn a_newer_version_is_reported() {
        let parsed = parse_tolerant(br#"{"version": 99, "preferred": ["light.flur"]}"#).unwrap();
        assert_eq!(parsed.overlay.preferred, ["light.flur"]);
        assert!(parsed.dropped[0].starts_with("version"));
    }
}
