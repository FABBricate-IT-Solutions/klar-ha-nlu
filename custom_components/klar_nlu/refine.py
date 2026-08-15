"""LLM-only rewrite of finished NLU replies."""

from __future__ import annotations

import logging
import re
from typing import Any
from uuid import uuid4

try:
    from homeassistant.components import conversation
    from homeassistant.core import Context, HomeAssistant
except ImportError:  # stdlib tests load this module without Home Assistant
    conversation = None  # type: ignore[assignment]
    Context = Any
    HomeAssistant = Any

try:
    from .fallback import can_use_fallback_agent
except ImportError:  # stdlib tests load this module without a package

    def can_use_fallback_agent(controls_home: bool, chat: bool = False) -> bool:
        del chat
        return not controls_home

try:
    from .refine_voices import _PERSONALITY, _RULES
except ImportError:  # stdlib tests load this module without a package
    from refine_voices import _PERSONALITY, _RULES

_LOGGER = logging.getLogger(__name__)
_INTENT = re.compile(r"\bHass[A-Z][A-Za-z]+\b")
_DIGITS = re.compile(r"\d+")
_NUM_WORD = re.compile(
    r"\b(?:null|eins|zwei|drei|vier|fünf|sechs|sieben|acht|neun|zehn|"
    r"elf|zwölf|dreizehn|vierzehn|fünfzehn|sechzehn|siebzehn|achtzehn|neunzehn|"
    r"zwanzig|dreissig|dreißig|vierzig|fünfzig|sechzig|siebzig|achtzig|neunzig|"
    r"hundert|tausend|(?:ein|zwei|drei|vier|fünf|sechs|sieben|acht|neun)und(?:zwanzig|"
    r"dreissig|dreißig|vierzig|fünfzig|sechzig|siebzig|achtzig|neunzig)|"
    r"zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|"
    r"thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|"
    r"thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand)\b",
    re.IGNORECASE,
)

_INPUT = {
    "de": "{speech}",
    "en": "{speech}",
}

_THINKING_OFF = {"chat_template_kwargs": {"enable_thinking": False}}
_MODEL_KEYS = ("chat_model", "model", "llm_model")


def should_refine(
    enabled: bool,
    agent_id: str | None,
    speech: str,
    home: bool,
) -> bool:
    return bool(enabled and agent_id and speech.strip() and home)


def refine_prompt(pack: str, personality: str, extra: str | None) -> str:
    rules = _RULES.get(pack, _RULES["de"])
    voice, shots = (_PERSONALITY.get(personality) or _PERSONALITY["default"]).get(
        pack,
        _PERSONALITY["default"]["de"],
    )
    custom = (extra or "").strip()
    voice = voice.rstrip(".")
    if pack == "en":
        prompt = (
            f"{rules}\n\nVoice: {voice}.\n"
            f"Sound like this character. Vary the wording. "
            f"Do not stamp the same opening every time.\n"
            f"Examples:\n{shots}"
        )
        if custom:
            prompt = f"{prompt}\nAdditional style instruction: {custom}"
        return prompt
    prompt = (
        f"{rules}\n\nStimme: {voice}.\n"
        f"Klinge wie diese Figur. Variiere die Formulierung. "
        f"Klebe nicht jedes Mal dieselbe Eröffnung davor.\n"
        f"Beispiele:\n{shots}"
    )
    if custom:
        prompt = f"{prompt}\nZusätzliche Stil-Anweisung: {custom}"
    return prompt


def refine_input(speech: str, pack: str) -> str:
    template = _INPUT.get(pack, _INPUT["de"])
    return template.format(speech=speech.strip())


def clean_refined(text: str) -> str:
    speech = (text or "").strip().strip("\"'`“”«»")
    if "\n" in speech:
        speech = " ".join(line.strip() for line in speech.splitlines() if line.strip())
    return speech.strip()


def accept_refined(original: str, refined: str) -> str | None:
    speech = clean_refined(refined)
    if not speech or speech.endswith(("...", "…")):
        return None
    if speech.endswith("?") and not original.rstrip().endswith("?"):
        return None
    if _INTENT.search(speech):
        return None
    source_nums = set(_DIGITS.findall(original))
    result_nums = set(_DIGITS.findall(speech))
    if source_nums != result_nums:
        return None
    if not source_nums and _NUM_WORD.search(speech):
        return None
    if len(speech) > max(len(original) * 6, 280):
        return None
    return speech


