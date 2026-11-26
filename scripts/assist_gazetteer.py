"""The gazetteer fallback of Home Assistant's default conversation agent, without Home Assistant.

Core's `DefaultAgent` asks hassil first. Only when hassil has no match, and the language is
English, it hands the sentence to `gazetteer-matcher` (see
`homeassistant/components/conversation/gazetteer.py`). `scripts/bench_assist.py` runs the
same two steps so the benchmark compares Klar with the agent as it ships today, not with
hassil alone. Recognition only: nothing is executed.
"""

from __future__ import annotations

from typing import Any

from gazetteer_matcher import GazetteerMatcher, TargetReference

# The language the matcher has a vocabulary for. Core's `GazetteerFallback.LANGUAGE`.
LANGUAGES = ("en",)
TARGET_SLOTS = ("name", "area", "floor", "domain", "device_class")


def domain_of(entity_id: str) -> str:
    return entity_id.split(".", 1)[0]


def build_home(home: dict) -> dict:
    """The matcher's gazetteer from a Klar `home_config.yaml`, shaped like Core's `async_build_home`."""
    floors = {}
    for floor in home.get("floors") or []:
        fid = str(floor.get("id") or floor.get("name"))
        floors[fid] = {"name": floor.get("name") or fid, "aliases": list(floor.get("aliases") or [])}
    areas = {}
    for area in home.get("areas") or []:
        areas[area["id"]] = {
            "name": area.get("name") or area["id"],
            "aliases": list(area.get("aliases") or []),
            "floor": area.get("floor"),
        }
    entities = {}
    for device in home.get("devices") or []:
        spec = {
            "name": device.get("name") or device["id"],
            "aliases": list(device.get("aliases") or []),
            "domain": domain_of(device["id"]),
            "area": device.get("area_id"),
        }
        if device.get("device_class"):
            spec["device_class"] = device["device_class"]
        entities[device["id"]] = spec
    for kind, items in (("scene", home.get("scenes") or []), ("script", home.get("scripts") or [])):
        for item in items:
            raw_id, name = (item["id"], item.get("name")) if isinstance(item, dict) else (str(item), None)
            sid = raw_id if "." in raw_id else f"{kind}.{raw_id}"
            entities[sid] = {"name": name or sid, "aliases": [], "domain": kind, "area": None}
    return {"floors": floors, "areas": areas, "entities": entities}


class GazetteerAgent:
    """Interprets sentences the way Core's fallback does, remembering targets for "it"/"them"."""

    def __init__(self, home: dict, language: str) -> None:
        self.language = language
        self.supported = language.split("-", 1)[0] in LANGUAGES
        self.matcher = GazetteerMatcher(home=build_home(home)) if self.supported else None
        self._previous: tuple[TargetReference, ...] = ()

    def reset(self) -> None:
        """A new conversation: nothing for a pronoun to refer back to."""
        self._previous = ()

    def interpret(self, text: str) -> list[dict[str, Any]]:
        """Return the intents the matcher accepts for `text`, or none."""
        if self.matcher is None:
            return []
        result = self.matcher.interpret(text, previous_targets=self._previous)
        if not result.accepted or not result.frames:
            return []
        # Core only takes over a sentence it can also answer.
        if any(frame.response_key is None for frame in result.frames):
            return []
        self._previous = result.targets
        return [{"name": frame.intent, "slots": frame_slots(frame.slots)} for frame in result.frames]


def frame_slots(slots: dict[str, Any]) -> dict[str, Any]:
    """The matcher keeps ids under `name`; Klar's gold calls that `entity_id`."""
    out = {key: value for key, value in slots.items() if value is not None}
    name = out.get("name")
    if isinstance(name, str) and "." in name and "entity_id" not in out:
        out["entity_id"] = name
    return out
