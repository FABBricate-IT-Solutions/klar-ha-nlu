# Sprachen

[Deutsch](languages.md) · [English](en/languages.md)

Jede kompilierte Assist-Locale nutzt denselben `LanguagePack`-Weg. Deutsch und Englisch sind handgeschriebene Referenzpacks; die übrigen Packs sind generiert. Eine Locale ist **Referenz**, **Beta** oder **experimentell** (siehe [Stufen](#stufen)); `GET /api/v2/languages` listet die Locales, die gewählt werden können, jeweils mit ihrer Stufe.

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

## Stufen

Eine Stufe sagt, wie weit eine Locale geprüft ist. Sie nimmt kein Pack weg. Veröffentlichte Tabelle (`scripts/lang_packs/tiers.py`, `GET /api/v2/languages`):

| Stufe | Codes | Gate |
|-------|-------|------|
| **Referenz** | `de`, `en`, `de-AT`, `de-CH`, `en-GB` | Volle An/Aus/Negation-Minimalpaare |
| **Beta** | `es`, `nl`, `pt`, `pt-BR`, `it`, `fr`, `id`, `uk`, `hi` | Keine falsche Aktion + Gold-Sätze (`tests/datasets/gold/`) |
| **Experimentell** | jede andere kompilierte Locale | Hinter `experimental_languages` |

`tests/language_tiers.rs` und `tests/language_gold.rs` entscheiden. Beworbene Tags (`zh`, `zh-Hans`, `no`, `nn`, …) laufen über die Varianten-Tabelle und lassen sich pinnen. In einem zweisprachigen Haus folgt die Antwortsprache der erkannten Äußerung, nicht dem alphabetisch ersten Pack.

Die Engine-Einstellung `experimental_languages` (standardmäßig aus) ist der Schalter:

- Eine Locale, die schon konfiguriert ist (`languages`, `default_language`), bleibt nutzbar und gelistet, egal welche Stufe. Ein Update nimmt einem Haus keine Sprache weg.
- Ein Request, der eine Sprache pinnt (`language` bei `POST /api/v2/parse`, die Sprache der Assist-Pipeline), wird in dieser Sprache beantwortet, egal in welcher Stufe.
- Mit ausgeschaltetem Schalter wird eine experimentelle Locale nicht zur Neuauswahl gelistet und ein Settings-Save, der eine hinzufügt, mit `422` abgelehnt. Referenz und Beta sind immer angeboten.


## Catalog-Modell

`Catalog` merge't die **gepinnten** Packs pro Request. Assist und `POST /api/v2/parse` sollen `language` senden. Leeres `Settings.languages` heißt: jede kompilierte Locale darf gepinnt werden — nicht „de+en mergen“. Ein Request ohne Sprache bindet dann das Standardpack (`Settings.default_language`, Englisch, wenn nichts gesetzt ist), nie einen leeren Catalog. Alle kompilierten Locales auf einmal zu binden kollidiert Tokens (z. B. deutsches `an`); die Engine liefert in dem Fall einen leeren Catalog statt eines Riesen-Merges. Eine neue experimentelle Locale bei ausgeschaltetem Schalter wird mit `422` abgelehnt.

Eine kurze explizite Liste wie `["de", "en"]` merge't diese Packs weiter für ungepinnten Parse. Das ist eine Nutzerwahl, keine Support-Kaste.

### Regionale Packs

`de-AT`, `de-CH`, `en-GB` und `pt-BR` sind regionale Packs. Ihre Registry-Zeile nennt ein `parent` (`de`, `en`, `pt`), und der Catalog bindet das regionale Pack über dieses Elternpack: Erst werden die Wörter des Elternpacks gelesen, das regionale Pack ergänzt eigene und gewinnt bei gleichem Schlüssel. Antworten und Haushaltstexte bleiben regional. `GET /api/v2/languages` nennt das `parent` jeder Locale.

Ein regionales Pack enthält nur, was die Region anders sagt. In `scripts/lang_packs` ist das `like` (ein regionales Wort und das Standardwort, für das es steht: `"aa": "an"`; der Generator schreibt es in jede Liste des Elternpacks, in der das Standardwort steht, mit derselben Verbklasse) und `add` (Wörter ohne einzelnes Standardwort, nach Pack-Pfad). Ein regionales Wort, das das Elternpack schon kennt, lehnt der Generator ab.

Ein regionales Pack hat keine eigenen Sätze. `tests/regional_overlays.rs`, die Paritätsmatrix und die Full-Home-Stufen fahren die Sätze des Elternpacks unter dem regionalen Code und erwarten dieselben Entscheidungen.

`parse()` bindet `Settings.languages`; Hilfsfunktionen lesen `catalog()`. Neue Engine-Felder gehören auf `LanguagePack` und in das bestehende `extend_sets!`.

## Pack anlegen

1. Kompaktes Lexikon in `scripts/lang_packs/`. Zielsprachige Wörter bevorzugen; `scripts/lang_packs/pack_lint.py` schlägt fehl, wenn Chat/Speech-Felder noch englische Stub-Füller tragen.
2. `python3 scripts/lang_packs/generate.py`
3. Rust wie Handcode reviewen: `rustfmt`, eindeutige Tokens nach `fold_latin`, keine Kommentar-Narration.
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

`fold_latin` (NFKC, Kleinbuchstaben, dann Fold pro Skript) mappt `ä` → `ae`, `é` → `e`, `ç` → `c`, `ı/ş/ğ`, `ș/ț`. Pack-Tabellen halten lateinische Digraphen, die schon zu diesem Fold passen (z. B. `oeffnen`); Sprache und Labels laufen zur Parse-Zeit weiter durch `fold_latin`. CJK/Thai-Splits sind script-gated; lateinisches `tokenize` bleibt Space-Split.

## Home Assistant

Die Integration liest `custom_components/klar_nlu/languages.py` (generiert). Assist-Sprachen setzt die Operator-UI (Einstellungen): leer = jede kompilierte Locale. Default folgt der Pipeline-/Request-Sprache. Ein einzelnes gepinntes Pack beschränkt weiter nur das Parsing. `pt-BR` und `de-CH` werden nicht auf ISO-639-1 gestutzt.

Operator-Chrome (die offizielle Klar-App) ist **nicht** der Assist-Pin und **nicht** das Home-Assistant-Profil. Setzen in App → Einstellungen → Operator-Sprache, oder mit `KLAR_UI_LOCALE` vor dem ersten Speichern. Gespeicherte Einstellung gewinnt. Integrationsformulare bleiben bei Home Assistant `translations/{lang}.json`.

## Tests

- `tests/assist_langs.rs` — Execute-Smoke je kompilierter Locale (inkl. de/en)
- `tests/parity_langs.rs` — dieselbe Wohn+Familie+m0+m2-Rubrik je kompilierter Locale
- `tests/datasets/assist/{code}/representative.yaml` — Representative-Gate
- `tests/language.rs` — Pin, Isolation, Overlays, Household-Cues
- `tests/regional_overlays.rs` — ein regionales Pack entscheidet auf den Suiten des Elternpacks wie dieses
- `tests/language_tiers.rs` — Minimalpaare je Locale entscheiden die Stufe; leeres `languages` bindet das Standardpack
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
cargo nextest run --locked -E 'test(/assist_langs|language|parity_langs|voice_suite/)'
```

## Natürliche niederländische Befehle

Das niederländische Paket unterstützt getrennte Partikeln (`doe … uit`,
`schakel … aan`), zusammengesetzte Verben (`uitdoen`, `aandoen`, `uitschakelen`,
`inschakelen`), höfliche Modalfragen und die Pluralformen `lampen`/`lichten`.
Die Regeln gelten für verschiedene Räume und Geräte, ohne einzelne Sätze zu
speichern. Quelle: `scripts/lang_packs/lexicons_west.py`; bei Änderungen nur das
niederländische Paket neu erzeugen.

Fensterfragen wie `Welke ramen staan er open?` erzeugen eine reine Leseabfrage
`HassGetState` mit `domain: binary_sensor`, `device_class: window`, `state: on`
und `response: list`. Geschlossene Fenster verwenden `state: off`. Ein bekannter
Raum begrenzt die Abfrage; unbekannte Räume und widersprüchliche Zustände werden
abgelehnt. Home Assistant ermittelt den aktuellen Zustand der für Assist
freigegebenen Fenstersensoren. Die niederländische Antwort nennt nur passende
Sensoren und meldet auch leere Ergebnisse. `response` steuert Klars Sprachausgabe
und wird vor dem HA-Intent-Aufruf entfernt.

Verneinungen (`niet`, `geen`, `nooit`, `nee`, `zonder`) werden vor eigenen
Satzregeln, Bestätigung und Kontextwiederholung geprüft. Die gesamte Anfrage
wird abgelehnt und ausstehende Rückfragen/Bestätigungen werden gelöscht.
Auch verneinte Abfragen und Freitext mit diesen Wörtern werden vorsorglich
abgelehnt, bis der Geltungsbereich sicher darstellbar ist. Verneinung wird weder
als Füllwort entfernt noch in einen gegenteiligen Befehl umgewandelt.

Regressionstests: `cargo nextest run --locked -E 'test(/dutch/)'`, niederländische Paritätstests sowie
HA-Dispatch-, Raumabfrage- und Sprachsnapshot-Tests. Die Tests verwenden lokale
Testdaten und greifen nicht auf eine laufende Home-Assistant-Installation zu.
