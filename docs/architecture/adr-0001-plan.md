# Umsetzungsplan — ADR 0001

[Deutsch](adr-0001-plan.md) · [English](adr-0001-plan.en.md)

Rahmen: [ADR 0001](adr-0001-rules-and-trainer.md). Jede Stufe ist ein eigenes PR **gegen `staging`**. Defaults bleiben heutiges Assist-Verhalten, bis jemand eine Spur ändert. Kein Kalender — Abhängigkeit und Risiko steuern die Reihenfolge.

## Lieferkanal: Staging, kein Hauptrelease

Diese Arbeit ist ein **langer Staging-Zyklus**. Nichts davon ist ein Stable-/CalVer-Release, solange nicht ausdrücklich `staging` → `main` freigegeben wird.

| Was | Festlegung |
|-----|------------|
| Basis jedes Umsetzungs-PRs | `staging` (geschützt, per PR mergen) |
| Dieses Plan-/ADR-PR | ebenfalls `staging` |
| Nach Merge auf `staging` | bestehender Staging-Workflow: Prerelease-Tag `{CalVer}-staging.{sha7}`, Image-Tag `staging`, nie `latest` |
| Testen | HA **Release-Kanal = Staging** (`http://klar-nlu-staging:10520` / GitHub-Prerelease) |
| `staging` → `main` | **nicht** Teil dieses Plans. Eigener Promote-PR, erst nach langem Feilen |
| CalVer / `latest` | unberührt, bis dieser Promote bewusst kommt |

Staging-CI fährt Quality+Security wie Release, **nicht** die wöchentliche `parity_langs`-Vollmatrix. Die Locale-Invariante gilt trotzdem: `assist_langs` bleibt PR-Gate; `parity_langs` vor jedem Seed-/Parse-Merge lokal oder per `workflow_dispatch`/`language-parity.yml`, nicht „erst auf main“.

Kein `--admin`, kein Direkt-Push auf `staging` oder `main`.

## Locale-Invariante

Alles gilt für **jede kompilierte Assist-Locale** in `GET /api/v2/languages` (heute 67, inkl. Varianten wie `de-CH`, `pt-BR`, `zh-CN`, `sr-Latn`). de/en sind Hand-Referenzpacks und Oracle-Graphen, **keine Support-Kaste**. Kein `match LangId` in `src/parse/`. Ein Feature, das nur auf Deutsch/Englisch grün ist, ist nicht fertig.

| Schicht | Wie alle Locales mitkommen |
|---------|----------------------------|
| Match | sprachunabhängig (`PolicyId`). Catalog-Ids sind stabil; UI-Texte über Operator-i18n-Keys, nicht hardcodiertes Deutsch. |
| Lexikon | jedes Pack ist die Seed-DB dieser Locale. Overlay `SetDelta` hängt am gebundenen Catalog, nicht an `de`. |
| Govern-Safety | `when.domain=lock` ist sprachlos — **ein** Seed-Bundle für alle Packs, nicht 67 Übersetzungen. |
| Phrase-Seeds / Household | nur über Generator für **alle** Packs im selben PR, analog `scripts/lang_packs`. Nie nur de/en. |
| Trainer-Validate | Dry-Run gegen Representative + Parity **der gebundenen Locale**, nicht nur `familienhaus_de`. |
| Operator-Chrome | jede kompilierte Assist-Locale (`web/src/i18n/en.ts`, `de.ts` und `messages/*.json`) bekommt dieselben Keys, übersetzt — keine englischen Reste. Assist-Qualität hängt nicht am Chrome. |

**Gate jeder Stufe, die Parse oder Seeds berührt:** `cargo nextest run --locked --test assist_langs --test parity_langs` (volle Matrix, kein Fail-Fast), plus die Oracle-Suiten. Russisch bleibt außen vor (kein Pack).

## Festgelegte Entscheidungen

| Punkt | Festlegung |
|-------|------------|
| Match | kompiliert + Overlay (`enabled`, `precedence`), keine Matching-DSL |
| Lexikon | Pack = Seed-DB; Slang nur `LanguageOverlay` `SetDelta` |
| Govern-Seed | ein sprachloses Safety-Bundle für alle Locales; Phrase-Seeds nur generiert für die ganze Matrix |
| Locales | alle kompilierten LangIds; de/en = Oracles, nicht Extra-Support |
| Trainer | nur Operator-UI; Engine ohne Netz; Propose = Context + Validate |
| `compiled_risky` | Untergrenze an, bis Seed-Parity bitgleich ist |
| `origin: trainer` | bleibt sichtbar |
| Precedence | Speichern erlaubt, Evaluate warnt, Reset ein Klick |
| Apply | pro Spur bestätigt |
| Lexikon-Tokens | Preview Pflicht; Apply nur nach grünem Dry-Run oder explizitem Override |
| Overlay-Pfade | Nomina/Cues erweitern; Verben nur neue Tokens |
| Household → Seed | nicht in v1 |

