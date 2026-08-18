"""Device combinations and exceptions: light X and Y, all except X."""

from __future__ import annotations

from catalog import ALL_ROOMS, _action, _case

DEVICE_PAIRS = (
    (("light.living_ceiling", "living", "ceiling"), ("light.living_lamp", "living", "lamp"), "full"),
    (("light.kitchen_ceiling", "kitchen", "ceiling"), ("light.kitchen_island", "kitchen", "island"), "full"),
    (("light.living_ceiling", "living", "ceiling"), ("light.kitchen_island", "kitchen", "island"), "full"),
    (("light.office_ceiling", "office", "ceiling"), ("light.hallway_light", "hallway", "light"), "full"),
    (("light.garden", "garden", "light"), ("light.garage_light", "garage", "light"), "full"),
    (("light.master_ceiling", "master_bedroom", "ceiling"), ("light.master_bedside_left", "master_bedroom", "bedside"), "full"),
    (("light.family_ceiling", "family_room", "ceiling"), ("light.dining_pendant", "dining", "light"), "full"),
    (("light.basement", "basement", "light"), ("light.laundry_light", "laundry", "light"), "full"),
)

CROSS_DOMAIN = (
    (("fan.master_fan", "master_bedroom", "fan"), ("light.kitchen_island", "kitchen", "island"), "full"),
    (("cover.living_blinds", "living", "cover"), ("light.living_ceiling", "living", "ceiling"), "full"),
    (("switch.washing_machine", "laundry", "washer"), ("switch.dryer", "laundry", "dryer"), "full"),
)

ROOM_OFF = (
    ("living", "kitchen", "full"),
    ("office", "hallway", "full"),
    ("garage", "garden", "full"),
    ("master_bedroom", "main_bath", "full"),
    ("family_room", "dining", "full"),
)

ROOM_TRIPLES = (
    ("living", "dining", "kitchen", "off", "full"),
    ("office", "hallway", "garage", "on", "full"),
    ("garden", "basement", "laundry", "off", "full"),
)

EXCEPT_FIXTURES = (
    ("light.kitchen_island", "kitchen", "island", "full"),
    ("light.living_lamp", "living", "lamp", "full"),
    ("light.garden", "garden", "light", "full"),
    ("light.office_ceiling", "office", "ceiling", "full"),
    ("light.basement", "basement", "light", "full"),
)

EXCEPT_ON_ROOMS = ("kitchen", "office", "garden", "living", "garage")
EXCEPT_TWO = (("kitchen", "office", "full"), ("living", "master_bedroom", "full"), ("garage", "garden", "full"))
FLOOR_ROOMS = {
    "ground": ("entryway", "living", "dining", "kitchen", "family_room", "laundry", "powder_room", "garage", "office", "garden"),
    "upper": ("master_bedroom", "bedroom_2", "bedroom_3", "bedroom_4", "master_bath", "main_bath", "hallway"),
    "basement": ("basement",),
}


def _except_off_others(skip_entity: str, skip_area: str) -> list[dict]:
    conds = [_action(area=area, domain="light", state="off") for area in ALL_ROOMS if area != skip_area]
    if skip_area == "kitchen":
        conds = [row for row in conds if row.get("area") != "kitchen"]
        conds.append(_action(entity="light.kitchen_ceiling", state="off"))
    elif skip_area == "living":
        conds = [row for row in conds if row.get("area") != "living"]
        other = "light.living_ceiling" if skip_entity != "light.living_ceiling" else "light.living_lamp"
        conds.append(_action(entity=other, state="off"))
    return conds


