"""Download and supervise the Klar binary next to Home Assistant."""

from __future__ import annotations

import asyncio
import logging
import os
import platform
import secrets
from pathlib import Path

from aiohttp import ClientError, ClientSession, ClientTimeout
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers import issue_registry as ir
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .archive import extract_klar_archive, require_sha256
from .const import (
    CHANNEL_STAGING,
    CONF_ALLOW_LLM_TOOLS,
    CONF_CALENDAR_LLM,
    CONF_LANGUAGES,
    CONF_NLU_RAG,
    CONF_PERSONALITY,
    CONF_PRODUCT_IN_ENGINE,
    CONF_QUIET_ACK,
    CONF_REFINE_PROMPT,
    CONF_REFINE_SPEECH,
    CONF_TOKEN,
    CONF_URL,
    DEFAULT_ALLOW_LLM_TOOLS,
    DEFAULT_CALENDAR_LLM,
    DEFAULT_NLU_RAG,
    DEFAULT_PERSONALITY,
    DEFAULT_QUIET_ACK,
    DEFAULT_REFINE_SPEECH,
    DEFAULT_URL,
    DOMAIN,
    ENGINE_VERSION,
    GITHUB_REPO,
    LANGUAGE_ALL,
    LANGUAGE_SYSTEM,
    pick_staging_release,
    resolve_channel,
    resolve_personality,
)

_LOGGER = logging.getLogger(__name__)

_ASSETS = {
    "x86_64": "klar-linux-x86_64.tar.gz",
    "amd64": "klar-linux-x86_64.tar.gz",
    "aarch64": "klar-linux-aarch64.tar.gz",
    "arm64": "klar-linux-aarch64.tar.gz",
}

_READY_TRIES = 30


def _read_stamp(path: Path) -> str:
    return path.read_text(encoding="utf-8").strip() if path.is_file() else ""


def _release_urls(version: str) -> tuple[str, ...]:
    tag = version.lstrip("v")
    base = f"https://api.github.com/repos/{GITHUB_REPO}/releases/tags"
    return (f"{base}/{tag}", f"{base}/v{tag}")


class UnsupportedMachineError(RuntimeError):
    """Home Assistant is running on an architecture we do not ship."""


