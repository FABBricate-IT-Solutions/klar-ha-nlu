"""Wohnung DE spoken variants (STT slips, listings, follow-ups)."""

from pathlib import Path

from . import de_spoken_more
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


def write(de: Path) -> None:
    dump(de / "area" / "spoken.yaml", "\n".join(_area()))
    dump(de / "devices" / "spoken.yaml", "\n".join(_devices()))
    dump(de / "query_area" / "spoken.yaml", "\n".join(_queries()))
    dump(de / "query_devices" / "spoken.yaml", "\n".join(_device_queries()))
    dump(de / "multiple_intents" / "spoken.yaml", "\n".join(_multi()))
    dump(de / "state_persistance" / "spoken.yaml", _followups())
    de_spoken_more.write(de)

def _area() -> list[str]:
    return [
        case(
            "stt_licht_wohnzimmer_aus",
            action_area("wohnzimmer", "light", "off"),
            [
                "Dicht im Wohnzimmer aus",
                "Mach das Dicht im Wohnzimmer aus",
                "Lichte im Wohnzimmer aus",
                "Wohnzimmer Lichte aus",
                "Wohnzimmer Lampen aus",
                "Wohnzimmers Licht aus",
                "Mach mal das Licht im Wohnzimmer aus",
                "Bitte Licht Wohnzimmer aus",
                "Beleuchtung im Wohnzimmer aus",
                "Wohnzimer Licht aus",
                "Licht im Wohnzimer aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "stt_licht_wohnzimmer_an",
            action_area("wohnzimmer", "light", "on"),
            [
                "Dicht im Wohnzimmer an",
                "Lichte Wohnzimmer an",
                "Wohnzimmer Beleuchtung an",
                "Mach bitte das Licht im Wohnzimmer an",
                "Kannst du mal das Licht im Wohnzimmer anmachen",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "stt_kuche_lichte",
            action_area("kuche", "light", "off"),
            [
                "Dicht in der Küche aus",
                "Küche Lichte aus",
                "Kuche Licht aus",
                "Lampen in der Küche aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "stt_schlafzimmer_dativ",
            action_area("schlafzimmer", "light", "off"),
            [
                "Lichte im Schlafzimmer aus",
                "Dicht im Schlafzimmer aus",
                "Schlafzimer Lichte aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "alle_lampen_aus",
            action_entity("light.alle_lichter", "off"),
            [
                "Alle Lampen aus",
                "Überall Lichte aus",
                "Mach überall die Lichter aus",
                "Alle Dicht aus",
                "Überall Dicht aus",
                "Alle Beleuchtung aus",
                "Die ganzen Lichter aus",
                "Sämtliche Lichte aus",
                "Gesamte Beleuchtung aus",
            ],
        ),
        case(
            "alle_ganzen_an",
            action_entity("light.alle_lichter", "on"),
            [
                "Mach die ganzen Lichter an",
                "Die ganzen Lampen an",
                "Mach mal die ganzen Lichter an",
                "Bitte sämtliche Lichter an",
            ],
        ),
        case(
            "alle_ausser_kugel_stt",
            except_kugel("off"),
            [
                "Alle Lichte aus außer der Kugel",
                "Alle Dicht aus außer der Kugel",
                "Alle Lampen aus ohne die Kugel",
            ],
            forbid=KUGEL_FORBID,
        ),
        case(
            "alle_ausser_schlaf_stt",
            except_rooms("off"),
            [
                "Alle Lichte außer Schlafzimmer aus",
                "Alle Dicht außer Schlafzimmer aus",
                "Alle Lampen außer dem Schlafzimmer aus",
            ],
            forbid=ROOM_FORBID,
        ),
        case(
            "wohnzimmer_gruen",
            action_color("area", "wohnzimmer", "green"),
            [
                "Wohnzimmer Licht auf Grün",
                "Licht im Wohnzimmer auf grün",
            ],
            forbid=GROUP_FORBID,
            speech_forbids=["prozent", "?"],
        ),
        case(
            "esszimmer_weiss",
            action_color("area", "esszimmer", "white"),
            [
                "Esszimmer Licht auf Weiß",
                "Licht im Esszimmer auf weiss",
            ],
            forbid=GROUP_FORBID,
            speech_forbids=["prozent", "?"],
        ),
    ]

def _devices() -> list[str]:
    return [
        case(
            "kugel_aus",
            action_entity("light.schlafzimmer_kugel", "off"),
            ["Kugel aus", "Mach die Kugel aus", "Schalt die Kugel aus", "Die Kugel bitte aus"],
            forbid=["light.alle_lichter", "light.schlafzimmer_decke"],
        ),
        case(
            "deckenlampe_an",
            action_entity("light.schlafzimmer_decke", "on"),
            ["Deckenlampe an", "Mach die Decke an", "Schalt die Deckenlampe ein"],
            forbid=["light.schlafzimmer_kugel", "light.alle_lichter"],
        ),
        case(
            "tv_aus",
            action_entity("switch.schlafzimmer_tv", "off"),
            ["TV aus", "Schlafzimmer TV aus", "Mach den Fernseher aus", "Fernseher aus"],
            forbid=["light.alle_lichter", "scene.filmabend"],
        ),
        case(
            "trockner_an",
            action_entity("switch.kuche_trockner", "on"),
            ["Trockner an", "Mach den Trockner an", "Schalt den Trockner ein"],
        ),
        case(
            "luefter_aus",
            action_entity("fan.arc_casual", "off"),
            ["Lüfter aus", "Mach den Ventilator aus", "Ventilator aus"],
            forbid=["light.arbeitszimmer"],
        ),
        case(
            "rollo_auf",
            action_entity("cover.wohnzimmer_rollo", "on"),
            ["Rollo auf", "Mach das Rollo auf", "Öffne das Rollo im Wohnzimmer"],
        ),
    ]

def _queries() -> list[str]:
    return [
        case(
            "status_licht_wohnzimmer_casual",
            query_area("wohnzimmer", "light"),
            [
                "Ist das Licht im Wohnzimmer an",
                "Brennt das Licht im Wohnzimmer",
                "Wohnzimmer Licht Status",
            ],
        ),
        case(
            "status_licht_kuche_casual",
            query_area("kuche", "light"),
            ["Ist in der Küche Licht an", "Küche Licht Status", "Brennt in der Küche Licht"],
        ),
    ]

def _device_queries() -> list[str]:
    return [
        case(
            "status_kugel",
            query_entity("light.schlafzimmer_kugel"),
            ["Ist die Kugel an", "Status Kugel", "Wie ist der Status der Kugel"],
            forbid=["light.alle_lichter", "schlafzimmer"],
        ),
        case(
            "status_tv",
            query_entity("switch.schlafzimmer_tv"),
            ["Ist der Fernseher an", "Status TV", "Läuft der TV"],
        ),
        case(
            "status_trockner",
            query_entity("switch.kuche_trockner"),
            ["Ist der Trockner an", "Läuft der Trockner", "Status Trockner"],
        ),
        case(
            "status_pc",
            query_entity("switch.pc_steckdose"),
            ["Ist der PC an", "Status PC", "Läuft der PC"],
        ),
    ]

def _multi() -> list[str]:
    three = "\n".join(
        [
            action_area("wohnzimmer", "light", "off"),
            action_area("esszimmer", "light", "off"),
            action_area("kuche", "light", "off"),
        ]
    )
    return [
        case(
            "drei_raeume_aus",
            three,
            [
                "Licht Wohnzimmer Küche und Esszimmer aus",
                "Wohnzimmer, Küche und Esszimmer Licht aus",
                "Mach Wohnzimmer Esszimmer und Küche Licht aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "drei_raeume_an",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "on"),
                    action_area("esszimmer", "light", "on"),
                    action_area("kuche", "light", "on"),
                ]
            ),
            [
                "Licht Wohnzimmer Küche und Esszimmer an",
                "Wohnzimmer, Küche und Esszimmer Licht an",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "ess_und_wohn_aus",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("esszimmer", "light", "off"),
                ]
            ),
            [
                "Ess und Wohnzimmer lichte aus",
                "Esszimmer und Wohnzimmer Dicht aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "kugel_und_decke_aus",
            "\n".join(
                [
                    action_entity("light.schlafzimmer_kugel", "off"),
                    action_entity("light.schlafzimmer_decke", "off"),
                ]
            ),
            [
                "Kugel und Deckenlampe aus",
                "Mach Kugel und Decke aus",
                "Schalt die Kugel und die Deckenlampe aus",
            ],
            forbid=["light.alle_lichter"],
        ),
    ]

def _followups() -> str:
    return """
- name: wohnzimmer_dann_kueche
  conditions:
    - {type: action, area: kuche, domain: light, state: 'on'}
  sentences:
    - - Licht im Wohnzimmer an
      - und die Küche auch
    - - Mach das Licht im Wohnzimmer an
      - auch in der Küche
- name: kugel_dann_aus
  conditions:
    - {type: action, entity_id: light.schlafzimmer_kugel, state: 'off'}
  sentences:
    - - Kugel an
      - aus
    - - Mach die Kugel an
      - mach sie aus
- name: wohnzimmer_dann_rot
  conditions:
    - type: action
      area: wohnzimmer
      domain: light
      attributes:
        color: red
  speech_has:
    - rot
  speech_forbids:
    - prozent
    - '?'
  sentences:
    - - Licht im Wohnzimmer an
      - auf Rot
    - - Mach das Licht im Wohnzimmer an
      - mach es rot
"""

