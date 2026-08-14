from __future__ import annotations

import logging
from typing import Any

import aiohttp
from homeassistant.components import conversation
from homeassistant.components.conversation import (
    AssistantContent,
    ChatLog,
    ConversationEntity,
    ConversationInput,
    ConversationResult,
)
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers import intent
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import CONF_FALLBACK_AGENT, CONF_URL, DEFAULT_URL

_LOGGER = logging.getLogger(__name__)

_CHAT_ONLY = (
    "Du antwortest nur im Gespräch. Steuere keine Geräte "
    "und rufe keine Home-Assistant-Werkzeuge auf."
)


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([KlarConversationEntity(hass, entry)])


def _home_intents(intents: list[Any]) -> list[dict[str, Any]]:
    return [
        item
        for item in intents
        if isinstance(item, dict) and item.get("name") and item["name"] != "Unknown"
    ]


def _speech_from_result(result: ConversationResult) -> str:
    speech = result.response.speech or {}
    plain = speech.get("plain") or {}
    return str(plain.get("speech") or "")


class KlarConversationEntity(ConversationEntity):
    _attr_name = "Klar NLU"
    _attr_supported_features = conversation.ConversationEntityFeature.CONTROL

    def __init__(self, hass: HomeAssistant, entry: ConfigEntry) -> None:
        self.hass = hass
        self._entry = entry
        self._attr_unique_id = entry.entry_id
        self._attr_supported_languages = ["de"]

    @property
    def _url(self) -> str:
        return self._entry.data.get(CONF_URL, DEFAULT_URL).rstrip("/")

    def _fallback_agent_id(self) -> str | None:
        agent_id = self._entry.options.get(CONF_FALLBACK_AGENT)
        if not agent_id or agent_id == self.entity_id:
            return None
        return agent_id

    async def _async_handle_message(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
    ) -> ConversationResult:
        payload = await self._parse(user_input.text, user_input.conversation_id)
        speech = payload.get("speech") or "Erledigt."
        intents = _home_intents(payload.get("intents") or [])
        clarify = bool(payload.get("clarify"))
        conversation_id = payload.get("conversation_id") or user_input.conversation_id

        if not clarify and not intents and not payload.get("unreachable"):
            fallback = await self._fallback(user_input, chat_log)
            if fallback is not None:
                return fallback

        for item in intents:
            await self._handle_intent(user_input, item)

        chat_log.async_add_assistant_content_without_tools(
            AssistantContent(agent_id=user_input.agent_id, content=speech)
        )
        response = intent.IntentResponse(language=user_input.language or "de")
        response.async_set_speech(speech)
        return ConversationResult(
            conversation_id=conversation_id,
            response=response,
            continue_conversation=clarify,
        )

    async def _fallback(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
    ) -> ConversationResult | None:
        agent_id = self._fallback_agent_id()
        if not agent_id:
            return None
        extra = getattr(user_input, "extra_system_prompt", None)
        prompt = f"{extra}\n{_CHAT_ONLY}" if extra else _CHAT_ONLY
        try:
            result = await conversation.async_converse(
                self.hass,
                user_input.text,
                user_input.conversation_id,
                user_input.context,
                language=user_input.language,
                agent_id=agent_id,
                device_id=user_input.device_id,
                satellite_id=getattr(user_input, "satellite_id", None),
                extra_system_prompt=prompt,
            )
        except Exception as err:  # noqa: BLE001 — other agent is a system boundary
            _LOGGER.warning("LLM-Fallback fehlgeschlagen: %s", err)
            return None
        last = chat_log.content[-1] if chat_log.content else None
        if getattr(last, "role", None) != "assistant":
            speech = _speech_from_result(result)
            if speech:
                chat_log.async_add_assistant_content_without_tools(
                    AssistantContent(agent_id=user_input.agent_id, content=speech)
                )
        return result

    async def _parse(self, text: str, conversation_id: str | None) -> dict[str, Any]:
        url = f"{self._url}/api/parse"
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    url,
                    json={"text": text, "conversation_id": conversation_id},
                    timeout=aiohttp.ClientTimeout(total=5),
                ) as resp:
                    resp.raise_for_status()
                    return await resp.json()
        except Exception as err:  # noqa: BLE001 — boundary to the local engine
            _LOGGER.warning("Klar nicht erreichbar: %s", err)
            return {
                "speech": "Klar antwortet gerade nicht.",
                "intents": [],
                "unreachable": True,
            }

    async def _handle_intent(self, user_input: ConversationInput, item: dict) -> None:
        name = item.get("name")
        if not name:
            return
        slots = {s["name"]: {"value": s["value"]} for s in item.get("slots") or []}
        try:
            await intent.async_handle(
                self.hass,
                "klar_nlu",
                name,
                slots,
                user_input.text,
                user_input.context,
                user_input.language or "de",
            )
        except intent.IntentHandleError as err:
            _LOGGER.debug("Intent %s nicht ausgeführt: %s", name, err)
