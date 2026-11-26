#!/usr/bin/env python3
"""Write the generated in-domain held-out cases (package 6.10 / Q-08).

The held-out corpus used to be 87 % out-of-domain, so a high score said little about
commands. This generator adds in-domain sentences for the two family homes
(`family_home_en`, `familienhaus_de`): device control, brightness, state queries,
temperature, covers, whole-room lights, two-command sentences, ASR-style typos and
sentences that must be confirmed. Every sentence is written here from Klar's own
templates and the entity names in `home_config.yaml`. Nothing comes from nlu.to.

Gold is derived from the home, never from Klar's output. Run `python3
scripts/eval_heldout/generate.py` after changing a template; CI re-runs it
(`scripts/check_generated.sh`) and fails on a diff.
"""

from __future__ import annotations

import sys
from pathlib import Path

import yaml

sys.path.insert(0, str(Path(__file__).resolve().parent))

from templates import LANGS  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "tests" / "datasets"
HOMES = {"en": "family_home_en", "de": "familienhaus_de"}
CASES_PER_FILE = 50
GEN_PREFIX = "gen_"

ONOFF_DOMAINS = ("light", "switch", "fan")
COVER_ENTITIES = ("cover.living_blinds", "cover.master_blinds", "cover.bedroom2_blinds", "cover.bedroom3_blinds")
CONFIRM_ENTITIES = {"lock.front_door": ("lock", "unlock"), "cover.garage_door": ("open",)}
THERMOSTATS = ("climate.ground_thermostat", "climate.upper_thermostat")
TEMPERATURES = (18, 19, 20, 21, 22, 23, 24)
BRIGHTNESS = (15, 30, 45, 60, 75, 90)
PER_ENTITY = 4


def load_home(code: str) -> dict:
    return yaml.safe_load((DATA / HOMES[code] / "home_config.yaml").read_text(encoding="utf-8"))


def quote(text: str) -> str:
    return "'" + text.replace("'", "''") + "'"


def case_yaml(name: str, split: str, code: str, sentences: list[str], expect: str) -> str:
    lines = [f"- name: {name}", f"  split: {split}", f"  language: {code}", "  sentences:"]
    lines += [f"    - {quote(sentence)}" for sentence in sentences]
    lines.append("  nlu_expect:")
    lines += [f"    {line}" for line in expect.splitlines()]
    return "\n".join(lines) + "\n"


def intents_expect(intents: list[tuple[str, dict]]) -> str:
    out = ["intents:"]
    for name, slots in intents:
        flat = ", ".join(f"{key}: {value}" for key, value in slots.items())
        out += [f"  - intent: {name}", f"    slots: {{{flat}}}"]
    return "\n".join(out)


def pick(pool: list[str], start: int, count: int) -> list[str]:
    return [pool[(start + step) % len(pool)] for step in range(count)]


def typo(word: str, mode: int) -> str:
    """One dropped interior letter, as in the hand-written ASR split. Short words stay intact."""
    if len(word) < 6:
        return word
    at = 2 + mode % (len(word) - 4)
    return word[:at] + word[at + 1 :]


def typo_name(name: str, mode: int) -> str:
    words = name.split(" ")
    long_words = [index for index, word in enumerate(words) if len(word) >= 6]
    if not long_words:
        return name
    index = long_words[mode % len(long_words)]
    words[index] = typo(words[index], mode)
    return " ".join(words)


