# API

[Deutsch](api.md) · [English](en/api.md)

Zwei Schnittstellen: HTTP auf Port **10520**, Wyoming auf **10500**.

## HTTP

### `POST /api/parse`

```json
{ "text": "Licht im Wohnzimmer an", "conversation_id": "optional-id", "language": "de", "personality": "butler" }
```

Antwort:

```json
{
  "text": "Licht im Wohnzimmer an",
  "intents": [
    {
      "name": "HassTurnOn",
      "slots": [
        { "name": "area", "value": "wohnzimmer" },
        { "name": "domain", "value": "light" }
      ]
    }
  ],
  "speech": "Sehr wohl. Wohnzimmerlicht ist an.",
  "clarify": false,
  "conversation_id": "optional-id",
  "chat": false,
  "briefing": false,
  "personality": "butler"
}
```

`text` ist auf 4096 Zeichen begrenzt. Der HTTP-Body ist auf 16 KiB begrenzt.

`language` ist optional (`de`, `en` oder ein BCP-47-Tag wie `en-US`). Ist es gesetzt, bindet Klar nur dieses Paket — Assist kann so zwischen Deutsch und Englisch umschalten. `speech` folgt dem gesetzten Paket.

`personality` ist optional und setzt auf diesem Endpunkt eine Formel vor `speech` (`Sehr wohl.`, `Aye.`, …). Home Assistant speichert die Auswahl in der Integration und schickt sie bei jedem Parse; die Engine-Settings sind nur für die Klar-UI. Die LLM-Verfeinerung in der HA-Integration formuliert den Satz danach in der Stimme um und klebt die Formel nicht wieder davor.

`clarify: true` bedeutet: keine Intents ausführen, die Frage in `speech` vorlesen, dieselbe `conversation_id` für die Antwort behalten.

Leere `intents` = kein Hausbefehl. Die HA-Integration leitet das an den Fallback-Agenten weiter, falls einer gesetzt ist.

`chat: true` bedeutet: Klar hat bewusst keinen Haus-Intent ausgegeben und der Satz darf an den Fallback-Agenten. `briefing: true` markiert News-/Briefing-Dialoge, damit Follow-ups nicht versehentlich Geräte steuern.

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
  "support_bundle": false
}
```

| Feld | Werte |
|------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (Geräte auflösen) oder `context_only` (nur Räume) |
| `languages` | Paket-Codes. Unbekannte Codes werden ignoriert. Leer fällt auf `de`+`en` zurück. |
| `support_bundle` | `true` speichert Parse-Verkehr unter `/data/support_bundle.jsonl` (max. 2000 Einträge). Überlebt Neustarts. Erststart auch über `KLAR_SUPPORT_BUNDLE=1`. |

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

Rohprotokoll als JSONL (`klar-support-bundle.jsonl`). Jede Zeile: Zeitstempel, Quelle (`http`/`wyoming`), Sprache, Anfrage, Intents, Sprachausgabe.

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