## Stufe 0 — Vertrag, kein Verhalten

Ziel: dieselben Daten, die die UI später zeichnet, existieren im JSON. Parse-Ergebnis und Scorecard unverändert.

**API (additiv, `schema_version` bleibt `2.0`)**

`PolicyTrace` um optionale Felder erweitern (`skip_serializing_if`):

```json
{
  "match": { "id": "area_command", "score": 0.93, "origin": "engine" },
  "seed": null,
  "house": { "id": "prefer-decke", "hit": "prefer_entity", "origin": "operator" },
  "band": "execute",
  "compiled_risky": false,
  "discarded": [{ "id": "grounded_entities", "score": 0.88, "reason": "lower_score" }]
}
```

`GET /api/v2/policies/catalog` — read-only Match-Katalog aus `PolicyId` (`id`, `precedence`, `summary_key`). Kein Overlay, keine locale-spezifischen Texte in der Engine.

**Code:** `src/types/outcome.rs`, `src/nlu/draft.rs` `safety_decision`, `src/parse/policy.rs` (Katalogzeilen), `web/src/types.ts`, `web/src/parseContract.ts`, `tests/contract.rs`.

**Gate:** `cargo nextest run --locked --test contract --test policy --test assist_langs`; Web-Contract akzeptiert die neuen optionalen Keys. Vollmatrix `parity_langs` wenn die Stufe Parse-Felder anfasst; sonst Contract reicht.

**Risiko:** gering. Alte Clients ignorieren unbekannte Felder; Confirm/Clarify serialisieren weiter keinen Plan.

## Stufe 1 — Pfad sichtbar, Spuren starr

Ziel: Tab Regeln zeigt drei Spalten. Evaluate und Labor zeichnen denselben Pfad. Noch keine Toggles an Match/Seed.

**UI**

- `web/src/components/PolicyPath.tsx` — drei Pflichtknoten + Band; `—` = geprüft, nicht getroffen; Klick setzt aktive Spur + Zeile.
- `RulesPage`: Grid drei Spalten. Match aus Catalog (read-only). Sprache: Lexikon-Deltas aus `GET /api/lang/overlay` (read-only) + leere Govern-Liste. Haus: heutiger Editor.
- Labor: `.flow` / `processPath` durch `PolicyPath` ersetzen (Lesen, nicht Schreiben).
- Strings in `web/src/i18n/en.ts` und `de.ts`; jede andere kompilierte Assist-Locale wird mit denselben Keys generiert (kein englischer Fallback).

**Gate:** manuell im Browser Regeln + Labor, einmal `de` und einmal eine generierte Locale (z. B. `ja` oder `ar`). Evaluate mit `language` gepinnt. Contract + `assist_langs` grün.

**Risiko:** gering. Kein Parse-Umbau. Auf schmalen Screens Spalten untereinander, eine offen.

## Stufe 2 — Match- und Lexikon-Steuerung

Ziel: Operator kann Match an/aus und Precedence ziehen; Lexikon-Deltas in derselben Spur schreiben. Defaults = heutiges Verhalten.

**Overlay** in `klar_nlu.json`:

```json
"match_controls": [{ "id": "media", "enabled": false, "precedence": 3 }]
```

Unbekannte Id → `400`. Fehlende Zeile = Engine-Default. Reset = Zeile löschen.

**Code:** `src/home/overlay.rs`, `src/io/policies.rs` (Bundle um `match_controls`), `src/parse/clause.rs` (disabled skippen, Precedence aus Overlay), `src/parse/policy.rs`. Lexikon: bestehende `POST /api/lang/overlay` aus der Spur Sprache aufrufen; `src/lang/validate.rs` `set_field` um fehlende Nomina/Cues erweitern (nicht Verben).

**Tests:** leeres Overlay ≡ heutige Kandidatenliste; `media` aus → keine `PolicyId::Media`-Kandidaten; Locale-Smokes mit Default-Overlay grün. `tests/language.rs` Overlay-add am **gebundenen** Pack (kein de-only Token).

**Gate:** `assist_langs`, `parity_langs`, `policy`, `language`. Evaluate zeigt Warnung, wenn ein Disable ein Smoke-Muster treffen würde (best effort: bekannte PolicyIds wie `area_command` / `all_lights`); Speichern trotzdem. Leeres Overlay darf **keine** Locale gegenüber `main` verschieben.

**Risiko:** mittel. Falsches Disable bricht Assist im Haus, nicht in CI, solange Defaults getestet werden.

## Stufe 3 — Govern-Seed für alle Locales, Verhalten gleich

