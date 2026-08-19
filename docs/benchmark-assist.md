# Benchmark: Klar NLU gegen Home Assistant Assist

[Deutsch](benchmark-assist.md) · [English](en/benchmark-assist.md)

Klar NLU existiert, weil der eingebaute Assist-Agent (`conversation.home_assistant`, HassIL-Templates) auf denselben Haus-Sätzen oft nicht reicht: mehrere Klauseln, Nachfragen, Timer, freie Formulierungen, gemischte Aliase. Dieser Vergleich misst das.

Die erste veröffentlichte Runde ist **Deutsch und Englisch** (19. August 2026). Weitere Sprachen folgen auf denselben Suiten (`full_home`, Parity, Assist-Smokes).

## Ergebnis

Gleiche Oracles wie in [tests](testing.md), gleicher Home-Graph je Suite. Assist erkennt nur (HassIL) — **kein** `conversation.process`, keine echten Geräte.

| | Assist | Klar NLU |
|---|---|---|
| 9.922 Sätze, DE/EN-Suiten | **31,3 %** | **100 %** (Gates, 0 Fail) |
| Hausbefehle ohne OOD-Ablehnung | **24,5 %** | 100 % |
| Lehrbuch-Lichtphrasen (`full_home` quick) | **~94 %** | 100 % |
| Multi-Intent, Timer, Klärung, Follow-up | **0 %** | 100 % |

**Klar trifft auf diesen Testsätzen etwa 3,2× so oft wie Assist.** Bei echten Hausbefehlen (ohne geschenkte OOD-Treffer, weil nichts matched) **4,1×**. Bei Sätzen, die Assist-Templates ohnehin vorsehen, ist der Abstand klein (etwa 1,06×).

Klar-Zahlen: `cargo nextest run` auf `voice_suite` und `full_home` de/en, `KLAR_FULL_HOME=1`, Profil `default` (`test-threads = num-cpus`). Assist: HassIL 3.11.0 und `home-assistant-intents` 2026.7.30, Slot-Listen `name` / `area` / `floor` aus jeder `home_config.yaml` (wie der Default-Agent in HA Core).

## Suiten

| Suite | Sätze | Assist | Klar | Faktor |
|-------|------:|-------:|-----:|-------:|
| Wohnung DE (`wohnung_mittel`) | 406 | 37,2 % | 100 % | 2,7× |
| Wohnung EN (`wohnung_en`) | 129 | 5,4 % | 100 % | 18× |
| Familienhaus DE | 2.596 | 39,5 % | 100 % | 2,5× |
| Family home EN | 4.316 | 21,2 % | 100 % | 4,7× |
| Full-home DE quick | 63 | 93,7 % | 100 % | 1,07× |
| Full-home EN quick | 72 | 94,4 % | 100 % | 1,06× |
| Full-home DE full | 1.123 | 37,7 % | 100 % | 2,7× |
| Full-home EN full | 1.217 | 37,5 % | 100 % | 2,7× |

Wohnung EN ist ein Ausreißer nach unten: Areas heißen *Wohnzimmer* / *Küche*, die Sätze sind englisch (`turn on the living room lights`). Assist matched `{area}` nur als ganzen Listeneintrag; Alias `living` reicht nicht für `living room`. Klar setzt Aliase zusammen.

Die Familien-Suiten enthalten auch Held-out/OOD. Assist bekommt dort 100 % „Reject“, weil nichts matched — das hebt die Gesamtquote. Ohne diese 899 Sätze: Assist **24,5 %**.

Klar-Gates zählen gegen die Oracles; der Parser ist nur noch V2.

## Wo Assist bricht

| Kategorie | Assist | Klar |
|-----------|-------:|-----:|
| Musik | 85 % | 100 % |
| Listen | 41 % | 100 % |
| Lichter | 37 % | 100 % |
| Klima | 6 % | 100 % |
| Statusfragen | 3 % | 100 % |
| Timer | 0 % | 100 % |
| Mehrere Intents | 0 % | 100 % |
| Klärung | 0 % | 100 % |
| Follow-up (`mach es aus`) | 0 % | 100 % |

