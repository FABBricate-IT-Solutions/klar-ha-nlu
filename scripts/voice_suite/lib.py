"""Shared YAML helpers for the Wohnung voice suite."""

from pathlib import Path
import textwrap

ROOT = Path(__file__).resolve().parents[2] / "tests" / "datasets"

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
  - {id: light.wohn_und_esszimmer, area_id: wohnung, name: Wohn und Esszimmer}
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

KUGEL_FORBID = [
    "light.alle_lichter",
    "light.wohn_und_esszimmer",
    "light.schlafzimmer_kugel",
]
ROOM_FORBID = [
    "schlafzimmer",
    "light.schlafzimmer_kugel",
    "light.schlafzimmer_decke",
    "light.schlafzimmer_licht",
    "light.alle_lichter",
    "light.wohn_und_esszimmer",
]
GROUP_FORBID = ["light.alle_lichter", "light.wohn_und_esszimmer"]

ROOMS = [
    ("wohnzimmer", "Wohnzimmer", "im Wohnzimmer", "light.wohnzimmer"),
    ("esszimmer", "Esszimmer", "im Esszimmer", "light.esszimmer"),
    ("kuche", "Küche", "in der Küche", "light.kuche_kuche"),
    ("arbeitszimmer", "Arbeitszimmer", "im Arbeitszimmer", "light.arbeitszimmer"),
    ("flur", "Flur", "im Flur", None),
    ("badezimmer", "Bad", "im Bad", None),
]


def dump(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(body).lstrip("\n"), encoding="utf-8")


def case(
    name: str,
    conditions: str,
    sentences: list[str],
    forbid: list[str] | None = None,
    speech_has: list[str] | None = None,
    speech_forbids: list[str] | None = None,
) -> str:
    lines = [f"- name: {name}", "  conditions:", conditions]
    if forbid:
        lines.append("  forbid:")
        for item in forbid:
            lines.append(f"    - {item}")
    if speech_has:
        lines.append("  speech_has:")
        for item in speech_has:
            lines.append(f"    - {item}")
    if speech_forbids:
        lines.append("  speech_forbids:")
        for item in speech_forbids:
            lines.append(f"    - {item!r}")
    lines.append("  sentences:")
    for sentence in sentences:
        lines.append(f"    - {sentence!r}")
    return "\n".join(lines) + "\n"


def action_entity(eid: str, state: str) -> str:
    return f"    - {{type: action, entity_id: {eid}, state: {state!r}}}"


def action_area(area: str, domain: str, state: str) -> str:
    return f"    - {{type: action, area: {area}, domain: {domain}, state: {state!r}}}"


def action_temp(eid: str, temp: int) -> str:
    return f"    - type: action\n      entity_id: {eid}\n      attributes:\n        temperature: {temp}"


def action_color(target_key: str, target_val: str, color: str) -> str:
    return f"    - type: action\n      {target_key}: {target_val}\n      domain: light\n      attributes:\n        color: {color}"


def action_bri(target_key: str, target_val: str, bri: int) -> str:
    return f"    - type: action\n      {target_key}: {target_val}\n      domain: light\n      attributes:\n        brightness: {bri}"


def query_area(area: str, domain: str) -> str:
    return f"    - {{type: query, area: {area}, domain: {domain}}}"


def query_entity(eid: str) -> str:
    return f"    - {{type: query, entity_id: {eid}}}"


def except_kugel(state: str) -> str:
    return "\n".join(
        [
            action_area("wohnzimmer", "light", state),
            action_area("esszimmer", "light", state),
            action_area("kuche", "light", state),
            action_area("arbeitszimmer", "light", state),
            action_entity("light.schlafzimmer_decke", state),
            action_entity("light.schlafzimmer_licht", state),
        ]
    )


def except_rooms(state: str) -> str:
    return "\n".join(
        [
            action_area("wohnzimmer", "light", state),
            action_area("esszimmer", "light", state),
            action_area("kuche", "light", state),
            action_area("arbeitszimmer", "light", state),
        ]
    )


def except_kugel_color(color: str) -> str:
    return "\n".join(
        [
            action_color("area", "wohnzimmer", color),
            action_color("area", "esszimmer", color),
            action_color("area", "kuche", color),
            action_color("area", "arbeitszimmer", color),
            action_color("entity_id", "light.schlafzimmer_decke", color),
            action_color("entity_id", "light.schlafzimmer_licht", color),
        ]
    )


def except_rooms_bri(bri: int) -> str:
    return "\n".join(
        [
            action_bri("area", "wohnzimmer", bri),
            action_bri("area", "esszimmer", bri),
            action_bri("area", "kuche", bri),
            action_bri("area", "arbeitszimmer", bri),
        ]
    )
