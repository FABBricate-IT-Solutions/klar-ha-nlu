#!/usr/bin/env python3
"""Score Home Assistant's default agent (HassIL + official intents + gazetteer) on Klar suites.

The agent is scored as Core runs it: hassil first, and for English the gazetteer matcher for
what hassil cannot match (`scripts/assist_gazetteer.py`). Both figures are written, hassil
alone and the whole agent, per suite and per language, with the versions and the commit the
run belongs to. Does not call conversation.process: recognition only, no device control.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from importlib import metadata
from collections import Counter, defaultdict
from pathlib import Path

import yaml
from hassil import recognize_best
from hassil.intents import Intents, TextSlotList
from home_assistant_intents import get_intents

sys.path.insert(0, str(Path(__file__).resolve().parent))

from assist_gazetteer import LANGUAGES as GAZETTEER_LANGUAGES  # noqa: E402
from assist_gazetteer import GazetteerAgent  # noqa: E402

REPO = Path(__file__).resolve().parents[1]
DATA = REPO / "tests" / "datasets"
SKIP = {"type", "entity_id", "area", "domain", "state", "attributes", "minutes", "hours", "seconds", "item", "name"}
ATTR_KEYS = (
    "temperature",
    "brightness",
    "percentage",
    "color",
    "position",
    "search_query",
    "media_id",
    "media_type",
    "media_class",
    "artist",
    "enqueue",
    "radio_mode",
    "volume_step",
    "volume_level",
    "is_volume_muted",
)
SUITES = {
    "wohnung_mittel": ("de", DATA / "wohnung_mittel", None),
    "wohnung_en": ("en", DATA / "wohnung_en", None),
    "familienhaus_de": ("de", DATA / "familienhaus_de", None),
    "family_home_en": ("en", DATA / "family_home_en", None),
    "full_home_de_quick": ("de", DATA / "full_home/de/quick", DATA / "full_home/de/home_config.yaml"),
    "full_home_en_quick": ("en", DATA / "full_home/en/quick", DATA / "full_home/en/home_config.yaml"),
    "full_home_de_full": ("de", DATA / "full_home/de/full", DATA / "full_home/de/home_config.yaml"),
    "full_home_en_full": ("en", DATA / "full_home/en/full", DATA / "full_home/en/home_config.yaml"),
    # The held-out corpus the scorecard gates, on the family homes it is written for.
    "heldout_en": ("en", DATA / "family_home_en" / "m7_heldout", DATA / "family_home_en" / "home_config.yaml"),
    "heldout_de": ("de", DATA / "familienhaus_de" / "m7_heldout", DATA / "familienhaus_de" / "home_config.yaml"),
}
DEFAULT_SUITES = [name for name in SUITES if not name.startswith("heldout_")]
PACKAGES = ("hassil", "home-assistant-intents", "gazetteer-matcher")
MODES = ("assist", "hassil")


def load_yaml(path: Path):
    return yaml.safe_load(path.read_text(encoding="utf-8")) or []


def domain_of(entity_id: str | None) -> str | None:
    return entity_id.split(".", 1)[0] if entity_id and "." in entity_id else None


def unique_tuples(rows):
    seen, out = set(), []
    for row in rows:
        key = (str(row[0]).casefold(), str(row[1]), str(row[2] if len(row) > 2 else {}))
        if key not in seen:
            seen.add(key)
            out.append(row)
    return out


def add_names(rows, entities, entity_id, names, context, area=None):
    domain = context["domain"]
    entities[entity_id] = {"area": area, "domain": domain, "name": names[0]}
    for name in names:
        if name:
            rows.append((str(name), entity_id, context))


def build_slots(home: dict):
    area_rows, floor_rows, name_rows, entities = [], [], [], {}
    for area in home.get("areas") or []:
        aid = area["id"]
        for name in [area.get("name") or aid, aid, *(area.get("aliases") or [])]:
            if name:
                area_rows.append((str(name), aid))
    for floor in home.get("floors") or []:
        fid = floor.get("id") or floor.get("name")
        for name in [floor.get("name") or fid, fid, *(floor.get("aliases") or [])]:
            if name:
                floor_rows.append((str(name), str(fid)))
    for device in home.get("devices") or []:
        eid = device["id"]
        domain = domain_of(eid) or "unknown"
        context = {"domain": domain}
        if device.get("device_class"):
            context["device_class"] = device["device_class"]
        add_names(name_rows, entities, eid, [device.get("name") or eid, *(device.get("aliases") or [])], context, device.get("area_id"))
    for kind, items in (("scene", home.get("scenes") or []), ("script", home.get("scripts") or [])):
        for item in items:
            if isinstance(item, dict):
                sid = item.get("id") or f"{kind}.{item.get('name')}"
                names = [item.get("name") or sid, sid]
            else:
                sid, names = str(item), [str(item)]
            if not sid.startswith(f"{kind}."):
                sid = f"{kind}.{sid}"
            add_names(name_rows, entities, sid, names, {"domain": kind})
    return {
        "area": TextSlotList.from_tuples(unique_tuples(area_rows), allow_template=False),
        "name": TextSlotList.from_tuples(unique_tuples(name_rows), allow_template=False),
        "floor": TextSlotList.from_tuples(unique_tuples(floor_rows), allow_template=False),
    }, entities


def cond_attrs(cond: dict) -> dict:
    attrs = dict(cond.get("attributes") or {})
    attrs.update({k: v for k, v in cond.items() if k not in SKIP})
    return attrs


def expected_intent_names(cond: dict) -> list[str]:
    attrs = cond_attrs(cond)
    if "temperature" in attrs:
        return ["HassClimateSetTemperature"]
    if "brightness" in attrs or "color" in attrs:
        return ["HassLightSet"]
    if "percentage" in attrs:
        return ["HassFanSetSpeed"]
    if "position" in attrs:
        return ["HassSetPosition"]
    if "search_query" in attrs or "media_id" in attrs:
        return ["HassMediaSearchAndPlay", "MassPlayMedia"]
    if "volume_level" in attrs or "volume_step" in attrs:
        return ["HassSetVolume", "HassSetVolumeRelative"]
    if "is_volume_muted" in attrs:
        return ["HassMediaPlayerMute", "HassMediaPlayerUnmute", "HassMediaUnpause", "HassTurnOn"]
    kind = cond.get("type") or "action"
    if kind == "query":
        return ["HassGetState", "HassClimateGetTemperature"]
    if kind in {"shopping_list", "todo_list"} or cond.get("item"):
        return ["HassListAddItem", "HassListCompleteItem", "HassShoppingListAddItem", "HassShoppingListCompleteItem"]
    entity = cond.get("entity_id") or ""
    if cond.get("minutes") is not None or cond.get("hours") is not None or cond.get("seconds") is not None or entity.startswith("timer."):
        return ["HassStartTimer", "HassIncreaseTimer", "HassDecreaseTimer", "HassTimerStatus", "HassPauseTimer", "HassCancelTimer"]
    if entity.startswith("vacuum."):
        return ["HassVacuumReturnToBase", "HassTurnOff"] if cond.get("state") == "off" else ["HassVacuumStart", "HassVacuumReturnToBase", "HassTurnOn"]
    if entity.startswith("scene.") or entity.startswith("script."):
        return ["HassTurnOn"]
    return {
        "paused": ["HassMediaPause"],
        "playing": ["HassMediaUnpause", "HassTurnOn"],
        "next": ["HassMediaNext"],
        "previous": ["HassMediaPrevious"],
        "off": ["HassTurnOff"],
        "closed": ["HassTurnOff"],
        "unlocked": ["HassTurnOff"],
        "open": ["HassTurnOn"],
        "locked": ["HassTurnOn"],
    }.get(cond.get("state"), ["HassTurnOn"])


def slot_attrs_ok(slots: dict, cond: dict) -> bool:
    for key in ATTR_KEYS:
        if key not in cond_attrs(cond):
            continue
        got = slots.get(key)
        return str(got) == str(cond_attrs(cond)[key]) if got is not None else False
    return True


def target_ok(slots: dict, cond: dict, entities: dict) -> bool:
    wanted = cond.get("entity_id")
    if wanted:
        if slots.get("entity_id") == wanted:
            return slot_attrs_ok(slots, cond)
        entity = entities.get(wanted)
        return bool(entity and slots.get("area") == entity["area"] and slots.get("domain") in (None, entity["domain"])) and slot_attrs_ok(slots, cond)
    area = cond.get("area")
    if not area:
        return slot_attrs_ok(slots, cond)
    domain = cond.get("domain")
    if slots.get("area") == area:
        if domain and slots.get("domain") not in (None, domain):
            ent = entities.get(slots.get("entity_id") or "")
            return bool(ent and ent["area"] == area and ent["domain"] == domain) and slot_attrs_ok(slots, cond)
        return slot_attrs_ok(slots, cond)
    ent = entities.get(slots.get("entity_id") or "")
    return bool(ent and ent["area"] == area and (domain is None or ent["domain"] == domain)) and slot_attrs_ok(slots, cond)


def exact_ok(expected: dict, intents: list[dict]) -> tuple[bool, str]:
    if expected.get("reject"):
        return (not intents), "reject" if not intents else "wrong_intent"
    if expected.get("clarify"):
        return False, "no_clarify"
    wanted = expected.get("intents") or []
    if len(wanted) != len(intents):
        return False, "missing_multi" if len(wanted) > 1 else "wrong_intent"
    for gold, got in zip(wanted, intents):
        if gold.get("intent") != got["name"]:
            return False, "wrong_intent"
        gold_slots = {k: str(v) for k, v in (gold.get("slots") or {}).items()}
        got_slots = {k: str(v) for k, v in got["slots"].items() if k in gold_slots}
        if gold_slots != got_slots:
            return False, "wrong_target"
    return True, "ok"


def result_slots(result) -> dict:
    slots = dict(result.intent_data.slots or {})
    for key, entity in result.entities.items():
        slots[key] = entity.value
    if "name" in slots and "entity_id" not in slots and isinstance(slots["name"], str) and "." in slots["name"]:
        slots["entity_id"] = slots["name"]
    return {k: v for k, v in slots.items() if v is not None}


def category_of(group: str, case: dict) -> str:
    stem = group.lower()
    for needle, label in (
        ("multi", "multi"),
        ("combo", "multi"),
        ("clarif", "clarify"),
        ("query", "query"),
        ("timer", "timers"),
        ("list", "lists"),
        ("climate", "climate"),
        ("cover", "covers"),
        ("lock", "locks"),
        ("fan", "fans"),
        ("switch", "switches"),
        ("media", "media"),
        ("music", "music"),
        ("vacuum", "vacuum"),
        ("scene", "scenes"),
        ("light", "lights"),
        ("state", "followup"),
        ("persist", "followup"),
        ("device", "devices"),
        ("area", "area"),
    ):
        if needle in stem:
            return label
    expect = case.get("nlu_expect") or {}
    if expect.get("clarify"):
        return "clarify"
    if expect.get("reject"):
        return "reject"
    if len(expect.get("intents") or []) > 1:
        return "multi"
    return "other"


def load_cases(suite_dir: Path):
    cases = []
    for path in sorted(p for p in suite_dir.rglob("*.yaml") if p.name != "home_config.yaml"):
        group = str(path.relative_to(suite_dir).with_suffix("")).replace("\\", "/")
        data = load_yaml(path)
        if isinstance(data, dict):
            data = [data]
        cases.extend((group, case) for case in data or [] if isinstance(case, dict))
    return cases


def dialogues(sentences):
    if not sentences:
        return []
    return sentences if isinstance(sentences[0], list) else [[s] for s in sentences]


def recognize(turns, intents, slots, language, agent):
    """Run the turns through hassil, then the gazetteer when hassil has no match.

    Returns the last turn's parse for the whole agent and for hassil alone, or an error name.
    """
    hassil_parsed, assist_parsed = [], []
    agent.reset()
    for text in turns:
        try:
            result = recognize_best(text, intents, slot_lists=slots, language=language, best_slot_name="name")
        except Exception as exc:  # noqa: BLE001
            return None, f"error:{type(exc).__name__}"
        hassil_parsed = [{"name": result.intent.name, "slots": result_slots(result)}] if result is not None else []
        assist_parsed = hassil_parsed or agent.interpret(text)
    return {"assist": assist_parsed, "hassil": hassil_parsed}, None


def score_case(case, parsed, entities):
    expected = case.get("nlu_expect")
    if expected:
        return exact_ok(expected, parsed)
    if not parsed or not case.get("conditions"):
        return False, "unmatched"
    if any(intent["slots"].get("entity_id") == bad or intent["slots"].get("area") == bad for bad in case.get("forbid") or [] for intent in parsed):
        return False, "wrong_target"
    if all(any(i["name"] in expected_intent_names(c) and target_ok(i["slots"], c, entities) for i in parsed) for c in case["conditions"]):
        return True, "ok"
    if parsed and parsed[0]["name"] not in expected_intent_names(case["conditions"][0]):
        return False, "wrong_intent"
    return False, "wrong_target"


def commit_sha() -> str:
    for name in ("KLAR_COMMIT_SHA", "GITHUB_SHA"):
        if os.environ.get(name, "").strip():
            return os.environ[name].strip()
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO, capture_output=True, text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def versions() -> dict[str, str]:
    return {name: metadata.version(name) for name in PACKAGES}


def rate(stats: Counter) -> dict:
    return {"n": stats["n"], "ok": stats["pass"], "fail": stats["fail"], "accuracy": pct(stats["pass"], stats["n"])}


def pct(ok: int, n: int) -> float:
    return round(100.0 * ok / n, 2) if n else 0.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=REPO / "target" / "assist_bench.json")
    parser.add_argument("--suite", action="append", choices=sorted(SUITES), help="Repeat to pick suites. Default: all DE/EN voice suites.")
    parser.add_argument("--commit", default=None, help="Commit the run belongs to. Default: KLAR_COMMIT_SHA, GITHUB_SHA or git HEAD.")
    args = parser.parse_args()
    names = args.suite or DEFAULT_SUITES
    commit = args.commit or commit_sha()
    compiled = {}
    for lang in sorted({SUITES[name][0] for name in names}):
        raw = get_intents(lang)
        if not raw:
            raise SystemExit(f"no Home Assistant intents for {lang}")
        compiled[lang] = Intents.from_dict(raw)
        print(f"loaded {lang} intents={len(compiled[lang].intents)}", flush=True)

    overall = {mode: Counter() for mode in MODES}
    by_cat = {mode: defaultdict(Counter) for mode in MODES}
    by_lang = {mode: defaultdict(Counter) for mode in MODES}
    by_lang_domain = {mode: defaultdict(Counter) for mode in MODES}
    by_reason, samples = Counter(), defaultdict(list)
    suites, t0 = {}, time.monotonic()
    for name in names:
        lang, suite_dir, home_path = SUITES[name]
        home_file = home_path or suite_dir / "home_config.yaml"
        if not home_file.exists():
            home_file = suite_dir.parent / "home_config.yaml"
        home = load_yaml(home_file)
        slots, entities = build_slots(home)
        agent = GazetteerAgent(home, lang)
        stats = {mode: Counter() for mode in MODES}
        cat_stats = {mode: defaultdict(Counter) for mode in MODES}
        started, n = time.monotonic(), 0
        for group, case in load_cases(suite_dir):
            cat = category_of(group, case)
            domain = "out_of_domain" if (case.get("nlu_expect") or {}).get("reject") else "in_domain"
            for turns in dialogues(case.get("sentences") or []):
                n += 1
                parsed_by_mode, error = recognize(turns, compiled[lang], slots, lang, agent)
                for mode in MODES:
                    if error:
                        ok, reason, parsed = False, error, []
                    else:
                        parsed = parsed_by_mode[mode]
                        ok, reason = score_case(case, parsed, entities)
                    key = "pass" if ok else "fail"
                    for bucket in (stats[mode], cat_stats[mode][cat], overall[mode], by_cat[mode][cat], by_lang[mode][lang], by_lang_domain[mode][(lang, domain)]):
                        bucket["n"] += 1
                        bucket[key] += 1
                    if mode == "assist":
                        stats[mode][reason] += 1
                        by_reason[reason] += 1
                        if not ok and len(samples[reason]) < 8:
                            samples[reason].append({"suite": name, "group": group, "case": case.get("name"), "text": turns[-1] if turns else "", "reason": reason, "parsed": parsed})
                if n % 400 == 0:
                    print(f"  {name} {n} … {stats['assist']['pass']}/{stats['assist']['n']}", flush=True)
        skip = {"n", "pass", "fail"}
        suites[name] = {
            "language": lang,
            **rate(stats["assist"]),
            "hassil_only": rate(stats["hassil"]),
            "reasons": {k: v for k, v in stats["assist"].items() if k not in skip},
            "categories": {k: rate(v) for k, v in cat_stats["assist"].items()},
            "seconds": round(time.monotonic() - started, 2),
        }
        print(f"{name}: assist {suites[name]['accuracy']:.1f}%  hassil only {suites[name]['hassil_only']['accuracy']:.1f}%  ({suites[name]['seconds']:.1f}s)", flush=True)

    summary = {
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "commit": commit,
        "engine": "hassil+home-assistant-intents+gazetteer-matcher",
        "versions": versions(),
        "gazetteer_languages": list(GAZETTEER_LANGUAGES),
        "suites": suites,
        "languages": {
            lang: {
                **rate(by_lang["assist"][lang]),
                "hassil_only": rate(by_lang["hassil"][lang]),
                # Free credit for "nothing matched" on out-of-domain sentences is kept apart from real commands.
                **{
                    domain: {**rate(by_lang_domain["assist"][(lang, domain)]), "hassil_only": rate(by_lang_domain["hassil"][(lang, domain)])}
                    for domain in ("in_domain", "out_of_domain")
                },
            }
            for lang in sorted(by_lang["assist"])
        },
        "overall": {**rate(overall["assist"]), "hassil_only": rate(overall["hassil"]), "seconds": round(time.monotonic() - t0, 2)},
        "by_category": {k: rate(v) for k, v in sorted(by_cat["assist"].items())},
        "by_reason": dict(by_reason),
        "samples": samples,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(summary["overall"], indent=2))
    print("wrote", args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
