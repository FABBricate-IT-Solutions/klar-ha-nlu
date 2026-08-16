# API

[Deutsch](api.md) · [English](en/api.md)

Zwei Schnittstellen: HTTP auf Port **10520**, Wyoming auf **10500**.

**Breaking:** `POST /api/parse` entfällt. Clients und die Home-Assistant-Integration müssen `POST /api/v2/parse` und `schema_version: "2.0"` sprechen. Engine und Integration zusammen aktualisieren.

## HTTP

### `POST /api/v2/parse`

```json
{ "text": "Licht im Wohnzimmer an", "conversation_id": "optional-id", "language": "de", "personality": "butler" }
```

Antwort:

```json
{
  "schema_version": "2.0",
  "text": "Licht im Wohnzimmer an",
  "conversation_id": "optional-id",
  "decision": { "type": "execute" },
  "speech": "Sehr wohl. Wohnzimmerlicht ist an.",
  "confidence": 0.96,
  "margin": 0.18,
  "selected_candidate_id": "selected-000",
  "plan": {
    "confidence": 0.96,
    "margin": 0.18,
    "evidence": [],
    "steps": [{
      "index": 0,
      "intent": {
        "name": "HassTurnOn",
        "slots": [
          { "name": "area", "value": "wohnzimmer" },
          { "name": "domain", "value": "light" }
        ]
      },
      "confidence": 0.96,
      "evidence": []
    }]
  },
  "candidates": [],
  "evidence": [],
  "trace": { "stages": [], "discarded": [] },
  "briefing": false
}
```

`text` ist auf 4096 Zeichen begrenzt. Der HTTP-Body ist auf 16 KiB begrenzt.

`language` ist optional (`de`, `en` oder ein BCP-47-Tag wie `en-US`). Ist es gesetzt, bindet Klar nur dieses Paket — Assist kann so zwischen Deutsch und Englisch umschalten. `speech` folgt dem gesetzten Paket.

`personality` ist optional und setzt auf diesem Endpunkt eine Formel vor `speech` (`Sehr wohl.`, `Aye.`, …). Home Assistant speichert die Auswahl in der Integration und schickt sie bei jedem Parse; die Engine-Settings sind nur für die Klar-UI. Die LLM-Verfeinerung in der HA-Integration formuliert den Satz danach in der Stimme um und klebt die Formel nicht wieder davor.

`decision.type` ist einer von `execute`, `clarify`, `confirm`, `reject`, `chat` oder `error`. Nur `execute` enthält `plan`, vollständige `candidates` und `selected_candidate_id`; Clients dürfen Intents ausschließlich in diesem Fall ausführen. Bei allen anderen Entscheidungen ist `candidates` leer und es werden nirgends Intent- oder Slot-Daten serialisiert. `confirm` enthält nur Prompt und eine opake Kandidaten-ID. Der vorgeschlagene Plan bleibt ausschließlich in der Session, bis dieselbe `conversation_id` bejaht wird.

Confidence ist evidenzbasiert und zwischen Intents vergleichbar. Feste Bänder: execute bei Confidence ≥ 0.80 und Margin ≥ 0.05, wenn kein konkurrierender vollständiger Plan näher als 0.05 liegt; confirm für riskante Schloss-/Cover-zu-Aktionen oder große Pläne ab 0.62; clarify bei konkurrierenden vollständigen Plänen unter 0.05 Margin oder Confidence zwischen 0.70 und dem Execute-Band; reject unter 0.70 (unter 0.62 wenn riskant), bei Out-of-Domain (Wetter, Trivia, leere Füllwörter) oder wenn kein geerdeter Schritt übrig bleibt. Fuzzy-, Session- und Inferenz-Evidenz darf exakte Lexikon+Resolver-Evidenz nicht überholen; inferierte Aktionen bekommen nie 1.0. Mehrsatz-Pläne behalten nur unabhängig gültige Schritte. Nach `nein` fällt ein Confirm weg; nach `ja` wird der gespeicherte Plan gegen den aktuellen Home-Graph neu geprüft. `chat` bleibt auf News, Briefing-Follow-ups und explizites LLM-Opt-in beschränkt.

`candidates`, `evidence` und `trace` erklären Ranking und verworfene Alternativen. Alle Listen und Detailtexte sind serverseitig begrenzt.

### Auth und Fehler

Lesende HTTP-Endpunkte sind für Loopback und das Home-Assistant-Supervisor-Netz erlaubt. Schreibende Endpunkte sind ohne Token nur von Loopback erlaubt. Von anderen Hosts braucht der Request einen Token:

```http
x-klar-token: secret
Authorization: Bearer secret
```

Der Token kommt aus `--token`, `KLAR_TOKEN` oder `--token-file`.

| Status | Bedeutung |
|--------|-----------|
| `400` | ungültige Custom Sentence oder Entity-ID |
| `401` | Token fehlt oder passt nicht |
| `404` | Entity nicht gefunden |
| `413` | Parse-Text zu lang |

### `GET` / `POST /api/settings`

```json
{
  "personality": "default",
  "mode": "full",
  "languages": ["de", "en"],
  "support_bundle": false,
  "support_bundle_raw_text": false,
  "confirm_risky_actions": true,
  "semantic_adapters": false
}
```

