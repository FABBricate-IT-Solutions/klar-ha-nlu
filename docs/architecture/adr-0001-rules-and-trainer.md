# ADR 0001 — Sichtbare Regeln, Sprach-Seeds und LLM-Trainer

[Deutsch](adr-0001-rules-and-trainer.md) · [English](adr-0001-rules-and-trainer.en.md)

Status: **vorgeschlagen** (noch keine Implementierung)

Klar bleibt eine deterministische, lokale NLU. Ein LLM darf das Haus **einrichten**, nicht den Parse-Pfad **fahren**.

## Idee, wie wir sie verstehen

Heute steckt Verhalten an zwei Stellen, die beide „Policy“ heißen, aber verschiedene Jobs haben:

1. **Match** — kompilierte Klausel-Strategien in Rust (`PolicyId` in `src/parse/policy.rs` und `src/parse/clause.rs`): *wie* ein Satz zu einem Intent wird (`area_command`, `grounded_entities`, `media`, …).
2. **Govern** — Overlay-Regeln (`PolicyRule` in `src/types/policy.rs`, Tab **Regeln**): *ob* ein schon erkannter Plan ausgeführt, bestätigt, blockiert, bevorzugt oder in eine Antwort/ein Script umgebogen wird.

Zusätzlich liegen **Safety** (`requires_confirmation` für Schlösser/Cover, Infra-Filter, Confidence-Schwellen) und **Haushaltsphrasen** (Uhr, Wetter, Erklären, Undo) fest im Code plus Sprachpack.

Die gewünschte Richtung:

- Ein LLM tritt als **Trainer** auf: es sieht den Home-Graph (Räume, Geräte, Tags, Lücken) und schlägt **passende Govern-Regeln** vor — dieselben `PolicyRule`-Objekte, die der Tab Regeln schon speichern kann.
- Alles, was heute **im Code** feststeckt und Verhalten ändert, soll **sichtbar** sein: gleiche Liste, gleiches Evaluate, gleicher Why-Trace. Sonst bleibt „warum Confirm?“ ein Geheimnis hinter `compiled_risky`.
- Verhalten soll **hausgenau** anpassbar sein, besonders damit der Trainer etwas hat, das er schreiben darf, ohne die Engine umzubauen.

Offene Strategiefrage: alles in die Rules Engine heben (plus vorbefüllte Regel-Datenbanken pro Sprache) — oder Match kompiliert lassen und nur Govern/Safety/Haushaltsdefaults datengetrieben machen.

## Ist-Zustand (knapp)

```
Text
  → tokenize / Katalog der gebundenen Sprache
  → PolicyId-Matching (kompiliert) → Kandidaten
  → Ranking + Evidence
  → Overlay-PolicyRule (Govern) + compiled_risky
  → Band: execute / confirm / clarify / reject / chat
```

Bereits vorhanden und nutzbar:

| Baustein | Ort | Lücke |
|----------|-----|--------|
| Overlay-`PolicyRule` (max. 64) | `klar_nlu.json`, `GET/POST /api/v2/policies` | leer beim Start, kein Seed, kein Origin |
| Evaluate | `POST /api/v2/policies/evaluate` | zeigt Overlay-Treffer + `compiled_risky`, nicht *welche* kompilierte Match-Policy und *warum* riskant |
| `PolicyTrace` | `ParseOutcome` | nur Overlay-Regel + Flag |
| `IntentCandidate.policy` | Ranking | PolicyId-String, nicht in der Regeln-UI |
| Sprachpacks | `src/lang/packs/{code}/` | Lexikon und Haushaltsphrasen, keine Govern-Seeds |
| Household-Route | `src/nlu/household.rs` | Phrase→Aktion im Code |
| Safety | `src/nlu/validation.rs` `risky_intent` | unsichtbar, nicht abschaltbar über Regeln |
| Infra | Tags + `infra_needles.txt` | nur teilweise Overlay (`infra_id` / Tags) |
| Trainer | — | fehlt. LLM in HA redet oder formuliert um, schreibt keine Regeln |

Zwei verschiedene Dinge heißen Policy. Der Trainer darf nur **Govern** schreiben. **Match** ist ein Algorithmus.

## Drei Strategien

### A — Nur UI/Trace, Engine unverändert

Katalog der PolicyIds + erweiterter Why-Trace. Overlay-Regeln bleiben hausgemacht.