def build(code: str) -> dict[str, list[str]]:
    lang = LANGS[code]
    home = load_home(code)
    devices = home["devices"]
    areas = {area["id"]: area["name"] for area in home["areas"]}
    by_id = {device["id"]: device for device in devices}
    named = [device for device in devices if device["id"].split(".")[0] in ONOFF_DOMAINS]
    cases: dict[str, list[str]] = {"control": [], "typo": [], "multi": [], "clarify": []}

    for index, device in enumerate(named):
        domain, slug = device["id"].split(".")
        pools = lang["onoff"][domain]
        for verb, intent in (("on", "HassTurnOn"), ("off", "HassTurnOff")):
            sentences = [t.format(n=device["name"]) for t in pick(pools[verb], index, PER_ENTITY)]
            expect = intents_expect([(intent, {"entity_id": device["id"]})])
            cases["control"].append(case_yaml(f"gen_{verb}_{slug}", "control", code, sentences, expect))

    for index, device in enumerate(d for d in devices if d["id"].startswith("light.")):
        level = BRIGHTNESS[index % len(BRIGHTNESS)]
        sentences = [t.format(n=device["name"], p=level) for t in pick(lang["brightness"], index, 2)]
        expect = intents_expect([("HassLightSet", {"entity_id": device["id"], "brightness": level})])
        cases["control"].append(case_yaml(f"gen_dim_{device['id'].split('.')[1]}", "control", code, sentences, expect))

    area_heads = {name.split(" ")[0].casefold() for name in areas.values()}
    queryable = [
        d
        for d in devices
        if d["id"].split(".")[0] in ("light", "switch", "lock")
        and not (d["id"].startswith("light.") and d["name"].split(" ")[0].casefold() in area_heads and d["name"].split(" ")[-1].casefold() in ("light", "licht"))
    ]
    for index, device in enumerate(queryable):
        state_pool = lang["query_lock"] if device["id"].startswith("lock.") else lang["query"]
        sentences = [t.format(n=device["name"]) for t in pick(state_pool, index, 2)]
        expect = intents_expect([("HassGetState", {"entity_id": device["id"]})])
        cases["control"].append(case_yaml(f"gen_state_{device['id'].split('.')[1]}", "control", code, sentences, expect))

    for index, entity in enumerate(THERMOSTATS):
        for step, degrees in enumerate(TEMPERATURES):
            sentences = [t.format(n=by_id[entity]["name"], d=degrees) for t in pick(lang["climate"], index + step, 2)]
            expect = intents_expect([("HassClimateSetTemperature", {"entity_id": entity, "temperature": degrees})])
            cases["control"].append(case_yaml(f"gen_climate_{entity.split('.')[1]}_{degrees}", "control", code, sentences, expect))

    for index, entity in enumerate(COVER_ENTITIES):
        name = by_id[entity]["name"]
        sentences = [t.format(n=name) for t in pick(lang["cover_open"], index, 3)]
        expect = intents_expect([("HassTurnOn", {"entity_id": entity})])
        cases["control"].append(case_yaml(f"gen_open_{entity.split('.')[1]}", "control", code, sentences, expect))

    lights_per_area: dict[str, int] = {}
    for device in devices:
        if device["id"].startswith("light."):
            lights_per_area[device["area_id"]] = lights_per_area.get(device["area_id"], 0) + 1
    rooms = sorted(area for area, count in lights_per_area.items() if count > 1)
    for index, area in enumerate(rooms):
        for verb, intent in (("on", "HassTurnOn"), ("off", "HassTurnOff")):
            sentences = [t.format(a=areas[area]) for t in pick(lang["room"][verb], index, 3)]
            expect = intents_expect([(intent, {"area": area, "domain": "light"})])
            cases["control"].append(case_yaml(f"gen_room_{verb}_{area}", "control", code, sentences, expect))

    for index, device in enumerate(named):
        domain = device["id"].split(".")[0]
        mate = named[(index * 7 + 3) % len(named)]
        if mate["id"] == device["id"]:
            mate = named[(index + 1) % len(named)]
        first_verb, second_verb = ("on", "off") if index % 2 == 0 else ("off", "on")
        template = lang["multi"][index % len(lang["multi"])]
        first = lang["onoff"][domain][first_verb][0].format(n=device["name"])
        second = lang["multi_second"][second_verb].format(n=mate["name"])
        sentence = template.format(a=first, b=second)
        intents = [
            ("HassTurnOn" if first_verb == "on" else "HassTurnOff", {"entity_id": device["id"]}),
            ("HassTurnOn" if second_verb == "on" else "HassTurnOff", {"entity_id": mate["id"]}),
        ]
        cases["multi"].append(case_yaml(f"gen_multi_{index}", "multi", code, [sentence], intents_expect(intents)))

    for index, device in enumerate(named):
        domain = device["id"].split(".")[0]
        for verb, intent in (("on", "HassTurnOn"), ("off", "HassTurnOff")):
            sentences = []
            for variant in range(2):
                mode = index * 5 + variant * 3 + (0 if verb == "on" else 1)
                broken = typo_name(device["name"], mode)
                if broken == device["name"]:
                    continue
                sentences.append(lang["typo"][verb][(index + variant) % len(lang["typo"][verb])].format(n=broken))
            if sentences:
                expect = intents_expect([(intent, {"entity_id": device["id"]})])
                cases["typo"].append(case_yaml(f"gen_typo_{verb}_{device['id'].split('.')[1]}", "typo", code, sentences, expect))

    for entity, verbs in CONFIRM_ENTITIES.items():
        for verb in verbs:
            sentences = [t.format(n=by_id[entity]["name"]) for t in lang["confirm"][verb]]
            cases["clarify"].append(case_yaml(f"gen_confirm_{verb}_{entity.split('.')[1]}", "clarify", code, sentences, "clarify: true\nintents: []"))
    return cases


def write(code: str) -> int:
    out = DATA / HOMES[code] / "m7_heldout"
    for stale in out.glob(f"{GEN_PREFIX}*.yaml"):
        stale.unlink()
    total = 0
    for split, blocks in build(code).items():
        for number, start in enumerate(range(0, len(blocks), CASES_PER_FILE), start=1):
            chunk = blocks[start : start + CASES_PER_FILE]
            (out / f"{GEN_PREFIX}{split}_{number}.yaml").write_text("\n".join(chunk), encoding="utf-8")
        total += sum(block.count("\n    - ") for block in blocks)
    return total


def main() -> int:
    for code in HOMES:
        print(f"{code}: {write(code)} generated sentences")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
