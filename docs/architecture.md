# Architektur

[Deutsch](architecture.md) · [English](en/architecture.md)

Klar ist eine regelbasierte NLU. Ein Satz wird tokenisiert, gegen Wortlisten geprüft und in Home-Assistant-Intents übersetzt. Es gibt kein neuronales Netz in der Engine.

## Ablauf

```
Text
  → fold_latin / tokenize
  → Füllwörter streichen (bitte, please, the, …)
  → detect_actions          Verbklassen aus den Sprachpaketen
  → split_clauses           und / and / dann, wenn ein neues Verb kommt
  → resolve                 Räume und Geräte aus dem Home-Graph
  → fill_intent             HassTurnOn, HassLightSet, …
  → speak                   kurze Bestätigung
```

`parse()` in `src/parse.rs` ist der Einstieg. Vor dem Parse bindet Klar die in `Settings.languages` gewählten Pakete (`de`, `en`, …).

## Schichten

| Modul | Aufgabe |
|-------|---------|
| `src/lang/` | Wortlisten pro Sprache, zusammengeführt im Catalog |
| `src/lexicon.rs` | Verbklasse → `Action` (On, CoverOpen, SetTemp, …) |
| `src/normalize.rs` | Token, Akzente, Füller |
| `src/numbers.rs` | Zahlwörter und Ziffern |
| `src/split.rs` | Klauseln, Follow-up-Leuchten |
| `src/resolve.rs` | Entity- und Area-Treffer |
| `src/parse.rs` / `parse_help.rs` | Orchestrierung, Clarify, Intents |
| `src/session.rs` | letztes Ziel, offene Rückfrage |
| `src/registry.rs` | HA Entity-/Area-Registry oder Default-Wohnung |
| `src/respond.rs` | gesprochene Bestätigung |
| `src/web.rs` | HTTP |
| `src/wyoming.rs` | Wyoming Intent |

## Home-Graph

Beim Start liest Klar `core.entity_registry` und `core.area_registry` aus `--config-dir` (meist `/config`). Fehlt die Registry, gilt `default_home()`.

Geräte werden über Namen, Aliase, Tags und Area getroffen. Generische Wörter (`Licht`, `light`) bleiben auf Area-Ebene, wenn mehrere Leuchten im Raum sind — dann fragt Klar nach.

## Sitzung

Dieselbe `conversation_id` teilt sich eine `Session`:

- letztes Gerät / letzter Raum / letzte Domain
- offene Clarify-Liste (`Meinst du Decke oder Lampe?`)
- `ja` / `yes` wiederholt den letzten Schalt-Intent

## Intents

Klar erzeugt die üblichen Assist-Intents, unter anderem:

`HassTurnOn`, `HassTurnOff`, `HassToggle`, `HassLightSet`, `HassClimateSetTemperature`, `HassGetState`, `HassSetPosition`, `HassFanSetSpeed`, `HassStartTimer`, `HassIncreaseTimer`, `HassShoppingListAddItem`, `HassMediaPause`, `HassMediaNext`, `HassVacuumStart`

Slots: `entity_id`, `area`, `domain`, plus je nach Aktion `brightness`, `temperature`, `position`, `percentage`, `color`, `duration`.

## Grenzen

- Kein freies Weltwissen. „Erzähl einen Witz“ bleibt leer — in HA übernimmt dann der Fallback-Agent.
- Keine Werkzeuge im Motor. Geräte laufen nur über die erkannten Intents.
- Dateien unter 500 Zeilen halten; neue Sprache = neues Paket, nicht eine längere `match`-Liste.
