# Benchmark: Klar NLU vs Home Assistant Assist

[Deutsch](../benchmark-assist.md) · [English](benchmark-assist.md)

Klar NLU exists because the built-in Assist agent (`conversation.home_assistant`, HassIL templates) often misses the same household sentences: several clauses, follow-ups, timers, loose phrasing, mixed aliases. This comparison measures that.

The first published round is **German and English** (19 August 2026). More languages will use the same suites (`full_home`, parity, Assist smokes).

## Result

Same oracles as in [tests](testing.md), same home graph per suite. Assist only recognizes (HassIL) — **no** `conversation.process`, no live devices.

| | Assist | Klar NLU |
|---|---|---|
| 9,922 utterances, DE/EN suites | **31.3%** | **100%** (gates, 0 fail) |
| Home commands, excluding OOD rejects | **24.5%** | 100% |
| Textbook light phrases (`full_home` quick) | **~94%** | 100% |
| Multi-intent, timers, clarify, follow-up | **0%** | 100% |

**On these test sentences Klar hits about 3.2× as often as Assist.** On real home commands (without free OOD credit when nothing matches) **4.1×**. On sentences Assist templates already cover, the gap is small (about 1.06×).

Klar numbers: `cargo nextest run` on `voice_suite` and `full_home` de/en, `KLAR_FULL_HOME=1`, default profile (`test-threads = num-cpus`). Assist: HassIL 3.11.0 and `home-assistant-intents` 2026.7.30, slot lists `name` / `area` / `floor` from each `home_config.yaml` (same shape as HA Core’s default agent).

## Suites

| Suite | Utterances | Assist | Klar | Factor |
|-------|-----------:|-------:|-----:|-------:|
| Apartment DE (`wohnung_mittel`) | 406 | 37.2% | 100% | 2.7× |
| Apartment EN (`wohnung_en`) | 129 | 5.4% | 100% | 18× |
| Family home DE | 2,596 | 39.5% | 100% | 2.5× |
| Family home EN | 4,316 | 21.2% | 100% | 4.7× |
| Full-home DE quick | 63 | 93.7% | 100% | 1.07× |
| Full-home EN quick | 72 | 94.4% | 100% | 1.06× |
| Full-home DE full | 1,123 | 37.7% | 100% | 2.7× |
| Full-home EN full | 1,217 | 37.5% | 100% | 2.7× |

Apartment EN is an outlier: areas are named *Wohnzimmer* / *Küche* while sentences are English (`turn on the living room lights`). Assist matches `{area}` only as a whole list value; alias `living` is not `living room`. Klar composes aliases.

Family suites also include held-out/OOD. Assist scores 100% “reject” there because nothing matches — that lifts the overall rate. Without those 899 utterances: Assist **24.5%**.

Klar gates score against the oracles; the parser is V2 only.

## Where Assist breaks

| Category | Assist | Klar |
|----------|-------:|-----:|
| Music | 85% | 100% |
| Lists | 41% | 100% |
| Lights | 37% | 100% |
| Climate | 6% | 100% |
| Status queries | 3% | 100% |
| Timers | 0% | 100% |
| Multi-intent | 0% | 100% |
| Clarify | 0% | 100% |
| Follow-up (`turn it off`) | 0% | 100% |

Of 6,815 Assist failures, 5,840 are **no template match**. The rest is usually the wrong target or intent (`close the door` as cover instead of lock).

Examples:

- `Turn on the bedroom 3 lights and turn off the entryway lights` — Assist: no match; Klar: two intents.
- `Set living room heat to 23` — Assist: climate, often the wrong target.
- `What's the status of the kitchen` — Assist: often no match.

## Method

1. Each YAML suite under `tests/datasets/` loads the same `home_config.yaml` Klar uses.
2. HassIL gets the same names, aliases, areas, and floors as slot lists, plus domain context — same idea as `default_agent.py` in Home Assistant.
3. `recognize_best(..., best_slot_name="name")` like Core.
4. The last turn of a dialogue is checked against `conditions` or `nlu_expect` (same semantics as `tests/voice_suite_support`).
5. Klar runs through nextest on the same files.

Do **not** run `conversation.process` against a live instance: the agent would execute intents. Local HassIL is Assist NLU without side effects.

Assist is stateless here (no Klar session follow-up). That matches the default agent without an LLM.

## Reproduce it

Rust 1.85+, Python 3.12+, [cargo-nextest](https://nexte.st/). No live Home Assistant required.

### Klar (reference)

```bash
cargo nextest run --test voice_suite --test full_home --no-capture \
  -E 'binary(voice_suite) or test(full_home_quick_de) or test(full_home_quick_en) or test(full_home_full_de) or test(full_home_full_en)'
```

`full` for de/en needs `KLAR_FULL_HOME=1`. Nextest uses every core (`test-threads = num-cpus` in `.config/nextest.toml`). Do not pass `--test-threads=1`.

Lines like `394 Sätze  394 ok  0 fehl  100.0%` are the Klar rate.

### Assist (HassIL)

```bash
python3 -m venv .venv-assist
.venv-assist/bin/pip install 'hassil>=2' home-assistant-intents pyyaml
.venv-assist/bin/python scripts/bench_assist.py
```

JSON is written to `target/assist_bench.json`. Subset of suites:

```bash
.venv-assist/bin/python scripts/bench_assist.py --suite wohnung_mittel --suite full_home_de_quick
.venv-assist/bin/python scripts/bench_assist.py --out /tmp/assist_bench.json
```

Assist’s rate is `overall.accuracy` or `suites.<name>.accuracy`. Compare that to Klar nextest 100% fail=0 on the DE/EN gates.

## More languages

This round is DE/EN because those are the hand-written reference packs and the strictest gates. The same suites already exist for compiled locales (`tests/datasets/full_home/{code}/`, `tests/datasets/parity/`).

Planned:

- `full_home` quick + full for more HassIL languages (fr, nl, es, it, …) once Klar locale gates are `fail == 0`
- Apartment parity against Assist templates in that language
- No silent lexicon merge; every locale stays first-class, see [languages](languages.md)

HassIL intents are not equally deep for every Klar pack. A later run will say per language whether Assist templates exist at all — not only whether Klar can parse the sentence.

## See also

- [Tests](testing.md) — suites, oracles, nextest
- [Languages](languages.md) — packs and locales
- [Architecture](architecture.md) — why Klar stays rule-based
- [Getting started](getting-started.md) — point the Assist pipeline at Klar
