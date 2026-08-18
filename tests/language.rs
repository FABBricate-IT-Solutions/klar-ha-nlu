use klar_nlu::home::default_home;
use klar_nlu::lang::{
    catalog_for, import_hassil, install_runtime_packs, load_runtime_dir, parse_hassil, pin_language, preview, reset_runtime_packs,
    validate_pack, validate_path, ExternalPack, LocaleError, LocaleId, VerbKind,
};
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{ParseDecision, Settings};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

fn packs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs")
}

fn lock_packs() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|err| err.into_inner())
}

fn load_shipped() {
    reset_runtime_packs();
    load_runtime_dir(&packs_dir()).expect("shipped packs");
}

#[test]
fn bcp47_chain_and_unknown_pin() {
    let _guard = lock_packs();
    reset_runtime_packs();
    let us = LocaleId::parse("en-US").unwrap();
    assert_eq!(us.fallback_chain().iter().map(|item| item.tag.as_str()).collect::<Vec<_>>(), ["en-US", "en"]);
    assert_eq!(pin_language("de-DE").unwrap(), "de-DE");
    assert!(matches!(pin_language("ru"), Err(LocaleError::Unknown(_))));
    assert!(matches!(pin_language("ru-RU"), Err(LocaleError::Unknown(_))));
    assert!(matches!(pin_language("zz"), Err(LocaleError::Unknown(_))));
}

#[test]
fn every_compiled_pack_id_matches_registry() {
    for id in klar_nlu::lang::LangId::all() {
        assert_eq!(id.pack().id.code(), id.code(), "pack table must not swallow {id:?}");
    }
}

#[test]
fn every_compiled_locale_has_assist_and_parity_suites() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assist = std::fs::read_to_string(root.join("tests/assist_langs.rs")).expect("assist_langs.rs");
    for id in klar_nlu::lang::LangId::all() {
        let code = id.code();
        assert!(assist.contains(&format!("(\"{code}\",")), "missing assist smoke for {code}");
        if code == "de" || code == "en" {
            continue;
        }
        for suite in ["wohnung_mittel", "familienhaus_de", "m0_exact", "m2_floors"] {
            let dir = root.join("tests/datasets/parity").join(code).join(suite);
            assert!(dir.is_dir(), "missing parity suite {} for {code}", dir.display());
        }
    }
    assert!(root.join("tests/datasets/wohnung_mittel").is_dir());
    assert!(root.join("tests/datasets/wohnung_en").is_dir());
    assert!(root.join("tests/datasets/familienhaus_de").is_dir());
    assert!(root.join("tests/datasets/family_home_en").is_dir());
}

#[test]
fn every_compiled_pack_has_household_cues() {
    for id in klar_nlu::lang::LangId::all() {
        let house = &id.pack().household;
        assert!(!house.teach.is_empty(), "{:?} teach", id.code());
        assert!(!house.undo.is_empty(), "{:?} undo", id.code());
        assert!(!house.clock.is_empty(), "{:?} clock", id.code());
        assert!(!house.weather.is_empty(), "{:?} weather", id.code());
    }
}

#[test]
fn de_catalog_stays_isolated_from_other_compiled_packs() {
    let de = catalog_for(&["de".into()]);
    assert_eq!(de.verb("an"), Some(VerbKind::On));
    assert_eq!(de.verb("allume"), None);
    assert_eq!(de.verb("accendi"), None);
    let en = catalog_for(&["en".into()]);
    assert_eq!(en.verb("on"), Some(VerbKind::OnParticle));
    assert_eq!(en.verb("allume"), None);
    assert!(!klar_nlu::lang::LangId::all().iter().any(|id| id.code() == "ru"));
}

#[test]
fn empty_or_all_codes_do_not_merge_a_default_pair() {
    let empty = catalog_for(&[]);
    assert_eq!(empty.verb("an"), None);
    assert_eq!(empty.verb("on"), None);
    let all = klar_nlu::lang::LangId::compiled_codes();
    let merged_all = catalog_for(&all);
    assert_eq!(merged_all.verb("an"), None);
    assert_eq!(merged_all.verb("on"), None);
}

#[test]
fn shipped_packs_validate() {
    let _guard = lock_packs();
    reset_runtime_packs();
    let report = validate_path(&packs_dir()).expect("registry");
    assert!(report.contains("de-AT ok"), "{report}");
    assert!(report.contains("en-GB ok"), "{report}");
    assert!(report.contains("fr ok"), "{report}");
}

