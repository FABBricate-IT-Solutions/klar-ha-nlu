# ADR 0003 — Die Engine besitzt die Assist-Produktlogik; HA bleibt die Plattform

[Deutsch](adr-0003-python-rust-boundary.md) · [English](adr-0003-python-rust-boundary.en.md)

Status: **vorgeschlagen** — Richtung für einen Staging-Zyklus. Umsetzung: [Plan](adr-0003-plan.md). Läuft auf **`staging`**, kein Hauptrelease.

Klar bleibt eine deterministische, lokale NLU. `nlu::parse` hat kein Netz und kein Modell. Ein LLM darf **reden und umformulieren**; es darf den Parse **nicht fahren**. Produktregeln, die nicht Home-Assistant-Plattformkleber sind, gehören in die Rust-Engine, nicht nach `custom_components/klar_nlu/`.

Dieses ADR ersetzt weder [ADR 0002](adr-0002-openai-llm-client.md) (ausgehender OpenAI-kompatibler Client) noch [ADR 0001](adr-0001-rules-and-trainer.md) (sichtbares Match / Sprache / Haus). Es zieht die Besitzlinie zu Ende: Python ist Kleber; die Engine besitzt Sprach-Produktregeln, Assist-Systemprompts und Vorlagen nach dem Execute.

## Kontext

Die Home-Assistant-Integration hat rund 8500 Zeilen. Etwa ein Drittel ist Produktlogik, die konzeptionell nach Rust gehört (Refine-Accept/Prompt, Yarn/Chat/RAG-Prompts, gesprochene Sätze nach dem Execute, Quiet-Ack-Eignung). Der Rest **muss** Python bleiben: Binary-Lebenszyklus, Config-Flow, Registry-Sync, HA-Service-Ausführung, ConversationEntity/ChatLog-Orchestrierung, Restfelder eines alten Agenten, ESPHome/assist_satellite-Chime, Entities/Panel/Services.

Heute ist diese Trennung verletzt:

- Assist-LLM-Pfade wachsen noch einen **parallelen OpenAI-SDK** (`refine.py` `_async_refine_raw`, `stream.py` `iter_completion_tokens`, `conversation.py` `_fallback` / `_stream_fallback`) neben `/api/v2/llm/chat`. ADR 0002 verbietet, das zu vergrößern.
- Refine-**Accept**-Regeln, Wetter-Erfindungs-Wächter und Stimmen-Prompts leben in Python, obwohl HTTP schon über Klar läuft.
- Fallback-/Yarn-/RAG-**Systemprompts** und das Protokoll `KLAR_PARSE:` / `KLAR_ACT:` leben in Python. HA soll nur Assist-Deltas streamen.
- `src/parse/respond.rs` dokumentiert bereits: Assist **überschreibt** die Parse-Sprache mit `speech.py` nach dem HA-Intent, damit der gesprochene Satz zum tatsächlich Gelaufenen passt. Diese Vorlagen brauchen einen **lebenden HA-Zustandsschnappschuss**; der Home-Graph reicht nicht (Klima `current_temperature`, MASS `media_title` / `volume_level`, Kalenderzeilen, lokale Uhr).
- Quiet-Ack-Eignung ist eine Produktregel (`quiet_ack_applies`); `play_chime` ist Plattformkleber.

`scripts/lang_packs/` (rund 12k Zeilen) bleibt Python-Codegen. Generatoren nicht nach Rust umschreiben. Vorlagen nach dem Execute wandern, indem derselbe Generator nach `src/lang/packs/*/speech.rs` emittiert (de/en weiter von Hand, wie heute).

## Entscheidung

### Besitz

| Schicht | Besitzer | Nicht |
|---------|----------|--------|
| `nlu::parse`, Ranking, Policy, Pack-Lexikon | Engine | Kein Modell, kein HA-I/O |
| OpenAI-kompatibles HTTP + SSE (`src/llm/`) | Engine | Kein zweites Python-SDK für neue Pfade |
| Refine-Prompt, `accept_refined`, Wetter-/Zahlen-/Stempel-Wächter, Stimmenblöcke | Engine | Python darf keine zweite Quelle der Wahrheit behalten |
| Yarn / nur-Chat / News / Kalender / RAG-Systemprompts; `KLAR_*`-Protokoll | Engine | HA setzt diese Strings nicht zusammen |
| Sprachvorlagen nach dem Execute (Acks, Queries, Media, Kalender, Uhr, Etage/Raum) | Engine, aus einem **Schnappschuss**, den HA nach dem Execute schickt | Engine darf lebenden HA-Zustand nicht selbst abgreifen |
| Quiet-Ack-**Eignung** | Engine (Flag am Execute-Plan / Outcome) | Chime abspielen |
| ChatLog-Deltas, ConversationEntity, `async_converse`-Legacy | Python | `conversation.py` nicht als Rust-HA-Plugin neu schreiben |
| Engine-Prozess, Config-Flow, Registry-Sync, HA-Services, Expose, ESPHome-Chime | Python | — |
| `contracts.py` `validate_v2_payload` | Python (Client-Schema-Wächter) | Nicht „deduplizieren“ nach Rust |
| `intents.py` `_fold_latin` / `_umlaut_eq` | Python einfrieren; Area-Resolve gegen HA-Registry ist Kleber | Nicht wachsen; kanonisches Fold ist `src/parse/normalize.rs` |
| `llm_endpoint.py` Restfelder eines HA-Agenten | Python | In Engine-Settings kopieren; Chat nicht fahren |
| Lang-Pack-**Generatoren** | Python `scripts/lang_packs/` | Generator nicht portieren |

### Parse bleibt offline

