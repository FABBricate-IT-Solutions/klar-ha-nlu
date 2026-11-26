# API

[Deutsch](api.md) · [English](en/api.md)

Zwei Schnittstellen: HTTP auf Port **10520**, Wyoming auf **10500**.

**Breaking:** `POST /api/parse` entfällt. Clients und die Home-Assistant-Integration müssen `POST /api/v2/parse` und `schema_version: "2.0"` sprechen. Engine und Integration zusammen aktualisieren.

## HTTP

Routenindex aus dem Axum-Router (`python3 scripts/gen_api_routes.py`). Auth ist die Capability aus `src/io/auth.rs`, oder `öffentlich`, wenn die Route außerhalb der Token-Middleware liegt (Health und `/`).

<!-- BEGIN GENERATED ROUTES -->

| Method | Path | Auth | Zweck |
|--------|------|------|---------|
| `GET` | `/` | öffentlich | React-Operator-UI (oder Redirect nach `web/dist`). |
| `POST` | `/api/assignment/apply` | Write | Raum-Zuordnungsvorschläge anwenden. |
| `POST` | `/api/assignment/undo` | Write | Letzten Assignment-Batch rückgängig. |
| `DELETE` | `/api/bundle` | Write | Support-Bundle-Status; DELETE leert ihn. |
| `GET` | `/api/bundle` | Read | Support-Bundle-Status; DELETE leert ihn. |
| `POST` | `/api/bundle/clear` | Write | Support-Bundle-Protokoll leeren (POST-Alias). |
| `GET` | `/api/bundle/dataset` | Read | Als falsch markierte Fälle als nlu_expect-YAML mit redigiertem Home-Ausschnitt laden. |
| `GET` | `/api/bundle/entries` | Read | Aufgezeichnete Turns listen oder löschen. |
| `POST` | `/api/bundle/entries` | Write | Aufgezeichnete Turns listen oder löschen. |
| `GET` | `/api/bundle/protocol` | Read | Redigiertes JSONL-Protokoll laden. |
| `GET` | `/api/custom` | Read | Liste der Custom-Phrasen. |
| `POST` | `/api/custom` | Write | Liste der Custom-Phrasen. |
| `GET` | `/api/dashboard` | Read | Dashboard-Payload für die React-UI. |
| `GET` | `/api/entities` | Read | Home-Graph; POST Tags / Aliase / preferred / area. |
| `POST` | `/api/entities` | PersistParse | Home-Graph; POST Tags / Aliase / preferred / area. |
| `GET` | `/api/gaps` | Read | Kalibrierungs-Rest und Räume. |
| `POST` | `/api/lang/explain` | Read | Preview plus Stages / Evidence / matched custom. |
| `GET` | `/api/lang/overlay` | Read | Sprach-Overlay + Custom-Sätze. |
| `POST` | `/api/lang/overlay` | Write | Sprach-Overlay + Custom-Sätze. |
| `POST` | `/api/lang/preview` | Read | Parse mit ungespeichertem Overlay; installiert nicht. |
| `POST` | `/api/lang/rollback` | Write | Frühere Overlay-History-Zeile wiederherstellen. |
| `GET` | `/api/settings` | Read | Engine-Settings-Store (GET/POST vollständig, PATCH partiell). |
| `PATCH` | `/api/settings` | Write | Engine-Settings-Store (GET/POST vollständig, PATCH partiell). |
| `POST` | `/api/settings` | Write | Engine-Settings-Store (GET/POST vollständig, PATCH partiell). |
| `GET` | `/api/ui` | Read | Persistenter UI-Zustand im Overlay. |
| `POST` | `/api/ui` | Write | Persistenter UI-Zustand im Overlay. |
| `GET` | `/api/v2/conversations` | Read | Gesprächsjournal (letzte 200 Turns / 24 h). |
| `POST` | `/api/v2/conversations/mark-wrong` | Write | Journal-Turn mit gemeintem Intent/Ziel markieren; nlu_expect-Fall exportieren. |
| `GET` | `/api/v2/conversations/{id}` | Read | Turns einer Conversation-ID. |
| `GET` | `/api/v2/health` | öffentlich | Öffentliche Build-Identität, Home-Alter, Overlay, LLM, Wyoming, Ops-Zähler (ohne Token). |
| `POST` | `/api/v2/home` | Write | Live-Home-Graph von der Home-Assistant-Integration. |
| `GET` | `/api/v2/intents` | Read | Liste der bekannten Intent-Namen. |
| `GET` | `/api/v2/languages` | Read | Wählbare Locales (Tier + offered). |
| `GET` | `/api/v2/last-turn` | Read | Neuester Journal-Turn, oder null. |
| `POST` | `/api/v2/llm/assist` | Write | Engine-eigenes Yarn/Chat/RAG/Kalender/News-Assist. |
| `POST` | `/api/v2/llm/chat` | Write | Rohes Chat/SSE gegen das Engine-LLM. |
| `POST` | `/api/v2/llm/custom-voice` | Write | Custom-Voice-Block über das Endpoint-LLM erzeugen. |
| `GET` | `/api/v2/llm/endpoint` | Read | OpenAI-kompatibler Upstream (Key nie im GET). |
| `POST` | `/api/v2/llm/endpoint` | Write | OpenAI-kompatibler Upstream (Key nie im GET). |
| `POST` | `/api/v2/llm/models` | Write | Modelle eines genannten Upstreams listen (Write). |
| `POST` | `/api/v2/llm/refine` | Write | Engine-eigenes Refine fertiger NLU-Sprache. |
| `GET` | `/api/v2/llm/voice` | Read | Persönlichkeits-Vorschau (Flavor + Refine-Prompt). |
| `POST` | `/api/v2/parse` | Read | V2-Parse (`schema_version: "2.0"`). |
| `GET` | `/api/v2/policies` | Read | Policy-Bundle der Rules-Registerkarte. |
| `POST` | `/api/v2/policies` | Write | Policy-Bundle der Rules-Registerkarte. |
| `GET` | `/api/v2/policies/catalog` | Read | Eingebauter Match-Katalog und Safety-Seeds. |
| `POST` | `/api/v2/policies/evaluate` | Read | Dry-Run-Parse gegen optionale Policies. |
| `POST` | `/api/v2/policies/propose/validate` | Read | Trainer-Vorschlag prüfen, ohne anzuwenden. |
| `GET` | `/api/v2/policies/trainer-context` | Read | Trainer-Kontext für das Rules-LLM. |
| `POST` | `/api/v2/policies/trainer/chat` | Write | Trainer-Chat SSE (Write-Token). |
| `POST` | `/api/v2/policies/trainer/consent` | Write | Ausstehende Trainer-Write-Tools freigeben oder ablehnen. |
| `POST` | `/api/v2/session/undo-priors` | Write | Vorherige Helligkeit/Position/Temperatur/Lautstärke auf den letzten Execute stempeln, damit Undo sie wiederherstellen kann. |
| `GET` | `/api/v2/settings/backup` | Write | Allowlistetes Settings-tar.gz laden. |
| `POST` | `/api/v2/settings/restore` | Write | Settings aus Backup-Archiv wiederherstellen. |
| `GET` | `/api/v2/setup/token` | Read | Supervisor/Loopback holt das automatisch erzeugte Schreib-Token (frische App-Installation). |
| `GET` | `/api/v2/speech/asr_boost` | Read | Whisper-Bias-Liste für externes STT. |
| `POST` | `/api/v2/speech/render` | Read | Sprache nach Execute aus HA-Zustandsschnappschuss. |
| `GET` | `/api/v2/wrong-cases` | Read | Vom Operator markierte Regressionsfälle listen. |
| `GET` | `/api/v2/wrong-cases/dataset` | Read | Als falsch markierte Fälle als nlu_expect-YAML laden. |
| `GET` | `/api/v2/wrong-cases/dry-run` | Read | Gespeicherte Haus-Regressionsfälle trocken gegen die laufende Engine laufen lassen. |
| `GET` | `/api/v2/wrong-cases/update-dialog` | Read | Nach Engine-Update: wie viele gespeicherte Fälle sich ändern, und welche Sätze. |
| `POST` | `/api/v2/wrong-cases/update-dialog` | Write | Nach Engine-Update: wie viele gespeicherte Fälle sich ändern, und welche Sätze. |

