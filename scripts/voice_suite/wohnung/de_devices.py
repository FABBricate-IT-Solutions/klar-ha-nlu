"""Wohnung DE: devices, queries, multi, clarify, follow-up."""

from pathlib import Path

from ..lib import (
    GROUP_FORBID,
    ROOMS,
    action_area,
    action_color,
    action_entity,
    action_temp,
    case,
    dump,
    query_area,
    query_entity,
)


def write(de: Path) -> None:
    dump(de / "devices" / "geraete.yaml", "\n".join(_devices()))
    dump(
        de / "devices" / "klima.yaml",
        case(
            "klima_schlaf_21",
            action_temp("climate.schlafzimmer_ac", 21),
            [
                "Klimaanlage auf 21 Grad",
                "Klima auf 21",
                "Stell die Klimaanlage auf einundzwanzig",
            ],
        ),
    )
    dump(de / "query_area" / "status.yaml", "\n".join(_area_queries()))
    dump(de / "query_devices" / "status.yaml", "\n".join(_device_queries()))
    dump(de / "multiple_intents" / "multi.yaml", "\n".join(_multi()))
    dump(de / "clarifications" / "lampen.yaml", _clarify())
    dump(de / "state_persistance" / "followup.yaml", _persist())


def _devices() -> list[str]:
    out = []
    for _area, name, _prep, eid in ROOMS:
        if not eid:
            continue
        out.append(
            case(
                f"device_{eid.split('.')[1]}_an",
                action_entity(eid, "on"),
                [f"Mach das {name} Licht an", f"Schalt {name} ein", f"{name} an"],
            )
        )
    out.extend(
        [
            case(
                "kugel_an",
                action_entity("light.schlafzimmer_kugel", "on"),
                ["Mach die Kugel an", "Kugel an", "Schalt die Kugel ein", "Mach mal die Kugel an"],
            ),
            case(
                "deckenlampe_aus",
                action_entity("light.schlafzimmer_decke", "off"),
                [
                    "Deckenlampe aus",
                    "Decknlampe aus",
                    "Mach die Deckenlampe aus",
                    "Schalt die Decke aus",
                    "Kannst du die Decke ausmachen",
                ],
            ),
            case(
                "r2d2_saugen",
                "    - {type: action, entity_id: vacuum.r2d2}",
                ["R2D2 soll saugen", "Staubsauger starten", "R2D2 saugen", "Saugroboter an"],
            ),
            case(
                "waschmaschine_an",
                action_entity("switch.badezimmer_waschmaschine", "on"),
                ["Waschmaschine an", "Mach die Waschmaschine an", "Schalt die Waschmaschine ein"],
            ),
            case(
                "spuelmaschine_aus",
                action_entity("switch.kuche_spulmaschine", "off"),
                ["Spülmaschine aus", "Mach die Spülmaschine aus", "Spuelmaschine aus"],
            ),
            case(
                "pc_an",
                action_entity("switch.pc_steckdose", "on"),
                ["PC an", "Mach den PC an", "Schalt die PC Steckdose ein"],
            ),
            case(
                "luefter_50",
                "    - type: action\n      entity_id: fan.arc_casual\n      attributes:\n        percentage: 50",
                ["Lüfter auf 50 Prozent", "Ventilator 50", "Setze den Lüfter auf 50"],
            ),
            case(
                "rollo_zu",
                action_entity("cover.wohnzimmer_rollo", "off"),
                ["Rollo zu", "Mach das Rollo im Wohnzimmer zu", "Schließ das Rollo"],
            ),
            case(
                "tuer_abschliessen",
                action_entity("lock.wohnungstuer", "on"),
                ["Wohnungstür abschließen", "Schließ die Tür ab", "Tür verriegeln"],
            ),
            case(
                "filmabend",
                "    - {type: action, entity_id: scene.filmabend}",
                ["Szene Filmabend", "Starte Filmabend", "Filmabend Szene an"],
            ),
        ]
    )
    return out


def _area_queries() -> list[str]:
    out = []
    for area, name, prep, _ in ROOMS:
        if area == "flur":
            continue
        out.append(
            case(
                f"status_licht_{area}",
                query_area(area, "light"),
                [
                    f"Wie ist der Status vom Licht {prep}",
                    f"Ist das Licht {prep} an",
                    f"Licht {name} Status",
                    f"Brennt das Licht {prep}",
                ],
            )
        )
    out.append(
        case(
            "temperatur_wohnung",
            "    - {type: query, area: wohnung, domain: climate}",
            [
                "Wie warm ist es in der Wohnung",
                "Temperatur Wohnung",
                "Wie ist die Temperatur zuhause",
            ],
        )
    )
    out.append(
        case(
            "temperatur_wohnzimmer",
            query_area("wohnzimmer", "climate"),
            [
                "Wie warm ist es im Wohnzimmer",
                "Temperatur Wohnzimmer",
                "Wie kalt ist es im Wohnzimmer",
            ],
        )
    )
    out.append(
        case(
            "status_alle_lichter",
            query_entity("light.alle_lichter"),
            [
                "Wie ist der Status von Alle Lichter",
                "Wie ist der Status der Leuchten",
                "Wie ist der Status von allen Lichtern",
            ],
            forbid=[
                "wohnzimmer",
                "esszimmer",
                "kuche",
                "light.wohnzimmer",
                "light.esszimmer",
                "light.wohn_und_esszimmer",
                "wohnung",
            ],
        )
    )
    out.append(
        case(
            "status_wohn_und_esszimmer",
            query_entity("light.wohn_und_esszimmer"),
            [
                "Wie ist der Status von Wohn und Esszimmer",
                "Status Wohn und Esszimmer",
            ],
            forbid=[
                "wohnzimmer",
                "esszimmer",
                "light.wohnzimmer",
                "light.esszimmer",
                "light.alle_lichter",
                "wohnung",
            ],
        )
    )
    return out


