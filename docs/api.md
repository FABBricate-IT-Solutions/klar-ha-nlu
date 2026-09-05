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

`language` ist optional (`de`, `en`, `fr` oder ein BCP-47-Tag wie `en-US`). Ist es gesetzt, bindet Klar nur dieses Paket. `speech` folgt dem gesetzten Paket.

`personality` ist optional und setzt auf diesem Endpunkt eine Formel vor `speech` (`Sehr wohl.`, `Aye.`, …). Die Engine-Settings sind die Quelle (`GET`/`POST /api/settings`); die Operator-UI Settings-Seite ist der Editor. Home Assistant lässt Persönlichkeit am Parse weg, wenn gespeicherte Engine-Settings da sind, und fällt nur auf übrige Integrationsoptionen zurück, wenn der Cache leer ist. Die LLM-Verfeinerung formuliert den Satz danach in der Stimme um und klebt die Formel nicht wieder davor.

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
  "semantic_adapters": false,
  "nlu_rag": false
}
```

| Feld | Werte |
|------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (Geräte auflösen) oder `context_only` (nur Räume) |
| `languages` | Paket-Codes. Unbekannte Codes werden ignoriert. Leer heißt: jede kompilierte Locale ist aktiv; der Catalog bindet trotzdem pro Request. Nicht alle Lexika mergen — Token-Kollisionen zerlegen Assist. |
| `support_bundle` | `true` speichert Parse-Verkehr unter `/data/support_bundle.jsonl` (max. 2000 Einträge). Überlebt Neustarts. Erststart auch über `KLAR_SUPPORT_BUNDLE=1`. |
| `support_bundle_raw_text` | `true` erlaubt Rohtext und Sprachausgabe im Download. Standard aus. Conversation-IDs werden immer gehasht, Entity- und Area-Namen immer pseudonymisiert. |
| `confirm_risky_actions` | `true` verlangt vor riskanten Aktionen wie Sperren/Entsperren und breiten sicherheitsrelevanten Steuerungen eine Bestätigung. |
| `semantic_adapters` | `true` befragt lokale typisierte Adapter nach einem Ranking-Reject. Standard aus. Vorschläge werden revalidiert und überschreiben Execute/Confirm/Clarify/Chat nicht. |
| `nlu_rag` | `true` hängt nur bei `chat` und `reject` einen gematchten Ausschnitt an. Standard aus. Nie Assist-Werkzeuge; der HA-Fallback darf einen Befehl nur über Klar-Werkzeuge nachziehen. `POST /api/v2/parse` kann `nlu_rag` pro Request setzen. |
| `refine_speech` | `true` formuliert fertige NLU-Sprache mit dem Engine-LLM um. Standard aus. |
| `extra_prompt` | Hausregel als User-Nachricht. Leer = nur Pack-Stimme. Ersetzt die Persönlichkeit nicht. |
| `quiet_ack` | `true` spielt bei einfachem An/Aus einen Chime statt TTS. Standard aus. |
| `calendar_llm` | `true` lässt das Engine-LLM Kalendertermine sprechen. Standard aus. |
| `allow_llm_tools` | `true` lässt das Engine-Chat-Modell nach dem Klar-Parse Home-Assistant-Assist-Werkzeuge nutzen (2026.9, prefixierte Namen). Standard aus. |

### `GET` / `POST /api/v2/llm/endpoint`

OpenAI-kompatibler Upstream der Engine. `GET` liefert `{ "configured", "base_url", "model" }` — nie den API-Key. `POST` setzt `base_url`, `api_key`, `model` in `data_dir/llm_endpoint.json` (nicht im Overlay). Leerer `api_key` behält den gespeicherten Key. `configured: false` löscht die Datei. `KLAR_LLM_*` gewinnt beim Start. Config in der Operator-UI; Assist braucht keine andere Home-Assistant-LLM-Integration. `nlu::parse` nutzt ihn nicht.

### `POST /api/v2/llm/chat`

```json
{ "messages": [{ "role": "user", "content": "…" }], "stream": true, "temperature": 0.2, "max_tokens": 2048 }
```

Write-Token nötig (wie Overlay). `stream: true` (Standard) sendet SSE `data: {"type":"delta"|"done"|"error",…}`. `stream: false` antwortet mit JSON `{"type":"done","text":"…"}`. 503 wenn kein Endpoint.

### `POST /api/v2/llm/refine`

```json
{ "speech": "Wohnzimmer Licht ist an.", "language": "de", "personality": "butler", "extra_prompt": "", "stream": false }
```

Engine baut den Refine-Systemprompt (Pack + Stimme) und schickt Extra als User-Nachricht, ruft das Modell (`temperature` 0.65, `max_tokens` 192) und prüft `accept_refined`. JSON `{"type":"done","text":"…","accepted":true}`. Abgelehnt: `text` ist das Original, `accepted` false. Caps: `speech` ≤ 4096, `extra_prompt` ≤ 2048. Write-Token. 503 ohne Endpoint. Python-Prompt nicht mitschicken.

### `POST /api/v2/llm/assist`

```json
{
  "text": "erzähl einen Witz",
  "language": "de",
  "personality": "butler",
  "kind": "auto",
  "allow_tools": false,
  "nlu_rag": false,
  "retrieval": null,
  "facts": null,
  "history": [["user", "…"], ["assistant", "…"]],
  "extra_system": null,
  "stream": true
}
```

Die Engine besitzt Yarn/Chat/RAG/Kalender/News-Prompts und `yarn_canned` / `yarn_nudge`. `kind`: `auto` | `yarn` | `chat` | `rag` | `calendar` | `news` | `news_follow`. `auto` nutzt `yarn_request` / RAG-Flag. `facts` sind Schlagzeilen oder Kalender-Readback aus HA. Persönlichkeit sitzt hier — `refine_prompt` nicht zusätzlich voranstellen. SSE ergänzt `{"type":"tool","tool":"klar.parse","text":"licht an"}` / `klar.act`, damit TTS nie `KLAR_PARSE:` spricht. Write-Token. 503 ohne Endpoint. Fehlende Route fällt geschlossen fehl — Home Assistant baut den Python-Prompt nicht nach.

### `POST /api/v2/speech/render`

Post-Execute-Snapshot aus Home Assistant. Die Engine interpoliert Pack-Templates zu einem faktischen Satz (`source: "post_execute"`). Persönlichkeit kommt später beim Assist-Finish. Assist ruft das nach Execute; fehlende Route fällt geschlossen fehl (kein Python-`from_handled`). Write-Token nötig.

```json
{
  "schema_version": "1",
  "language": "de",
  "personality": "default",
  "now": "2026-09-05T19:22:00+02:00",
  "intent": {
    "name": "HassTurnOn",
    "slots": [{"name": "area", "value": "wohnzimmer"}]
  },
  "outcome": "success",
  "entities": [
    {
      "entity_id": "light.wohnzimmer",
      "name": "Wohnzimmer",
      "domain": "light",
      "state": "on",
      "area": "wohnzimmer",
      "area_name": "Wohnzimmer",
      "device_class": null,
      "attributes": {
        "current_temperature": null,
        "temperature_unit": null,
        "unit_of_measurement": null,
        "hvac_action": null,
        "hvac_mode": null,
        "volume_level": null,
        "is_volume_muted": null,
        "media_title": null,
        "media_artist": null,
        "media_album_name": null
      }
    }
  ],
  "calendar_events": [],
  "media_queue": []
}
```

Antwort: `{ "speech": "Licht im Wohnzimmer ist an.", "quiet_ack": false, "source": "post_execute" }`. `outcome` ist `success` | `partial` | `error`. `now` ist Pflicht (Uhr). Unbekannte Attribut-Keys werden verworfen. Fehlendes `schema_version` → `400`. Caps: 32 Entities, 16 Kalenderzeilen, 8 Queue-Titel, Attributwerte ≤ 256 Zeichen. Fehlendes Klima-/MASS-Attribut → Unavailable-Satz, keine Erfindung.

### `POST /api/v2/policies/trainer/chat`

```json
{ "message": "Funzel als Licht-Wort", "layer": "language", "language": "de", "history": [] }
```

Die Engine lädt Trainer-Context, streamt das Modell, extrahiert JSON und hängt `proposal` plus `validate` an den Stream. Apply bleibt ein menschlicher Schreib-Call. Siehe [trainer-prompt.md](architecture/trainer-prompt.md).

### `POST /api/v2/home`

Live-Home-Graph von der Home-Assistant-Integration. `schema_version` muss `"1"` sein. Caps und Fehlercodes: [home-assistant.md](home-assistant.md#registry-sync-ha-ist-quelle). Nach einem gültigen Push ist HA die laufende Quelle; die `.storage`-Überwachung überschreibt diesen Graph nicht mehr.

### `GET /api/v2/languages`

Metadaten der kompilierten Packs (`code`, `native_name`, `script`, `variants`). Das ist die erstklassige Assist-Locale-Liste in der Binary.

### `GET` / `POST /api/lang/overlay`

Benutzer-Sprach-Overlay plus Custom Sentences, unter `/data/klar_nlu.json`.

```json
{
  "custom": [{ "phrase": "filmabend", "intent": "HassTurnOn", "slots": { "entity_id": "scene.film" } }],
  "language": { "sets": {} },
  "label": "save"
}
```

Antwort enthält `history` (`hash`, `label`, `saved_at`). Ohne `language` im POST bleiben gespeicherte Set-Deltas. Ungültiges Custom/Overlay → `422`. Write-Token außer Loopback.

### `POST /api/lang/preview`

Parse mit optional ungespeichertem `custom` und `language_overlay`. Installiert das Overlay nicht. Gleiche Textgrenze wie Parse (`413` wenn zu lang). Optionales `language` pinnt ein Pack (`422` wenn unbekannt).

### `POST /api/lang/explain`

Gleicher Body wie Preview. Liefert `decision`, `confidence`, `speech`, `stages`, `evidence` und `matched_custom` — ohne Live-Overlay.

### `POST /api/lang/rollback`

`{ "hash": "…" }` stellt diese History-Zeile wieder her. Ohne `hash` die letzte gespeicherte Revision. `404`, wenn nichts da ist.

### `GET` / `POST /api/v2/policies`

Policy-Bundle für den Tab **Regeln**: `{ "policies": […], "speech_bank": { "entries": […] } }`.

Jede Regel: `id`, `enabled`, `label`, `when` (optional `intent` / `domain` / `area` / `entity_id` / `floor` / `name` / `phrase`), `effect`, optional `prefer` / `payload`. Effects: `confirm`, `block`, `allow`, `prefer_entity`, `prefer_area`, `reply`, `script`, `template`, `llm`. Höchstens 64 Regeln. Ungültiger Body → `400`. POST schreibt ins Overlay.

### `POST /api/v2/policies/evaluate`

Dry-Run-Parse mit optionalen `policies` (sonst der gespeicherte Satz). Body: `{ "text", "language?", "policies?" }`. Antwort: `outcome`, `compiled_risky`, `matched_rule`, `hit`, `speech_variant`.

### `GET /api/v2/conversations`

Gesprächsjournal: letzte **200** Turns, **24 Stunden**, Datei `/data/conversations.jsonl`. Felder: `conversation_id`, `ts_ms`, optionales `text`, `decision`, `speech`, `confidence`, `briefing`, `evidence_kinds`, `last_names`, optional `confirm_prompt` / `candidate_id`. Roh-`text` nur bei `support_bundle_raw_text`. Confirm/Clarify enthalten keinen Plan.

### `GET /api/v2/conversations/{id}`

Turns einer `conversation_id` (`400`, wenn die Id länger als 128 Zeichen ist).

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

`locale` ist nur Operator-Chrome. Eine gespeicherte Einstellung (`locale_set: true`) gewinnt. Bis dahin nutzt `GET` `KLAR_UI_LOCALE`, sonst `en`. Das ist nicht der Assist-Pin und nicht `Accept-Language`.

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

React-Operator-UI aus `web/` (Home, Gespräche, Regeln, Haus / Zuordnung, Labor, Einstellungen). Im Image unter `/usr/share/klar/ui`, nach `npm run build` unter `web/dist`. `web/index.html` ist der Vite-Mount, keine eigenständige Testseite.

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
