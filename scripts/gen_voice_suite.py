#!/usr/bin/env python3
"""Generate YAML voice tests for Klar — German first, English smoke."""

from pathlib import Path
import textwrap

ROOT = Path(__file__).resolve().parents[1] / "tests" / "datasets"


def dump(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(body).lstrip("\n"), encoding="utf-8")


HOME = """
name: Wohnung Mittel
language: de
difficulty: medium
areas:
  - id: wohnzimmer
    name: Wohnzimmer
    aliases: [wohnraum, living, livingroom]
  - id: esszimmer
    name: Esszimmer
    aliases: [ess, dining]
  - id: schlafzimmer
    name: Schlafzimmer
    aliases: [schlaf, bedroom]
  - id: kuche
    name: Küche
    aliases: [kueche, kuche, kitchen]
  - id: badezimmer
    name: Badezimmer
    aliases: [bad, bathroom]
  - id: arbeitszimmer
    name: Arbeitszimmer
    aliases: [buero, office]
  - id: flur
    name: Flur
    aliases: [diele, hallway]
  - id: balkon
    name: Balkon
    aliases: [terrasse, balcony]
  - id: wohnung
    name: Wohnung
    aliases: [haus, zuhause, home, apartment]
devices:
  - {id: light.wohnzimmer, area_id: wohnzimmer, name: Wohnzimmer Licht}
  - {id: light.esszimmer, area_id: esszimmer, name: Esszimmer Licht}
  - {id: light.kuche_kuche, area_id: kuche, name: Küche Licht}
  - {id: light.arbeitszimmer, area_id: arbeitszimmer, name: Arbeitszimmer}
  - {id: light.schlafzimmer_kugel, area_id: schlafzimmer, name: Kugel}
  - {id: light.schlafzimmer_decke, area_id: schlafzimmer, name: Deckenlampe}
  - {id: light.schlafzimmer_licht, area_id: schlafzimmer, name: Schlafzimmer Licht}
  - {id: light.alle_lichter, area_id: wohnung, name: Alle Lichter}
  - {id: climate.better_thermostat_wohnzimmer, area_id: wohnzimmer, name: Heizung Wohnzimmer}
  - {id: climate.better_thermostat_esszimmer, area_id: esszimmer, name: Heizung Esszimmer}
  - {id: climate.better_thermostat_schlafzimmer, area_id: schlafzimmer, name: Heizung Schlafzimmer}
  - {id: climate.better_thermostat_badezimmer, area_id: badezimmer, name: Heizung Bad}
  - {id: climate.schlafzimmer_ac, area_id: schlafzimmer, name: Klimaanlage}
  - {id: vacuum.r2d2, area_id: wohnzimmer, name: R2D2}
  - {id: switch.pc_steckdose, area_id: arbeitszimmer, name: PC Steckdose}
  - {id: switch.schlafzimmer_tv, area_id: schlafzimmer, name: Schlafzimmer TV}
  - {id: switch.badezimmer_waschmaschine, area_id: badezimmer, name: Waschmaschine}
  - {id: switch.kuche_trockner, area_id: kuche, name: Trockner}
  - {id: switch.kuche_spulmaschine, area_id: kuche, name: Spülmaschine}
  - {id: fan.arc_casual, area_id: arbeitszimmer, name: Lüfter}
  - {id: cover.wohnzimmer_rollo, area_id: wohnzimmer, name: Rollo Wohnzimmer}
  - {id: lock.wohnungstuer, area_id: flur, name: Wohnungstür}
  - {id: scene.filmabend, area_id: wohnzimmer, name: Filmabend}
"""


def case(name: str, conditions: str, sentences: list[str]) -> str:
    lines = [f"- name: {name}", "  conditions:", conditions, "  sentences:"]
    for s in sentences:
        lines.append(f"    - {s!r}")
    return "\n".join(lines) + "\n"


def action_entity(eid: str, state: str) -> str:
    return f"    - {{type: action, entity_id: {eid}, state: {state!r}}}"


def action_area(area: str, domain: str, state: str) -> str:
    return f"    - {{type: action, area: {area}, domain: {domain}, state: {state!r}}}"


def action_temp(eid: str, temp: int) -> str:
    return (
        f"    - type: action\n"
        f"      entity_id: {eid}\n"
        f"      attributes:\n"
        f"        temperature: {temp}"
    )


def action_bri(target_key: str, target_val: str, bri: int) -> str:
    return (
        f"    - type: action\n"
        f"      {target_key}: {target_val}\n"
        f"      domain: light\n"
        f"      attributes:\n"
        f"        brightness: {bri}"
    )