- Günstig, kein Benchmark-Risiko.
- Trainer hat nichts Standardmäßiges, das er klonen oder überschreiben kann. Safety bleibt unsichtbar.

### B — Alles in die Rules Engine (Match als Daten)

`area_command`, Resolver, Session-Replay, Media-vs-Licht als DSL in vorbefüllten Datenbanken pro Sprache. Trainer schreibt beliebige Matching-Regeln.

- Ein Modell, maximale Flexibilität.
- Die heutigen PolicyIds sind **Funktionen** (Session, Compounds, Media-Claim, Ranking-Caps), keine `when`/`effect`-Zeilen. Eine DSL, die das ausdrückt, ist eine zweite Sprache plus Interpreter. Das 9.922-Satz-DE/EN-Gate und die Locale-Parity würden an generiertem Matching hängen. Ein LLM, das Match-Regeln erfindet, kann Assist leise zerlegen. Das widerspricht „kein Netz in der Engine“.

**Nicht der Weg**, solange Klar lokal, deterministisch und benchmark-stabil bleiben soll.

### C — Hybrid (Empfehlung)

Drei Schichten mit klaren Schreibrechten:

```
Match (kompiliert, read-only im Katalog)
  PolicyId + Resolver + Ranking + Schwellen

Govern (datengetrieben, sichtbar, überschreibbar)
  Sprach-Seed  →  Haus-Overlay (Operator / Trainer)
  confirm / block / allow / prefer / reply / script / template / llm

Invarianten (kompiliert, selten, immer im Trace)
  Plan-Validierung, Expose-Filter, Schema, optionale Safety-Untergrenze
```

Das LLM sieht das Haus und schreibt **nur Haus-Overlay-Regeln** (plus optional Aliase, Tags, Custom Sentences). Es ändert weder PolicyIds noch Wortlisten der Packs.

Vorbefüllte „Datenbanken pro Sprache“ gibt es dann **zwei**, und sie bleiben getrennt:

1. **Lexikon-Pack** (schon da): Verben, Nomina, Haushaltsphrasen.
2. **Govern-Seed** (neu): Default-Regeln dieser Sprache, als echte `PolicyRule[]`.

## Warum C und nicht B

- Der Trainer braucht ein **enges Schema**. `PolicyRule` hat das schon (`when` + `effect` + `prefer`/`payload`). `sanitize_rules` und Evaluate existieren.
- Sichtbarkeit heißt nicht „derselbe Interpreter“. Match-Policies können als Katalogzeilen erscheinen (`id`, Label, Precedence, „was sie tun“), ohne dass ihre Rust-Funktion zur editierbaren Regel wird.
- Safety, die heute in `risky_intent` steckt, *kann* als Seed-Regel erscheinen (`when.domain = lock` → `confirm`). Dann sieht man sie, kann sie überschreiben, und Evaluate zeigt `matched_rule` statt nur `compiled_risky`.
- Sprach-Seeds geben dem Trainer Vorlagen: „in diesem Haus gibt es `lock.Haustuer` und `cover.rollo_wohnzimmer` — instanziiere die Seed-Regeln auf diese Entity-Ids.“

## Schichtenvertrag

### 1. Match-Katalog (Engine, read-only)

Jede `PolicyId` wird eine Katalogzeile, z. B.:

```json
{
  "id": "area_command",
  "layer": "match",
  "origin": "engine",
  "editable": false,
  "precedence": 8,
  "summary": "Raum + Domain ohne Gerätenamen → Area-Intent"
}
```

Im Parse-Trace steht immer die gewählte Match-Policy (heute schon `candidates[].policy`), plus Verlierer und Margin. Die Regeln-UI zeigt diese Zeilen als **Engine**, nicht als editierbare Liste.

Nicht in diesen Katalog als editierbare Regeln: Tokenizer, Fuzzy, Session-Memory, Compound-Split. Das bleibt Code. Der Trace sagt nur, *dass* sie gegriffen haben.

### 2. Govern-Seed pro Sprache (neu)

Mit dem Lexikon-Pack ausgeliefert, z. B. `src/lang/packs/de/govern.json` (oder generiert wie die Packs). Inhalt sind normale `PolicyRule`s mit stabilen Ids:

