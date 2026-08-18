"""Language-neutral Assist cases for a full smart home (quick + full).

Quick is the CI gate: only cases every compiled locale can realize with pack
verbs. Cases that already work in de/en but not yet in generated packs stay in
full so they remain in the big dataset without blocking language work.
"""

from __future__ import annotations

QUICK_ROOMS = ("living", "kitchen", "master_bedroom", "office", "garage", "hallway", "garden", "basement")
ALL_ROOMS = (
    "entryway",
    "living",
    "dining",
    "kitchen",
    "family_room",
    "laundry",
    "powder_room",
    "garage",
    "office",
    "garden",
    "master_bedroom",
    "bedroom_2",
    "bedroom_3",
    "bedroom_4",
    "master_bath",
    "main_bath",
    "hallway",
    "basement",
)
LIGHTS = (
    ("light.living_ceiling", "living", "ceiling"),
    ("light.living_lamp", "living", "lamp"),
    ("light.kitchen_ceiling", "kitchen", "ceiling"),
    ("light.kitchen_island", "kitchen", "island"),
    ("light.master_ceiling", "master_bedroom", "ceiling"),
    ("light.master_bedside_left", "master_bedroom", "bedside"),
    ("light.office_ceiling", "office", "ceiling"),
    ("light.garden", "garden", "light"),
    ("light.basement", "basement", "light"),
    ("light.hallway_light", "hallway", "light"),
    ("light.family_ceiling", "family_room", "ceiling"),
    ("light.dining_pendant", "dining", "light"),
    ("light.garage_light", "garage", "light"),
    ("light.bedroom2_ceiling", "bedroom_2", "ceiling"),
    ("light.bedroom3_ceiling", "bedroom_3", "ceiling"),
    ("light.bedroom4_ceiling", "bedroom_4", "ceiling"),
    ("light.entryway_light", "entryway", "light"),
    ("light.laundry_light", "laundry", "light"),
    ("light.main_bath_light", "main_bath", "light"),
    ("light.powder_room_light", "powder_room", "light"),
    ("light.master_ensuite", "master_bedroom", "ensuite"),
    ("light.master_bedside_right", "master_bedroom", "bedside"),
    ("light.master_bath_light", "master_bath", "light"),
)
COVERS = (
    ("cover.living_blinds", "living"),
    ("cover.master_blinds", "master_bedroom"),
    ("cover.bedroom2_blinds", "bedroom_2"),
    ("cover.bedroom3_blinds", "bedroom_3"),
    ("cover.garage_door", "garage"),
)
CLIMATES = (
    ("climate.ground_thermostat", "living"),
    ("climate.upper_thermostat", "master_bedroom"),
    ("climate.master_ac", "master_bedroom"),
)
FANS = (("fan.family_fan", "family_room"), ("fan.master_fan", "master_bedroom"), ("fan.bedroom2_fan", "bedroom_2"))
LOCKS = (("lock.front_door", "entryway"), ("lock.garage_entry", "garage"))
SWITCHES = (
    ("switch.dishwasher", "kitchen", "dishwasher"),
    ("switch.washing_machine", "laundry", "washer"),
    ("switch.dryer", "laundry", "dryer"),
    ("switch.rangehood", "kitchen", "rangehood"),
    ("switch.master_bath_fan", "master_bath", "bathfan"),
    ("switch.main_bath_fan", "main_bath", "bathfan"),
)
COLORS = ("red", "blue", "green", "white", "yellow", "black")
BRIGHT = (20, 40, 50, 60, 80, 100)


def _case(name, group, tier, conditions, kind, **slots):
    row = {"name": name, "group": group, "tier": tier, "conditions": conditions, "kind": kind}
    row.update(slots)
    return row


def _action(entity=None, area=None, domain=None, state=None, **attrs):
    cond = {"type": "action"}
    if entity:
        cond["entity_id"] = entity
    if area:
        cond["area"] = area
    if domain:
        cond["domain"] = domain
    if state is not None:
        cond["state"] = state
    if attrs:
        cond["attributes"] = attrs
    return cond


def _query(entity=None, area=None, domain=None):
    cond = {"type": "query"}
    if entity:
        cond["entity_id"] = entity
    if area:
        cond["area"] = area
    if domain:
        cond["domain"] = domain
    return cond


