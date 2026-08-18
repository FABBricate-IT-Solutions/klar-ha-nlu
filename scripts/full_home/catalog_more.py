"""Full-tier extras so every locale approaches English family-home breadth."""

from __future__ import annotations

from catalog import (
    ALL_ROOMS,
    CLIMATES,
    COVERS,
    FANS,
    LIGHTS,
    LOCKS,
    _action,
    _case,
    _query,
)

SEARCHES = (
    "queen",
    "beatles",
    "radiohead",
    "abba",
    "coldplay",
    "adele",
    "prince",
    "bowie",
    "nirvana",
    "eagles",
    "madonna",
    "mozart",
    "daft punk",
    "biafra",
)
PLAYERS = (
    ("media_player.living_music", "living"),
    ("media_player.kitchen_music", "kitchen"),
    ("media_player.office_music", "office"),
)
LIST_ITEMS = ("milk", "bread", "eggs", "apples", "coffee")
TIMER_MINS = (5, 15, 20, 30, 60)
FAN_SPEEDS = (20, 60, 80)
QUERY_MORE = (
    ("light.kitchen_ceiling", "kitchen", "light"),
    ("light.office_ceiling", "office", "light"),
    ("light.garden", "garden", "light"),
    ("light.basement", "basement", "light"),
    ("cover.master_blinds", "master_bedroom", "cover"),
    ("cover.garage_door", "garage", "cover"),
    ("lock.garage_entry", "garage", "lock"),
    ("climate.upper_thermostat", "master_bedroom", "climate"),
    ("climate.master_ac", "master_bedroom", "climate"),
    ("media_player.family_tv", "family_room", "media_player"),
    ("media_player.living_music", "living", "media_player"),
    ("binary_sensor.kitchen_motion", "kitchen", "binary_sensor"),
    ("sensor.living_humidity", "living", "sensor"),
    ("fan.master_fan", "master_bedroom", "fan"),
    ("vacuum.robot", "living", "vacuum"),
)


def extra() -> list[dict]:
    out = []
    for entity, area in PLAYERS:
        for query in SEARCHES:
            if entity == "media_player.living_music" and query == "queen":
                continue
            out.append(
                _case(
                    f"play_{query}_{area}",
                    "music",
                    "full",
                    [_action(entity=entity, search_query=query)],
                    "play_search",
                    area=area,
                    query=query,
                )
            )
    for item in LIST_ITEMS:
        tier = "both" if item == "milk" else "full"
        if item != "milk":
            out.append(_case(f"list_add_{item}", "lists", tier, [{"type": "todo_list", "item": item}], "list_add", item=item))
        out.append(_case(f"list_done_{item}", "lists", "full", [{"type": "todo_list", "item": item}], "list_done", item=item))
    for minutes in TIMER_MINS:
        out.append(
            _case(
                f"timer_oven_{minutes}",
                "timers",
                "full",
                [_action(entity="timer.oven", minutes=minutes)],
                "timer_start",
                timer="oven",
                n=minutes,
            )
        )
    for entity, area in FANS:
        for n in FAN_SPEEDS:
            out.append(_case(f"{entity}_speed_{n}", "fans", "full", [_action(entity=entity, percentage=n)], "fan_speed", area=area, n=n))
    for entity, area, domain in QUERY_MORE:
        out.append(_case(f"query_{entity}", "query", "full", [_query(entity=entity)], "query_entity", area=area, domain=domain, entity=entity))
    for left, right in (("basement", "garden"), ("hallway", "office"), ("dining", "kitchen"), ("bedroom_2", "bedroom_3")):
        out.append(
            _case(
                f"multi_{left}_{right}",
                "multi",
                "full",
                [_action(area=left, domain="light", state="on"), _action(area=right, domain="light", state="on")],
                "multi_and",
                area=left,
                area2=right,
            )
        )
    out.append(_case("scene_morning", "scenes", "full", [_action(entity="scene.good_morning", state="on")], "scene", scene="morning"))
    out.append(_case("scene_kids", "scenes", "full", [_action(entity="scene.kids_bedtime", state="on")], "scene", scene="kids"))
    for entity, area in CLIMATES:
        out.append(_case(f"{entity}_get_full", "climate", "full", [_query(entity=entity)], "get_temp", area=area, entity=entity))
    for entity, area in COVERS[:2]:
        out.append(_case(f"{entity}_open_extra", "covers", "full", [_action(entity=entity, state="open")], "open_cover", area=area))
    for entity, area in LOCKS:
        out.append(_case(f"{entity}_lock_extra", "locks", "full", [_action(entity=entity, state="locked")], "lock", area=area))
    for entity, area, fixture in LIGHTS[8:12]:
        out.append(_case(f"{entity}_on_extra", "lights", "full", [_action(entity=entity, state="on")], "on_fixture", area=area, fixture=fixture, entity=entity))
    for entity, area, fixture in LIGHTS:
        if entity in {row[0] for row in QUERY_MORE} or entity == "light.living_ceiling":
            continue
        out.append(
            _case(
                f"query_{entity}",
                "query",
                "full",
                [_query(entity=entity)],
                "query_entity",
                area=area,
                domain="light",
                entity=entity,
            )
        )
    for entity, area in PLAYERS:
        out.append(
            _case(
                f"radio_{area}",
                "music",
                "full",
                [_action(entity=entity, media_id="queen", radio_mode="true")],
                "play_radio",
                area=area,
                query="queen",
            )
        )
        out.append(_case(f"now_{area}", "music", "full", [_query(entity=entity)], "now_playing", area=area))
    assert ALL_ROOMS
    return out