Von 6.815 Assist-Fehlern sind 5.840 **kein Template-Match**. Der Rest ist meist das falsche Ziel oder der falsche Intent (`Tür zu` als Cover statt Lock).

Beispiele:

- `Mach die Lichter in Kinderzimmer 3 an und Mach die Lichter am Eingang aus` — Assist: kein Match; Klar: zwei Intents.
- `Heizung Wohnzimmer auf 23 Grad` — Assist: Klima, oft falsches Ziel.
- `Wie ist der Status der Küche` — Assist: oft kein Match.

## Methode

1. Jede YAML-Suite unter `tests/datasets/` lädt dieselbe `home_config.yaml` wie Klar.
2. HassIL bekommt dieselben Namen, Aliase, Areas und Floors als Slot-Listen, plus Domain-Context — analog zu `default_agent.py` in Home Assistant.
3. `recognize_best(..., best_slot_name="name")` wie Core.
4. Der letzte Turn eines Dialogs wird gegen `conditions` oder `nlu_expect` geprüft (gleiche Semantik wie `tests/voice_suite_support`).
5. Klar läuft über nextest gegen dieselben Dateien.

**Nicht** `conversation.process` gegen eine Live-Instanz: der Agent würde Intents ausführen. HassIL lokal ist die Assist-NLU ohne Nebenwirkungen.

Assist ist zustandslos (kein Klar-Session-Follow-up). Das ist Absicht: so arbeitet der Default-Agent ohne LLM.

## Selbst nachfahren

Rust 1.85+, Python 3.12+, [cargo-nextest](https://nexte.st/). Kein Live-Home-Assistant nötig.

### Klar (Referenz)

```bash
cargo nextest run --test voice_suite --test full_home --no-capture \
  -E 'binary(voice_suite) or test(full_home_quick_de) or test(full_home_quick_en) or test(full_home_full_de) or test(full_home_full_en)'
```

`full` für de/en braucht `KLAR_FULL_HOME=1`. Nextest nutzt alle Kerne (`test-threads = num-cpus` in `.config/nextest.toml`). Nicht `--test-threads=1` setzen.

Ausgabezeilen wie `394 Sätze  394 ok  0 fehl  100.0%` sind die Klar-Quote.

### Assist (HassIL)

```bash
python3 -m venv .venv-assist
.venv-assist/bin/pip install 'hassil>=2' home-assistant-intents pyyaml
.venv-assist/bin/python scripts/bench_assist.py
```

JSON landet in `target/assist_bench.json`. Einzelne Suiten:

```bash
.venv-assist/bin/python scripts/bench_assist.py --suite wohnung_mittel --suite full_home_de_quick
.venv-assist/bin/python scripts/bench_assist.py --out /tmp/assist_bench.json
```

Die Assist-Quote ist `overall.accuracy` bzw. `suites.<name>.accuracy`. Vergleich: Klar-nextest 100 % Fail=0 auf den DE/EN-Gates gegen diese Assist-Zahl.

## Weitere Sprachen

Diese Runde ist DE/EN, weil das die handgeschriebenen Referenzpacks und die härtesten Gates sind. Dieselben Suiten existieren bereits für die kompilierten Locales (`tests/datasets/full_home/{code}/`, `tests/datasets/parity/`).

Geplant:

- `full_home` quick + full für weitere HassIL-Sprachen (fr, nl, es, it, …), sobald die Locale-Gates bei Klar `fail == 0` sind
- Parity-Wohnung gegen Assist-Templates derselben Sprache
- Kein stilles Mischen von Lexika; jede Locale bleibt erstklassig, siehe [Sprachen](languages.md)

HassIL-Intents gibt es nicht für jedes Klar-Pack gleich tief. Ein späterer Lauf sagt dann pro Sprache, ob Assist-Templates überhaupt existieren — nicht nur, ob Klar den Satz kann.

## Siehe auch

- [Tests](testing.md) — Suiten, Oracles, nextest
- [Sprachen](languages.md) — Packs und Locales
- [Architektur](architecture.md) — warum Klar regelbasiert bleibt
- [Einstieg](getting-started.md) — Assist-Pipeline auf Klar stellen
