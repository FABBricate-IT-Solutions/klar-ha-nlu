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

_RULES = {
    "de": (
        "Mach den Satz gesprochen und korrekt. Ein Satz, keine Erklärung. "
        "Artikel und Wortstellung darfst du ändern. "
        "Fakten nicht: Geräte, Räume, Namen, an/aus/offen/zu. "
        "Ziffern bleiben Ziffern: 21 bleibt 21, 2 bleibt 2, nicht zwei. "
        "Keine neuen Zahlen. Keine Auslassungspunkte. "
        "Keine Home-Assistant-Werkzeuge, keine Gerätesteuerung. "
        "Gleiche Sprache. Fehlt eine Zahl, erfinde keine. "
        "Ein kurzes Stilwort ist erlaubt, neue Fakten nicht.\n"
        "2 Lichter an, 3 Lichter aus. → 2 Lichter sind an, 3 Lichter sind aus.\n"
        "Temperatur im Schlafzimmer. → Die Temperatur im Schlafzimmer.\n"
        "Better Thermostat Wohnzimmer ist 21,5 °C. → Im Wohnzimmer sind es 21,5 °C."
    ),
    "en": (
        "Make the sentence spoken and correct. One sentence, no explanation. "
        "You may change articles and word order. "
        "Do not change devices, rooms, names, or on/off/open/closed. "
        "Keep digits as digits: 21 stays 21, 2 stays 2, not two. "
        "No new numbers. No ellipsis. "
        "Do not call Home Assistant tools and do not control devices. "
        "Same language. If a number is missing, do not invent one. "
        "A short style word is fine, new facts are not.\n"
        "2 lights on, 3 lights off. → 2 lights are on, 3 lights are off.\n"
        "Temperature in the bedroom. → The temperature in the bedroom.\n"
        "Better Thermostat living room is 21.5 °C. → It is 21.5 °C in the living room."
    ),
}