def combos() -> list[dict]:
    out = []
    for (left, right, tier) in DEVICE_PAIRS:
        l_ent, l_area, l_fix = left
        r_ent, r_area, r_fix = right
        out.append(
            _case(
                f"and_{l_ent}_{r_ent}_on",
                "combo",
                tier,
                [_action(entity=l_ent, state="on"), _action(entity=r_ent, state="on")],
                "multi_fixtures",
                area=l_area,
                area2=r_area,
                fixture=l_fix,
                fixture2=r_fix,
            )
        )
        out.append(
            _case(
                f"and_{l_ent}_{r_ent}_off",
                "combo",
                "full",
                [_action(entity=l_ent, state="off"), _action(entity=r_ent, state="off")],
                "multi_fixtures_off",
                area=l_area,
                area2=r_area,
                fixture=l_fix,
                fixture2=r_fix,
            )
        )
    for (left, right, tier) in CROSS_DOMAIN:
        l_ent, l_area, l_fix = left
        r_ent, r_area, r_fix = right
        out.append(
            _case(
                f"and_{l_ent}_{r_ent}_on",
                "combo",
                tier,
                [_action(entity=l_ent, state="on"), _action(entity=r_ent, state="on")],
                "multi_fixtures",
                area=l_area,
                area2=r_area,
                fixture=l_fix,
                fixture2=r_fix,
            )
        )
    for left, right, tier in ROOM_OFF:
        out.append(
            _case(
                f"off_{left}_{right}",
                "combo",
                tier,
                [_action(area=left, domain="light", state="off"), _action(area=right, domain="light", state="off")],
                "multi_off",
                area=left,
                area2=right,
            )
        )
    for a, b, c, state, tier in ROOM_TRIPLES:
        out.append(
            _case(
                f"{state}_{a}_{b}_{c}",
                "combo",
                tier,
                [
                    _action(area=a, domain="light", state=state),
                    _action(area=b, domain="light", state=state),
                    _action(area=c, domain="light", state=state),
                ],
                "multi_three_off" if state == "off" else "multi_three",
                area=a,
                area2=b,
                area3=c,
            )
        )
    for entity, area, fixture, tier in EXCEPT_FIXTURES:
        out.append(
            _case(
                f"except_{entity}",
                "combo",
                tier,
                _except_off_others(entity, area),
                "all_except" if fixture == "light" else "except_fixture",
                area=area,
                skip_fixture=fixture,
                forbid=[entity, area],
            )
        )
    for skip in EXCEPT_ON_ROOMS:
        conds = [_action(area=area, domain="light", state="on") for area in ALL_ROOMS if area != skip]
        out.append(
            _case(
                f"all_on_except_{skip}",
                "combo",
                "full",
                conds,
                "all_except_on",
                area=skip,
                forbid=[skip],
            )
        )
    out.append(
        _case(
            "except_in_kitchen_island",
            "combo",
            "full",
            [_action(entity="light.kitchen_ceiling", state="off")],
            "except_in_area",
            area="kitchen",
            skip_fixture="island",
            forbid=["light.kitchen_island"],
        )
    )
    out.append(
        _case(
            "except_in_living_lamp",
            "combo",
            "full",
            [_action(entity="light.living_ceiling", state="off")],
            "except_in_area",
            area="living",
            skip_fixture="lamp",
            forbid=["light.living_lamp"],
        )
    )
    out.append(
        _case(
            "except_island_on",
            "combo",
            "full",
            [_action(area=area, domain="light", state="on") for area in ALL_ROOMS if area != "kitchen"]
            + [_action(entity="light.kitchen_ceiling", state="on")],
            "except_fixture_on",
            area="kitchen",
            skip_fixture="island",
            forbid=["light.kitchen_island"],
        )
    )
    for left, right, tier in EXCEPT_TWO:
        conds = [_action(area=area, domain="light", state="off") for area in ALL_ROOMS if area not in {left, right}]
        out.append(
            _case(
                f"except_{left}_{right}",
                "combo",
                tier,
                conds,
                "except_two",
                area=left,
                area2=right,
                forbid=[left, right],
            )
        )
    for floor_id, skip, tier in (("ground", "kitchen", "full"), ("upper", "master_bedroom", "full"), ("ground", "office", "full")):
        rooms = FLOOR_ROOMS[floor_id]
        out.append(
            _case(
                f"floor_{floor_id}_except_{skip}",
                "combo",
                tier,
                [_action(area=area, domain="light", state="off") for area in rooms if area != skip],
                "floor_except",
                floor=floor_id,
                area=skip,
                forbid=[skip],
            )
        )
    out.append(
        _case(
            "covers_living_master_open",
            "combo",
            "full",
            [_action(area="living", domain="cover", state="open"), _action(area="master_bedroom", domain="cover", state="open")],
            "multi_covers",
            area="living",
            area2="master_bedroom",
        )
    )
    out.append(
        _case(
            "covers_living_master_close",
            "combo",
            "full",
            [_action(area="living", domain="cover", state="closed"), _action(area="master_bedroom", domain="cover", state="closed")],
            "multi_covers_close",
            area="living",
            area2="master_bedroom",
        )
    )
    out.append(
        _case(
            "climate_living_master_21",
            "combo",
            "full",
            [_action(area="living", domain="climate", temperature=21), _action(area="master_bedroom", domain="climate", temperature=21)],
            "multi_climate",
            area="living",
            area2="master_bedroom",
            n=21,
        )
    )
    out.append(
        _case(
            "bright_living_kitchen_50",
            "combo",
            "full",
            [_action(area="living", domain="light", brightness=50), _action(area="kitchen", domain="light", brightness=50)],
            "multi_bright",
            area="living",
            area2="kitchen",
            n=50,
        )
    )
    out.append(
        _case(
            "color_living_kitchen_red",
            "combo",
            "full",
            [_action(area="living", domain="light", color="red"), _action(area="kitchen", domain="light", color="red")],
            "multi_color",
            area="living",
            area2="kitchen",
            color="red",
        )
    )
    out.append(
        _case(
            "query_living_kitchen_lights",
            "combo",
            "full",
            [{"type": "query", "area": "living", "domain": "light"}, {"type": "query", "area": "kitchen", "domain": "light"}],
            "query_two",
            area="living",
            area2="kitchen",
            domain="light",
        )
    )
    out.append(
        _case(
            "temp_living_and_light_kitchen_off",
            "combo",
            "full",
            [{"type": "query", "entity_id": "climate.ground_thermostat"}, _action(area="kitchen", domain="light", state="off")],
            "query_and_off",
            area="living",
            area2="kitchen",
        )
    )
    out.append(
        _case(
            "locks_front_and_garage",
            "combo",
            "full",
            [_action(entity="lock.front_door", state="locked"), _action(entity="lock.garage_entry", state="locked")],
            "multi_locks",
            area="entryway",
            area2="garage",
        )
    )
    out.append(
        _case(
            "scene_and_powder_off",
            "combo",
            "full",
            [_action(entity="scene.movie_night", state="on"), _action(entity="light.powder_room_light", state="off")],
            "scene_and_off",
            scene="film",
            area="powder_room",
        )
    )
    return out