| Beispiel-Id | when | effect | Zweck |
|-------------|------|--------|--------|
| `seed:confirm-lock` | domain `lock` | `confirm` | sichtbare Form von `risky_intent` für Schlösser |
| `seed:confirm-cover-close` | domain `cover`, intent `HassTurnOff` | `confirm` | Rollo zu |
| `seed:block-area-lock` | domain `lock` + area gesetzt | `block` | kein „alle Schlösser im Stockwerk“ |
| `seed:prefer-climate` | — | `prefer_entity` | nur wenn `preferred_climate` gesetzt; Trainer füllt `prefer` |

Haushaltsphrasen, die schon wie Regeln aussehen (`reply` / `script`), können in einer späteren Phase hier landen. Uhr/Wetter/Undo bleiben zuerst Code, bis der Seed denselben Vertrag erfüllt (Tests in `tests/policy.rs` / Household-Unit).

Seed-Regeln zählen **nicht** gegen das 64er Haus-Quota. Haus-Regeln mit derselben Id **ersetzen** den Seed. Zusätzliche Haus-Regeln stehen **davor** (erste passende gewinnt, wie heute).

### 3. Haus-Overlay (Operator + Trainer)

Genau das heutige Bundle in `klar_nlu.json`. Neu: `origin` (`operator` \| `trainer`) und `replaces` (Seed-Id). Evaluate läuft gegen den **gemergten** Satz Seed⊕Haus.

### 4. Invarianten

Auch wenn jemand `seed:confirm-lock` abschaltet, bleiben `validate_plan`, Assist-Expose und Schema. Ob `compiled_risky` als unsichtbare Untergrenze erhalten bleibt, ist eine offene Frage unten — Empfehlung: **anfangs ja**, Trace unterscheidet `hit: confirm` (Regel) vs. `compiled_risky: true` (Untergrenze). Wenn Seeds die Tests 1:1 abdecken, kann die Untergrenze hinter einem Setting zurücktreten.

## LLM-Trainer

Kein Parse-Hot-Path. Kein Werkzeug, das Intents ausführt. Optional, Operator-ausgelöst.

```
Home-Graph (sichtbare Entities/Areas/Floors/Tags)
  + Govern-Seed der gebundenen Sprache
  + aktuelle Haus-Regeln
  + Gaps (unbenannte Geräte, fehlende Areas)
  + optionales redigiertes Journal
      → Trainer-Prompt mit JSON-Schema = PolicyRule[]
      → sanitize_rules
      → Grounding: entity_id / area / prefer existieren im Graph
      → Dry-Run: Evaluate auf Haus-Smokes + Locale-Smokes
      → Diff in der UI: übernehmen / ablehnen / einzeln editieren
```

Was der Trainer schreiben **darf**: `PolicyRule`, optional Aliase, `nlu_ignore`/`infra`-Tags, Custom Sentences.

Was er **nicht** darf: PolicyIds, Ranking-Schwellen, Wortlisten, neue Effects außerhalb des Enums, Entity-Ids, die nicht im Graph sind.

Haus-Beispiele, die der Seed nicht kennen kann:

- `when.entity_id = climate.kinderzimmer` → `block` (Kinder-AC nachts)
- `when.phrase = „gute nacht“` → `script.good_night`
- `prefer_entity` für die Wohnzimmer-Decke, wenn mehrere Lichter heißen „Lampe“
- `confirm` nur für `lock.Haustuer`, nicht für das Schuppen-Schloss

Das Modell sitzt dort, wo schon der Fallback-Agent ist (HA). Die Engine bleibt ohne Netz. Der Prompt bekommt nur den Graph-Snapshot und das Schema, keine Assist-Tools.

## Sichtbarkeit („was passiert und warum“)

Ein Why-Trace pro Turn, eine UI-Liste mit drei Origins:

```
matched_match:     area_command          (Engine)
matched_govern:    seed:confirm-lock     (Seed, sichtbar)
overridden_by:     house:allow-shed-lock (Haus, Trainer)
compiled_risky:    false
band:              execute
```

„Was hast du gehört?“ (`household.explain`) und `POST /api/lang/explain` sollen dieselben Ids sprechen, nicht nur `Entscheidung: confirm`.

Der Evaluator im Tab Regeln merkt Seed- vs. Haus-Treffer und die Match-Policy des Rankings.

