"""Rewrite DE oracle cases into native parity sentences."""

from __future__ import annotations

from lex import color_word, domain_noun, domain_of, entity_info, room, unique

SCENES = {
    "good_night": "good_night",
    "leaving_home": "leaving",
    "movie_night": "film",
    "dinner_time": "dinner",
    "kids_bedtime": "kids",
    "good_morning": "morning",
}


def switch_label(lex: dict, ident: str, info: dict) -> str:
    blob = ident.lower()
    if "spul" in blob or "spuel" in blob or "dish" in blob:
        return lex.get("dishwasher") or lex["device"]
    if "wasch" in blob or "wash" in blob:
        return lex.get("washer") or lex["device"]
    if "trock" in blob or "dryer" in blob:
        return lex.get("dryer") or "dryer"
    if "pc" in blob or "steck" in blob:
        return "pc"
    if "tv" in blob:
        return "tv"
    area = info.get("area") or ""
    return f"{lex['switch']} {room(lex, area)}" if area else lex["switch"]


def scene_title(lex: dict, name: str) -> str:
    key = SCENES.get(name, name)
    return {"good_night": lex["good_night"], "leaving": lex["leaving"], "film": lex["film"]}.get(key, key)


def script_label(lex: dict, ident: str) -> str:
    return f"{lex['scene']} {ident.split('.')[-1]}"


def action_verb(lex: dict, cond: dict) -> str:
    state = str(cond.get("state") or "")
    kind = cond.get("type") or "action"
    attrs = cond.get("attributes") or {}
    domain = domain_of(cond)
    if kind == "query":
        return lex["query"]
    if attrs.get("brightness") is not None or attrs.get("color") or attrs.get("temperature") is not None or attrs.get("percentage") is not None or attrs.get("position") is not None:
        return lex["set"]
    if domain == "cover":
        if state in {"off", "closed"}:
            return lex["close"]
        if state in {"on", "open"}:
            return lex["open"]
    if domain == "lock":
        if state in {"off", "unlocked"}:
            return lex["unlock"]
        return lex["lock_v"]
    if domain == "media_player" and state == "paused":
        return lex["pause"]
    if state in {"off", "closed", "unlocked", "paused"}:
        return lex["off"]
    if state == "open":
        return lex["open"]
    return lex["on"]


def target_words(lex: dict, cond: dict, suite: str) -> str:
    area = cond.get("area")
    entity = cond.get("entity_id")
    domain = domain_of(cond)
    if entity:
        ident = str(entity)
        info = entity_info(suite, ident)
        if ident.startswith("timer."):
            return f"{lex['timer']} {ident.split('.')[-1]}"
        if ident.startswith("scene.") or ident.startswith("script."):
            return script_label(lex, ident)
        if domain == "climate" and ("ac" in ident or "klima" in f"{ident} {info['name']}".lower()):
            where = room(lex, str(area or info["area"]))
            return f"ac {where}".strip()
        if domain == "binary_sensor":
            return f"sensor {lex['door']} {room(lex, str(area or info['area']))}".strip()
        if domain == "media_player" and cond.get("state") == "paused":
            return f"tv {room(lex, str(area or info['area']))}".strip()
        if domain in {"climate", "fan", "cover"} and (area or info["area"]):
            return f"{domain_noun(lex, domain)} {room(lex, str(area or info['area']))}"
        if "alle" in ident:
            return f"{lex['all']} {lex['light']}"
        if "wohn_und" in ident:
            return "wohn und esszimmer"
        if domain == "light" and any(key in ident or key in info["name"].lower() for key in ("kugel", "globe")):
            return lex["globe"]
        if domain == "light" and info["area"] and any(key in ident or key in info["name"].lower() for key in ("decke", "ceiling")):
            return f"{lex['ceiling']} {room(lex, info['area'])}"
        if domain == "light" and info["area"] and any(key in ident or key in info["name"].lower() for key in ("lampe", "lamp")):
            return f"{lex['lamp']} {room(lex, info['area'])}"
        if domain == "light" and info["area"] and any(key in ident or key in info["name"].lower() for key in ("nacht", "bedside")):
            return f"{lex['bedside']} {room(lex, info['area'])}"
        if domain == "light" and info["area"] and any(key in ident or key in info["name"].lower() for key in ("insel", "island")):
            return f"{lex['island']} {room(lex, info['area'])}"
        if domain == "light" and info["area"] and "ensuite" in ident:
            return f"ankleide {room(lex, info['area'])}"
        if domain == "light" and info["area"] and any(ch.isdigit() for ch in info["name"]):
            return f"{lex['ceiling']} {room(lex, info['area'])}"
        if domain == "light" and info["area"] and info["area"] not in {"wohnung", "home"}:
            return f"{domain_noun(lex, domain)} {room(lex, info['area'])}"
        if ident.startswith("scene.") or ident.startswith("script."):
            return script_label(lex, ident)
        if domain == "switch":
            return switch_label(lex, ident, info)
        if info["area"]:
            return f"{domain_noun(lex, domain)} {room(lex, info['area'])}"
        return domain_noun(lex, domain)
    if area:
        return f"{area_noun(lex, domain, cond)} {room(lex, str(area))}"
    return domain_noun(lex, domain)