def _device_queries() -> list[str]:
    return [
        case(
            "status_waschmaschine",
            query_entity("switch.badezimmer_waschmaschine"),
            ["Ist die Waschmaschine an", "Status Waschmaschine", "Läuft die Waschmaschine"],
        ),
        case(
            "status_r2d2",
            query_entity("vacuum.r2d2"),
            ["Was macht R2D2", "Status Staubsauger", "Ist der Sauger an"],
        ),
    ]


def _multi() -> list[str]:
    return [
        case(
            "wz_und_kuche_licht",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "on"),
                    action_area("kuche", "light", "on"),
                ]
            ),
            [
                "Mach das Licht im Wohnzimmer und in der Küche an",
                "Licht Wohnzimmer und Küche an",
                "Schalt Wohnzimmer und Küche ein",
            ],
        ),
        case(
            "licht_und_heizung",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "on"),
                    "    - type: action\n      attributes:\n        temperature: 23",
                ]
            ),
            [
                "Mach das Licht im Wohnzimmer an und stell die Heizung auf 23",
                "Licht Wohnzimmer an und Heizung auf 23 Grad",
                "Wohnzimmer Licht an und Heizung dreiundzwanzig",
            ],
        ),
        case(
            "zwei_raeume_und_heizung",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "on"),
                    action_area("kuche", "light", "on"),
                    "    - type: action\n      attributes:\n        temperature: 23",
                ]
            ),
            [
                "Mach das Licht im Wohnzimmer und in der Küche an und stell die Heizung auf 23",
            ],
        ),
        case(
            "wohn_und_ess_aus",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("esszimmer", "light", "off"),
                ]
            ),
            [
                "Wohn und Esszimmer lichte aus",
                "Wohn und Esszimmer lichter aus",
                "Wohnzimmer und Esszimmer Lichter aus",
                "Mach Wohn und Esszimmer Licht aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "wohn_und_ess_rot",
            "\n".join(
                [
                    action_color("area", "wohnzimmer", "red"),
                    action_color("area", "esszimmer", "red"),
                ]
            ),
            [
                "Wohn und Esszimmer auf Rot",
                "Wohn und Esszimmer Licht auf rot",
            ],
            forbid=GROUP_FORBID,
            speech_has=["rot"],
            speech_forbids=["prozent", "?"],
        ),
        case(
            "alle_ausser_schlaf_und_kueche",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("esszimmer", "light", "off"),
                    action_area("arbeitszimmer", "light", "off"),
                ]
            ),
            ["Alle Lichter außer Schlafzimmer und Küche aus"],
            forbid=[
                "schlafzimmer",
                "kuche",
                "light.kuche_kuche",
                "light.alle_lichter",
                "light.wohn_und_esszimmer",
            ],
        ),
    ]


def _clarify() -> str:
    return """
- name: schlafzimmer_lampe_kugel
  conditions:
    - {type: action, entity_id: light.schlafzimmer_kugel, state: 'on'}
  sentences:
    - - Mach die Lampe im Schlafzimmer an
      - Kugel
    - - Schlafzimmer Lampe an
      - die Kugel
- name: schlafzimmer_lampe_decke
  conditions:
    - {type: action, entity_id: light.schlafzimmer_decke, state: 'on'}
  sentences:
    - - Mach die Lampe im Schlafzimmer an
      - Deckenlampe
    - - Schlafzimmer Lampe an
      - die Decke
"""


def _persist() -> str:
    return """
- name: wohnzimmer_dann_aus
  conditions:
    - {type: action, entity_id: light.wohnzimmer, state: 'off'}
  sentences:
    - - Licht im Wohnzimmer an
      - mach sie aus
    - - Mach das Licht im Wohnzimmer an
      - aus
- name: heizung_dann_grad
  conditions:
    - type: action
      entity_id: climate.better_thermostat_wohnzimmer
      attributes:
        temperature: 21
  sentences:
    - - Heizung Wohnzimmer auf 23 Grad
      - auf 21
    - - Stell die Heizung im Wohnzimmer auf 23
      - mach 21 Grad
"""