class KlarEngine:
    """Klar child process started by the integration."""

    def __init__(self, hass: HomeAssistant, channel: str = "stable") -> None:
        self.hass = hass
        self.channel = resolve_channel(channel)
        self._proc: asyncio.subprocess.Process | None = None
        self.token: str | None = None

    @property
    def bindir(self) -> Path:
        return Path(self.hass.config.path("klar_nlu"))

    @property
    def binary(self) -> Path:
        return self.bindir / "klar"

    async def async_start(self) -> None:
        if await self._ping():
            _LOGGER.info("Klar already listens on %s", DEFAULT_URL)
            return
        await self._ensure_binary()
        await self._spawn()
        await self._wait_ready()
        ir.async_delete_issue(self.hass, DOMAIN, "engine_down")
        asyncio.create_task(self._watch())

    async def async_stop(self) -> None:
        proc = self._proc
        self._proc = None
        if proc is None or proc.returncode is not None:
            return
        proc.terminate()
        try:
            await asyncio.wait_for(proc.wait(), 8)
        except TimeoutError:
            proc.kill()
            await proc.wait()

    def _stamp_for(self, version: str) -> str:
        return f"{self.channel}:{version.lstrip('v')}"

    async def _ensure_binary(self) -> None:
        stamp = self.bindir / "version"
        current = await self.hass.async_add_executor_job(_read_stamp, stamp)
        present = await self.hass.async_add_executor_job(self.binary.is_file)
        session = async_get_clientsession(self.hass)
        timeout = ClientTimeout(total=180)
        if self.channel != CHANNEL_STAGING:
            wanted = self._stamp_for(ENGINE_VERSION)
            if present and current in {wanted, ENGINE_VERSION}:
                return
        try:
            release = await self._fetch_release(session, timeout)
            version = str(release.get("tag_name") or ENGINE_VERSION).lstrip("v")
            wanted = self._stamp_for(version)
            present = await self.hass.async_add_executor_job(self.binary.is_file)
            if present and current == wanted:
                return
            machine = platform.machine().lower()
            asset = _ASSETS.get(machine)
            if asset is None:
                raise UnsupportedMachineError(
                    f"No Klar build for {machine}. Run the engine yourself and pick the URL."
                )
            chosen = next(
                (
                    item
                    for item in release.get("assets") or []
                    if item.get("name") == asset
                ),
                None,
            )
            if not chosen:
                raise RuntimeError(f"Release has no asset {asset}")
            async with session.get(chosen["browser_download_url"], timeout=timeout) as resp:
                resp.raise_for_status()
                blob = await resp.read()
        except ClientError as err:
            raise RuntimeError(f"Could not download Klar: {err}") from err
        require_sha256(chosen.get("digest"), blob)
        await self.hass.async_add_executor_job(self._extract, blob, wanted)

    async def _fetch_release(
        self, session: ClientSession, timeout: ClientTimeout
    ) -> dict:
        if self.channel == CHANNEL_STAGING:
            url = f"https://api.github.com/repos/{GITHUB_REPO}/releases?per_page=30"
            async with session.get(url, timeout=timeout) as resp:
                resp.raise_for_status()
                data = await resp.json()
            chosen = pick_staging_release(data)
            if chosen is not None:
                return chosen
            raise RuntimeError(
                "No staging prerelease on GitHub. Stay on stable or wait for a staging merge."
            )
        for url in _release_urls(ENGINE_VERSION):
            async with session.get(url, timeout=timeout) as resp:
                if resp.status == 404:
                    continue
                resp.raise_for_status()
                data = await resp.json()
                if isinstance(data, dict):
                    return data
        raise RuntimeError(
            f"No GitHub release {ENGINE_VERSION.lstrip('v')}. Start Klar yourself."
        )

    def _extract(self, blob: bytes, stamp: str) -> None:
        extract_klar_archive(blob, self.bindir)
        (self.bindir / "version").write_text(stamp, encoding="utf-8")

    def _ensure_token(self) -> str:
        path = self.bindir / "token"
        self.bindir.mkdir(parents=True, exist_ok=True)
        if path.is_file():
            token = path.read_text(encoding="utf-8").strip()
            if token:
                return token
        token = secrets.token_hex(16)
        path.write_text(token, encoding="utf-8")
        return token

    async def _spawn(self) -> None:
        self.token = await self.hass.async_add_executor_job(self._ensure_token)
        env = os.environ.copy()
        env["KLAR_TOKEN"] = self.token
        env["KLAR_UI_DIR"] = str(self.bindir / "ui")
        try:
            self._proc = await asyncio.create_subprocess_exec(
                str(self.binary),
                "--config-dir",
                str(self.hass.config.config_dir),
                "--data-dir",
                str(self.bindir),
                "--http",
                "127.0.0.1:10520",
                "--wyoming",
                "127.0.0.1:10500",
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                env=env,
            )
        except FileNotFoundError as err:
            raise RuntimeError(
                f"Klar binary not runnable at {self.binary} ({err}). "
                "Home Assistant Core is Alpine musl; the release tarball must be musl."
            ) from err
        asyncio.create_task(self._pipe_log(self._proc.stdout, logging.DEBUG))
        asyncio.create_task(self._pipe_log(self._proc.stderr, logging.WARNING))
        _LOGGER.info("Started Klar engine pid=%s", self._proc.pid)

    async def _pipe_log(self, stream: asyncio.StreamReader | None, level: int) -> None:
        if stream is None:
            return
        while True:
            line = await stream.readline()
            if not line:
                return
            _LOGGER.log(level, "klar: %s", line.decode(errors="replace").rstrip())

    async def _watch(self) -> None:
        proc = self._proc
        if proc is None:
            return
        await proc.wait()
        if self._proc is None:
            return
        ir.async_create_issue(
            self.hass,
            DOMAIN,
            "engine_down",
            is_fixable=False,
            severity=ir.IssueSeverity.ERROR,
            translation_key="engine_down",
        )

    async def _wait_ready(self) -> None:
        for _ in range(_READY_TRIES):
            if self._proc is not None and self._proc.returncode is not None:
                raise RuntimeError(f"Klar engine exited with {self._proc.returncode}")
            if await self._ping():
                return
            await asyncio.sleep(0.4)
        raise RuntimeError("Klar engine did not become ready")

    async def _ping(self) -> bool:
        session = async_get_clientsession(self.hass)
        try:
            async with session.get(
                DEFAULT_URL + "/",
                timeout=ClientTimeout(total=2),
            ) as resp:
                return resp.status < 500
        except ClientError:
            return False