def area_noun(lex: dict, domain: str | None, cond: dict | None = None) -> str:
    noun = domain_noun(lex, domain)
    query = (cond or {}).get("type") == "query"
    if query:
        return noun
    if domain == "switch":
        return f"{lex['all']} {lex['switch']}"
    if domain == "light":
        return f"{lex['all']} {noun}"
    return noun


def phrase(lex: dict, cond: dict, suite: str) -> str:
    attrs = cond.get("attributes") or {}
    verb = action_verb(lex, cond)
    target = target_words(lex, cond, suite)
    if attrs.get("brightness") is not None:
        return f"{lex['set']} {target} {attrs['brightness']}"
    if attrs.get("color"):
        return f"{lex['set']} {target} {color_word(lex, attrs['color'])}"
    if attrs.get("temperature") is not None:
        if target == lex["climate"]:
            target = f"{lex['climate']} {room(lex, 'wohnzimmer')}"
        return f"{lex['set']} {target} {attrs['temperature']}"
    if attrs.get("percentage") is not None:
        return f"{lex['set']} {target} {attrs['percentage']}"
    if attrs.get("position") is not None:
        return f"{lex['set']} {target} {attrs['position']}"
    if cond.get("minutes") is not None:
        return f"{lex['on']} {target} {cond['minutes']} {lex['minutes']}"
    return f"{verb} {target}"


def except_targets(name: str, lex: dict, case: dict) -> list[str]:
    targets: list[str] = []
    if "kugel" in name:
        targets.append(lex["globe"])
    if "island" in name or "insel" in name:
        targets.append(lex["island"])
    if "schlaf" in name:
        targets.append(room(lex, "schlafzimmer"))
    if "kueche" in name or "kitchen" in name or name.endswith("kuche") or "_kuche" in name or "und_kueche" in name:
        targets.append(room(lex, "kuche"))
    if "arbeits" in name:
        targets.append(room(lex, "arbeitszimmer"))
    if not targets:
        for item in case.get("forbid") or []:
            token = str(item)
            if "island" in token or "insel" in token:
                targets.append(lex["island"])
            elif token in {"schlafzimmer", "kuche", "kitchen", "arbeitszimmer"}:
                targets.append(room(lex, "kuche" if token == "kitchen" else token))
            elif "kugel" in token:
                targets.append(lex["globe"])
    return unique(targets)


def except_sentence(case: dict, lex: dict) -> str:
    conds = case.get("conditions") or [{}]
    first = conds[0]
    attrs = first.get("attributes") or {}
    excluded = f" {lex['except']} ".join(except_targets(str(case.get("name") or ""), lex, case)) or room(lex, "kuche")
    if first.get("type") == "query":
        return f"{lex['query']} {lex['all']} {lex['light']} {lex['except']} {excluded}"
    if attrs.get("color"):
        return f"{lex['set']} {lex['all']} {lex['light']} {lex['except']} {excluded} {color_word(lex, attrs['color'])}"
    if attrs.get("brightness") is not None:
        return f"{lex['set']} {lex['all']} {lex['light']} {lex['except']} {excluded} {attrs['brightness']}"
    return f"{action_verb(lex, first)} {lex['all']} {lex['light']} {lex['except']} {excluded}"


