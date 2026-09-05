from __future__ import annotations

from typing import Any
from urllib.parse import urlparse

import voluptuous as vol
from homeassistant import config_entries
from homeassistant.core import callback
from homeassistant.data_entry_flow import FlowResult
from homeassistant.helpers import selector

from .const import (
    CHANNEL_STABLE,
    CHANNEL_STAGING,
    CONF_ALLOW_LLM_TOOLS,
    CONF_ASSIST_FILTER,
    CONF_CALENDAR_LLM,
    CONF_CHANNEL,
    CONF_FALLBACK_AGENT,
    CONF_LANGUAGES,
    CONF_MODE,
    CONF_NLU_RAG,
    CONF_PERSONALITY,
    CONF_QUIET_ACK,
    CONF_REFINE_PROMPT,
    CONF_REFINE_SPEECH,
    CONF_TOKEN,
    CONF_URL,
    DEFAULT_ASSIST_FILTER,
    DEFAULT_CHANNEL,
    DEFAULT_URL,
    DOMAIN,
    MODE_LOCAL,
    MODE_REMOTE,
    channel_for_addon_slug,
    is_managed_engine_url,
    resolve_channel,
    resolve_engine_target,
)

# Leftover product knobs from older options forms. Assist falls back to these
# only when the engine settings cache is empty. The operator UI owns them now.
_PRODUCT_OPTION_KEYS = (
    CONF_PERSONALITY,
    CONF_LANGUAGES,
    CONF_REFINE_SPEECH,
    CONF_REFINE_PROMPT,
    CONF_NLU_RAG,
    CONF_QUIET_ACK,
    CONF_CALENDAR_LLM,
    CONF_ALLOW_LLM_TOOLS,
)


def _on_supervisor(hass: Any) -> bool:
    components = getattr(getattr(hass, "config", None), "components", None) or []
    return "hassio" in components


def _options_schema() -> vol.Schema:
    return vol.Schema(
        {
            vol.Optional(CONF_MODE, default=MODE_LOCAL): selector.SelectSelector(
                selector.SelectSelectorConfig(
                    options=[MODE_LOCAL, MODE_REMOTE],
                    translation_key="engine_mode",
                    mode=selector.SelectSelectorMode.LIST,
                )
            ),
            vol.Optional(CONF_FALLBACK_AGENT): selector.ConversationAgentSelector(
                selector.ConversationAgentSelectorConfig()
            ),
            vol.Optional(CONF_URL): str,
            vol.Optional(CONF_TOKEN): str,
            vol.Optional(CONF_ASSIST_FILTER, default=DEFAULT_ASSIST_FILTER): (
                selector.BooleanSelector()
            ),
            vol.Optional(CONF_CHANNEL, default=DEFAULT_CHANNEL): selector.SelectSelector(
                selector.SelectSelectorConfig(
                    options=[CHANNEL_STABLE, CHANNEL_STAGING],
                    translation_key="release_channel",
                    mode=selector.SelectSelectorMode.LIST,
                )
            ),
        }
    )


