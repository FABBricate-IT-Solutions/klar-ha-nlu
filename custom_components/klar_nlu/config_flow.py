from __future__ import annotations

from typing import Any

import voluptuous as vol
from homeassistant import config_entries
from homeassistant.core import callback
from homeassistant.data_entry_flow import FlowResult
from homeassistant.helpers import selector

from .const import (
    CONF_FALLBACK_AGENT,
    CONF_MODE,
    CONF_URL,
    DEFAULT_ADDON_URL,
    DEFAULT_URL,
    DOMAIN,
    MODE_LOCAL,
    MODE_REMOTE,
)

OPTIONS_SCHEMA = vol.Schema(
    {
        vol.Optional(CONF_FALLBACK_AGENT): selector.ConversationAgentSelector(
            selector.ConversationAgentSelectorConfig()
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
        vol.Optional(CONF_URL, default=DEFAULT_URL): str,
    }
)


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
            return self.async_create_entry(
                title="Klar NLU",
                data={CONF_MODE: mode, CONF_URL: url},
            )
        return self.async_show_form(step_id="user", data_schema=USER_SCHEMA)

    async def async_step_hassio(self, discovery_info) -> FlowResult:
        return await self.async_step_user(
            {CONF_MODE: MODE_REMOTE, CONF_URL: DEFAULT_ADDON_URL}
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
            agent = user_input.get(CONF_FALLBACK_AGENT) or None
            return self.async_create_entry(
                data={CONF_FALLBACK_AGENT: agent} if agent else {}
            )
        return self.async_show_form(
            step_id="init",
            data_schema=self.add_suggested_values_to_schema(
                OPTIONS_SCHEMA, self.config_entry.options
            ),
        )
