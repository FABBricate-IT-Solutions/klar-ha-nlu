"""English mirrors of the German spoken families."""

from pathlib import Path

from ..lib import (
    GROUP_FORBID,
    KUGEL_FORBID,
    ROOM_FORBID,
    action_area,
    action_color,
    action_entity,
    case,
    dump,
    except_kugel,
    except_rooms,
    query_area,
    query_entity,
)


def write(en: Path) -> None:
    dump(en / "area" / "spoken_synonyms.yaml", "\n".join(_syn_area()))
    dump(en / "devices" / "spoken.yaml", "\n".join(_syn_devices()))
    dump(en / "query_area" / "spoken.yaml", "\n".join(_syn_queries()))
    dump(en / "query_devices" / "spoken.yaml", "\n".join(_syn_device_queries()))
    dump(en / "state_persistance" / "spoken.yaml", _syn_followups())

def _syn_area() -> list[str]:
    return [
        case(
            "all_synonyms_off",
            action_entity("light.alle_lichter", "off"),
            [
                "Turn off all lamps",
                "Switch off every light",
                "Turn off the entire lighting",
                "Turn off the whole lights",
                "Turn off every lamp",
            ],
        ),
        case(
            "all_synonyms_on",
            action_entity("light.alle_lichter", "on"),
            [
                "Turn on all lights",
                "Turn on every light",
                "Turn on the entire lighting",
                "Turn on the whole lights",
                "Switch on every lamp",
            ],
        ),
        case(
            "all_except_globe_synonyms",
            except_kugel("off"),
            [
                "Turn off every light except the globe",
                "Turn off the entire lighting except the globe",
                "Turn off the whole lights except the globe",
            ],
            forbid=KUGEL_FORBID,
        ),
        case(
            "all_except_bedroom_synonyms",
            except_rooms("off"),
            [
                "Turn off every light except the bedroom",
                "Turn off the entire lighting except the bedroom",
                "Turn off all lights not in the bedroom",
            ],
            forbid=ROOM_FORBID,
        ),
        case(
            "living_green",
            action_color("area", "wohnzimmer", "green"),
            [
                "Set the living room lights to green",
                "Living room lights to green",
            ],
            forbid=GROUP_FORBID,
            speech_forbids=["percent", "?"],
        ),
        case(
            "dining_white",
            action_color("area", "esszimmer", "white"),
            [
                "Set the dining room lights to white",
                "Dining room lights to white",
            ],
            forbid=GROUP_FORBID,
            speech_forbids=["percent", "?"],
        ),
    ]

def _syn_devices() -> list[str]:
    return [
        case(
            "globe_off",
            action_entity("light.schlafzimmer_kugel", "off"),
            ["Turn off the globe", "Globe off", "Switch off the globe"],
            forbid=["light.alle_lichter", "light.schlafzimmer_decke"],
        ),
        case(
            "ceiling_on",
            action_entity("light.schlafzimmer_decke", "on"),
            ["Turn on the ceiling light", "Ceiling light on"],
            forbid=["light.schlafzimmer_kugel", "light.alle_lichter"],
        ),
        case(
            "tv_off",
            action_entity("switch.schlafzimmer_tv", "off"),
            ["Turn off the TV", "Bedroom TV off", "Turn off the bedroom TV"],
            forbid=["light.alle_lichter", "scene.filmabend"],
        ),
        case(
            "dryer_on",
            action_entity("switch.kuche_trockner", "on"),
            ["Turn on the dryer", "Dryer on"],
        ),
        case(
            "fan_off",
            action_entity("fan.arc_casual", "off"),
            ["Turn off the fan", "Fan off"],
            forbid=["light.arbeitszimmer"],
        ),
        case(
            "blinds_open",
            action_entity("cover.wohnzimmer_rollo", "on"),
            ["Open the blinds", "Open the living room blinds"],
        ),
        case(
            "pc_on",
            action_entity("switch.pc_steckdose", "on"),
            ["Turn on the PC", "PC on"],
        ),
        case(
            "globe_and_ceiling_off",
            "\n".join(
                [
                    action_entity("light.schlafzimmer_kugel", "off"),
                    action_entity("light.schlafzimmer_decke", "off"),
                ]
            ),
            ["Turn off the globe and the ceiling light", "Globe and ceiling off"],
            forbid=["light.alle_lichter"],
        ),
    ]

def _syn_queries() -> list[str]:
    return [
        case(
            "status_living_light",
            query_area("wohnzimmer", "light"),
            [
                "Is the living room light on",
                "What's the status of the living room lights",
                "Living room light status",
            ],
        ),
        case(
            "status_kitchen_light",
            query_area("kuche", "light"),
            ["Is the kitchen light on", "Kitchen light status"],
        ),
    ]

def _syn_device_queries() -> list[str]:
    return [
        case(
            "status_globe",
            query_entity("light.schlafzimmer_kugel"),
            ["Is the globe on", "Globe status", "What's the status of the globe"],
            forbid=["light.alle_lichter", "schlafzimmer"],
        ),
        case(
            "status_tv",
            query_entity("switch.schlafzimmer_tv"),
            ["Is the TV on", "Bedroom TV status"],
        ),
        case(
            "status_pc",
            query_entity("switch.pc_steckdose"),
            ["Is the PC on", "PC status"],
        ),
        case(
            "status_dryer",
            query_entity("switch.kuche_trockner"),
            ["Is the dryer on", "Dryer status"],
        ),
    ]

def _syn_followups() -> str:
    return """
- name: living_then_kitchen
  conditions:
    - {type: action, area: kuche, domain: light, state: 'on'}
  sentences:
    - - Turn on the living room lights
      - and the kitchen too
    - - Turn on the living room lights
      - the kitchen as well
- name: globe_then_off
  conditions:
    - {type: action, entity_id: light.schlafzimmer_kugel, state: 'off'}
  sentences:
    - - Turn on the globe
      - turn it off
    - - Globe on
      - off
- name: living_then_red
  conditions:
    - type: action
      area: wohnzimmer
      domain: light
      attributes:
        color: red
  speech_has:
    - rot
  speech_forbids:
    - percent
    - '?'
  sentences:
    - - Turn on the living room lights
      - to red
    - - Turn on the living room lights
      - make it red
"""

