//! Recorded failure ceilings for suites that are not at zero yet.
//!
//! `tests/fixtures/suite_bounds.yaml` maps suite → locale → number of failing sentences.
//! A locale without an entry has the ceiling 0. More failures than recorded fail the test.
#![allow(dead_code)]

use super::RunStats;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) const BOUNDS_FILE: &str = "tests/fixtures/suite_bounds.yaml";

type Bounds = BTreeMap<String, BTreeMap<String, usize>>;

fn parse_bounds(raw: &str) -> Bounds {
    serde_yaml::from_str(raw).unwrap_or_else(|error| panic!("{BOUNDS_FILE}: {error}"))
}

fn checked_in_bounds() -> Bounds {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BOUNDS_FILE);
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    parse_bounds(&raw)
}

fn bound_in(bounds: &Bounds, suite: &str, code: &str) -> usize {
    bounds.get(suite).and_then(|locales| locales.get(code)).copied().unwrap_or(0)
}

/// `Err` when the suite fails more sentences than recorded, `Ok(true)` when the ceiling can drop.
fn check_bound(suite: &str, code: &str, fail: usize, bound: usize) -> Result<bool, String> {
    if fail > bound {
        return Err(format!("{suite} {code}: {fail} failing sentences, recorded ceiling is {bound} ({BOUNDS_FILE})"));
    }
    Ok(fail < bound)
}

pub(crate) fn assert_within_bound(suite: &str, code: &str, stats: &RunStats) {
    let bound = bound_in(&checked_in_bounds(), suite, code);
    match check_bound(suite, code, stats.fail, bound) {
        Err(message) => panic!("{message}\n{}", stats.fails.join("\n")),
        Ok(true) => println!("{suite} {code}: {} failing sentences, lower the ceiling {bound} in {BOUNDS_FILE}", stats.fail),
        Ok(false) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "full_home_full:\n  de: 1\n  cy: 57\n";

    #[test]
    fn more_failures_than_recorded_is_an_error() {
        let bounds = parse_bounds(SAMPLE);
        let error = check_bound("full_home_full", "de", 2, bound_in(&bounds, "full_home_full", "de")).expect_err("2 > 1");
        assert!(error.contains("2 failing sentences") && error.contains("ceiling is 1"), "{error}");
    }

    #[test]
    fn recorded_count_passes_and_fewer_asks_to_lower_the_ceiling() {
        let bounds = parse_bounds(SAMPLE);
        let bound = bound_in(&bounds, "full_home_full", "cy");
        assert_eq!(check_bound("full_home_full", "cy", 57, bound), Ok(false));
        assert_eq!(check_bound("full_home_full", "cy", 12, bound), Ok(true));
    }

    #[test]
    fn locale_or_suite_without_entry_must_be_at_zero() {
        let bounds = parse_bounds(SAMPLE);
        assert_eq!(bound_in(&bounds, "full_home_full", "fr"), 0);
        assert_eq!(bound_in(&bounds, "parity_conversation", "de"), 0);
        assert!(check_bound("full_home_full", "fr", 1, 0).is_err());
        assert_eq!(check_bound("full_home_full", "fr", 0, 0), Ok(false));
    }

    #[test]
    #[should_panic(expected = "recorded ceiling is 0")]
    fn assert_panics_above_the_checked_in_ceiling() {
        let stats = RunStats { ok: 0, fail: 1, fails: vec!["x".into()] };
        assert_within_bound("no_such_suite", "de", &stats);
    }

    #[test]
    fn checked_in_file_lists_known_suites_and_no_zero_entries() {
        for (suite, locales) in checked_in_bounds() {
            assert!(["full_home_full", "parity_conversation"].contains(&suite.as_str()), "unknown suite {suite} in {BOUNDS_FILE}");
            for (code, bound) in locales {
                assert!(bound > 0, "{suite} {code}: drop the entry instead of recording 0");
            }
        }
    }
}