def merge_engine_settings(
    data: object,
    personality: str,
    languages: list[str] | None,
    pipeline: dict[str, object] | None = None,
) -> dict | None:
    if not isinstance(data, dict):
        return None
    out = dict(data)
    out["personality"] = resolve_personality(personality)
    if languages is not None:
        out["languages"] = list(languages)
    if pipeline:
        for key, value in pipeline.items():
            out[key] = value
    return out


def merge_ui_locale(data: object, locale: str) -> dict | None:
    if not isinstance(data, dict):
        return None
    out = dict(data)
    out["locale"] = locale
    return out


def product_options_nondefault(options: object) -> bool:
    """True when leftover HA options still carry operator product knobs."""
    if not isinstance(options, dict):
        return False
    if resolve_personality(options.get(CONF_PERSONALITY)) != DEFAULT_PERSONALITY:
        return True
    languages = options.get(CONF_LANGUAGES)
    if isinstance(languages, list):
        pinned = [code for code in languages if code not in {LANGUAGE_SYSTEM, LANGUAGE_ALL}]
        if pinned:
            return True
    elif languages not in {None, "", LANGUAGE_SYSTEM, LANGUAGE_ALL}:
        return True
    if str(options.get(CONF_REFINE_PROMPT) or "").strip():
        return True
    flags = (
        (CONF_REFINE_SPEECH, DEFAULT_REFINE_SPEECH),
        (CONF_NLU_RAG, DEFAULT_NLU_RAG),
        (CONF_QUIET_ACK, DEFAULT_QUIET_ACK),
        (CONF_CALENDAR_LLM, DEFAULT_CALENDAR_LLM),
        (CONF_ALLOW_LLM_TOOLS, DEFAULT_ALLOW_LLM_TOOLS),
    )
    return any(bool(options.get(key, default)) != bool(default) for key, default in flags)


def cached_engine_settings(hass: HomeAssistant, entry: ConfigEntry) -> dict[str, object]:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id) or {}
    payload = stored.get("engine_settings")
    return payload if isinstance(payload, dict) else {}


def store_engine_settings(
    hass: HomeAssistant, entry: ConfigEntry, payload: dict[str, object]
) -> None:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id)
    if stored is not None:
        stored["engine_settings"] = payload


def _settings_headers(token: str | None) -> dict[str, str]:
    return {"X-Klar-Token": token} if token else {}


async def async_fetch_settings(
    hass: HomeAssistant, url: str, token: str | None = None
) -> dict[str, object] | None:
    session = async_get_clientsession(hass)
    timeout = ClientTimeout(total=3)
    try:
        async with session.get(
            f"{url.rstrip('/')}/api/settings",
            headers=_settings_headers(token),
            timeout=timeout,
        ) as resp:
            resp.raise_for_status()
            payload = await resp.json()
    except (ClientError, TimeoutError, OSError, ValueError) as err:
        _LOGGER.debug("Klar settings not fetched: %s", err)
        return None
    return payload if isinstance(payload, dict) else None


async def async_put_settings(
    hass: HomeAssistant,
    url: str,
    payload: dict[str, object],
    token: str | None = None,
) -> dict[str, object] | None:
    session = async_get_clientsession(hass)
    timeout = ClientTimeout(total=3)
    try:
        async with session.post(
            f"{url.rstrip('/')}/api/settings",
            json=payload,
            headers=_settings_headers(token),
            timeout=timeout,
        ) as resp:
            resp.raise_for_status()
            body = await resp.json()
    except (ClientError, TimeoutError, OSError, ValueError) as err:
        _LOGGER.debug("Klar settings not written: %s", err)
        return None
    return body if isinstance(body, dict) else payload


