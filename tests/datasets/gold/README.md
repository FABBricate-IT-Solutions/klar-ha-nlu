# Gold natural sentences (every shipped locale)

One `{code}.yaml` per compiled locale, driven by `tests/language_gold.rs`.
Sentences are hand-written natural speech in real orthography. They are the
non-tautological gate for Q-02: never generate them from pack lexicons or
`scripts/parity/`. Seeded from the plan evidence probe
(`klar-nlu-plan-evidence/lang/natural.py`, 29 locales) plus hand-authored
sentences for the other 35. Not speaker-verified end to end; Cornish (`kw`) is
the lowest-confidence set.

## Schema (`deny_unknown_fields`)

- `rooms`: `living`, `kitchen`, `bedroom`, `hall` in the language's own names.
- `devices`: `light_living`, `light_kitchen`, `light_bedroom`, `cover_bedroom`,
  `climate_living`, `lock_front`, `shopping_list`.
- `cases`: `text`, `expect`, optional `kind`, optional `soft`.

`expect` is `Head[@area][:slot=value]`, joined with `+` for multi-command
sentences, or `CONFIRM` / `CHAT`. Heads: `TurnOn`, `TurnOff`, `LightSet`,
`ClimateSetTemperature`, `GetState`, `GetTemp`, `StartTimer`, `ListAdd`,
`MediaPause`. Each locale's home also has one music-tagged speaker so
`MediaPause` resolves.

## Gates

- Structure: at least 10 cases, unique texts, rooms and devices present, and
  coverage of power on, power off, `ListAdd` and `CHAT`.
- Family: a state-changing result in the wrong action family fails, and so does
  "on" coming back as "off" or the reverse. Reads, rejects and clarifications are not
  wrong families.
- Exact: every case without `soft` must match intent, area and slots.
- Core: at least 5 hard (non-`soft`) cases per locale.
- Ratchet: a `soft` case that now passes fails the test; drop the marker.

`soft: "known gap: ..."` records a sentence the pack cannot resolve yet. It
stays a probe, never a pass. Most gaps are rejects or clarifications
(timers, time and CHAT questions, percent words, definite-form lights in
`nb`/`da`/`sv`, on/off in `ar`/`he`/`is`/`mn`).

## Inheritance

`de-AT` and `de-CH` use `de.yaml`; `en-GB` uses `en.yaml`. The mapping lives in
`INHERITS` in the test, which also asserts these locales have no file of their
own and that `LangId::parent()` agrees. `pt-BR`, `zh-TW` and `zh-HK` have
their own files.

## Known soft gaps that still execute something

- `ca`, `gl`: "un temporitzador de cinc minuts" starts a 1-minute timer.
- `kw`: a half-brightness and a 21 degree sentence get the wrong value.
- `et`, `lt`: a two-command sentence executes only the first command.
- `lb`: "Maach d'Musek paus" splits into on plus pause, so the sentence uses
  "Pauséier d'Musek".
