"""Wohnung EN smoke, spoken mirrors, and extra pairs."""

from pathlib import Path

from . import en_spoken
from ..lib import (
    GROUP_FORBID,
    KUGEL_FORBID,
    ROOM_FORBID,
    action_area,
    action_color,
    action_entity,
    action_temp,
    case,
    dump,
    except_kugel,
    except_kugel_color,
    except_rooms,
    query_area,
    query_entity,
)


def write(en: Path) -> None:
    dump(en / "area" / "lights.yaml", "\n".join(_lights()))
    dump(
        en / "devices" / "climate.yaml",
        case(
            "living_heat_23",
            action_temp("climate.better_thermostat_wohnzimmer", 23),
            [
                "Set the living room heater to 23 degrees",
                "Living room thermostat 23",
                "Set heating in the living room to 23",
            ],
        ),
    )
    dump(en / "query_area" / "status.yaml", "\n".join(_queries()))
    dump(
        en / "multiple_intents" / "multi.yaml",
        case(
            "kitchen_and_living_on",
            "\n".join(
                [
                    action_area("kuche", "light", "on"),
                    action_area("wohnzimmer", "light", "on"),
                ]
            ),
            ["Turn on the kitchen and living room lights"],
        ),
    )
    dump(
        en / "multiple_intents" / "spoken.yaml",
        case(
            "living_and_dining_off",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("esszimmer", "light", "off"),
                ]
            ),
            [
                "Turn off the living and dining room lights",
                "Turn off living and dining lights",
            ],
            forbid=GROUP_FORBID,
        ),
    )
    dump(en / "state_persistance" / "followup.yaml", _persist())
    dump(en / "area" / "spoken_more.yaml", "\n".join(_spoken_area()))
    dump(en / "multiple_intents" / "listings.yaml", "\n".join(_listings()))
    dump(en / "area" / "spoken_pairs.yaml", "\n".join(_pairs_area()))
    dump(en / "multiple_intents" / "spoken_pairs.yaml", "\n".join(_pairs_multi()))
    en_spoken.write(en)

def _lights() -> list[str]:
    return [
        case(
            "kitchen_lights_on",
            action_area("kuche", "light", "on"),
            [
                "Turn on the kitchen lights",
                "Kitchen lights on",
                "Switch on the lights in the kitchen",
            ],
        ),
        case(
            "living_lights_off",
            action_area("wohnzimmer", "light", "off"),
            [
                "Turn off the living room lights",
                "Living room lights off",
                "Switch off the lights in the living room",
                "Turn off the livingrom lights",
            ],
        ),
        case(
            "all_lights_off",
            action_entity("light.alle_lichter", "off"),
            ["Turn off all lights", "All lights off", "Turn off every lamp", "Turn off the entire lighting"],
        ),
        case(
            "all_lights_on",
            action_entity("light.alle_lichter", "on"),
            [
                "Turn on all lights",
                "Turn on every light",
                "Turn on the whole lights",
                "Switch on the entire lighting",
            ],
        ),
        case(
            "bedroom_to_red",
            action_color("area", "schlafzimmer", "red"),
            [
                "Set the bedroom lights to red",
                "Bedroom lights to red",
                "Bedrooms light to red",
            ],
            forbid=GROUP_FORBID,
            speech_has=["rot"],
            speech_forbids=["percent", "?"],
        ),
        case(
            "bedroom_to_blue",
            action_color("area", "schlafzimmer", "blue"),
            [
                "Set the bedroom lights to blue",
                "Bedroom lights to blue",
            ],
            forbid=GROUP_FORBID,
            speech_has=["blau"],
            speech_forbids=["percent", "?"],
        ),
        case(
            "all_except_globe",
            except_kugel("off"),
            [
                "Turn off all lights except the globe",
                "All lights off except Kugel",
                "Turn off all lights but not the globe",
                "Turn off all lights except for the globe",
                "Turn off every light except the globe",
            ],
            forbid=KUGEL_FORBID,
        ),
        case(
            "all_except_bedroom",
            except_rooms("off"),
            [
                "Turn off all lights except the bedroom",
                "All lights off except the bedroom",
                "Turn off all lights but not the bedroom",
                "Turn off all lights except for the bedroom",
            ],
            forbid=ROOM_FORBID,
        ),
        case(
            "all_on_except_globe",
            except_kugel("on"),
            [
                "Turn on all lights except the globe",
                "All lights on but not the globe",
            ],
            forbid=KUGEL_FORBID,
        ),
        case(
            "all_red_except_globe",
            except_kugel_color("red"),
            [
                "Set all lights to red except the globe",
                "All lights to red except the globe",
            ],
            forbid=KUGEL_FORBID,
            speech_has=["rot"],
            speech_forbids=["percent", "?"],
        ),
    ]