<!-- END GENERATED ROUTES -->

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

`text` ist auf 4096 Zeichen begrenzt. Der HTTP-Body ist auf 16 KiB begrenzt. `news_topic` steht nur bei einer Nachrichtenansage mit Thema neben `briefing` und fehlt sonst.

`language` ist optional (`de`, `en`, `fr` oder ein BCP-47-Tag wie `en-US`). Ist es gesetzt, bindet Klar nur dieses Paket. `speech` folgt dem gesetzten Paket.

`personality` ist optional und setzt auf diesem Endpunkt eine Formel vor `speech` (`Sehr wohl.`, `Aye.`, …). Die Engine-Settings sind die Quelle (`GET`/`POST /api/settings`); die Operator-UI Settings-Seite ist der Editor. Home Assistant lässt Persönlichkeit am Parse weg, wenn gespeicherte Engine-Settings da sind, und fällt nur auf übrige Integrationsoptionen zurück, wenn der Cache leer ist. Die LLM-Verfeinerung formuliert den Satz danach in der Stimme um und klebt die Formel nicht wieder davor.

`decision.type` ist einer von `execute`, `clarify`, `confirm`, `reject`, `chat` oder `error`. Nur `execute` enthält `plan`, vollständige `candidates` und `selected_candidate_id`; Clients dürfen Intents ausschließlich in diesem Fall ausführen. Bei allen anderen Entscheidungen ist `candidates` leer und es wird kein Plan, Intent oder Slot serialisiert. `confirm` enthält einen Prompt, der Aktion und Ziel nennt („Soll ich wirklich Haustür aufschließen?“), und eine opake Kandidaten-ID; `evidence` und `trace` sagen weiterhin, was verstanden wurde (zum Beispiel `action:off`, `target_entity:lock.front_door`), damit die Entscheidung erklärbar bleibt, sind aber nicht ausführbar. Der vorgeschlagene Plan bleibt ausschließlich in der Session, bis dieselbe `conversation_id` bejaht wird. Eine offene `confirm`- oder `clarify`-Frage gilt 60 Sekunden, wird genau einmal beantwortet und von einem vollständigen neuen Befehl ersetzt. Eine `conversation_id`, die leer ist, länger als 128 Zeichen ist oder Steuerzeichen enthält, benennt nie eine geteilte Session: `/api/parse` behandelt eine leere Id wie eine fehlende und antwortet in den beiden anderen Fällen mit `400`.