#[test]
fn scoped_overlay_does_not_last_win_verbs() {
    let _guard = lock_packs();
    reset_runtime_packs();
    let de = catalog_for(&["de".into()]);
    let colliding = ExternalPack::from_yaml(
        r#"
klar_lang_pack: "2.0"
id: bad
bcp47: [de-XX]
extends: de
verbs:
  an: Off
"#,
    )
    .unwrap();
    let report = validate_pack(&colliding, Some(de));
    assert!(!report.ok());
    assert!(report.errors.iter().any(|issue| issue.path == "verbs.an"), "{report:?}");
    install_runtime_packs(vec![colliding]);
    let catalog = catalog_for(&["de-XX".into()]);
    assert_eq!(catalog.verb("an"), Some(VerbKind::On));
    reset_runtime_packs();
}

#[test]
fn locale_overlay_adds_tokens_without_editing_rust() {
    let _guard = lock_packs();
    load_shipped();
    assert_eq!(pin_language("fr").unwrap(), "fr");
    assert_eq!(pin_language("fr-FR").unwrap(), "fr-FR");
    let fr = catalog_for(&["fr".into()]);
    assert_eq!(fr.verb("allume"), Some(VerbKind::On));
    assert!(fr.light_nouns.contains("lumiere"));
    let en = catalog_for(&["en".into()]);
    assert_eq!(en.verb("allume"), None);
    assert_eq!(en.verb("colour"), None);
    let gb = catalog_for(&["en-GB".into()]);
    assert_eq!(gb.verb("colour"), Some(VerbKind::Color));
    assert!(gb.light_nouns.contains("bulb"));
    reset_runtime_packs();
}

#[test]
fn morphology_hooks_are_pack_configurable() {
    let _guard = lock_packs();
    reset_runtime_packs();
    let pack = ExternalPack::from_yaml(
        r#"
klar_lang_pack: "2.0"
id: de-morph
bcp47: [de-XX]
extends: de
morphology:
  room_suffixes: [ern]
  color_suffixes: [xyz]
  linking:
    - morpheme: en
      min_rest_len: 5
      require_noun: false
"#,
    )
    .unwrap();
    install_runtime_packs(vec![pack]);
    let catalog = catalog_for(&["de-XX".into()]);
    assert!(catalog.morphology.effective_room_suffixes().contains(&"ern"));
    assert!(catalog.morphology.effective_room_suffixes().contains(&"en"));
    assert_eq!(catalog.color("rotem"), Some("red"));
    assert_eq!(catalog.color("rotxyz"), Some("red"));
    let _bound = klar_nlu::lang::bind(&["de-XX".into()]);
    assert!(klar_nlu::parse::normalize::inflected_eq("schlafzimmerern", "schlafzimmer"));
    reset_runtime_packs();
}

#[test]
fn hassil_import_keeps_plain_phrases_and_reports_templates() {
    let raw = r#"
language: de
intents:
  HassTurnOn:
    data:
      - sentences:
          - "filmabend"
          - "schalte {name} ein"
        slots:
          domain: light
        lists:
          name:
            values: [licht]
"#;
    let imported = parse_hassil(raw, "de", "custom.yaml").unwrap();
    assert_eq!(imported.imported, 1);
    assert_eq!(imported.pack.intents[0].phrase, "filmabend");
    assert_eq!(imported.pack.extends, "de");
    assert!(imported.unsupported.iter().any(|row| row.contains("template") || row.contains("lists")), "{:?}", imported.unsupported);
    let french = parse_hassil("intents: {}\n", "fr", "fr.yaml").unwrap();
    assert_eq!(french.pack.extends, "fr");
}

#[test]
fn cli_preview_pins_english() {
    let out = preview("Turn on the office light", "en", None, None).unwrap();
    assert!(out.contains("language=en"), "{out}");
    assert!(out.contains("decision=execute"), "{out}");
}

#[test]
fn cli_import_hassil_dry_run() {
    let path = std::env::temp_dir().join("klar-m4-hassil.yaml");
    std::fs::write(
        &path,
        r#"
language: de
intents:
  HassTurnOn:
    data:
      - sentences:
          - "filmabend"
"#,
    )
    .unwrap();
    let out = import_hassil(&path, None, Some("de"), true).unwrap();
    assert!(out.contains("imported=1"), "{out}");
    assert!(out.contains("filmabend"), "{out}");
}

#[test]
fn named_command_still_executes_after_locale_pin() {
    let home = default_home();
    let settings = Settings { languages: vec!["en-US".into()], ..Settings::default() };
    let mut session = Session::default();
    let outcome = parse("Turn on the office light", &home, &mut session, &[], &settings);
    assert!(matches!(outcome.decision, ParseDecision::Execute), "{outcome:#?}");
}
