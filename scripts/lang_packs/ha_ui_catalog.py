"""Assemble flattened HA UI strings from per-locale field maps."""

from __future__ import annotations

from lang_packs.ha_ui_asia import ASIA
from lang_packs.ha_ui_europe import EUROPE
from lang_packs.ha_ui_europe_north import EUROPE_NORTH
from lang_packs.ha_ui_indic import INDIC
from lang_packs.ha_ui_mena import MENA
from lang_packs.ha_ui_more import MORE
from lang_packs.ha_ui_west import WEST

# Assist pack code -> translations/*.json stem. None = strings.json (English).
HA_FILE = {
    "en": None,
    "zh-CN": "zh-Hans",
    "zh-TW": "zh-Hant",
}

PACKS: dict[str, dict[str, str]] = {}
PACKS.update(EUROPE)
PACKS.update(EUROPE_NORTH)
PACKS.update(ASIA)
PACKS.update(MORE)
PACKS.update(MENA)
PACKS.update(WEST)
PACKS.update(INDIC)

_BRAND = {
    "title": "Klar NLU",
    "butler": "Butler",
    "party": "Party",
    "hippie": "Hippie",
    "gollum": "Gollum",
    "jarvis": "Jarvis",
}


def expand(fields: dict[str, str]) -> dict[str, str]:
    f = {**_BRAND, **fields}
    normal, casual, caring = f["normal"], f["casual"], f["caring"]
    grumpy, sarcastic, pirate = f["grumpy"], f["sarcastic"], f["pirate"]
    return {
        "config.step.user.title": f["title"],
        "config.step.user.description": f["user_desc"],
        "config.step.user.data.mode": f["mode"],
        "config.step.user.data.channel": f["channel"],
        "config.step.user.data.url": f["url"],
        "config.step.user.data.token": f["token"],
        "config.error.invalid_url": f["invalid"],
        "config.abort.already_configured": f["already"],
        "options.step.init.title": f["title"],
        "options.step.init.description": f["opt_desc"],
        "options.step.init.data.mode": f["mode"],
        "options.step.init.data.personality": f["personality"],
        "options.step.init.data.languages": f["languages"],
        "options.step.init.data.fallback_agent": f["fallback"],
        "options.step.init.data.allow_llm_tools": f.get(
            "allow_llm_tools", "Allow Assist tools on the chit-chat agent"
        ),
        "options.step.init.data.refine_speech": f["refine_speech"],
        "options.step.init.data.refine_prompt": f["refine_prompt"],
        "options.step.init.data.url": f["url"],
        "options.step.init.data.token": f["token"],
        "options.step.init.data.assist_filter": f["assist_filter"],
        "options.step.init.data.nlu_rag": f["nlu_rag"],
        "options.step.init.data.quiet_ack": f["quiet_ack"],
        "options.step.init.data.calendar_llm": f["calendar_llm"],
        "options.step.init.data.channel": f["channel"],
        "options.step.init.data_description.personality": f["help_personality"],
        "options.step.init.data_description.refine_speech": f["help_refine_speech"],
        "options.step.init.data_description.refine_prompt": f["help_refine_prompt"],
        "options.step.init.data_description.token": f["help_token"],
        "options.step.init.data_description.assist_filter": f["help_assist"],
        "options.step.init.data_description.nlu_rag": f["help_rag"],
        "options.step.init.data_description.quiet_ack": f["help_quiet"],
        "options.step.init.data_description.calendar_llm": f["help_calendar_llm"],
        "options.step.init.data_description.allow_llm_tools": f.get(
            "help_allow_llm_tools",
            "Off by default. On: Klar still calls the chit-chat agent even if that agent can control Home Assistant. The model may turn lights and run scripts. Off: Klar skips the LLM when that agent has Assist tools.",
        ),
        "options.step.init.data_description.languages": f["help_languages"],
        "options.step.init.data_description.channel": f["help_channel"],
        "options.error.invalid_url": f["invalid"],
        "selector.engine_mode.options.local": f["local"],
        "selector.engine_mode.options.remote": f["remote"],
        "selector.release_channel.options.stable": f["stable"],
        "selector.release_channel.options.staging": f["staging"],
        "selector.nlu_language.options.system": f["system"],
        "selector.nlu_language.options.all": f["all"],
        "selector.personality.options.default": normal,
        "selector.personality.options.butler": f["butler"],
        "selector.personality.options.locker": casual,
        "selector.personality.options.fuersorglich": caring,
        "selector.personality.options.party": f["party"],
        "selector.personality.options.grantig": grumpy,
        "selector.personality.options.sarkastisch": sarcastic,
        "selector.personality.options.pirat": pirate,
        "selector.personality.options.hippie": f["hippie"],
        "selector.personality.options.gollum": f["gollum"],
        "selector.personality.options.jarvis": f["jarvis"],
        "entity.select.personality.name": f["personality"],
        "entity.select.personality.state.default": normal,
        "entity.select.personality.state.butler": f["butler"],
        "entity.select.personality.state.locker": casual,
        "entity.select.personality.state.fuersorglich": caring,
        "entity.select.personality.state.party": f["party"],
        "entity.select.personality.state.grantig": grumpy,
        "entity.select.personality.state.sarkastisch": sarcastic,
        "entity.select.personality.state.pirat": pirate,
        "entity.select.personality.state.hippie": f["hippie"],
        "entity.select.personality.state.gollum": f["gollum"],
        "entity.select.personality.state.jarvis": f["jarvis"],
        "entity.switch.quiet_ack.name": f["quiet_name"],
        "entity.sensor.last_heard.name": f["heard"],
        "entity.sensor.last_decision.name": f["decision"],
        "entity.sensor.last_speech.name": f["speech"],
        "entity.sensor.last_area.name": f["area"],
        "issues.engine_down.title": f["down_title"],
        "issues.engine_down.description": f["down_desc"],
    }