`POST /api/v2/parse` ruft kein LLM, holt keinen HA-Zustand und rendert keine Sprache nach dem Execute. Parse-Sprache bleibt die Wyoming-/Lab-/Vor-Execute-Zeile aus `respond.rs`. Assist ersetzt sie weiter, nachdem Geräte gelaufen sind.

Quiet-Ack-**Eignung** darf ein Boolean aus dem Execute-Plan sein (ein `HassTurnOn`/`HassTurnOff` an Licht/Schalter). Das ist weiter offline. Python bestätigt den Erfolg nach dem Dispatch und spielt dann den Chime.

### LLM: Zweck-Routen, kein zweiter Client

`POST /api/v2/llm/chat` bleibt der Rohtransport (Messages rein, SSE oder JSON raus). Neue Assist-Produktaufrufe schicken **keinen** vorgebauten Systemprompt aus Python.

| Route | Rolle |
|-------|--------|
| `POST /api/v2/llm/chat` | Rohe Messages. Operator/Debug. Unverändert. |
| `POST /api/v2/llm/refine` | Engine baut den Refine-Prompt aus Pack + Persönlichkeit + Extra; fährt das Modell; wendet `accept_refined` an; liefert akzeptierte Sprache oder das Original. |
| `POST /api/v2/llm/assist` | Engine klassifiziert Yarn/Chat/RAG/Kalender/News (oder ehrt ein explizites `kind`); baut den Systemprompt; streamt dieselben SSE-Events wie Chat. Strukturierte `tool`-Events für `klar.parse` / `klar.act` — keine geleakte `KLAR_PARSE:`-Zeile in TTS. |
| `POST /api/v2/policies/trainer/chat` | Unverändert (ADR 0001 / 0002). |

Python streamt Assist-Deltas aus diesen Events (`engine_llm.py` + `stream.py` ChatLog-Kleber). `async_converse` bleibt der **dokumentierte Legacy-Pfad**, wenn ein alter HA-Agent noch konfiguriert ist **und** die Engine 503 liefert (kein Endpoint). `client.chat.completions.create` nicht weiter ausbauen.

`agent_has_home_control` / `can_use_fallback_agent` bleiben Python: sie lesen `ConversationEntityFeature`. Die Engine bekommt `allow_tools: bool`; sie inspiziert keine HA-Agentenobjekte.

### Sprache nach dem Execute: Schnappschussvertrag

Ein naiver Port von `speech.py` ohne lebenden Zustand ist blind: Parse-`respond.rs` sieht keine Klima-Attribute, kein MASS-Now-Playing und keine HA-`matched_states` nach dem Intent.

Neue **Write**-Route, nicht auf dem Parse-Pfad:

`POST /api/v2/speech/render`

- Request: schemaversionierter Schnappschuss (Allow-List der Entity-Felder, optionale Kalenderzeilen, optionale Media-Queue, `now` in der Hauszeitzone, Intent + Execute-Outcome, Sprache, Persönlichkeit).
- Response: `{ "speech": "…", "quiet_ack": false, "source": "post_execute" }`.
- Caps und unbekannte Keys: ablehnen oder droppen; niemals rohe HA-State-Objekte durchreichen.
- Persönlichkeits-**Präfix** einmal, am Assist-Finish (`style` / Refine), nicht im Renderer und nochmal in Python.

`POST /api/v2/home` bleibt der Graph-Sync. Nicht mit Live-Attributen überladen.

Solange der Renderer fehlt, behält Python `speech.py`. Nach Parity ruft Assist Render auf, `speech.py` wird 404-Fallback, danach gelöscht.

## Folgen

### Positiv

- Ein Besitzer für gesprochene Produktregeln und Assist-Prompts; jede Locale geht über Packs + `scripts/lang_packs`, nicht über ein wachsendes Python-Dict.
- ADR 0002 gilt in Assist, nicht nur im Trainer.
- Sprache nach dem Execute kann zum Gelaufenen passen, ohne HA-I/O in `nlu::parse`.

### Negativ

- Versionsversatz: eine alte Engine kennt `/llm/refine` oder `/speech/render` nicht. Staging liefert Engine + Integration zusammen; Python behält einen 404-Fallback für einen Bake, dann fällt das Duplikat.

- Zwei Sprachquellen gibt es **heute** (`respond.rs` vs `speech.py`). Der Renderer darf nicht beide TTS-en. Assist überschreibt Parse-Sprache nach dem Execute bereits; das bleibt.

### Neutral

- `conversation.py` bleibt der HA-Orchestrierer. Es wird dünner (keine Prompt-Strings, keine Accept-Regeln, keine Template-Formatierung).
- Fold-Helfer in `intents.py` einfrieren; nicht in eine gemeinsame Crate „aufräumen“.

## Nicht das

- `scripts/lang_packs/` in Rust neu schreiben
- `conversation.py` als Rust-Home-Assistant-Plugin neu schreiben
- LLM oder Netz in `nlu::parse`
- Neue `PolicyId`-Matcher (ADR 0001)
- `validate_v2_payload` löschen oder Python-`_fold_latin` wachsen lassen
- Diesen Zyklus `staging` → `main` ohne ausdrückliche Freigabe

## Links

- [Umsetzungsplan](adr-0003-plan.md)
- [ADR 0002 — OpenAI-kompatibler LLM-Client](adr-0002-openai-llm-client.md)
- [ADR 0001 — Sichtbare Regeln und Trainer](adr-0001-rules-and-trainer.md)
- `src/parse/respond.rs`, `src/llm/`, `custom_components/klar_nlu/{refine,fallback,rag_tools,speech,quiet,conversation,stream,engine_llm}.py`
