from __future__ import annotations

from typing import Any
from urllib.parse import urlparse

import voluptuous as vol
from homeassistant import config_entries
from homeassistant.core import callback
from homeassistant.data_entry_flow import FlowResult
from homeassistant.helpers import selector

from .const import (
    CONF_ASSIST_FILTER,
    CONF_FALLBACK_AGENT,
    CONF_LANGUAGES,
    CONF_MODE,
    CONF_PERSONALITY,
    CONF_REFINE_PROMPT,
    CONF_REFINE_SPEECH,
    CONF_TOKEN,
    CONF_URL,
    DEFAULT_ASSIST_FILTER,
    DEFAULT_PERSONALITY,
    DEFAULT_REFINE_PROMPT,
    DEFAULT_REFINE_SPEECH,
    DEFAULT_URL,
    DOMAIN,
    MODE_LOCAL,
    MODE_REMOTE,
    PERSONALITIES,
    SUPPORTED_LANGUAGES,
    resolve_personality,
)


def _options_schema(advanced: bool) -> vol.Schema:
    fields: dict[Any, Any] = {
        vol.Optional(CONF_PERSONALITY, default=DEFAULT_PERSONALITY): selector.SelectSelector(
            selector.SelectSelectorConfig(
                options=list(PERSONALITIES),
                mode=selector.SelectSelectorMode.DROPDOWN,
                translation_key="personality",
            )
        ),
        vol.Optional(CONF_LANGUAGES, default=list(SUPPORTED_LANGUAGES)): selector.SelectSelector(
            selector.SelectSelectorConfig(
                options=list(SUPPORTED_LANGUAGES),
                multiple=True,
                mode=selector.SelectSelectorMode.LIST,
                translation_key="languages",
            )
        ),
        vol.Optional(CONF_FALLBACK_AGENT): selector.ConversationAgentSelector(
            selector.ConversationAgentSelectorConfig()
        ),
        vol.Optional(CONF_REFINE_SPEECH, default=DEFAULT_REFINE_SPEECH): (
            selector.BooleanSelector()
        ),
        vol.Optional(CONF_REFINE_PROMPT, default=DEFAULT_REFINE_PROMPT): (
            selector.TextSelector(
                selector.TextSelectorConfig(
                    multiline=True,
                )
            )
        ),
        vol.Optional(CONF_URL): str,
        vol.Optional(CONF_TOKEN): str,
    }
    if advanced:
        fields[vol.Optional(CONF_ASSIST_FILTER, default=DEFAULT_ASSIST_FILTER)] = (
            selector.BooleanSelector()
        )
    return vol.Schema(fields)

USER_SCHEMA = vol.Schema(
    {
        vol.Required(CONF_MODE, default=MODE_LOCAL): selector.SelectSelector(
            selector.SelectSelectorConfig(
                options=[MODE_LOCAL, MODE_REMOTE],
                translation_key="engine_mode",
                mode=selector.SelectSelectorMode.LIST,
            )
        ),
        vol.Optional(CONF_URL, default=DEFAULT_URL): str,
        vol.Optional(CONF_TOKEN): str,
    }
)


def _valid_engine_url(url: str) -> bool:
    parsed = urlparse(url)
    return parsed.scheme in {"http", "https"} and bool(parsed.netloc) and not parsed.username


class KlarConfigFlow(config_entries.ConfigFlow, domain=DOMAIN):
    VERSION = 1

    async def async_step_user(self, user_input: dict | None = None) -> FlowResult:
        if self._async_current_entries():
            return self.async_abort(reason="already_configured")
        if user_input is not None:
            await self.async_set_unique_id(DOMAIN)
            mode = user_input.get(CONF_MODE, MODE_LOCAL)
            url = DEFAULT_URL if mode == MODE_LOCAL else (
                user_input.get(CONF_URL) or DEFAULT_URL
            )
            if not _valid_engine_url(url):
                return self.async_show_form(
                    step_id="user",
                    data_schema=USER_SCHEMA,
                    errors={"base": "invalid_url"},
                )
            data = {CONF_MODE: mode, CONF_URL: url}
            token = (user_input.get(CONF_TOKEN) or "").strip()
            if token:
                data[CONF_TOKEN] = token
            return self.async_create_entry(
                title="Klar NLU",
                data=data,
            )
        return self.async_show_form(step_id="user", data_schema=USER_SCHEMA)

    async def async_step_hassio(self, discovery_info) -> FlowResult:
        slug = getattr(discovery_info, "slug", None) or "klar-nlu"
        host = str(slug).replace("_", "-")
        return await self.async_step_user(
            {CONF_MODE: MODE_REMOTE, CONF_URL: f"http://{host}:10520"}
        )

    @staticmethod
    @callback
    def async_get_options_flow(
        config_entry: config_entries.ConfigEntry,
    ) -> config_entries.OptionsFlow:
        return KlarOptionsFlow()


class KlarOptionsFlow(config_entries.OptionsFlow):
    async def async_step_init(
        self, user_input: dict[str, Any] | None = None
    ) -> FlowResult:
        if user_input is not None:
            langs = [
                code
                for code in (user_input.get(CONF_LANGUAGES) or [])
                if code in SUPPORTED_LANGUAGES
            ] or list(SUPPORTED_LANGUAGES)
            personality = resolve_personality(user_input.get(CONF_PERSONALITY))
            data: dict[str, Any] = {
                CONF_LANGUAGES: langs,
                CONF_PERSONALITY: personality,
            }
            agent = user_input.get(CONF_FALLBACK_AGENT) or None
            if agent:
                data[CONF_FALLBACK_AGENT] = agent
            data[CONF_REFINE_SPEECH] = bool(user_input.get(CONF_REFINE_SPEECH))
            refine_prompt = (user_input.get(CONF_REFINE_PROMPT) or "").strip()
            if refine_prompt:
                data[CONF_REFINE_PROMPT] = refine_prompt
            url = (user_input.get(CONF_URL) or "").strip()
            if url:
                if not _valid_engine_url(url):
                    return self.async_show_form(
                        step_id="init",
                        data_schema=self.add_suggested_values_to_schema(
                            _options_schema(self.show_advanced_options), user_input
                        ),
                        errors={"base": "invalid_url"},
                    )
                data[CONF_URL] = url
            token = (user_input.get(CONF_TOKEN) or "").strip()
            if token:
                data[CONF_TOKEN] = token
            if CONF_ASSIST_FILTER in user_input:
                data[CONF_ASSIST_FILTER] = bool(user_input[CONF_ASSIST_FILTER])
            else:
                data[CONF_ASSIST_FILTER] = bool(
                    self.config_entry.options.get(
                        CONF_ASSIST_FILTER, DEFAULT_ASSIST_FILTER
                    )
                )
            return self.async_create_entry(data=data)
        suggested = {
            CONF_LANGUAGES: list(SUPPORTED_LANGUAGES),
            CONF_ASSIST_FILTER: DEFAULT_ASSIST_FILTER,
            CONF_PERSONALITY: DEFAULT_PERSONALITY,
            CONF_REFINE_PROMPT: DEFAULT_REFINE_PROMPT,
            CONF_REFINE_SPEECH: DEFAULT_REFINE_SPEECH,
            **self.config_entry.options,
        }
        if CONF_URL not in suggested:
            suggested[CONF_URL] = self.config_entry.data.get(CONF_URL, "")
        return self.async_show_form(
            step_id="init",
            data_schema=self.add_suggested_values_to_schema(
                _options_schema(self.show_advanced_options), suggested
            ),
        )
