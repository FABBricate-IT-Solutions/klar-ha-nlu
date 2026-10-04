# Sprachen

[Deutsch](languages.md) · [English](en/languages.md)

Jede kompilierte Assist-Locale ist erstklassig. Deutsch und Englisch sind handgeschriebene Referenzpacks; generierte Packs nutzen denselben `LanguagePack`-Weg und dieselbe Freigabe-UX. `GET /api/v2/languages` listet den kompilierten Satz.

## Unterstützte Locales

**67** kompilierte Assist-Locales (Quelle: `custom_components/klar_nlu/languages.py` / `GET /api/v2/languages`).

| Code | Name | Code | Name |
|------|------|------|------|
| `de` | Deutsch | `en` | English |
| `fr` | Français | `nl` | Nederlands |
| `es` | Español | `it` | Italiano |
| `pt` | Português | `pt-BR` | Português (Brasil) |
| `ca` | Català | `ro` | Română |
| `da` | Dansk | `nb` | Norsk Bokmål |
| `sv` | Svenska | `fi` | Suomi |
| `de-CH` | Schwyzerdütsch | `de-AT` | Deutsch (Österreich) |
| `en-GB` | English (UK) | `af` | Afrikaans |
| `cs` | Čeština | `sk` | Slovenčina |
| `pl` | Polski | `hu` | Magyar |
| `hr` | Hrvatski | `sl` | Slovenščina |
| `bg` | Български | `el` | Ελληνικά |
| `sr` | Српски | `sr-Latn` | Srpski |
| `uk` | Українська | `tr` | Türkçe |
| `zh-CN` | 简体中文 | `zh-TW` | 繁體中文（台灣） |
| `zh-HK` | 繁體中文（香港） | `ja` | 日本語 |
| `ko` | 한국어 | `th` | ภาษาไทย |
| `ar` | العربية | `he` | עברית |
| `fa` | فارسی | `ur` | اُردُو |
| `cy` | Cymraeg | `et` | Eesti |
| `eu` | Euskara | `ga` | Gaeilge |
| `gl` | Galician | `is` | islenska |
| `lb` | Letzebuergesch | `kw` | kernewek |
| `lt` | Lietuviu | `lv` | Latviesu |
| `id` | Bahasa Indonesia | `ms` | Bahasa Melayu |
| `sw` | Kiswahili | `vi` | Tieng Viet |
| `hi` | Hindi | `bn` | Bangla |
| `gu` | Gujarati | `kn` | Kannada |
| `ml` | Malayalam | `mr` | Marathi |
| `ta` | Tamil | `te` | Telugu |
| `pa` | Punjabi | `ne` | Nepali |
| `hy` | Armenian | `ka` | Georgian |
| `mn` | Mongolian | | |

`de` / `en` sind handgeschriebene Referenzpacks. Andere Locales sind generiert und smoke-getestet; die Qualität variiert.

**Nicht mitgeliefert:** Russisch (`ru`, `ru-RU`) — kein Pack, kein Registry-Eintrag, `pin_language("ru")` bleibt unbekannt.

YAML unter `packs/` bleibt für User-Overlays und `klar lang import-hassil`, nicht für Assist-Abdeckung. Packs werden nicht still zu einem Riesen-Default-Catalog gemerged.

HassIL ins Overlay importieren (nicht in einen gemergten Default-Catalog):

```bash
klar lang import-hassil --from pfad/zu/hassil --into /data --language de --dry-run
```

## Aufbau

- `src/lang/packs/{code}/` — `verbs.rs`, `speech.rs`, `pack.rs` (de/en hand-written reference; others generated)
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

Die Integration liest `custom_components/klar_nlu/languages.py` (generiert). Assist-Sprachen setzt die Operator-UI (Einstellungen): leer = jede kompilierte Locale. Default folgt der Pipeline-/Request-Sprache. Ein einzelnes gepinntes Pack beschränkt weiter nur das Parsing. `pt-BR` und `de-CH` werden nicht auf ISO-639-1 gestutzt.

Operator-Chrome (die offizielle Klar-App) ist **nicht** der Assist-Pin und **nicht** das Home-Assistant-Profil. Setzen in App → Einstellungen → Operator-Sprache, oder mit `KLAR_UI_LOCALE` vor dem ersten Speichern. Gespeicherte Einstellung gewinnt. Integrationsformulare bleiben bei Home Assistant `translations/{lang}.json`.

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

CI prüft, dass der Generator ein No-Op ist, und läuft die volle `parity_langs`-Matrix (jede kompilierte Locale, kein Fail-Fast). Lokal:

```bash
python3 scripts/lang_packs/generate.py
python3 scripts/parity/generate.py
python3 scripts/check_lang_packs.py
cargo nextest run --test assist_langs --test language --test parity_langs --test voice_suite
```