def join_conds(conds: list[dict], lex: dict, suite: str) -> str:
    if all(c.get("entity_id") and not (c.get("attributes") or {}) for c in conds) and len(conds) > 1:
        areas = unique([entity_info(suite, str(c["entity_id"]))["area"] for c in conds])
        domains = unique([domain_of(c) for c in conds])
        if len(areas) == 1 and areas[0] and len(domains) == 1 and "kugel" not in str(conds):
            return f"{action_verb(lex, conds[0])} {area_noun(lex, domains[0], conds[0])} {room(lex, areas[0])}"
        parts = [phrase(lex, item, suite) for item in conds]
        return f" {lex['then']} ".join(parts)
    parts: list[str] = []
    index = 0
    while index < len(conds):
        current = conds[index]
        attrs = current.get("attributes") or {}
        if current.get("area") and not attrs:
            verb = action_verb(lex, current)
            noun = area_noun(lex, current.get("domain"), current)
            areas = []
            while index < len(conds) and conds[index].get("area") and not (conds[index].get("attributes") or {}) and action_verb(lex, conds[index]) == verb:
                areas.append(room(lex, str(conds[index]["area"])))
                index += 1
            for area in unique(areas):
                parts.append(f"{verb} {noun} {area}")
            continue
        parts.append(phrase(lex, current, suite))
        index += 1
    return f" {lex['then']} ".join(parts)


def sentence_for(case: dict, lex: dict, suite: str) -> list[str] | list[list[str]]:
    expect = case.get("nlu_expect") or {}
    name = str(case.get("name") or "")
    sentences = case.get("sentences")
    multi = isinstance(sentences, list) and sentences and isinstance(sentences[0], list)
    if expect.get("reject"):
        return [lex["reject"]]
    if expect.get("clarify") and not expect.get("intents"):
        return [f"{lex['on']} {lex['light']} {room(lex, 'wohnzimmer')}"]
    if multi and ("vs" in name or "lampe" in name):
        return [clarify_turns(case, lex, suite)]
    if multi:
        return [multi_turn(case, lex, suite)]
    intents = expect.get("intents") or []
    if intents:
        return [join_intents(intents, lex, suite)]
    conds = case.get("conditions") or []
    if not conds:
        return [f"{lex['on']} {lex['light']}"]
    if any((c.get("type") in {"shopping_list", "todo_list"}) or c.get("item") for c in conds):
        item = next((c.get("item") for c in conds if c.get("item")), "apples")
        if any(word in str(item).lower() for word in ("kitchen", "floor", "toilet", "counter")):
            item = str(item).split()[0]
        chores = any(c.get("type") == "todo_list" or "chore" in str(c.get("list_name") or "").lower() for c in conds)
        verb = lex["done"] if any(c.get("complete") for c in conds) else lex["add"]
        return [f"{verb} {item} {'aufgabenliste' if chores else lex['list']}"]
    if name == "cancel_all_timers":
        return [f"{lex['off']} {lex['all']} {lex['timer']}"]
    if "kugel_und_decke" in name:
        return [f"{lex['globe']} {lex['and']} {lex['ceiling']} {room(lex, 'schlafzimmer')} {lex['off']}"]
    if "ausser" in name or "except" in name:
        return [except_sentence(case, lex)]
    if len(conds) > 1:
        return [join_conds(conds, lex, suite)]
    return [phrase(lex, conds[0], suite)]


def clarify_turns(case: dict, lex: dict, suite: str) -> list[str]:
    name = str(case.get("name") or "")
    conds = case.get("conditions") or [{}]
    area = str(conds[0].get("area") or entity_info(suite, str(conds[0].get("entity_id") or "")).get("area") or "")
    where = room(lex, area)
    left = name.split("_vs_")[0]
    if "wash" in name:
        return [f"{lex.get('on2', lex['on'])} {lex['device']} {room(lex, area)}", lex["yes"]]
    if "island" in left:
        pick = lex["island"]
    elif "kugel" in left or "kugel" in name:
        pick = lex["globe"]
    elif "ceiling" in left or "decke" in left:
        pick = lex["ceiling"]
    elif "lamp" in left:
        pick = lex["lamp"]
    elif "bedside" in left or "nacht" in left:
        pick = lex["bedside"]
    else:
        pick = lex["yes"]
    return [f"{lex['on']} {lex['light']} {where}", pick]


