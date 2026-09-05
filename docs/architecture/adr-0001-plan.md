# Umsetzungsplan — ADR 0001

[Deutsch](adr-0001-plan.md) · [English](adr-0001-plan.en.md)

Rahmen: [ADR 0001](adr-0001-rules-and-trainer.md). Jede Stufe ist ein eigenes PR. Defaults bleiben heutiges Assist-Verhalten, bis jemand eine Spur ändert. Kein Kalender — Abhängigkeit und Risiko steuern die Reihenfolge.

## Festgelegte Entscheidungen

| Punkt | Festlegung |
|-------|------------|
| Match | kompiliert + Overlay (`enabled`, `precedence`), keine Matching-DSL |
| Lexikon | Pack = Seed-DB; Slang nur `LanguageOverlay` `SetDelta` |
| Govern-Seed | `PolicyRule[]` pro Locale, Haus ersetzt per Id |
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

`GET /api/v2/policies/catalog` — read-only Match-Katalog aus `PolicyId` (id, precedence, summary). Kein Overlay.

**Code:** `src/types/outcome.rs`, `src/nlu/draft.rs` `safety_decision`, `src/parse/policy.rs` (Katalogzeilen), `web/src/types.ts`, `web/src/parseContract.ts`, `tests/contract.rs`.

**Gate:** `cargo nextest run --locked --test contract --test policy`; Web-Contract akzeptiert die neuen optionalen Keys. DE/EN-Voice-Suiten unverändert.

**Risiko:** gering. Alte Clients ignorieren unbekannte Felder; Confirm/Clarify serialisieren weiter keinen Plan.

## Stufe 1 — Pfad sichtbar, Spuren starr

Ziel: Tab Regeln zeigt drei Spalten. Evaluate und Labor zeichnen denselben Pfad. Noch keine Toggles an Match/Seed.

**UI**

- `web/src/components/PolicyPath.tsx` — drei Pflichtknoten + Band; `—` = geprüft, nicht getroffen; Klick setzt aktive Spur + Zeile.
- `RulesPage`: Grid drei Spalten. Match aus Catalog (read-only). Sprache: Lexikon-Deltas aus `GET /api/lang/overlay` (read-only) + leere Govern-Liste. Haus: heutiger Editor.
- Labor: `.flow` / `processPath` durch `PolicyPath` ersetzen (Lesen, nicht Schreiben).
- Strings in `web/src/i18n/en.ts` und `de.ts`; andere Locales fallen auf `en` zurück.

**Gate:** manuell im Browser Regeln + Labor mit `Licht im Wohnzimmer an` und einem Schloss-Satz. Contract weiter grün.

**Risiko:** gering. Kein Parse-Umbau. Auf schmalen Screens Spalten untereinander, eine offen.

## Stufe 2 — Match- und Lexikon-Steuerung

Ziel: Operator kann Match an/aus und Precedence ziehen; Lexikon-Deltas in derselben Spur schreiben. Defaults = heutiges Verhalten.

**Overlay** in `klar_nlu.json`:

```json
"match_controls": [{ "id": "media", "enabled": false, "precedence": 3 }]
```

Unbekannte Id → `400`. Fehlende Zeile = Engine-Default. Reset = Zeile löschen.

**Code:** `src/home/overlay.rs`, `src/io/policies.rs` (Bundle um `match_controls`), `src/parse/clause.rs` (disabled skippen, Precedence aus Overlay), `src/parse/policy.rs`. Lexikon: bestehende `POST /api/lang/overlay` aus der Spur Sprache aufrufen; `src/lang/validate.rs` `set_field` um fehlende Nomina/Cues erweitern (nicht Verben).

**Tests:** leeres Overlay ≡ heutige Kandidatenliste; `media` aus → keine `PolicyId::Media`-Kandidaten; Locale-Smokes mit Default-Overlay grün. `tests/language.rs` Overlay add `funzel`.

**Gate:** `assist_langs`, `parity_langs`, `policy`, `language`. Evaluate zeigt Warnung, wenn ein Disable ein Smoke-Muster treffen würde (best effort: bekannte PolicyIds wie `area_command` / `all_lights`); Speichern trotzdem.

**Risiko:** mittel. Falsches Disable bricht Assist im Haus, nicht in CI, solange Defaults getestet werden.

## Stufe 3 — Govern-Seed de/en, Verhalten gleich

Ziel: `risky_intent` / `allow_permitted` als sichtbare Seed-Regeln. `compiled_risky` bleibt Floor.

