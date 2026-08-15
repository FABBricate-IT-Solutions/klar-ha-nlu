"""Wohnung DE: room lights, except, heat."""

from pathlib import Path

from ..lib import (
    GROUP_FORBID,
    KUGEL_FORBID,
    ROOM_FORBID,
    ROOMS,
    action_area,
    action_bri,
    action_color,
    action_entity,
    action_temp,
    case,
    dump,
    except_kugel,
    except_kugel_color,
    except_rooms,
    except_rooms_bri,
)


def write(de: Path) -> None:
    dump(de / "area" / "lichter.yaml", "\n".join(_lights()))
    dump(de / "area" / "heizung.yaml", "\n".join(_heat()))


def _lights() -> list[str]:
    out = []
    for area, name, prep, _eid in ROOMS:
        if area in ("flur", "badezimmer"):
            continue
        out.append(
            case(
                f"{area}_lichter_an",
                action_area(area, "light", "on"),
                [
                    f"Mach das Licht {prep} an",
                    f"Licht {name} an",
                    f"Schalt die Lichter {prep} ein",
                    f"Kannst du das Licht {prep} anmachen",
                    f"{name} Licht an",
                    f"{name} Lichte an",
                    f"Dicht {prep} an",
                    f"Lampen {prep} an",
                    f"Bitte Licht {name} an",
                ],
                forbid=GROUP_FORBID,
            )
        )
        out.append(
            case(
                f"{area}_lichter_aus",
                action_area(area, "light", "off"),
                [
                    f"Mach das Licht {prep} aus",
                    f"Licht {name} aus",
                    f"Schalt die Lichter {prep} aus",
                    f"{name} Licht aus",
                    f"{name} Lichte aus",
                    f"{name} Lichter aus",
                    f"Dicht {prep} aus",
                    f"Lampen {prep} aus",
                    f"Beleuchtung {prep} aus",
                ],
                forbid=GROUP_FORBID,
            )
        )
    out.append(
        case(
            "alle_lichter_aus",
            action_entity("light.alle_lichter", "off"),
            [
                "Alle Lichter aus",
                "Mach überall das Licht aus",
                "Schalt alle Lampen aus",
                "Alle Lichte aus",
                "Mach die ganzen Lichter aus",
                "Sämtliche Lichter aus",
                "Mach alles Licht aus",
            ],
        )
    )
    out.append(
        case(
            "alle_lichter_an",
            action_entity("light.alle_lichter", "on"),
            [
                "Alle Lichter an",
                "Mach die ganzen Lichter an",
                "Mach überall das Licht an",
                "Sämtliche Lampen an",
                "Komplette Beleuchtung an",
                "Schalt jedes Licht an",
                "Mach das ganze Licht an",
            ],
        )
    )
    out.append(
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
    out.append(
        case(
            "schlafzimmer_auf_rot",
            action_color("area", "schlafzimmer", "red"),
            [
                "Schlafzimmern Licht auf Rot",
                "Schlafzimmern auf Rot",
                "Schlafzimmer Licht auf Rot",
                "Licht im Schlafzimmer auf Rot",
            ],
            forbid=GROUP_FORBID,
            speech_has=["rot"],
            speech_forbids=["prozent", "?"],
        )
    )
    out.append(
        case(
            "schlafzimmer_auf_blau",
            action_color("area", "schlafzimmer", "blue"),
            [
                "Schlafzimmer Licht auf Blau",
                "Licht im Schlafzimmer auf Blau",
            ],
            forbid=GROUP_FORBID,
            speech_has=["blau"],
            speech_forbids=["prozent", "?"],
        )
    )
    out.append(
        case(
            "alle_ausser_kugel",
            except_kugel("off"),
            [
                "Alle Lichter aus außer der Kugel",
                "Alle lichter aus ausser der Kugel",
                "Alle Lichter ohne die Kugel aus",
                "Alle Lichter aus aber nicht die Kugel",
                "Alle Lichter aus bis auf die Kugel",
                "Die ganzen Lichter aus außer der Kugel",
            ],
            forbid=KUGEL_FORBID,
        )
    )
    out.append(
        case(
            "alle_ausser_schlafzimmer",
            except_rooms("off"),
            [
                "Alle Lichter außer Schlafzimmer ausschalten",
                "Alle Lichter ausser Schlafzimmer aus",
                "Alle Lichter aus außer dem Schlafzimmer",
                "Alle Lichter außer Schlafzimmern aus",
                "Alle Lichter aus nicht im Schlafzimmer",
                "Alle Lichter nicht in dem Schlafzimmer aus",
            ],
            forbid=ROOM_FORBID,
        )
    )
    out.append(
        case(
            "alle_an_ausser_kugel",
            except_kugel("on"),
            [
                "Alle Lichter an außer der Kugel",
                "Alle Lichter an ohne die Kugel",
            ],
            forbid=KUGEL_FORBID,
        )
    )
    out.append(
        case(
            "alle_an_ausser_schlafzimmer",
            except_rooms("on"),
            [
                "Alle Lichter an außer Schlafzimmer",
                "Alle Lichter an nicht im Schlafzimmer",
            ],
            forbid=ROOM_FORBID,
        )
    )
    out.append(
        case(
            "alle_auf_rot_ausser_kugel",
            except_kugel_color("red"),
            [
                "Alle Lichter auf Rot außer der Kugel",
                "Alle Lichter auf rot ausser der Kugel",
            ],
            forbid=KUGEL_FORBID,
            speech_has=["rot"],
            speech_forbids=["prozent", "?"],
        )
    )
    out.append(
        case(
            "alle_auf_20_ausser_schlafzimmer",
            except_rooms_bri(20),
            [
                "Alle Lichter auf 20 Prozent außer Schlafzimmer",
                "Alle Lichter auf 20 außer dem Schlafzimmer",
            ],
            forbid=ROOM_FORBID,
        )
    )
    return out


def _heat() -> list[str]:
    climates = [
        ("wohnzimmer", "Wohnzimmer", "im Wohnzimmer", "climate.better_thermostat_wohnzimmer"),
        ("esszimmer", "Esszimmer", "im Esszimmer", "climate.better_thermostat_esszimmer"),
        ("schlafzimmer", "Schlafzimmer", "im Schlafzimmer", "climate.better_thermostat_schlafzimmer"),
        ("badezimmer", "Bad", "im Bad", "climate.better_thermostat_badezimmer"),
    ]
    return [
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
        for area, name, prep, eid in climates
    ]