def query_area(area: str, domain: str) -> str:
    return f"    - {{type: query, area: {area}, domain: {domain}}}"


def query_entity(eid: str) -> str:
    return f"    - {{type: query, entity_id: {eid}}}"


def main() -> None:
    de = ROOT / "wohnung_mittel"
    en = ROOT / "wohnung_en"
    dump(de / "home_config.yaml", HOME)
    dump(en / "home_config.yaml", HOME.replace("language: de", "language: en"))

    rooms = [
        ("wohnzimmer", "Wohnzimmer", "im Wohnzimmer", "light.wohnzimmer"),
        ("esszimmer", "Esszimmer", "im Esszimmer", "light.esszimmer"),
        ("kuche", "Küche", "in der Küche", "light.kuche_kuche"),
        ("arbeitszimmer", "Arbeitszimmer", "im Arbeitszimmer", "light.arbeitszimmer"),
        ("flur", "Flur", "im Flur", None),
        ("badezimmer", "Bad", "im Bad", None),
    ]

    area_licht = []
    for area, name, prep, _eid in rooms:
        if area in ("flur", "badezimmer"):
            continue
        area_licht.append(
            case(
                f"{area}_lichter_an",
                action_area(area, "light", "on"),
                [
                    f"Mach das Licht {prep} an",
                    f"Licht {name} an",
                    f"Schalt die Lichter {prep} ein",
                    f"Kannst du das Licht {prep} anmachen",
                    f"{name} Licht an",
                ],
            )
        )
        area_licht.append(
            case(
                f"{area}_lichter_aus",
                action_area(area, "light", "off"),
                [
                    f"Mach das Licht {prep} aus",
                    f"Licht {name} aus",
                    f"Schalt die Lichter {prep} aus",
                    f"{name} Licht aus",
                ],
            )
        )
    area_licht.append(
        case(
            "alle_lichter_aus",
            action_entity("light.alle_lichter", "off"),
            [
                "Alle Lichter aus",
                "Mach überall das Licht aus",
                "Wohnung Licht aus",
                "Schalt alle Lampen aus",
            ],
        )
    )
    area_licht.append(
        case(
            "wohnzimmer_helligkeit",
            action_bri("area", "wohnzimmer", 45),
            [
                "Setze das Licht im Wohnzimmer auf 45 Prozent",
                "Wohnzimmer Helligkeit 45",
                "Dimme das Wohnzimmer auf 45",
                "Licht Wohnzimmer 45 Prozent",
            ],
        )
    )
    dump(de / "area" / "lichter.yaml", "\n".join(area_licht))

    heizung = []
    climates = [
        ("wohnzimmer", "Wohnzimmer", "im Wohnzimmer", "climate.better_thermostat_wohnzimmer"),
        ("esszimmer", "Esszimmer", "im Esszimmer", "climate.better_thermostat_esszimmer"),
        ("schlafzimmer", "Schlafzimmer", "im Schlafzimmer", "climate.better_thermostat_schlafzimmer"),
        ("badezimmer", "Bad", "im Bad", "climate.better_thermostat_badezimmer"),
    ]
    for area, name, prep, eid in climates:
        heizung.append(
            case(
                f"heizung_{area}_23",
                action_temp(eid, 23),
                [
                    f"Heizung {name} auf 23 Grad",
                    f"Stell die Heizung {prep} auf 23",
                    f"Temperatur {name} 23",
                    f"Heizung {name} auf dreiundzwanzig Grad",
                ],
            )
        )
    dump(de / "area" / "heizung.yaml", "\n".join(heizung))

    devices = []
    for area, name, prep, eid in rooms:
        if not eid:
            continue
        devices.append(
            case(
                f"device_{eid.split('.')[1]}_an",
                action_entity(eid, "on"),
                [
                    f"Mach das {name} Licht an",
                    f"Schalt {name} ein",
                    f"{name} an",
                ],
            )
        )
    devices.append(
        case(
            "kugel_an",
            action_entity("light.schlafzimmer_kugel", "on"),
            ["Mach die Kugel an", "Kugel an", "Schalt die Kugel ein"],
        )
    )
    devices.append(
        case(
            "deckenlampe_aus",
            action_entity("light.schlafzimmer_decke", "off"),
            ["Deckenlampe aus", "Mach die Deckenlampe aus", "Schalt die Decke aus"],
        )
    )
    devices.append(
        case(
            "r2d2_saugen",
            "    - {type: action, entity_id: vacuum.r2d2}",
            ["R2D2 soll saugen", "Staubsauger starten", "R2D2 saugen", "Saugroboter an"],
        )
    )
    devices.append(
        case(
            "waschmaschine_an",
            action_entity("switch.badezimmer_waschmaschine", "on"),
            ["Waschmaschine an", "Mach die Waschmaschine an", "Schalt die Waschmaschine ein"],
        )
    )
    devices.append(
        case(
            "spuelmaschine_aus",
            action_entity("switch.kuche_spulmaschine", "off"),
            ["Spülmaschine aus", "Mach die Spülmaschine aus", "Spuelmaschine aus"],
        )
    )
    devices.append(
        case(
            "pc_an",
            action_entity("switch.pc_steckdose", "on"),
            ["PC an", "Mach den PC an", "Schalt die PC Steckdose ein"],
        )
    )
    devices.append(
        case(
            "luefter_50",
            "    - type: action\n      entity_id: fan.arc_casual\n      attributes:\n        percentage: 50",
            ["Lüfter auf 50 Prozent", "Ventilator 50", "Setze den Lüfter auf 50"],
        )
    )
    devices.append(
        case(
            "rollo_zu",
            action_entity("cover.wohnzimmer_rollo", "off"),
            ["Rollo zu", "Mach das Rollo im Wohnzimmer zu", "Schließ das Rollo"],
        )
    )
    devices.append(
        case(
            "tuer_abschliessen",
            action_entity("lock.wohnungstuer", "on"),
            ["Wohnungstür abschließen", "Schließ die Tür ab", "Tür verriegeln"],
        )
    )
    devices.append(
        case(
            "filmabend",
            "    - {type: action, entity_id: scene.filmabend}",
            ["Szene Filmabend", "Starte Filmabend", "Filmabend Szene an"],
        )
    )
    dump(de / "devices" / "geraete.yaml", "\n".join(devices))

    klima_dev = []
    klima_dev.append(
        case(
            "klima_schlaf_21",
            action_temp("climate.schlafzimmer_ac", 21),
            [
                "Klimaanlage auf 21 Grad",
                "Klima auf 21",
                "Stell die Klimaanlage auf einundzwanzig",
            ],
        )
    )
    dump(de / "devices" / "klima.yaml", "\n".join(klima_dev))

    q_area = []
    for area, name, prep, _ in rooms:
        if area in ("flur",):
            continue
        q_area.append(
            case(
                f"status_licht_{area}",
                query_area(area, "light"),
                [
                    f"Wie ist der Status vom Licht {prep}",
                    f"Ist das Licht {prep} an",
                    f"Licht {name} Status",
                ],
            )
        )
    q_area.append(
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
    q_area.append(
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
    dump(de / "query_area" / "status.yaml", "\n".join(q_area))

    q_dev = []
    q_dev.append(
        case(
            "status_waschmaschine",
            query_entity("switch.badezimmer_waschmaschine"),
            ["Ist die Waschmaschine an", "Status Waschmaschine", "Läuft die Waschmaschine"],
        )
    )
    q_dev.append(
        case(
            "status_r2d2",
            query_entity("vacuum.r2d2"),
            ["Was macht R2D2", "Status Staubsauger", "Ist der Sauger an"],
        )
    )
    dump(de / "query_devices" / "status.yaml", "\n".join(q_dev))

    multi = []
    multi.append(
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
        )
    )
    multi.append(
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
        )
    )
    multi.append(
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
        )
    )
    dump(de / "multiple_intents" / "multi.yaml", "\n".join(multi))

    clar = """
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
    dump(de / "clarifications" / "lampen.yaml", clar)

    persist = """
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
    dump(de / "state_persistance" / "followup.yaml", persist)

    # English smoke — same home, English sentences only
    dump(
        en / "area" / "lights.yaml",
        "\n".join(
            [
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
                    ],
                ),
                case(
                    "all_lights_off",
                    action_entity("light.alle_lichter", "off"),
                    ["Turn off all lights", "All lights off"],
                ),
            ]
        ),
    )
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
    dump(
        en / "query_area" / "status.yaml",
        case(
            "how_warm_home",
            "    - {type: query, area: wohnung, domain: climate}",
            ["How warm is it in the apartment", "What's the temperature at home"],
        ),
    )
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
        en / "state_persistance" / "followup.yaml",
        """
- name: living_then_off
  conditions:
    - {type: action, entity_id: light.wohnzimmer, state: 'off'}
  sentences:
    - - Turn on the living room lights
      - turn it off
""",
    )

    print(f"wrote {de} and {en}")


if __name__ == "__main__":
    main()