## Was bewusst *nicht* in die Rules Engine wandert

- `PolicyId`-Funktionen und ihre Precedence
- Confidence-Bänder (`EXECUTE_MIN_CONFIDENCE` …) — höchstens später als dokumentierte Settings, nicht als LLM-Output
- Resolver / Fuzzy / Compounds
- Sprachlexikon (das *ist* schon die Sprach-Datenbank)

Sonst trainiert das LLM die NLU selbst. Genau das soll Klar nicht sein.

## Phasen (technisch, ohne Kalender)

1. **Vokabular + Katalog + Why-Trace**  
   Match-Katalog API, `PolicyTrace` um Match-Id, Seed-Id, Risky-Grund erweitern. Regeln-UI: Engine-Liste read-only. Kein Verhaltenswechsel. Gate: Contract-Tests für Trace-Felder; DE/EN-Scorecard unverändert.

2. **Safety als Seed, Verhalten gleich**  
   `risky_intent` und `allow_permitted` als Seed-Regeln für `de`/`en`. `compiled_risky` bleibt Untergrenze, bis Parity-Tests zeigen, dass Seeds denselben Band erzeugen. Quota: Seed getrennt vom Haus-Limit.

3. **Govern-Seeds für alle kompilierten Locales**  
   Wie Lexikon-Packs: Referenz `de`/`en` handgeschrieben, Rest generiert oder dünn (Safety-Universalien sind sprachunabhängig; Phrase-Seeds nicht). Generator-Freshness wie bei `scripts/lang_packs`.

4. **Trainer-Endpoint**  
   `POST /api/v2/policies/propose` liefert einen Vorschlag (kein Speichern). UI: Diff, Evaluate, Apply. Prompt und Schema versioniert. Grounding-Tests mit `tests/datasets/familienhaus_de` und `family_home_en`.

5. **Optional: Household-Phrasen → Phrase-Regeln**  
   Nur wenn Seed+Overlay denselben Undo/Explain/Clock-Vertrag erfüllen. Sonst bleiben sie Code.

Jede Phase braucht ein eigenes PR; dieses ADR ist die Klammer, kein Implementierungs-Diff.

## Offene Punkte (Entscheidung vor Phase 2/4)

1. Darf der Operator `seed:confirm-lock` wirklich abschalten, oder bleibt `compiled_risky` immer die Untergrenze?
2. Läuft der Trainer nur in der Klar-UI (Empfehlung für v1) oder auch als Assist-Gespräch („richte mein Haus ein“)?
3. Werden Haushaltsphrasen in Phase 5 zu Regeln, oder bleiben sie Lexikon+Code?
4. Soll `origin: trainer` nach Apply wie Operator-Regeln aussehen, oder bleibt die Herkunft dauerhaft sichtbar?
5. Infra-Needles: weiter kompiliert, oder Trainer taggt nur Graph-Entities (`infra` / `nlu_ignore`)?

Empfehlungen dazu: (1) Untergrenze anfangs an, Setting später. (2) Nur UI. (3) Später. (4) Herkunft behalten. (5) Tags am Graph, Needles als Default-Vorschlag.

## Folgen, wenn wir C umsetzen

- Tab Regeln wird die Wahrheit: Engine (sichtbar, starr), Sprache (Seed, überschreibbar), Haus (editierbar, trainierbar).
- Der Trainer hat ein begrenztes Schreibrecht und einen Dry-Run gegen dasselbe Evaluate wie der Mensch.
- Match bleibt schnell, getestet und lokal.
- Die 64-Regel-Grenze gilt nur fürs Haus; Seeds sind ein zweites Bundle.
- Risiko von Phase 2: Seed-Safety muss bitgleich zu `risky_intent` sein, sonst kippt Confirm auf Schlössern.

## Verweise

- Overlay-Regeln: `src/types/policy.rs`, `src/nlu/policy_route.rs`, `src/io/policies.rs`
- Match-Policies: `src/parse/policy.rs`, `src/parse/clause.rs`
- Safety: `src/nlu/draft.rs` `safety_decision`, `src/nlu/validation.rs` `risky_intent`
- Ranking-Katalog (Namen): `IntentCandidate.policy`
- Sprachpacks: [Sprachen](../languages.md)
- API: [API](../api.md) (`/api/v2/policies`, `/api/v2/policies/evaluate`)