Confidence ist evidenzbasiert und zwischen Intents vergleichbar. Feste Bänder: execute bei Confidence ≥ 0.80 und Margin ≥ 0.05, wenn kein konkurrierender vollständiger Plan näher als 0.05 liegt; confirm für riskante Schloss-/Cover-zu-Aktionen, für jeden Schritt, der ein Garagentor, ein Tor oder eine Tür bewegt (nach device_class, sonst nach Namen; auch Öffnen und Umschalten), oder Pläne ab acht Schritten, die nicht nur Lichter betreffen, ab 0.62; clarify bei konkurrierenden vollständigen Plänen unter 0.05 Margin oder Confidence zwischen 0.70 und dem Execute-Band; reject unter 0.70 (unter 0.62 wenn riskant), bei Out-of-Domain (Wetter, Trivia, leere Füllwörter) oder wenn kein geerdeter Schritt übrig bleibt. Fuzzy-, Session- und Inferenz-Evidenz darf exakte Lexikon+Resolver-Evidenz nicht überholen; inferierte Aktionen bekommen nie 1.0. Mehrsatz-Pläne behalten nur unabhängig gültige Schritte. Eine offene Frage beantworten nur die Ja-, Nein- und Abbruchwörter der aktiven Sprache, geprüft an den gesprochenen Wörtern: nach einem Nein- oder Abbruchwort fällt sie weg; nach einem Ja-Wort wird der gespeicherte Plan gegen den aktuellen Home-Graph neu geprüft. Eine Entweder-oder-`clarify`-Frage nimmt kein Ja an, sondern eine ihrer Optionen, Wort für Wort genannt. `chat` bleibt auf News, Briefing-Follow-ups und explizites LLM-Opt-in beschränkt. Allgemeine Fragen ohne Hausbezug lehnen mit Grund `out_of_domain` ab; Home Assistant gibt nur diesen Grund an das konfigurierte LLM weiter und spricht den Ablehnungssatz, wenn kein Modell antwortet. Ein fehlgeschlagener Gerätebefehl bleibt ein normaler Reject.

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

### `GET` / `POST` / `PATCH /api/settings`

`GET` liefert das volle Settings-Objekt. `POST` ersetzt es. `PATCH` merged ein Teilobjekt (unbekannte Keys → `422`).

