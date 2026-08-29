"""Ordered Home Assistant execution with first-class partial failure."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

from homeassistant.components.conversation import ConversationInput
from homeassistant.core import HomeAssistant

from .contracts import validate_execute_result
from .dispatch import IntentStepResult, handle_intent

_FAILED = {
    "de": "Das hat nicht geklappt.",
    "en": "That did not work.",
}
_TV_UNAVAILABLE = {
    "de": "Der Fernseher ist gerade nicht erreichbar.",
    "en": "That TV is not available.",
}
_FAILED_HINT = {
    "media_unavailable": {
        "de": "Der Player ist gerade nicht erreichbar.",
        "en": "That player is not available.",
    },
    "mass_target_unavailable": {
        "de": "Der Musik-Player ist gerade nicht erreichbar.",
        "en": "The music player is not available.",
    },
    "entity_not_exposed": {
        "de": "Dieses Gerät ist für Assist nicht freigegeben.",
        "en": "That device is not exposed to Assist.",
    },
}
_PARTIAL = {
    "de": "Ein Schritt ist fehlgeschlagen.",
    "en": "One step failed.",
}


async def execute_plan(
    hass: HomeAssistant,
    user_input: ConversationInput,
    intents: list[dict[str, Any]],
    pack: str,
    assistant: str | None,
    exposed: Callable[[str], bool],
) -> dict[str, Any]:
    steps: list[dict[str, Any]] = []
    spoken: list[str] = []
    for index, item in enumerate(intents):
        result = await handle_intent(hass, user_input, item, pack, assistant, exposed)
        step = _step(index, item, result)
        steps.append(step)
        if result.ok and result.speech:
            spoken.append(result.speech)
    successes = sum(1 for step in steps if step["status"] == "success")
    if successes == len(steps) and steps:
        outcome = "success"
        speech = " ".join(spoken)
    elif successes:
        outcome = "partial"
        extra = _PARTIAL.get(pack, _PARTIAL["en"])
        speech = " ".join([*spoken, extra]).strip()
    else:
        outcome = "error"
        hint = next((step.get("error") for step in steps if step.get("error")), None)
        speech = _error_speech(pack, str(hint or ""), intents)
    return validate_execute_result({"outcome": outcome, "speech": speech, "steps": steps})


def _error_speech(pack: str, hint: str, intents: list[dict[str, Any]]) -> str:
    if hint == "media_unavailable" and any(_tv_item(item) for item in intents):
        return _TV_UNAVAILABLE.get(pack) or _TV_UNAVAILABLE["en"]
    return (_FAILED_HINT.get(hint) or {}).get(pack) or _FAILED.get(pack, _FAILED["en"])


def _tv_item(item: dict[str, Any]) -> bool:
    blob = " ".join(
        str(slot.get("value") or "")
        for slot in item.get("slots") or []
        if isinstance(slot, dict) and slot.get("name") in {"entity_id", "name"}
    ).casefold()
    return "tv" in blob or "fernseher" in blob


def _step(index: int, item: dict[str, Any], result: IntentStepResult) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "index": index,
        "intent": str(item.get("name") or "unknown"),
        "status": "success" if result.ok else "error",
        "speech": result.speech,
        "error": None if result.ok else (result.error or "intent_failed"),
    }
    return payload
