"""Assist sentences captured from Home Assistant conversation.process."""

from pathlib import Path

from ..lib import ROOT, case, dump, query_area, query_entity

# Verified 2026-08-16 via conversation.klar_nlu on the live Wohnung.
HA_KUECHE = ["Wie ist der Status von Küche", "Wie ist der Status der Küche"]
HA_LICHTER = [
    "Wie ist der Status der Leuchten",
    "Wie ist der Status von allen Lichtern",
    "Wie ist der Status von Alle Lichter",
]
HA_KUGEL = ["Wie ist der Status von der Kugel"]

LICHTER_FORBID = [
    "wohnzimmer",
    "esszimmer",
    "kuche",
    "light.wohnzimmer",
    "light.esszimmer",
    "light.wohn_und_esszimmer",
    "wohnung",
]


def _kueche() -> str:
    return case("ha_status_kueche", query_area("kuche", "light"), HA_KUECHE, forbid=["light.alle_lichter"])


def _lichter() -> str:
    return case("ha_status_lichter", query_entity("light.alle_lichter"), HA_LICHTER, forbid=LICHTER_FORBID)


def write(de: Path) -> None:
    dump(
        de / "assist" / "ha_queries.yaml",
        "\n".join(
            [
                _kueche(),
                _lichter(),
                case(
                    "ha_status_kugel",
                    query_entity("light.schlafzimmer_kugel"),
                    HA_KUGEL,
                    forbid=["light.alle_lichter", "schlafzimmer"],
                ),
            ]
        ),
    )
    dump(
        ROOT / "wohnung_live" / "assist" / "ha_queries.yaml",
        "\n".join(
            [
                _kueche(),
                _lichter(),
                case(
                    "ha_status_kugel",
                    query_entity("light.schlafzimmer"),
                    HA_KUGEL,
                    forbid=["light.alle_lichter"],
                ),
            ]
        ),
    )