```json
{
  "personality": "default",
  "mode": "full",
  "languages": ["de", "en"],
  "support_bundle": false,
  "support_bundle_raw_text": false,
  "confirm_risky_actions": true,
  "semantic_adapters": false,
  "web_lookup": false,
  "news_briefing": false,
  "nlu_rag": false
}
```

| Feld | Werte |
|------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (Geräte auflösen) oder `context_only` (nur Räume) |
| `languages` | Paket-Codes. Unbekannte Codes werden ignoriert. Leer heißt: jede kompilierte Locale darf gepinnt werden; ein Request ohne Sprache bindet dann `default_language`. Nicht alle Lexika mergen — Token-Kollisionen zerlegen Assist. Eine experimentelle Locale hinzuzufügen, während `experimental_languages` aus ist, wird mit `422` abgelehnt; eine schon konfigurierte darf bleiben. |
| `default_language` | Das Pack, das ein Request bindet, wenn `languages` leer ist und der Request keine Sprache nennt. Leer heißt Englisch. Ein Code, den kein Pack beantwortet, wird mit `422` abgelehnt. |
| `experimental_languages` | `true` bietet experimentelle Locales zur Neuauswahl an (siehe [Sprachen](languages.md#stufen)). Standardmäßig aus. |
| `support_bundle` | `true` speichert Parse-Verkehr unter `/data/support_bundle.jsonl` (max. 2000 Einträge). Überlebt Neustarts. Erststart auch über `KLAR_SUPPORT_BUNDLE=1`. |
| `support_bundle_raw_text` | `true` erlaubt Rohtext, Sprachausgabe und Tokens im Download und im Journal. Standard aus. Conversation-IDs werden immer gehasht, Entity- und Area-Namen immer pseudonymisiert. |
| `confirm_risky_actions` | `true` verlangt vor riskanten Aktionen wie Sperren/Entsperren und breiten sicherheitsrelevanten Steuerungen eine Bestätigung. |
| `semantic_adapters` | `true` befragt lokale typisierte Adapter nach einem Ranking-Reject. Standard aus. Vorschläge werden revalidiert und überschreiben Execute/Confirm/Clarify/Chat nicht. |
| `nlu_rag` | `true` hängt nur bei `chat` und `reject` einen gematchten Ausschnitt an. Standard aus. Nie Assist-Werkzeuge; der HA-Fallback darf einen Befehl nur über Klar-Werkzeuge nachziehen. `POST /api/v2/parse` kann `nlu_rag` pro Request setzen. |
| `web_lookup` | `true` erlaubt Wikipedia/Wikidata bei Out-of-Domain-Chat. Standard aus. |
| `news_briefing` | `true` erlaubt RSS-Abrufe für Nachrichten. Standard aus. |
| `refine_speech` | `true` formuliert fertige NLU-Sprache mit dem Engine-LLM um. Standard aus. |
| `extra_prompt` | Hausregel als User-Nachricht. Leer = nur Pack-Stimme. Ersetzt die Persönlichkeit nicht. |
| `quiet_ack` | `true` spielt bei einfachem An/Aus einen Chime statt TTS. Standard aus. |
| `satellite_followup` | `true` lässt das Satellitenmikrofon nach einer fertigen Chat-Antwort offen. Standard aus, damit Mikrofon und Lautsprecher am selben Gerät die eigene Sprache nicht wieder hören. Rückfragen und Bestätigungen bleiben offen. Getippter Assist hat keine Satelliten-ID und bleibt unverändert. |
| `calendar_llm` | `true` lässt das Engine-LLM Kalendertermine sprechen. Standard aus. |
| `news_feeds` | Liste aus `{ "pack", "url" }`. Eine `http`- oder `https`-URL pro Pack-Code, ohne Benutzerinfo, höchstens acht Einträge. Leer heißt die eingebauten Feeds — Abruf nur bei `news_briefing`. |
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

Engine baut den Refine-Systemprompt (Pack + Stimme) und schickt Extra als User-Nachricht, ruft das Modell (`temperature` 0.65, `max_tokens` skaliert mit der Original-Länge, min 192, max 4096) und prüft `accept_refined`. JSON `{"type":"done","text":"…","accepted":true}`. Abgelehnt: `text` ist das Original, `accepted` false. Caps: `speech` ≤ 4096, `extra_prompt` ≤ 2048. Write-Token. 503 ohne Endpoint. Python-Prompt nicht mitschicken.

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

### `GET /api/v2/speech/asr_boost`

Gerankte Bias-Liste für externes Whisper (`wyoming-faster-whisper --initial-prompt`). Klar hostet **kein** STT/TTS und kein Whisper/Piper. Query: `language` (Pack-Code, Default aus Settings), `max_tokens` (Default 200, Cap 223). Auth wie andere Read-APIs (Supervisor/Loopback ohne Token, sonst Token).

Nur schwierige Namen: fuzzy-anfällige Entity-Namen/Aliasse (kurz, Umlaute, Rollo/Vorhang-Klasse, Bindestrich/Ziffern) und Custom-Phrase-Anker. Nicht enthalten: übliche Räume, Pack-Verben/Gerätewörter (`Licht`, `an`, …), volle Assist-Sätze, Live-Zustand, Secrets, Infra, nicht freigegebene Entities.

```json
{
  "schema_version": "1",
  "language": "de",
  "max_tokens": 200,
  "prompt": "Kugel Vorhang Rollo Studio-Vorhang Papiertonne …",
  "terms": [{"text": "Vorhang", "tier": 3, "entity_id": "cover.studio_vorhang"}],
  "dropped": 42,
  "updated_at": "a1b2c3…"
}
```

`updated_at` ist ein Inhalts-Fingerprint (nicht Wanduhr). Die Integration schreibt den Prompt nach Homograph-Sync nach `config/klar_nlu_asr_boost.txt` und per Service `klar_nlu.export_asr_boost`. Setup: [Home Assistant — ASR-Boost](home-assistant.md#asr-boost-whisper).

### `POST /api/v2/speech/render`

Post-Execute-Snapshot aus Home Assistant. Die Engine interpoliert Pack-Templates zu einem faktischen Satz (`source: "post_execute"`). Persönlichkeit kommt später beim Assist-Finish. Assist ruft das nach Execute; fehlende Route fällt geschlossen fehl (kein Python-`from_handled`). Gleiche Auth wie Parse: Supervisor-Netz und Loopback ohne Token, sonst Token.

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

Live-Home-Graph von der Home-Assistant-Integration. Der vollständige Request hat eine Byte-Grenze von 8 MiB, getrennt vom 16-KiB-Budget gewöhnlicher JSON-Routen. Die Caps für Sammlungen und Felder gelten weiterhin; nicht autorisierte Requests werden vor dem Lesen des Bodys abgewiesen. `schema_version` muss `"1"` sein. Caps und Fehlercodes: [home-assistant.md](home-assistant.md#registry-sync-ha-ist-quelle). Nach einem gültigen Push ist HA die laufende Quelle; die `.storage`-Überwachung überschreibt diesen Graph nicht mehr.

### `GET /api/v2/languages`

Die Locales, die jetzt gewählt werden können, mit `code`, `native_name`, `script`, `variants`, `tier` (`reference`, `beta` oder `experimental`) und `offered`. Referenz und Beta sind immer gelistet; experimentelle, wenn `experimental_languages` an ist oder wenn sie schon konfiguriert sind. `?all=true` listet jede kompilierte Locale; `offered` ist `false` bei denen, die der Schalter von einer Neuauswahl ausnimmt.

### `GET` / `POST /api/lang/overlay`

Benutzer-Sprach-Overlay plus Custom Sentences, unter `/data/klar_nlu.json`.

```json
{
  "custom": [{ "phrase": "filmabend", "intent": "HassTurnOn", "slots": { "entity_id": "scene.film" } }],
  "language": { "sets": {} },
  "label": "save"
}
```

Antwort enthält `history` (`hash`, `label`, `saved_at`). Ohne `language` im POST bleiben gespeicherte Set-Deltas, ohne `custom` die eigenen Sätze; `"custom": []` löscht sie. Ungültiges Custom/Overlay → `422`. Write-Token außer Loopback.

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

Kalibrieransicht für Entities, die noch schwache Namen, fehlende Areas oder keine hilfreichen Aliase/Tags haben. Die Antwort enthält `leftover` und `rooms` (inkl. `floor_id` je Raum). Das Overlay ist nicht enthalten.

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

Redigiertes JSONL (`klar-support-bundle.jsonl`). Conversation-IDs sind gehasht, Entity-/Area-Namen pseudonymisiert. Ohne `support_bundle_raw_text` sind Text, Speech und Tokens leer.

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
