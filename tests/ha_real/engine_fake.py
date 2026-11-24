"""A scripted stand-in for the Klar engine's HTTP API.

The integration talks to the engine through Home Assistant's shared aiohttp session.
These tests replace that session with the Home Assistant test mock and answer the three
routes a turn needs: ``/api/settings``, ``/api/v2/home``, ``/api/v2/parse`` and
``/api/v2/speech/render``. Every
parse reply goes through the integration's own contract validation, so a reply the real
engine could not send does not pass here either.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from typing import Any

from pytest_homeassistant_custom_component.test_util.aiohttp import (
    AiohttpClientMocker,
    AiohttpClientMockResponse,
)

ENGINE_URL = "http://klar-engine.test:10520"
TIME_ZONE = "US/Pacific"

Body = dict[str, Any]
Reply = Callable[["FakeEngine", Body], dict[str, Any]]


def intent(name: str, **slots: object) -> dict[str, Any]:
    """An intent as the engine sends it: every slot value is a string."""
    return {"name": name, "slots": [{"name": key, "value": str(value)} for key, value in slots.items()]}


def _envelope(body: Body, decision: dict[str, Any], speech: str) -> dict[str, Any]:
    return {
        "schema_version": "2.0",
        "text": str(body.get("text") or ""),
        "conversation_id": str(body.get("conversation_id") or ""),
        "decision": decision,
        "speech": speech,
        "confidence": 1.0,
        "margin": 1.0,
        "candidates": [],
        "evidence": [],
        "trace": {"stages": [], "discarded": []},
        "briefing": False,
    }


def execute(body: Body, intents: list[dict[str, Any]], speech: str = "") -> dict[str, Any]:
    plan = {
        "confidence": 1.0,
        "margin": 1.0,
        "evidence": [],
        "steps": [
            {"index": index, "intent": item, "confidence": 1.0, "evidence": []}
            for index, item in enumerate(intents)
        ],
    }
    payload = _envelope(body, {"type": "execute"}, speech)
    payload["plan"] = plan
    payload["selected_candidate_id"] = "c0"
    payload["candidates"] = [
        {
            "id": "c0",
            "plan": plan,
            "score": 1.0,
            "margin": 1.0,
            "policy": "test",
            "precedence": 0,
            "evidence": [],
        }
    ]
    return payload


def confirm(body: Body, prompt: str) -> dict[str, Any]:
    return _envelope(body, {"type": "confirm", "prompt": prompt, "candidate_id": "c0"}, prompt)


def clarify(body: Body, prompt: str, options: list[str]) -> dict[str, Any]:
    return _envelope(body, {"type": "clarify", "prompt": prompt, "options": options}, prompt)


def reject(body: Body, reason: str, speech: str) -> dict[str, Any]:
    return _envelope(body, {"type": "reject", "reason": reason}, speech)


class FakeEngine:
    """Answers engine routes from a script and records what the integration sent."""

    def __init__(self) -> None:
        self.settings: dict[str, Any] = {"languages": ["en"], "time_zone": TIME_ZONE}
        self.parse_bodies: list[Body] = []
        self.home_snapshots: list[Body] = []
        self.speech_snapshots: list[Body] = []
        self.spoken = "Done."
        self.pending: dict[str, list[dict[str, Any]]] = {}
        self._script: dict[str, Reply] = {}
        self.reachable = True

    def on(self, text: str, reply: Reply) -> None:
        self._script[text.casefold()] = reply

    def executes(self, text: str, intents: list[dict[str, Any]], speech: str = "") -> None:
        self.on(text, lambda _engine, body: execute(body, intents, speech))

    def asks_before(self, text: str, prompt: str, intents: list[dict[str, Any]]) -> None:
        """Confirm first; the plan runs only when the same session answers yes."""

        def ask(engine: FakeEngine, body: Body) -> dict[str, Any]:
            engine.pending[str(body["conversation_id"])] = intents
            return confirm(body, prompt)

        self.on(text, ask)
        self.on("yes", _answer_yes)

    def session_ids(self) -> list[str]:
        return [str(body.get("conversation_id") or "") for body in self.parse_bodies]

    def install(self, mocker: AiohttpClientMocker) -> None:
        mocker.get(f"{ENGINE_URL}/api/settings", side_effect=self._get_settings)
        mocker.post(f"{ENGINE_URL}/api/settings", side_effect=self._post_settings)
        mocker.post(f"{ENGINE_URL}/api/v2/home", side_effect=self._home)
        mocker.post(f"{ENGINE_URL}/api/v2/parse", side_effect=self._parse)
        mocker.post(f"{ENGINE_URL}/api/v2/speech/render", side_effect=self._render)
        rest = re.compile(rf"^{re.escape(ENGINE_URL)}/.*")
        for method in ("get", "post", "put", "delete"):
            mocker.request(method, rest, status=404, json={"error": "not_found"})

    def _response(self, method: str, url: Any, payload: Any, status: int = 200) -> AiohttpClientMockResponse:
        if not self.reachable:
            status, payload = 503, {"error": "down"}
        return AiohttpClientMockResponse(method, url, status=status, json=payload)

    async def _get_settings(self, method: str, url: Any, _data: Any) -> AiohttpClientMockResponse:
        return self._response(method, url, dict(self.settings))

    async def _post_settings(self, method: str, url: Any, data: Any) -> AiohttpClientMockResponse:
        if self.reachable and isinstance(data, dict):
            self.settings = dict(data)
        return self._response(method, url, dict(self.settings))

    async def _home(self, method: str, url: Any, data: Any) -> AiohttpClientMockResponse:
        if self.reachable and isinstance(data, dict):
            self.home_snapshots.append(data)
        return self._response(method, url, {"ok": True})

    async def _render(self, method: str, url: Any, data: Any) -> AiohttpClientMockResponse:
        if self.reachable and isinstance(data, dict):
            self.speech_snapshots.append(data)
        return self._response(method, url, {"speech": self.spoken, "source": "post_execute"})

    async def _parse(self, method: str, url: Any, data: Any) -> AiohttpClientMockResponse:
        body: Body = dict(data or {})
        if not self.reachable:
            return self._response(method, url, None)
        self.parse_bodies.append(body)
        reply = self._script.get(str(body.get("text") or "").casefold())
        if reply is None:
            return self._response(method, url, reject(body, "no_match", "I did not understand that."))
        return self._response(method, url, reply(self, body))


def _answer_yes(engine: FakeEngine, body: Body) -> dict[str, Any]:
    intents = engine.pending.pop(str(body["conversation_id"]), None)
    if intents is None:
        return reject(body, "nothing_to_confirm", "There is nothing to confirm.")
    return execute(body, intents)