| Feld | Werte |
|------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (Geräte auflösen) oder `context_only` (nur Räume) |
| `languages` | Paket-Codes. Unbekannte Codes werden ignoriert. Leer fällt auf `de`+`en` zurück. |
| `support_bundle` | `true` speichert Parse-Verkehr unter `/data/support_bundle.jsonl` (max. 2000 Einträge). Überlebt Neustarts. Erststart auch über `KLAR_SUPPORT_BUNDLE=1`. |
| `support_bundle_raw_text` | `true` erlaubt Rohtext und Sprachausgabe im Download. Standard aus. Conversation-IDs werden immer gehasht, Entity- und Area-Namen immer pseudonymisiert. |
| `confirm_risky_actions` | `true` verlangt vor riskanten Aktionen wie Sperren/Entsperren und breiten sicherheitsrelevanten Steuerungen eine Bestätigung. |
| `semantic_adapters` | `true` befragt lokale typisierte Adapter nach einem Ranking-Reject. Standard aus. Vorschläge werden revalidiert und überschreiben Execute/Confirm/Clarify/Chat nicht. |

### `GET` / `POST /api/custom`

Eigene Sätze, exakter oder unscharfer Treffer:

```json
[{ "phrase": "filmabend", "intent": "HassTurnOn", "slots": { "entity_id": "scene.film" } }]
```

Maximal 64 Einträge. Jede Phrase braucht mindestens vier Zeichen und höchstens 200 Bytes; `intent` muss ein bekannter Intent-Name sein.

### `GET /api/entities`

Geladener Home-Graph (Name, Area, Aliase, Tags).

### `POST /api/entities`

```json
{ "entity_id": "light.wohn_decke", "tags": ["decke"] }
```

Tags helfen der Auflösung, wenn der Anzeigename zu generisch ist.

`aliases`, `preferred` und optionale `area` werden im Overlay gespeichert und sofort auf den laufenden `HomeStore` angewendet.

### `GET /api/gaps`

Kalibrieransicht für Entities, die noch schwache Namen, fehlende Areas oder keine hilfreichen Aliase/Tags haben. Die Antwort enthält `leftover`, `rooms` und das aktuelle Overlay.

### `GET /api/dashboard`

Operator-Dashboard für die React-UI:

- `counts`: Graph gesamt, Assist-sichtbar, Räume, offene Zuordnungen, `high`/`medium`/`low`, Bundle-Einträge
- `coverage`: Trichter `all` → `assist` → `high` plus `leftover`
- `rooms`: Raum-Bereitschaft mit offenen Items je Raum
- `assignment`: Geräte mit `confidence`, `suggested_area` und Gründen
- `traffic`: Bundle-Aggregate, Tagesverlauf, letzte Sätze für Replay

### `GET` / `POST /api/ui`

Persistenter UI-Zustand unter `/data/klar_nlu.json`:

```json
{
  "tab": "dashboard",
  "locale": "de",
  "dismissed": ["light.hue_play_1"],
  "last_apply": [],
  "graph": { "light.schlafzimmer": { "x": 120.0, "y": 40.0 } }
}
```

`POST` braucht wie andere Schreibzugriffe den Token, außer von Loopback.

### `POST /api/assignment/apply`

Übernimmt alle nicht verworfenen Raumvorschläge mit Score ≥ 3. Die letzte Charge landet in `ui.last_apply`.

### `POST /api/assignment/undo`

Macht die letzte Auto-Apply-Charge rückgängig und leert `ui.last_apply`.

### `GET /api/bundle`

Status des Support-Bundles: `enabled`, `count`, `bytes`.

### `GET /api/bundle/entries`

Liste der gespeicherten Aufzeichnungen (neueste zuerst, max. 400). Felder: `id`, `ts_ms`, `source`, `text`, `speech`, `intents`.

### `POST /api/bundle/entries`

Auswahl löschen: `{ "ids": ["…"] }`. Antwort wie `GET /api/bundle/entries`.

### `GET /api/bundle/dataset`

Download als Voice-Suite-YAML (`klar-assist-dataset.yaml`).

### `GET /api/bundle/protocol`

Redigiertes JSONL (`klar-support-bundle.jsonl`). Conversation-IDs sind gehasht, Entity-/Area-Namen pseudonymisiert. Ohne `support_bundle_raw_text` steht statt Rohtext die Token-Replay-Kette; Speech ist leer.

### `POST /api/bundle/clear` / `DELETE /api/bundle`

Protokoll leeren.

### `GET /`

Lokale Test-UI (`web/index.html`).

## Wyoming

Klar spricht das Wyoming-Intent-Protokoll (eine JSON-Zeile pro Event).

`describe` → `info` mit Name, Version und `languages` aus den Settings.

`recognize` mit `data.text`:

| Ergebnis | Event |
|----------|--------|
| nichts erkannt | `not-recognized` |
| ein Intent | `intent` |
| mehrere | `intents-start` … `intent` … `intents-stop` |

Jeder Intent trägt `name`, `entities` (`name`/`value`) und `text` (Bestätigung).

Wyoming akzeptiert nur Loopback und das Supervisor-Netz. Es nutzt denselben `HomeStore`, dieselben Sessions und dieselben Settings wie HTTP. Die HA-Custom-Component nutzt HTTP, nicht Wyoming. Wyoming ist für Add-ons und Satelliten gedacht, die einen Intent-Server erwarten.