**Daten:** `src/lang/packs/de/govern.json`, `en/govern.json` — Ids `seed:confirm-lock`, `seed:confirm-cover-close`, `seed:block-area-lock`. Merge: Haus-Regeln davor, Haus mit gleicher Id ersetzt Seed, Seeds nicht in der 64.

**Code:** `src/types/policy.rs` (Origin/replaces optional), `src/nlu/draft.rs`, `src/nlu/policy_route.rs`. Toggle am Seed schreibt Haus-Override `enabled: false` mit `replaces`, löscht nicht das Pack.

**Tests:** dieselben Lock/Cover-Fälle in `src/nlu/validation.rs` und `tests/policy.rs` — Band identisch zu heute, `policy_trace.seed` gesetzt wenn Floor nicht allein greift.

**Gate:** bitgleiche Confirm/Reject-Matrix vs. `main` auf `familienhaus_de` / `family_home_en` plus `tests/policy.rs`.

**Risiko:** hoch. Kleiner when-Fehler ändert Schloss-Confirm. Deshalb Floor an lassen, bis diese Stufe grün ist.

## Stufe 4 — Seeds für die übrigen Locales

Ziel: jede kompilierte Locale hat denselben Safety-Universalsatz (sprachunabhängig). Phrase-Seeds nur wo das Pack Household-Phrasen hat — nicht in dieser Stufe.

Generator analog `scripts/lang_packs`: `de`/`en` Referenz kopieren. Freshness-Check wie Packs.

**Gate:** `parity_langs`; Confirm-Lock-Smoke je Locale, wo das Representative-Set ein Schloss hat, sonst Skip.

**Risiko:** niedrig, wenn Stufe 3 hält. Dünne Seeds sind besser als übersetzte Phrase-Falschlinge.

## Stufe 5 — Trainer (Context + Validate)

Ziel: LLM richtet ein, Engine bleibt ohne Netz.

1. `GET /api/v2/policies/trainer-context?layer=` — Graph-Sichtbares, Gaps, Catalog, Seed, aktuelle Overlays, Schema der Spur. Kein Roh-Journal ohne Settings.
2. UI oder HA-Agent erzeugt JSON (Prompt versioniert, im Repo unter `docs/architecture/trainer-prompt.md`, erst in diesem PR).
3. `POST /api/v2/policies/propose/validate` — `sanitize_*`, Grounding (Entity/Area/`prefer` im Graph, Match-Id im Catalog, Lexikon-Pfad in `set_field`), Dry-Run gegen Haus-Smokes + Locale-Smokes der gebundenen Sprache.
4. Drawer: Diff, Checkbox, Übernehmen ruft die bestehende Write-API der Spur.

**Schemas:** wie ADR-Tabelle (Match / Lexikon / Seed / Haus). Unbekannte Match-Id und fremde Entity → Zeile `rejected`, nicht 500.

**Tests:** Fixtures ohne LLM: Graph `familienhaus_de` → erwartete Vorschläge für Klima/Lock; `media_new_matcher` rejected; Lexikon-add mit Partikel `an` rejected.

**Gate:** Unit + Contract für Context/Validate. Kein Live-Modell in CI.

**Risiko:** mittel (Prompt-Drift). Validate ist die Kante; das Modell darf nur vorschlagen.

## Stufe 6 — später, nicht v1

- Household-Phrasen (Uhr, Wetter, Undo, Erklären) in den Govern-Seed, nur bei gleichem Vertrag wie `src/nlu/household.rs`.
- Trainer als Assist-Gespräch.
- `compiled_risky` hinter Setting, sobald Stufe 3/4 lange grün sind.
- Pfad-Chip in der Gesprächszeile.

## Reihenfolge und Stopps

```
0 Vertrag → 1 UI-Pfad → 2 Match/Lexikon-Overlay → 3 Seed-Safety de/en
                                                 → 4 Locale-Seeds
                                                 → 5 Trainer
```

Nicht parallel zu Stufe 3: Seed-Merge ändert `safety_decision`. Stufe 1 darf auf Stufe 0 landen, sobald Catalog+Trace da sind.

Stopp und zurück, wenn: DE/EN-Oracle-Suiten rot, Confirm-Lock driftet, Catalog merget Locales, Trainer ohne Validate speichert.

## Explizit draußen

- Neue `PolicyId` aus der UI oder dem Modell
- Pack-Dateien zur Laufzeit ersetzen
- Morphologie, `NumberStyle`, Tokenizer als Overlay
- Alle Lexika in einen Catalog mergen
- LLM in `nlu::parse`
- Assist-Tools für den Trainer
