# API

[Deutsch](api.md) · [English](en/api.md)

Zwei Schnittstellen: HTTP auf Port **10520**, Wyoming auf **10500**.

## HTTP

### `POST /api/parse`

```json
{ "text": "Licht im Wohnzimmer an", "conversation_id": "optional-id", "language": "de" }
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
  "speech": "Schalte wohnzimmer ein.",
  "clarify": false,
  "conversation_id": "…"
}
```

`language` ist optional (`de`, `en` oder ein BCP-47-Tag wie `en-US`). Ist es gesetzt, bindet Klar nur dieses Paket — Assist kann so zwischen Deutsch und Englisch umschalten. `speech` folgt dem gesetzten Paket.

`clarify: true` bedeutet: keine Intents ausführen, die Frage in `speech` vorlesen, dieselbe `conversation_id` für die Antwort behalten.

Leere `intents` = kein Hausbefehl. Die HA-Integration leitet das an den Fallback-Agenten weiter, falls einer gesetzt ist.

### `GET` / `POST /api/settings`

```json
{
  "personality": "default",
  "mode": "full",
  "languages": ["de", "en"]
}
```

| Feld | Werte |
|------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (Geräte auflösen) oder `context_only` (nur Räume) |
| `languages` | Paket-Codes. Unbekannte Codes werden ignoriert. Leer fällt auf `de`+`en` zurück. |

### `GET` / `POST /api/custom`

Eigene Sätze, exakter oder unscharfer Treffer:

```json
[{ "phrase": "filmabend", "intent": "HassTurnOn", "slots": { "entity_id": "scene.film" } }]
```

### `GET /api/entities`

Geladener Home-Graph (Name, Area, Aliase, Tags).

### `POST /api/entities`

```json
{ "entity_id": "light.wohn_decke", "tags": ["decke"] }
```

Tags helfen der Auflösung, wenn der Anzeigename zu generisch ist.

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

Die HA-Custom-Component nutzt HTTP, nicht Wyoming. Wyoming ist für Add-ons und Satelliten gedacht, die einen Intent-Server erwarten.