def lights() -> list[dict]:
    out = []
    for area in ALL_ROOMS:
        tier = "both" if area in QUICK_ROOMS else "full"
        out.append(_case(f"lights_{area}_on", "lights", tier, [_action(area=area, domain="light", state="on")], "on_area", area=area))
        out.append(_case(f"lights_{area}_off", "lights", tier, [_action(area=area, domain="light", state="off")], "off_area", area=area))
    for entity, area, fixture in LIGHTS:
        tier = "both" if entity in {row[0] for row in LIGHTS[:8]} else "full"
        out.append(_case(f"{entity}_on", "lights", tier, [_action(entity=entity, state="on")], "on_fixture", area=area, fixture=fixture, entity=entity))
        out.append(_case(f"{entity}_off", "lights", tier, [_action(entity=entity, state="off")], "off_fixture", area=area, fixture=fixture, entity=entity))
        for n in BRIGHT:
            out.append(
                _case(
                    f"{entity}_bright_{n}",
                    "lights",
                    "both" if n == 50 and entity == "light.living_ceiling" else "full",
                    [_action(entity=entity, brightness=n)],
                    "set_bright",
                    area=area,
                    fixture=fixture,
                    n=n,
                )
            )
    for entity, area, fixture in LIGHTS:
        for color in COLORS:
            out.append(
                _case(
                    f"{entity}_{color}",
                    "lights",
                    "both" if color == "red" and entity == "light.living_ceiling" else "full",
                    [_action(entity=entity, color=color)],
                    "set_color",
                    area=area,
                    fixture=fixture,
                    color=color,
                )
            )
    for skip in ("kitchen", "office", "garage", "master_bedroom"):
        conds = [_action(area=area, domain="light", state="off") for area in ALL_ROOMS if area != skip]
        out.append(
            _case(
                f"all_except_{skip}",
                "lights",
                "full",
                conds,
                "all_except",
                area=skip,
                forbid=[skip],
            )
        )
    for floor_id in ("ground", "upper", "basement"):
        out.append(
            _case(
                f"floor_{floor_id}_on",
                "lights",
                "full",
                [_action(domain="light", state="on")],
                "floor_on",
                floor=floor_id,
            )
        )
        out.append(_case(f"floor_{floor_id}_off", "lights", "full", [_action(domain="light", state="off")], "floor_off", floor=floor_id))
    return out


def climate_cover_lock() -> list[dict]:
    out = []
    for entity, area in CLIMATES:
        for temp in (18, 19, 20, 21, 22, 23, 24):
            out.append(
                _case(
                    f"{entity}_{temp}",
                    "climate",
                    "full",
                    [_action(entity=entity, temperature=temp)],
                    "set_temp",
                    area=area,
                    n=temp,
                    entity=entity,
                )
            )
        out.append(_case(f"{entity}_get", "climate", "full", [_query(entity=entity)], "get_temp", area=area, entity=entity))
    for entity, area in COVERS:
        out.append(_case(f"{entity}_open", "covers", "full", [_action(entity=entity, state="open")], "open_cover", area=area))
        out.append(_case(f"{entity}_close", "covers", "full", [_action(entity=entity, state="closed")], "close_cover", area=area))
        for pos in (0, 25, 40, 75, 100):
            out.append(_case(f"{entity}_pos_{pos}", "covers", "full", [_action(entity=entity, position=pos)], "set_pos", area=area, n=pos))
    for entity, area in LOCKS:
        out.append(_case(f"{entity}_lock", "locks", "full", [_action(entity=entity, state="locked")], "lock", area=area))
        out.append(_case(f"{entity}_unlock", "locks", "full", [_action(entity=entity, state="unlocked")], "unlock", area=area))
    return out


def appliances() -> list[dict]:
    out = []
    for entity, area in FANS:
        out.append(_case(f"{entity}_on", "fans", "full", [_action(entity=entity, state="on")], "fan_on", area=area))
        out.append(_case(f"{entity}_off", "fans", "full", [_action(entity=entity, state="off")], "fan_off", area=area))
        out.append(_case(f"{entity}_speed", "fans", "full", [_action(entity=entity, percentage=40)], "fan_speed", area=area, n=40))
    for entity, area, kind in SWITCHES:
        out.append(
            _case(
                f"{entity}_on",
                "switches",
                "full",
                [_action(entity=entity, state="on")],
                "switch_on",
                area=area,
                appliance=kind,
            )
        )
        out.append(_case(f"{entity}_off", "switches", "full", [_action(entity=entity, state="off")], "switch_off", area=area, appliance=kind))
    out.append(_case("vacuum_start", "vacuum", "both", [_action(entity="vacuum.robot", state="on")], "vac_start"))
    out.append(_case("vacuum_dock", "vacuum", "full", [_action(entity="vacuum.robot", state="off")], "vac_dock"))
    return out