def multi_turn(case: dict, lex: dict, suite: str) -> list[str]:
    conds = case.get("conditions") or []
    expect = case.get("nlu_expect") or {}
    intents = expect.get("intents") or []
    original = case.get("sentences")[0]
    name = str(case.get("name") or "")
    if intents:
        slots = (intents[0].get("slots") or {})
        area = slots.get("area")
        brightness = slots.get("brightness")
        first = f"{lex['query']} {lex['light']} {room(lex, str(area))}" if area else f"{lex['query']} {lex['light']}"
        second = str(brightness) if brightness is not None else followup(lex, conds[0] if conds else {}, original)
        return [first, second]
    cond = conds[0] if conds else {}
    target = target_words(lex, cond, suite) if conds else lex["light"]
    hint = str(original[0]).lower()
    if len(conds) > 1 and conds[0].get("entity_id"):
        areas = unique([entity_info(suite, str(item["entity_id"]))["area"] for item in conds if item.get("entity_id")])
        noun = domain_noun(lex, domain_of(cond))
        first = f" {lex['then']} ".join(f"{lex['query']} {noun} {room(lex, area)}" for area in areas if area)
        return [first, followup(lex, cond, original)]
    if "wohnzimmer_dann_kueche" in name or "dann_kueche" in name:
        return [f"{lex['on']} {lex['light']} {room(lex, 'wohnzimmer')}", f"{lex['on']} {lex['light']} {room(lex, 'kuche')}"]
    if cond.get("type") == "query" or "status" in hint or "ist" in hint:
        first = f"{lex['query']} {target}"
    elif "kugel" in name:
        first = f"{lex['on']} {lex['globe']}"
    else:
        first = f"{lex['on']} {target}"
    return [first, followup(lex, cond, original)]


def join_intents(intents: list[dict], lex: dict, suite: str) -> str:
    parts = []
    for item in intents:
        slots = item.get("slots") or {}
        name = item.get("intent") or "HassTurnOn"
        cond = {
            "entity_id": slots.get("entity_id"),
            "area": slots.get("area"),
            "domain": slots.get("domain"),
            "state": "off" if name == "HassTurnOff" else "on",
            "minutes": slots.get("minutes"),
            "attributes": {key: slots[key] for key in ("brightness", "color", "temperature", "percentage", "position") if key in slots},
        }
        if name in {"HassStartTimer", "HassIncreaseTimer"}:
            entity = str(slots.get("entity_id") or "timer.oven").split(".")[-1]
            if slots.get("minutes") is not None:
                parts.append(f"{lex['on']} {lex['timer']} {entity} {slots['minutes']} {lex['minutes']}")
            else:
                parts.append(f"{lex['on']} {lex['timer']} {entity}")
        elif name in {"HassListAddItem", "HassShoppingListAddItem"}:
            item = slots.get("item") or "apples"
            if "chore" in str(slots.get("entity_id") or ""):
                parts.append(f"{lex['add']} {item} aufgabenliste")
            else:
                parts.append(f"{lex['add']} {item} {lex['list']}")
        elif name in {"HassListCompleteItem", "HassShoppingListCompleteItem"}:
            item = slots.get("item") or "bread"
            parts.append(f"{lex['done']} {item} aufgabenliste")
        elif slots.get("floor"):
            verb = lex["off"] if name == "HassTurnOff" else lex["on"]
            parts.append(f"{verb} {lex['light']} {lex['floors'].get(str(slots['floor']), [str(slots['floor'])])[0]}")
        else:
            parts.append(phrase(lex, cond, suite))
    return f" {lex['then']} ".join(parts)


def followup(lex: dict, cond: dict, original: list[str]) -> str:
    attrs = cond.get("attributes") or {}
    state = str(cond.get("state") or "")
    domain = domain_of(cond)
    if attrs.get("color"):
        return color_word(lex, attrs["color"])
    if attrs.get("brightness") is not None or attrs.get("position") is not None:
        return str(attrs.get("brightness", attrs.get("position")))
    if domain == "cover" and state in {"off", "closed", "on", "open"}:
        return lex["close"] if state in {"off", "closed"} else lex["open"]
    if domain == "lock" and state in {"off", "unlocked", "on", "locked"}:
        return lex["unlock"] if state in {"off", "unlocked"} else lex["lock_v"]
    if state == "off":
        return lex["off"]
    if state == "on":
        return lex["on"]
    hint = original[1].lower() if len(original) > 1 else ""
    if "kugel" in hint:
        return lex["globe"]
    if "decke" in hint or "ceiling" in hint:
        return lex["ceiling"]
    if any(key in hint for key in ("küche", "kuche", "kueche")):
        return f"{lex['on']} {lex['light']} {room(lex, 'kuche')}"
    if hint and any(ch.isdigit() for ch in original[1]):
        return "".join(ch for ch in original[1] if ch.isdigit() or ch == " ")
    return lex["yes"]