def _entry_url_token(hass: HomeAssistant, entry: ConfigEntry) -> tuple[str, str | None]:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id) or {}
    url = str(
        stored.get("url")
        or entry.options.get(CONF_URL)
        or entry.data.get(CONF_URL)
        or DEFAULT_URL
    )
    token = stored.get("token") or entry.options.get(CONF_TOKEN) or entry.data.get(CONF_TOKEN)
    return url.rstrip("/"), str(token) if token else None


async def async_refresh_engine_settings(
    hass: HomeAssistant, entry: ConfigEntry
) -> dict[str, object]:
    url, token = _entry_url_token(hass, entry)
    fetched = await async_fetch_settings(hass, url, token)
    if fetched is not None:
        store_engine_settings(hass, entry, fetched)
        return fetched
    return cached_engine_settings(hass, entry)


async def async_seed_product_settings(
    hass: HomeAssistant,
    entry: ConfigEntry,
    url: str,
    personality: str,
    token: str | None = None,
    languages: list[str] | None = None,
    ui_locale: str | None = None,
    pipeline: dict[str, object] | None = None,
) -> dict[str, object] | None:
    """Copy leftover HA product options onto the engine once, then stop overwriting."""
    del ui_locale
    current = await async_fetch_settings(hass, url, token)
    if current is None:
        return None
    migrated = bool(entry.data.get(CONF_PRODUCT_IN_ENGINE))
    payload: dict[str, object] = current
    if not migrated and product_options_nondefault(dict(entry.options)):
        merged = merge_engine_settings(current, personality, languages, pipeline)
        if merged is None:
            return current
        posted = await async_put_settings(hass, url, merged, token)
        if posted is None:
            store_engine_settings(hass, entry, current)
            return current
        payload = posted
        migrated = True
        hass.config_entries.async_update_entry(
            entry, data={**dict(entry.data), CONF_PRODUCT_IN_ENGINE: True}
        )
    elif not migrated:
        hass.config_entries.async_update_entry(
            entry, data={**dict(entry.data), CONF_PRODUCT_IN_ENGINE: True}
        )
    store_engine_settings(hass, entry, payload)
    return payload


async def async_patch_engine_settings(
    hass: HomeAssistant, entry: ConfigEntry, patch: dict[str, object]
) -> dict[str, object] | None:
    url, token = _entry_url_token(hass, entry)
    current = cached_engine_settings(hass, entry)
    if not current:
        fetched = await async_fetch_settings(hass, url, token)
        current = fetched or {}
    if not current:
        return None
    posted = await async_put_settings(hass, url, {**current, **patch}, token)
    if posted is not None:
        store_engine_settings(hass, entry, posted)
    return posted


async def async_push_fallback_flag(
    hass: HomeAssistant, entry: ConfigEntry, fallback_llm: bool
) -> None:
    """HA leftover agent is glue; keep the engine boolean in sync without touching voice."""
    await async_patch_engine_settings(hass, entry, {"fallback_llm": fallback_llm})


async def async_push_llm_endpoint(
    hass: HomeAssistant,
    url: str,
    token: str | None,
    agent_id: object,
) -> None:
    """Copy an OpenAI-compatible agent onto the engine. Never stores the key in HA options."""
    from .const import engine_url_candidates
    from .llm_endpoint import openai_compatible_endpoint

    payload = openai_compatible_endpoint(hass, str(agent_id) if agent_id else None)
    if payload is None:
        return
    session = async_get_clientsession(hass)
    headers = {"X-Klar-Token": token} if token else {}
    timeout = ClientTimeout(total=3)
    body = {**payload, "configured": True}
    for base in engine_url_candidates(url):
        try:
            async with session.post(
                f"{base}/api/v2/llm/endpoint",
                json=body,
                headers=headers,
                timeout=timeout,
            ) as resp:
                if resp.status < 400:
                    return
        except (ClientError, TimeoutError, OSError) as err:
            _LOGGER.debug("Klar LLM endpoint not synced: %s", err)