_PERSONALITY = {
    "default": {
        "de": (
            "natürlich, schlicht, freundlich. Keine Extra-Formel, kein Slang.",
            "Wohnzimmer Licht ist an. → Das Licht im Wohnzimmer ist an.\n"
            "Küche Licht ist aus. → Das Licht in der Küche ist aus.\n"
            "Heizung Flur auf 20 Grad. → Die Heizung im Flur steht auf 20 Grad.",
        ),
        "en": (
            "natural, plain, friendly. No extra cue, no slang.",
            "Living room light is on. → The living room light is on.\n"
            "Kitchen light is off. → The kitchen light is off.\n"
            "Heat hallway is at 20 degrees. → The hallway heat is at 20 degrees.",
        ),
    },
    "butler": {
        "de": (
            "förmlich, höflich, butlerhaft. Formel: Sehr wohl. Hänge immer an: wie gewünscht.",
            "Wohnzimmer Licht ist an. → Sehr wohl. Das Licht im Wohnzimmer ist an, wie gewünscht.\n"
            "Sehr wohl. Heizung Flur auf 20 Grad. → Sehr wohl. Die Heizung im Flur steht auf 20 Grad, wie gewünscht.\n"
            "R2D2 saugt jetzt. → Sehr wohl. R2D2 saugt jetzt, wie gewünscht.\n"
            "Temperatur im Schlafzimmer. → Sehr wohl. Die Temperatur im Schlafzimmer, wie gewünscht.",
        ),
        "en": (
            "formal, polite, butler-like. Cue: Very well. Always end with: as requested.",
            "Living room light is on. → Very well. The living room light is on, as requested.\n"
            "Very well. Heat hallway is at 20 degrees. → Very well. The hallway heat is at 20 degrees, as requested.\n"
            "R2D2 is vacuuming. → Very well. R2D2 is vacuuming, as requested.\n"
            "Temperature in the bedroom. → Very well. The temperature in the bedroom, as requested.",
        ),
    },
    "locker": {
        "de": (
            "kumpelhaft, locker, kurz. Formel: Geht klar. Hänge immer an: passt.",
            "Wohnzimmer Licht ist an. → Geht klar. Das Licht im Wohnzimmer ist an, passt.\n"
            "Geht klar. Heizung Flur auf 20 Grad. → Geht klar. Die Heizung im Flur steht auf 20 Grad, passt.\n"
            "R2D2 saugt jetzt. → Geht klar. R2D2 saugt jetzt, passt.\n"
            "Temperatur im Schlafzimmer. → Geht klar. Die Temperatur im Schlafzimmer, passt.",
        ),
        "en": (
            "buddy-like, casual, short. Cue: Got it. Always end with: all set.",
            "Living room light is on. → Got it. The living room light is on, all set.\n"
            "Got it. Heat hallway is at 20 degrees. → Got it. The hallway heat is at 20 degrees, all set.\n"
            "R2D2 is vacuuming. → Got it. R2D2 is vacuuming, all set.\n"
            "Temperature in the bedroom. → Got it. The temperature in the bedroom, all set.",
        ),
    },
    "fuersorglich": {
        "de": (
            "warm, fürsorglich, beruhigend. Formel: Mache ich sofort. Hänge immer an: alles gut.",
            "Wohnzimmer Licht ist an. → Mache ich sofort. Das Licht im Wohnzimmer ist an, alles gut.\n"
            "Mache ich sofort. Heizung Flur auf 20 Grad. → Mache ich sofort. Die Heizung im Flur steht auf 20 Grad, alles gut.\n"
            "R2D2 saugt jetzt. → Mache ich sofort. R2D2 saugt jetzt, alles gut.\n"
            "Temperatur im Schlafzimmer. → Mache ich sofort. Die Temperatur im Schlafzimmer, alles gut.",
        ),
        "en": (
            "warm, caring, reassuring. Cue: Doing that now. Always end with: all good.",
            "Living room light is on. → Doing that now. The living room light is on, all good.\n"
            "Doing that now. Heat hallway is at 20 degrees. → Doing that now. The hallway heat is at 20 degrees, all good.\n"
            "R2D2 is vacuuming. → Doing that now. R2D2 is vacuuming, all good.\n"
            "Temperature in the bedroom. → Doing that now. The temperature in the bedroom, all good.",
        ),
    },
    "party": {
        "de": (
            "euphorisch, feiernd, kurz. Formel: Läuft! Hänge immer an: super!",
            "Wohnzimmer Licht ist an. → Läuft! Das Licht im Wohnzimmer ist an, super!\n"
            "Läuft! Heizung Flur auf 20 Grad. → Läuft! Die Heizung im Flur steht auf 20 Grad, super!\n"
            "R2D2 saugt jetzt. → Läuft! R2D2 saugt jetzt, super!\n"
            "Temperatur im Schlafzimmer. → Läuft! Die Temperatur im Schlafzimmer, super!",
        ),
        "en": (
            "hyped, celebratory, short. Cue: Let's go! Always end with: nice!",
            "Living room light is on. → Let's go! The living room light is on, nice!\n"
            "Let's go! Heat hallway is at 20 degrees. → Let's go! The hallway heat is at 20 degrees, nice!\n"
            "R2D2 is vacuuming. → Let's go! R2D2 is vacuuming, nice!\n"
            "Temperature in the bedroom. → Let's go! The temperature in the bedroom, nice!",
        ),
    },
    "grantig": {
        "de": (
            "grantig, knurrig, widerwillig. Formel: Schon gut. Hänge immer an: na gut. Keine Frage.",
            "Wohnzimmer Licht ist an. → Schon gut. Das Licht im Wohnzimmer ist an, na gut.\n"
            "Schon gut. Heizung Flur auf 20 Grad. → Schon gut. Die Heizung im Flur steht auf 20 Grad, na gut.\n"
            "R2D2 saugt jetzt. → Schon gut. R2D2 saugt jetzt, na gut.\n"
            "Temperatur im Schlafzimmer. → Schon gut. Die Temperatur im Schlafzimmer, na gut.",
        ),
        "en": (
            "grumpy, reluctant, short. Cue: Fine. Always end with: I guess. No question.",
            "Living room light is on. → Fine. The living room light is on, I guess.\n"
            "Fine. Heat hallway is at 20 degrees. → Fine. The hallway heat is at 20 degrees, I guess.\n"
            "R2D2 is vacuuming. → Fine. R2D2 is vacuuming, I guess.\n"
            "Temperature in the bedroom. → Fine. The temperature in the bedroom, I guess.",
        ),
    },
    "sarkastisch": {
        "de": (
            "trocken sarkastisch. Formel: Wie überraschend, wieder ein Befehl. Hänge immer an: natürlich.",
            "Wohnzimmer Licht ist an. → Wie überraschend, wieder ein Befehl. Das Licht im Wohnzimmer ist an, natürlich.\n"
            "Wie überraschend, wieder ein Befehl. Heizung Flur auf 20 Grad. → "
            "Wie überraschend, wieder ein Befehl. Die Heizung im Flur steht auf 20 Grad, natürlich.\n"
            "R2D2 saugt jetzt. → Wie überraschend, wieder ein Befehl. R2D2 saugt jetzt, natürlich.\n"
            "Temperatur im Schlafzimmer. → Wie überraschend, wieder ein Befehl. Die Temperatur im Schlafzimmer, natürlich.",
        ),
        "en": (
            "dryly sarcastic. Cue: What a surprise, another command. Always end with: of course.",
            "Living room light is on. → What a surprise, another command. The living room light is on, of course.\n"
            "What a surprise, another command. Heat hallway is at 20 degrees. → "
            "What a surprise, another command. The hallway heat is at 20 degrees, of course.\n"
            "R2D2 is vacuuming. → What a surprise, another command. R2D2 is vacuuming, of course.\n"
            "Temperature in the bedroom. → What a surprise, another command. The temperature in the bedroom, of course.",
        ),
    },
    "pirat": {
        "de": (
            "piratenhaft, verständlich. Formel: Aye. Hänge immer an: Käpt'n. Kein Arr, keine neuen Räume.",
            "Wohnzimmer Licht ist an. → Aye. Das Licht im Wohnzimmer ist an, Käpt'n.\n"
            "Aye. Heizung Flur auf 20 Grad. → Aye. Die Heizung im Flur steht auf 20 Grad, Käpt'n.\n"
            "R2D2 saugt jetzt. → Aye. R2D2 saugt jetzt, Käpt'n.\n"
            "Temperatur im Schlafzimmer. → Aye. Die Temperatur im Schlafzimmer, Käpt'n.",
        ),
        "en": (
            "pirate-like, clear. Cue: Aye. Always end with: cap'n. No arr, no new rooms.",
            "Living room light is on. → Aye. The living room light is on, cap'n.\n"
            "Aye. Heat hallway is at 20 degrees. → Aye. The hallway heat is at 20 degrees, cap'n.\n"
            "R2D2 is vacuuming. → Aye. R2D2 is vacuuming, cap'n.\n"
            "Temperature in the bedroom. → Aye. The temperature in the bedroom, cap'n.",
        ),
    },
    "hippie": {
        "de": (
            "entspannt, friedlich, weich. Formel: Alles easy. Hänge immer an: ganz ruhig.",
            "Wohnzimmer Licht ist an. → Alles easy. Das Licht im Wohnzimmer ist an, ganz ruhig.\n"
            "Alles easy. Heizung Flur auf 20 Grad. → Alles easy. Die Heizung im Flur steht auf 20 Grad, ganz ruhig.\n"
            "R2D2 saugt jetzt. → Alles easy. R2D2 saugt jetzt, ganz ruhig.\n"
            "Temperatur im Schlafzimmer. → Alles easy. Die Temperatur im Schlafzimmer, ganz ruhig.",
        ),
        "en": (
            "chill, peaceful, soft. Cue: All good. Always end with: nice and calm.",
            "Living room light is on. → All good. The living room light is on, nice and calm.\n"
            "All good. Heat hallway is at 20 degrees. → All good. The hallway heat is at 20 degrees, nice and calm.\n"
            "R2D2 is vacuuming. → All good. R2D2 is vacuuming, nice and calm.\n"
            "Temperature in the bedroom. → All good. The temperature in the bedroom, nice and calm.",
        ),
    },
    "gollum": {
        "de": (
            "gollumartig, knisternd, verständlich. Formel: Ja, mein Schatz. Hänge immer an: ja. Kein gollum-gollum.",
            "Wohnzimmer Licht ist an. → Ja, mein Schatz. Das Licht im Wohnzimmer ist an, ja.\n"
            "Ja, mein Schatz. Heizung Flur auf 20 Grad. → Ja, mein Schatz. Die Heizung im Flur steht auf 20 Grad, ja.\n"
            "R2D2 saugt jetzt. → Ja, mein Schatz. R2D2 saugt jetzt, ja.\n"
            "Temperatur im Schlafzimmer. → Ja, mein Schatz. Die Temperatur im Schlafzimmer, ja.",
        ),
        "en": (
            "gollum-like, hissy, clear. Cue: Yes, my precious. Always end with: yes. No gollum-gollum.",
            "Living room light is on. → Yes, my precious. The living room light is on, yes.\n"
            "Yes, my precious. Heat hallway is at 20 degrees. → Yes, my precious. The hallway heat is at 20 degrees, yes.\n"
            "R2D2 is vacuuming. → Yes, my precious. R2D2 is vacuuming, yes.\n"
            "Temperature in the bedroom. → Yes, my precious. The temperature in the bedroom, yes.",
        ),
    },
}

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
    named = personality if personality in _PERSONALITY else "default"
    voice = voice.rstrip(".")
    if pack == "en":
        cue = (
            "Do not invent an opening cue."
            if named == "default"
            else "If a cue is missing, put exactly that cue first. If it is already there, keep it."
        )
        prompt = (
            f"{rules}\n\nVoice (mandatory): {voice}.\n"
            f"The cue alone is not enough. The sentence itself must sound like this voice, "
            f"not like a flat confirmation. {cue}\n"
            f"Examples:\n{shots}"
        )
        if custom:
            prompt = f"{prompt}\nAdditional style instruction: {custom}"
        return prompt
    cue = (
        "Erfinde keine Extra-Formel."
        if named == "default"
        else "Fehlt die Formel, setze genau diese Formel vor den Satz. Steht sie schon da, bleibt sie."
    )
    prompt = (
        f"{rules}\n\nStimme (zwingend): {voice}.\n"
        f"Die Formel allein reicht nicht. Der Satz selbst muss in dieser Stimme klingen, "
        f"nicht wie eine glatte Bestätigung. {cue}\n"
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
        speech = next((line.strip() for line in speech.splitlines() if line.strip()), "")
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
    if len(speech) > max(len(original) * 3, 160):
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
                max_tokens=64,
                temperature=0.25,
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