def _queries() -> list[str]:
    return [
        case(
            "how_warm_home",
            "    - {type: query, area: wohnung, domain: climate}",
            ["How warm is it in the apartment", "What's the temperature at home"],
        ),
        case(
            "status_living_and_dining",
            query_entity("light.wohn_und_esszimmer"),
            [
                "Status of Wohn und Esszimmer",
                "What's the status of Wohn und Esszimmer",
            ],
            forbid=[
                "wohnzimmer",
                "esszimmer",
                "light.wohnzimmer",
                "light.esszimmer",
                "light.alle_lichter",
                "wohnung",
            ],
        ),
    ]

def _persist() -> str:
    return """
- name: living_then_off
  conditions:
    - {type: action, entity_id: light.wohnzimmer, state: 'off'}
  sentences:
    - - Turn on the living room lights
      - turn it off
"""

def _spoken_area() -> list[str]:
    return [
        case(
            "living_lamps_off",
            action_area("wohnzimmer", "light", "off"),
            [
                "Turn off the living room lamps",
                "Living room lighting off",
                "Please turn off the lights in the living room",
                "Turn off the livingrom lights",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "all_lamps_off",
            action_entity("light.alle_lichter", "off"),
            ["Turn off all lamps", "Switch off every light"],
        ),
        case(
            "all_except_bedroom_stt",
            except_rooms("off"),
            [
                "Turn off all lights except bedrooms",
            ],
            forbid=ROOM_FORBID,
        ),
    ]

def _listings() -> list[str]:
    return [
        case(
            "living_kitchen_dining_off",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("esszimmer", "light", "off"),
                    action_area("kuche", "light", "off"),
                ]
            ),
            [
                "Turn off the living room, kitchen and dining lights",
                "Turn off living kitchen and dining lights",
            ],
            forbid=GROUP_FORBID,
        ),
    ]

def _pairs_area() -> list[str]:
    return [
        case(
            "dining_lamps_on",
            action_area("esszimmer", "light", "on"),
            [
                "Turn on the dining room lamps",
                "Dining room lights on",
                "Please turn on the lights in the dining room",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "office_lights_off",
            action_area("arbeitszimmer", "light", "off"),
            [
                "Turn off the office lights",
                "Office lighting off",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "kitchen_to_blue",
            action_color("area", "kuche", "blue"),
            [
                "Set the kitchen lights to blue",
                "Kitchen lights to blue",
            ],
            forbid=GROUP_FORBID,
            speech_has=["blau"],
            speech_forbids=["percent", "?"],
        ),
    ]

def _pairs_multi() -> list[str]:
    return [
        case(
            "kitchen_and_dining_on",
            "\n".join(
                [
                    action_area("kuche", "light", "on"),
                    action_area("esszimmer", "light", "on"),
                ]
            ),
            [
                "Turn on the kitchen and dining room lights",
                "Turn on kitchen and dining lights",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "living_and_office_off",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("arbeitszimmer", "light", "off"),
                ]
            ),
            [
                "Turn off the living room and office lights",
                "Turn off living and office lights",
            ],
            forbid=GROUP_FORBID,
        ),
    ]