USER_SCHEMA = vol.Schema(
    {
        vol.Required(CONF_MODE, default=MODE_LOCAL): selector.SelectSelector(
            selector.SelectSelectorConfig(
                options=[MODE_LOCAL, MODE_REMOTE],
                translation_key="engine_mode",
                mode=selector.SelectSelectorMode.LIST,
            )
        ),
        vol.Optional(CONF_CHANNEL, default=DEFAULT_CHANNEL): selector.SelectSelector(
            selector.SelectSelectorConfig(
                options=[CHANNEL_STABLE, CHANNEL_STAGING],
                translation_key="release_channel",
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


def _keep_product_options(existing: dict[str, Any]) -> dict[str, Any]:
    return {key: existing[key] for key in _PRODUCT_OPTION_KEYS if key in existing}


class KlarConfigFlow(config_entries.ConfigFlow, domain=DOMAIN):
    VERSION = 1

    async def async_step_user(self, user_input: dict | None = None) -> FlowResult:
        if self._async_current_entries():
            return self.async_abort(reason="already_configured")
        if user_input is not None:
            await self.async_set_unique_id(DOMAIN)
            mode, url = resolve_engine_target(
                mode=user_input.get(CONF_MODE, MODE_LOCAL),
                channel=resolve_channel(user_input.get(CONF_CHANNEL)),
                url=user_input.get(CONF_URL),
                supervisor=_on_supervisor(self.hass),
            )
            channel = resolve_channel(user_input.get(CONF_CHANNEL))
            if not _valid_engine_url(url):
                return self.async_show_form(
                    step_id="user",
                    data_schema=USER_SCHEMA,
                    errors={"base": "invalid_url"},
                )
            data = {
                CONF_MODE: mode,
                CONF_URL: url,
                CONF_CHANNEL: channel,
            }
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
            {
                CONF_MODE: MODE_REMOTE,
                CONF_URL: f"http://{host}.local.hass.io:10520",
                CONF_CHANNEL: channel_for_addon_slug(slug),
            }
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
            data = _keep_product_options(dict(self.config_entry.options))
            agent = user_input.get(CONF_FALLBACK_AGENT) or None
            if agent:
                data[CONF_FALLBACK_AGENT] = agent
            channel = resolve_channel(user_input.get(CONF_CHANNEL))
            mode, url = resolve_engine_target(
                mode=user_input.get(
                    CONF_MODE,
                    self.config_entry.options.get(
                        CONF_MODE, self.config_entry.data.get(CONF_MODE, MODE_LOCAL)
                    ),
                ),
                channel=channel,
                url=(user_input.get(CONF_URL) or "").strip()
                or self.config_entry.options.get(CONF_URL)
                or self.config_entry.data.get(CONF_URL)
                or "",
                supervisor=_on_supervisor(self.hass),
            )
            data[CONF_MODE] = mode
            data[CONF_CHANNEL] = channel
            if url:
                if not _valid_engine_url(url):
                    return self.async_show_form(
                        step_id="init",
                        data_schema=self.add_suggested_values_to_schema(
                            _options_schema(), user_input
                        ),
                        errors={"base": "invalid_url"},
                    )
                data[CONF_URL] = url
            self.hass.config_entries.async_update_entry(
                self.config_entry,
                data={
                    **dict(self.config_entry.data),
                    CONF_MODE: mode,
                    CONF_URL: url,
                    CONF_CHANNEL: channel,
                },
            )
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
            CONF_ASSIST_FILTER: DEFAULT_ASSIST_FILTER,
            CONF_MODE: self.config_entry.options.get(
                CONF_MODE, self.config_entry.data.get(CONF_MODE, MODE_LOCAL)
            ),
            CONF_CHANNEL: resolve_channel(
                self.config_entry.options.get(
                    CONF_CHANNEL, self.config_entry.data.get(CONF_CHANNEL)
                )
            ),
            **self.config_entry.options,
        }
        if CONF_URL not in suggested:
            suggested[CONF_URL] = self.config_entry.data.get(CONF_URL, "")
        suggested[CONF_CHANNEL] = resolve_channel(
            suggested.get(CONF_CHANNEL, self.config_entry.data.get(CONF_CHANNEL))
        )
        suggested[CONF_MODE] = suggested.get(
            CONF_MODE, self.config_entry.data.get(CONF_MODE, MODE_LOCAL)
        )
        if is_managed_engine_url(suggested.get(CONF_URL)):
            suggested[CONF_MODE], suggested[CONF_URL] = resolve_engine_target(
                mode=suggested[CONF_MODE],
                channel=suggested[CONF_CHANNEL],
                url=suggested.get(CONF_URL),
                supervisor=_on_supervisor(self.hass),
            )
        return self.async_show_form(
            step_id="init",
            data_schema=self.add_suggested_values_to_schema(_options_schema(), suggested),
        )