def refine_extra_body() -> dict[str, Any]:
    return dict(_THINKING_OFF)


def speech_from_completion(result: Any) -> str:
    choices = getattr(result, "choices", None) or []
    if not choices:
        return ""
    message = getattr(choices[0], "message", None)
    return str(getattr(message, "content", None) or "").strip()


def _mapping(value: Any) -> dict[str, Any]:
    if isinstance(value, dict):
        return value
    data = getattr(value, "data", None)
    return data if isinstance(data, dict) else {}


def _first_model(*sources: Any) -> str | None:
    for source in sources:
        data = _mapping(source)
        for key in _MODEL_KEYS:
            model = str(data.get(key) or "").strip()
            if model:
                return model
    return None


def _openai_client(raw: Any) -> Any:
    if raw is not None and hasattr(raw, "chat"):
        return raw
    inner = getattr(raw, "client", None)
    if inner is not None and hasattr(inner, "chat"):
        return inner
    return None


def llm_client_and_model(hass: HomeAssistant, agent_id: str) -> tuple[Any, str] | None:
    if conversation is None:
        return None
    try:
        agent = conversation.async_get_agent(hass, agent_id)
    except Exception:  # noqa: BLE001 — agent lookup is a system boundary
        return None
    if agent is None:
        return None
    entry = getattr(agent, "entry", None) or getattr(agent, "_entry", None)
    client = _openai_client(getattr(entry, "runtime_data", None))
    if client is None:
        client = _openai_client(getattr(agent, "client", None) or getattr(agent, "_client", None))
    if client is None:
        return None
    model = _first_model(getattr(agent, "subentry", None), getattr(entry, "options", None), getattr(entry, "data", None))
    if not model:
        return None
    return client, model


def speech_from_result(result: Any) -> str:
    speech = getattr(result, "response", None)
    speech = getattr(speech, "speech", None) or {}
    plain = speech.get("plain") if isinstance(speech, dict) else None
    if not isinstance(plain, dict):
        return ""
    return str(plain.get("speech") or "").strip()


async def async_refine_speech(
    hass: HomeAssistant,
    agent_id: str,
    controls_home: bool,
    speech: str,
    context: Context,
    language: str | None,
    pack: str,
    personality: str,
    extra_prompt: str | None,
) -> str | None:
    if conversation is None:
        return None
    prompt = refine_prompt(pack, personality, extra_prompt)
    _LOGGER.debug("LLM-Refine Stimme %s", personality)
    user = refine_input(speech, pack)
    raw = await _async_refine_raw(
        hass, agent_id, user, prompt, language or pack, context, controls_home
    )
    return accept_refined(speech, raw or "")


async def _async_refine_raw(
    hass: HomeAssistant,
    agent_id: str,
    user: str,
    prompt: str,
    language: str,
    context: Context,
    controls_home: bool,
) -> str | None:
    resolved = llm_client_and_model(hass, agent_id)
    if resolved is not None:
        client, model = resolved
        try:
            result = await client.chat.completions.create(
                model=model,
                messages=[
                    {"role": "system", "content": prompt},
                    {"role": "user", "content": user},
                ],
                max_tokens=128,
                temperature=0.65,
                extra_body=refine_extra_body(),
            )
            return speech_from_completion(result)
        except Exception as err:  # noqa: BLE001 — client shape varies by agent
            _LOGGER.debug("LLM-Refine direkt fehlgeschlagen, converse: %s", err)
    if not can_use_fallback_agent(controls_home):
        _LOGGER.warning("LLM-Refine %s hat Assist-Werkzeuge — converse übersprungen", agent_id)
        return None
    try:
        result = await conversation.async_converse(
            hass,
            user,
            f"klar-refine-{uuid4()}",
            context,
            language=language,
            agent_id=agent_id,
            device_id=None,
            satellite_id=None,
            extra_system_prompt=prompt,
        )
    except Exception as err:  # noqa: BLE001 — other agent is a system boundary
        _LOGGER.warning("LLM-Refine fehlgeschlagen: %s", err)
        return None
    return speech_from_result(result)
