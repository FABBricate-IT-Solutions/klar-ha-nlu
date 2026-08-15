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
