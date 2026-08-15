# Architecture

[Deutsch](../architecture.md) · [English](architecture.md)

Klar is a rule-based NLU. A sentence is tokenized, checked against word lists, and turned into Home Assistant intents. There is no neural net in the engine.

## Pipeline

```
Text
  → fold_latin / tokenize
  → strip fillers (bitte, please, the, …)
  → detect_actions          verb classes from the language packs
  → split_clauses           und / and / then when a new verb appears
  → resolve                 rooms and devices from the home graph
  → fill_intent             HassTurnOn, HassLightSet, …
  → speak                   short confirmation
```

In Home Assistant the spoken line can get a personality cue and, if enabled, an LLM rewrite (`custom_components/klar_nlu/refine.py`). The engine itself stays rule-based.

`parse()` in `src/parse.rs` is the entry point. Before parsing, Klar binds the packs listed in `Settings.languages` (`de`, `en`, …).

## Layers

| Module | Role |
|--------|------|
| `src/lang/` | Per-language word lists, merged in the catalog |
| `src/lexicon.rs` | Verb class → `Action` (On, CoverOpen, SetTemp, …) |
| `src/normalize.rs` | Tokens, accents, fillers |
| `src/numbers.rs` | Number words and digits |
| `src/split.rs` | Clauses, follow-up lights |
| `src/resolve.rs` | Entity and area matches |
| `src/parse.rs` / `parse_help.rs` | Orchestration, clarify, intents |
| `src/session.rs` | Last target, open clarification |
| `src/registry.rs` | HA entity/area registry or default home |
| `src/respond.rs` | Spoken confirmation |
| `src/web.rs` | HTTP |
| `src/wyoming.rs` | Wyoming intent |

## Home graph

On startup Klar reads `core.entity_registry` and `core.area_registry` from `--config-dir` (usually `/config`). If the registry is missing, `default_home()` is used.

Devices are matched by name, aliases, tags, and area. Generic words (`Licht`, `light`) stay at area level when a room has several lights — then Klar asks.

## Session

The same `conversation_id` shares a `Session`:

- last device / last area / last domain
- open clarify list (`Do you mean the ceiling or the lamp?`)
- `ja` / `yes` replays the last switch intent

## Intents

Klar emits the usual Assist intents, including:

`HassTurnOn`, `HassTurnOff`, `HassToggle`, `HassLightSet`, `HassClimateSetTemperature`, `HassGetState`, `HassSetPosition`, `HassFanSetSpeed`, `HassStartTimer`, `HassIncreaseTimer`, `HassShoppingListAddItem`, `HassMediaPause`, `HassMediaNext`, `HassVacuumStart`

Slots: `entity_id`, `area`, `domain`, plus `brightness`, `temperature`, `position`, `percentage`, `color`, `duration` depending on the action.

## Limits

- No general world knowledge. “Tell me a joke” stays empty — in HA the fallback agent takes over.
- No tools in the engine. Devices run only through recognized intents. An optional LLM in HA may rewrite the finished confirmation; it does not get Assist tools for that step.
- Files stay under 500 lines; a new language is a new pack, not a longer `match` list.