Ziel: `risky_intent` / `allow_permitted` als sichtbare Seed-Regeln auf **jeder** gebundenen Locale. `compiled_risky` bleibt Floor.

**Daten:** ein sprachloses Bundle, z. B. `src/lang/govern_safety.json`, gebunden mit jedem Pack — nicht 67 Kopien, nicht nur `de`/`en`. Ids `seed:confirm-lock`, `seed:confirm-cover-close`, `seed:block-area-lock`. `when` nur Intent/Domain/Area, keine Phrase. Merge: Haus davor, gleiche Id ersetzt Seed, Seeds nicht in der 64.

Phrase- oder Household-Seeds gehören **nicht** hierher. Wenn sie später kommen: Generator schreibt sie für `LangId::all()` im selben PR.

**Code:** `src/types/policy.rs`, `src/nlu/draft.rs`, `src/nlu/policy_route.rs`, Bind in `src/nlu/context.rs` unabhängig von `LangId`. Toggle schreibt Haus-Override `enabled: false` mit `replaces`.

**Tests:** Lock/Cover-Matrix in `tests/policy.rs` (de) **und** dieselben Intents über `assist_langs` / Representative je Locale, soweit das Set ein Schloss oder Cover hat. Band identisch zu `main`. `policy_trace.seed` gesetzt, wenn nicht nur der Floor greift.

**Gate:** `policy`, `assist_langs`, `parity_langs`; bitgleiche Confirm/Reject-Oracles `familienhaus_de` / `family_home_en` bleiben die Graph-Referenz, gelten aber nicht als einzige Locales.

**Risiko:** hoch. Kleiner when-Fehler ändert Schloss-Confirm in allen Sprachen gleichzeitig. Floor an lassen, bis die volle Matrix grün ist.

## Stufe 4 — Trainer (Context + Validate)

Ziel: LLM richtet ein, Engine bleibt ohne Netz. Context und Validate sind **locale-scoped** (`language` am Request, sonst Assist-Pin).

1. `GET /api/v2/policies/trainer-context?layer=&language=` — Graph, Gaps, Catalog, Seed, Overlays, Schema. Lexikon-Vorschläge nur gegen den gebundenen Pack.
2. UI oder HA-Agent erzeugt JSON (Prompt versioniert, `docs/architecture/trainer-prompt.md` in diesem PR). Prompt listet die Locale, nicht „German house“.
3. `POST /api/v2/policies/propose/validate` — sanitize, Grounding, Dry-Run gegen Representative + Parity **dieser** Locale plus Haus-Smokes.
4. Drawer: Diff, Apply auf die Write-API der Spur.

**Tests:** Fixtures ohne LLM für mindestens eine Referenz (`familienhaus_de`) **und** eine generierte Locale (z. B. `tests/datasets/full_home/ja` oder Parity-Graph). `media_new_matcher` rejected; Lexikon-add eines Partikels der **gebundenen** Locale rejected (nicht hart `an` für jede Sprache).

**Gate:** Unit + Contract; `assist_langs` unverändert. Kein Live-Modell in CI.

**Risiko:** mittel (Prompt-Drift). Validate ist die Kante.

## Stufe 5 — später, nicht v1

- Household-Phrasen in den Seed: **Generator für alle Packs**, Vertrag wie `src/nlu/household.rs`, Gate `assist_langs` + `parity_langs`.
- Trainer als Assist-Gespräch (Pipeline-Sprache = Pack).
- `compiled_risky` hinter Setting, sobald Stufe 3 auf der vollen Matrix hält.
- Pfad-Chip in der Gesprächszeile.

## Reihenfolge und Stopps

```
0 Vertrag → 1 UI-Pfad → 2 Match/Lexikon-Overlay → 3 Seed-Safety (alle Locales)
                                                 → 4 Trainer (locale-scoped)
```

Nicht parallel zu Stufe 3: Seed-Merge ändert `safety_decision`. Stufe 1 darf auf Stufe 0 landen, sobald Catalog+Trace da sind.

Stopp und zurück, wenn: irgendeine Locale in `assist_langs` / `parity_langs` rot wird, Confirm-Lock driftet, Catalog Locales merget, Trainer ohne Validate speichert, eine Stufe nur für de/en landet, oder jemand `staging` → `main` ohne Freigabe öffnet.

## Explizit draußen

- Ein Promote `staging` → `main` ohne ausdrückliche Freigabe
- Eine Stufe oder ein Seed, das nur de/en beliefert
- Neue `PolicyId` aus der UI oder dem Modell
- Pack-Dateien zur Laufzeit ersetzen
- Morphologie, `NumberStyle`, Tokenizer als Overlay
- Alle Lexika in einen Catalog mergen
- LLM in `nlu::parse`
- Assist-Tools für den Trainer
