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
        speech = (_FAILED_HINT.get(str(hint)) or {}).get(pack) or _FAILED.get(pack, _FAILED["en"])
    return validate_execute_result({"outcome": outcome, "speech": speech, "steps": steps})


def _step(index: int, item: dict[str, Any], result: IntentStepResult) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "index": index,
        "intent": str(item.get("name") or "unknown"),
        "status": "success" if result.ok else "error",
        "speech": result.speech,
        "error": None if result.ok else (result.error or "intent_failed"),
    }
    return payload
