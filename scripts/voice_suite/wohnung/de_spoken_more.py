"""Extra Wohnung DE spoken pairs, colors, brightness, except, queries."""

from pathlib import Path

from ..lib import (
    GROUP_FORBID,
    ROOM_FORBID,
    action_area,
    action_bri,
    action_color,
    action_entity,
    case,
    dump,
    except_rooms,
    query_area,
    query_entity,
)


def write(de: Path) -> None:
    dump(de / "area" / "spoken_more.yaml", "\n".join(_more_area()))
    dump(de / "devices" / "spoken_more.yaml", "\n".join(_more_devices()))
    dump(de / "query_area" / "spoken_more.yaml", "\n".join(_more_queries()))
    dump(de / "query_devices" / "spoken_more.yaml", "\n".join(_more_device_queries()))
    dump(de / "multiple_intents" / "spoken_more.yaml", "\n".join(_more_multi()))

def _more_area() -> list[str]:
    return [
        case(
            "esszimmer_lichte_an",
            action_area("esszimmer", "light", "on"),
            [
                "Esszimmer Lichte an",
                "Dicht im Esszimmer an",
                "Lampen im Esszimmer an",
                "Mach bitte das Licht im Esszimmer an",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "arbeitszimmer_lampen_aus",
            action_area("arbeitszimmer", "light", "off"),
            [
                "Arbeitszimmer Lampen aus",
                "Dicht im Arbeitszimmer aus",
                "Lichte im Büro aus",
                "Bitte Licht im Arbeitszimmer aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "wohnzimmer_gelb",
            action_color("area", "wohnzimmer", "yellow"),
            [
                "Wohnzimmer Licht auf Gelb",
                "Licht im Wohnzimmer auf gelb",
            ],
            forbid=GROUP_FORBID,
            speech_forbids=["prozent", "?"],
        ),
        case(
            "kuche_blau",
            action_color("area", "kuche", "blue"),
            [
                "Küche Licht auf Blau",
                "Licht in der Küche auf blau",
            ],
            forbid=GROUP_FORBID,
            speech_has=["blau"],
            speech_forbids=["prozent", "?"],
        ),
        case(
            "esszimmer_30",
            action_bri("area", "esszimmer", 30),
            [
                "Esszimmer Helligkeit 30",
                "Licht im Esszimmer auf 30 Prozent",
                "Dimme das Esszimmer auf 30",
            ],
        ),
        case(
            "kuche_80",
            action_bri("area", "kuche", 80),
            [
                "Küche auf 80 Prozent",
                "Licht in der Küche auf 80",
            ],
        ),
        case(
            "alle_ausser_arbeitszimmer",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("esszimmer", "light", "off"),
                    action_area("kuche", "light", "off"),
                ]
            ),
            [
                "Alle Lichter außer Arbeitszimmer aus",
                "Alle Lichte außer dem Arbeitszimmer aus",
                "Alle Lampen aus außer dem Büro",
            ],
            forbid=["arbeitszimmer", "light.arbeitszimmer", "light.alle_lichter", "light.wohn_und_esszimmer"],
        ),
        case(
            "ueberall_ausser_schlaf_stt",
            except_rooms("off"),
            [
                "Überall die Lichter aus außer dem Schlafzimmer",
                "Alle Lampen außer Schlafzimmer aus",
                "Alle Dicht außer Schlafzimmer aus",
            ],
            forbid=ROOM_FORBID,
        ),
    ]

def _more_devices() -> list[str]:
    return [
        case(
            "kugel_casual",
            action_entity("light.schlafzimmer_kugel", "on"),
            [
                "Bitte die Kugel an",
                "Kannst du die Kugel anmachen",
                "Mach mal die Kugel an",
            ],
            forbid=["light.alle_lichter", "light.schlafzimmer_decke"],
        ),
        case(
            "decke_aus_casual",
            action_entity("light.schlafzimmer_decke", "off"),
            [
                "Mach mal die Decke aus",
                "Bitte die Deckenlampe aus",
                "Kannst du die Deckenlampe ausmachen",
            ],
            forbid=["light.schlafzimmer_kugel", "light.alle_lichter"],
        ),
        case(
            "spuelmaschine_an",
            action_entity("switch.kuche_spulmaschine", "on"),
            ["Spülmaschine an", "Mach die Spülmaschine an", "Schalt die Spülmaschine ein"],
        ),
        case(
            "waschmaschine_aus",
            action_entity("switch.badezimmer_waschmaschine", "off"),
            ["Waschmaschine aus", "Mach die Waschmaschine aus"],
        ),
        case(
            "pc_aus",
            action_entity("switch.pc_steckdose", "off"),
            ["PC aus", "Mach den PC aus", "Schalt den PC aus"],
        ),
    ]

def _more_queries() -> list[str]:
    return [
        case(
            "brennt_esszimmer",
            query_area("esszimmer", "light"),
            [
                "Brennt das Licht im Esszimmer",
                "Ist im Esszimmer Licht an",
                "Esszimmer Licht Status",
            ],
        ),
        case(
            "brennt_arbeitszimmer",
            query_area("arbeitszimmer", "light"),
            [
                "Brennt das Licht im Arbeitszimmer",
                "Ist das Licht im Büro an",
            ],
        ),
        case(
            "status_licht_schlaf_casual",
            query_area("schlafzimmer", "light"),
            [
                "Brennt das Licht im Schlafzimmer",
                "Schlafzimmer Licht Status",
            ],
        ),
    ]

def _more_device_queries() -> list[str]:
    return [
        case(
            "status_decke",
            query_entity("light.schlafzimmer_decke"),
            ["Ist die Deckenlampe an", "Status Deckenlampe", "Brennt die Deckenlampe"],
            forbid=["light.alle_lichter", "light.schlafzimmer_kugel"],
        ),
        case(
            "status_spuelmaschine",
            query_entity("switch.kuche_spulmaschine"),
            ["Ist die Spülmaschine an", "Läuft die Spülmaschine", "Status Spülmaschine"],
        ),
        case(
            "status_wasch_casual",
            query_entity("switch.badezimmer_waschmaschine"),
            ["Läuft die Waschmaschine noch", "Ist die Waschmaschine an"],
        ),
    ]

def _more_multi() -> list[str]:
    return [
        case(
            "wz_und_arbeit_aus",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("arbeitszimmer", "light", "off"),
                ]
            ),
            [
                "Licht Wohnzimmer und Arbeitszimmer aus",
                "Wohnzimmer und Büro Licht aus",
                "Mach Wohnzimmer und Arbeitszimmer Licht aus",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "kuche_und_ess_an",
            "\n".join(
                [
                    action_area("kuche", "light", "on"),
                    action_area("esszimmer", "light", "on"),
                ]
            ),
            [
                "Licht Küche und Esszimmer an",
                "Küche und Esszimmer Lichte an",
                "Mach Küche und Esszimmer Licht an",
            ],
            forbid=GROUP_FORBID,
        ),
        case(
            "wz_ess_arbeit_aus",
            "\n".join(
                [
                    action_area("wohnzimmer", "light", "off"),
                    action_area("esszimmer", "light", "off"),
                    action_area("arbeitszimmer", "light", "off"),
                ]
            ),
            [
                "Licht Wohnzimmer Esszimmer und Arbeitszimmer aus",
                "Wohnzimmer, Esszimmer und Arbeitszimmer Licht aus",
            ],
            forbid=GROUP_FORBID,
        ),
    ]