def media() -> list[dict]:
    out = []
    for entity, area in (("media_player.living_tv", "living"), ("media_player.family_tv", "family_room")):
        out.append(_case(f"{entity}_on", "media", "full", [_action(entity=entity, state="on")], "media_on", area=area))
        out.append(_case(f"{entity}_off", "media", "full", [_action(entity=entity, state="off")], "media_off", area=area))
        out.append(_case(f"{entity}_pause", "media", "full", [_action(entity=entity, state="paused")], "media_pause", area=area))
    out.append(
        _case(
            "play_queen",
            "music",
            "full",
            [_action(entity="media_player.living_music", search_query="queen")],
            "play_search",
            area="living",
            query="queen",
        )
    )
    out.append(
        _case(
            "play_album",
            "music",
            "full",
            [_action(entity="media_player.living_music", media_id="rumours", media_type="album", artist="fleetwood mac")],
            "play_album",
            area="living",
            query="rumours",
            artist="fleetwood mac",
        )
    )
    out.append(
        _case(
            "play_radio",
            "music",
            "full",
            [_action(entity="media_player.living_music", media_id="queen", radio_mode="true")],
            "play_radio",
            area="living",
            query="queen",
        )
    )
    out.append(
        _case(
            "queue_song",
            "music",
            "full",
            [_action(entity="media_player.living_music", media_id="bohemian rhapsody", enqueue="add")],
            "play_queue",
            area="living",
            query="bohemian rhapsody",
        )
    )
    out.append(_case("now_playing", "music", "full", [_query(entity="media_player.living_music")], "now_playing", area="living"))
    for vol in (10, 20, 30, 50, 80):
        out.append(
            _case(
                f"music_volume_{vol}",
                "music",
                "full",
                [_action(entity="media_player.living_music", volume_level=vol)],
                "media_vol",
                area="living",
                n=vol,
            )
        )
    out.append(_case("music_next", "music", "full", [_action(entity="media_player.living_music", state="next")], "media_next", area="living"))
    out.append(_case("music_prev", "music", "full", [_action(entity="media_player.living_music", state="previous")], "media_prev", area="living"))
    out.append(_case("kitchen_play", "music", "full", [_action(entity="media_player.kitchen_music", search_query="queen")], "play_search", area="kitchen", query="queen"))
    return out


def household() -> list[dict]:
    out = [
        _case("scene_film", "scenes", "both", [_action(entity="scene.movie_night", state="on")], "scene", scene="film"),
        _case("scene_night", "scenes", "full", [_action(entity="script.good_night", state="on")], "script", scene="good_night"),
        _case("scene_leave", "scenes", "full", [_action(entity="script.leaving_home", state="on")], "script", scene="leaving"),
        _case("scene_dinner", "scenes", "full", [_action(entity="scene.dinner_time", state="on")], "scene", scene="dinner"),
        _case("timer_oven", "timers", "full", [_action(entity="timer.oven", minutes=10)], "timer_start", timer="oven", n=10),
        _case("timer_laundry", "timers", "full", [_action(entity="timer.laundry", minutes=45)], "timer_start", timer="laundry", n=45),
        _case("timer_cancel", "timers", "full", [_action(entity="timer.abstract")], "timer_cancel"),
        _case("list_add", "lists", "full", [{"type": "todo_list", "item": "milk"}], "list_add", item="milk"),
        _case("list_done", "lists", "full", [{"type": "todo_list", "item": "milk"}], "list_done", item="milk"),
    ]
    return out


def queries_multi() -> list[dict]:
    out = []
    for entity, area, domain in (
        ("light.living_ceiling", "living", "light"),
        ("cover.living_blinds", "living", "cover"),
        ("lock.front_door", "entryway", "lock"),
        ("climate.ground_thermostat", "living", "climate"),
        ("media_player.living_tv", "living", "media_player"),
        ("binary_sensor.front_door_sensor", "entryway", "binary_sensor"),
        ("binary_sensor.living_window", "living", "binary_sensor"),
        ("sensor.living_temperature", "living", "sensor"),
    ):
        out.append(
            _case(
                f"query_{entity}",
                "query",
                "full",
                [_query(entity=entity)],
                "query_entity",
                area=area,
                domain=domain,
                entity=entity,
            )
        )
    out.append(
        _case(
            "multi_living_kitchen",
            "multi",
            "full",
            [_action(area="living", domain="light", state="on"), _action(area="kitchen", domain="light", state="on")],
            "multi_and",
            area="living",
            area2="kitchen",
        )
    )
    out.append(
        _case(
            "multi_off_lock",
            "multi",
            "full",
            [_action(area="living", domain="light", state="off"), _action(entity="lock.front_door", state="locked")],
            "multi_off_lock",
            area="living",
        )
    )
    for left, right in (
        ("kitchen", "dining"),
        ("office", "hallway"),
        ("garage", "garden"),
        ("master_bedroom", "main_bath"),
        ("family_room", "living"),
    ):
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
    for skip in ALL_ROOMS:
        if skip in {"kitchen", "office", "garage", "master_bedroom"}:
            continue
        conds = [_action(area=area, domain="light", state="off") for area in ALL_ROOMS if area != skip]
        out.append(_case(f"all_except_{skip}", "lights", "full", conds, "all_except", area=skip, forbid=[skip]))
    return out


def catalog() -> list[dict]:
    from catalog_combos import combos
    from catalog_more import extra

    return lights() + climate_cover_lock() + appliances() + media() + household() + queries_multi() + extra() + combos()


def for_tier(tier: str) -> list[dict]:
    return [case for case in catalog() if case["tier"] in {tier, "both"}]
