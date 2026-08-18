# Sprachen

[Deutsch](languages.md) · [English](en/languages.md)

Jede kompilierte Assist-Locale ist erstklassig. Deutsch und Englisch sind handgeschriebene Referenzpacks; generierte Packs nutzen denselben `LanguagePack`-Weg und dieselbe Freigabe-UX. `GET /api/v2/languages` listet den kompilierten Satz.

YAML unter `packs/` bleibt für User-Overlays und `klar lang import-hassil`, nicht für Assist-Abdeckung. Packs werden nicht still zu einem Riesen-Default-Catalog gemerged.

Russisch (`ru`, `ru-RU`) wird nicht mitgeliefert: kein Pack, kein Registry-Eintrag, `pin_language("ru")` bleibt unbekannt.

HassIL ins Overlay importieren (nicht in einen gemergten Default-Catalog):

```bash
klar lang import-hassil --from pfad/zu/hassil --into /data --language de --dry-run
```

## Aufbau

- `src/lang/de_pack.rs` / `en_pack.rs` — handgeschriebene Referenzpacks
- `src/lang/de.rs` / `en.rs` — Verbtabellen für diese Packs
- `src/lang/packs/{code}/` — generierte `verbs.rs`, `speech.rs`, `pack.rs`
- `src/lang/registry.rs` — kompilierte Ids, `from_code`, `pack()`, `GET /api/v2/languages`
- `scripts/lang_packs/` — Generator (HassIL-Harvest nur Bootstrap). `generate.py` nicht im Pre-Commit ausführen.

Ein generiertes Pack darf in die Binary, wenn diese Felder stehen und die Representative-Suite durchläuft.

## Catalog-Modell

`Catalog` merge't die **gepinnten** Packs pro Request. Assist und `POST /api/v2/parse` sollen `language` senden. Leeres `Settings.languages` heißt: jede kompilierte Locale ist für Assist aktiv — nicht „de+en mergen“. Alle Lexika in einen Catalog zu mergen kollidiert Tokens (z. B. deutsches `an`) und wird abgelehnt.

Eine kurze explizite Liste wie `["de", "en"]` merge't diese Packs weiter für ungepinnten Parse. Das ist eine Nutzerwahl, keine Support-Kaste.

`parse()` bindet `Settings.languages`; Hilfsfunktionen lesen `catalog()`. Neue Engine-Felder gehören auf `LanguagePack` und in das bestehende `extend_sets!`.

## Pack anlegen

1. Kompaktes Lexikon in `scripts/lang_packs/` (kein Stub, keine englischen Lückenfüller).
2. `python3 scripts/lang_packs/generate.py`
3. Rust wie Handcode reviewen: `rustfmt`, gefaltete eindeutige Tokens, keine Kommentar-Narration.
4. Dateien unter 500 Zeilen. Keine `match LangId`-Arme in `src/parse/`.
5. Bestehende Suiten müssen grün bleiben; derselbe Assist/Parity-Smoke für die neue Locale.

`LanguagePack` in `src/lang/groups.rs` ist die Checkliste. Leere Slices nur, wenn die Sprache das Konzept nicht hat.

## Verbklassen

`VerbKind` ist die Rolle eines Wortes, nicht die Home-Assistant-Aktion. Neue Klassen brauchen einen expliziten Arm in `src/parse/action.rs` (kein stilles `_ =>`).

## Zahlen

`NumberStyle`:

- `GermanUnd` — `einundzwanzig`
- `EnglishTens` — `twenty one`
- `ListedOnly` — nur Listenwörter (Default für neue Packs)

Ein neuer Kombinator ist eine neue Variante plus Tests. `De | En`-Matches nicht erweitern.

## Tokens

`fold_latin` mappt `ä` → `ae`, `é` → `e`, `ç` → `c`, `ı/ş/ğ`, `ș/ț`. Packs speichern die gefaltete Form. CJK/Thai-Splits sind script-gated; lateinisches `tokenize` bleibt Space-Split.

## Home Assistant

Die Integration liest `custom_components/klar_nlu/languages.py` (generiert). Die Optionen listen jede kompilierte Locale mit ihrem Eigennamen. Default-Freigabe ist der volle kompilierte Satz. Assist pinnt pro Request ein Pack. `pt-BR` und `de-CH` werden nicht auf ISO-639-1 gestutzt.

## Tests

- `tests/assist_langs.rs` — Execute-Smoke je kompilierter Locale (inkl. de/en)
- `tests/parity_langs.rs` — dieselbe Wohn+Familie+m0+m2-Rubrik je kompilierter Locale
- `tests/datasets/assist/{code}/representative.yaml` — Representative-Gate
- `tests/language.rs` — Pin, Isolation, Overlays, Household-Cues
- DE/EN-Voice-Suiten (`wohnung_mittel`, `wohnung_en`, `familienhaus_de`, `family_home_en`) sind die **Oracle**-Graphen; andere Locales legen native Sätze auf dieselben Graphen

## Datensatz-Generator (jede Locale, lokal)

Ein Befehl schreibt Parity-Overlays für jede generierte Locale (nicht Russisch):

```bash
python3 scripts/parity/generate.py
```

Er liest die DE-Oracles (`wohnung_mittel`, `familienhaus_de`, `m0_exact`, `m2_floors`) plus das Locale-Lexikon und schreibt `tests/datasets/parity/{code}/{suite}/`. Raum-Aliase: `tests/datasets/parity/rooms.yaml`.

DE und EN sind keine Overlays: sie **sind** die Oracles. Neu erzeugen mit `python3 scripts/gen_voice_suite.py` (Familie: `docs/testing.md`). Danach `scripts/parity/generate.py`, damit die anderen Locales mitziehen.

CI prüft, dass der Generator ein No-Op ist. Die volle 65-Locale-Matrix läuft nicht. Ändert ein PR ein Pack- oder Datensatz-Pfad, läuft die Suite dieser Locale (`scripts/ci_lang_tests.py`): de/en hart, andere nur Report. Lokal:

```bash
python3 scripts/lang_packs/generate.py
python3 scripts/parity/generate.py
python3 scripts/check_lang_packs.py
cargo nextest run --test assist_langs --test language --test parity_langs --test voice_suite
```
