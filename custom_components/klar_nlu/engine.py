"""Download and supervise the Klar binary next to Home Assistant."""

from __future__ import annotations

import asyncio
import logging
import os
import platform
import secrets
import time
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
    engine_headers,
    engine_url_candidates,
    pick_staging_release,
    resolve_channel,
    resolve_personality,
)
from .lang_select import engine_default_language
from .time_zone import time_zone_name, time_zone_needs_sync
from .units import ha_unit_system

_LOGGER = logging.getLogger(__name__)

_ASSETS = {
    "x86_64": "klar-linux-x86_64.tar.gz",
    "amd64": "klar-linux-x86_64.tar.gz",
    "aarch64": "klar-linux-aarch64.tar.gz",
    "arm64": "klar-linux-aarch64.tar.gz",
}

_READY_TRIES = 30
# H-15: do not GET settings on every Assist turn; cap alias fan-out.
SETTINGS_CACHE_TTL_S = 15.0
SETTINGS_FETCH_BUDGET_S = 4.0

# A-11: the child must not inherit Supervisor/Hass tokens from the parent.
_ENGINE_ENV_KEEP = (
    "PATH",
    "HOME",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "TZ",
    "TMPDIR",
    "TMP",
    "TEMP",
    "USER",
    "LOGNAME",
    "XDG_RUNTIME_DIR",
    "RUST_LOG",
    "KLAR_SUPPORT_BUNDLE",
)


def _engine_child_env(token: str, ui_dir: Path) -> dict[str, str]:
    env = {key: value for key, value in os.environ.items() if key in _ENGINE_ENV_KEEP and value}
    env["KLAR_TOKEN"] = token
    env["KLAR_UI_DIR"] = str(ui_dir)
    return env


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
        self._supervise = False
        self.token: str | None = None

    @property
    def bindir(self) -> Path:
        return Path(self.hass.config.path("klar_nlu"))

    @property
    def binary(self) -> Path:
        return self.bindir / "klar"

    async def async_start(self) -> None:
        if await self._ping():
            self.token = await self.hass.async_add_executor_job(self._existing_token)
            _LOGGER.info("Klar already listens on %s", DEFAULT_URL)
            await self.async_check_version(DEFAULT_URL, self.token)
            return
        await self._ensure_binary()
        await self._spawn()
        try:
            await self._wait_ready()
        except Exception:
            # H-14: a failed ready wait must not leave an orphan child.
            await self.async_stop()
            raise
        ir.async_delete_issue(self.hass, DOMAIN, "engine_down")
        await self.async_check_version(DEFAULT_URL, self.token)
        self._supervise = True
        asyncio.create_task(self._watch())

    async def async_stop(self) -> None:
        self._supervise = False
        await self._kill_child()

    async def _kill_child(self) -> None:
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
        except (ClientError, RuntimeError, UnsupportedMachineError) as err:
            # H-14: staging (and any download failure) still starts from a local binary.
            if present:
                _LOGGER.warning("Using existing Klar binary offline: %s", err)
                return
            if isinstance(err, UnsupportedMachineError):
                raise
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

    def _existing_token(self) -> str | None:
        path = self.bindir / "token"
        if not path.is_file():
            return None
        token = path.read_text(encoding="utf-8").strip()
        return token or None

    def _ensure_token(self) -> str:
        path = self.bindir / "token"
        self.bindir.mkdir(parents=True, exist_ok=True)
        token = path.read_text(encoding="utf-8").strip() if path.is_file() else ""
        if not token:
            token = secrets.token_hex(16)
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(token)
        os.chmod(path, 0o600)
        return token

    async def _spawn(self) -> None:
        self.token = await self.hass.async_add_executor_job(self._ensure_token)
        env = _engine_child_env(self.token, self.bindir / "ui")
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
        """Restart a crashed child with exponential backoff (H-14)."""
        delay = 1.0
        while self._supervise:
            proc = self._proc
            if proc is not None:
                code = await proc.wait()
                if not self._supervise:
                    return
                ir.async_create_issue(
                    self.hass,
                    DOMAIN,
                    "engine_down",
                    is_fixable=False,
                    severity=ir.IssueSeverity.ERROR,
                    translation_key="engine_down",
                )
                _LOGGER.warning(
                    "Klar engine exited (%s); restarting in %.1fs", code, delay
                )
                await asyncio.sleep(delay)
                if not self._supervise:
                    return
            try:
                await self._spawn()
                await self._wait_ready()
            except Exception as err:  # noqa: BLE001 — keep trying with backoff
                _LOGGER.error("Klar engine restart failed: %s", err)
                await self._kill_child()
                delay = min(delay * 2.0, 60.0)
                await asyncio.sleep(delay)
                continue
            ir.async_delete_issue(self.hass, DOMAIN, "engine_down")
            delay = 1.0

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
        for path in ("/api/v2/health", "/"):
            try:
                async with session.get(
                    DEFAULT_URL + path,
                    timeout=ClientTimeout(total=2),
                ) as resp:
                    if resp.status < 500:
                        return True
            except (ClientError, TimeoutError, OSError):
                continue
        return False

    async def async_check_version(self, url: str, token: str | None = None) -> None:
        await async_check_engine_version(self.hass, url, token)


async def async_check_engine_version(
    hass: HomeAssistant, url: str, token: str | None = None
) -> None:
    """Warn on integration/engine CalVer mix instead of looking 'not responding' (R-19)."""
    payload = await async_fetch_health(hass, url, token)
    if payload is None:
        return
    engine_version = str(payload.get("version") or "").lstrip("v")
    wanted = ENGINE_VERSION.lstrip("v")
    if not engine_version or engine_version == wanted:
        ir.async_delete_issue(hass, DOMAIN, "version_mismatch")
        return
    ir.async_create_issue(
        hass,
        DOMAIN,
        "version_mismatch",
        is_fixable=False,
        severity=ir.IssueSeverity.WARNING,
        translation_key="version_mismatch",
        translation_placeholders={
            "integration": wanted,
            "engine": engine_version,
        },
    )


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


def cached_unit_system(hass: HomeAssistant) -> str:
    # H-05: Home Assistant's configured units win over a stale engine cache.
    if ha_unit_system(hass) == "imperial":
        return "imperial"
    stored = hass.data.get(DOMAIN) or {}
    for payload in stored.values():
        if not isinstance(payload, dict):
            continue
        settings = payload.get("engine_settings")
        if isinstance(settings, dict) and str(settings.get("unit_system") or "").lower() == "imperial":
            return "imperial"
    return "metric"


def store_engine_settings(
    hass: HomeAssistant, entry: ConfigEntry, payload: dict[str, object]
) -> None:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id)
    if stored is not None:
        stored["engine_settings"] = payload
        stored["engine_settings_at"] = time.monotonic()


async def async_fetch_settings(
    hass: HomeAssistant, url: str, token: str | None = None
) -> dict[str, object] | None:
    session = async_get_clientsession(hass)
    headers = engine_headers(token)
    deadline = time.monotonic() + SETTINGS_FETCH_BUDGET_S
    for base in engine_url_candidates(url):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        timeout = ClientTimeout(total=min(3.0, remaining))
        try:
            async with session.get(
                f"{base}/api/settings",
                headers=headers,
                timeout=timeout,
            ) as resp:
                resp.raise_for_status()
                payload = await resp.json()
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            _LOGGER.debug("Klar settings not fetched from %s: %s", base, err)
            continue
        return payload if isinstance(payload, dict) else None
    return None


async def async_fetch_health(
    hass: HomeAssistant, url: str, token: str | None = None
) -> dict[str, object] | None:
    """Read `GET /api/v2/health` for version compare and doctor pages (R-19)."""
    session = async_get_clientsession(hass)
    headers = engine_headers(token)
    for base in engine_url_candidates(url):
        try:
            async with session.get(
                f"{base}/api/v2/health",
                headers=headers,
                timeout=ClientTimeout(total=2),
            ) as resp:
                if resp.status >= 400:
                    continue
                payload = await resp.json()
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            _LOGGER.debug("Klar health not fetched from %s: %s", base, err)
            continue
        return payload if isinstance(payload, dict) else None
    return None


async def async_put_settings(
    hass: HomeAssistant,
    url: str,
    payload: dict[str, object],
    token: str | None = None,
) -> dict[str, object] | None:
    session = async_get_clientsession(hass)
    headers = engine_headers(token)
    deadline = time.monotonic() + SETTINGS_FETCH_BUDGET_S
    for base in engine_url_candidates(url):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        timeout = ClientTimeout(total=min(3.0, remaining))
        try:
            async with session.post(
                f"{base}/api/settings",
                json=payload,
                headers=headers,
                timeout=timeout,
            ) as resp:
                resp.raise_for_status()
                body = await resp.json()
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            _LOGGER.debug("Klar settings not written to %s: %s", base, err)
            continue
        return body if isinstance(body, dict) else payload
    return None


async def async_patch_settings(
    hass: HomeAssistant,
    url: str,
    patch: dict[str, object],
    token: str | None = None,
) -> dict[str, object] | None:
    """Merge on the engine (D-08). Falls back to full POST only when PATCH is missing."""
    session = async_get_clientsession(hass)
    headers = engine_headers(token)
    deadline = time.monotonic() + SETTINGS_FETCH_BUDGET_S
    for base in engine_url_candidates(url):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        timeout = ClientTimeout(total=min(3.0, remaining))
        try:
            async with session.patch(
                f"{base}/api/settings",
                json=patch,
                headers=headers,
                timeout=timeout,
            ) as resp:
                if resp.status in {404, 405}:
                    return None
                resp.raise_for_status()
                body = await resp.json()
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            _LOGGER.debug("Klar settings not patched at %s: %s", base, err)
            continue
        return body if isinstance(body, dict) else None
    return None


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
    hass: HomeAssistant, entry: ConfigEntry, *, force: bool = False
) -> dict[str, object]:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id) or {}
    cached = cached_engine_settings(hass, entry)
    fetched_at = stored.get("engine_settings_at")
    if (
        not force
        and cached
        and isinstance(fetched_at, (int, float))
        and time.monotonic() - float(fetched_at) < SETTINGS_CACHE_TTL_S
    ):
        return cached
    url, token = _entry_url_token(hass, entry)
    fetched = await async_fetch_settings(hass, url, token)
    if fetched is not None:
        store_engine_settings(hass, entry, fetched)
        return fetched
    return cached


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
    """Apply a partial settings update without rewriting the whole document (D-08)."""
    url, token = _entry_url_token(hass, entry)
    patched = await async_patch_settings(hass, url, patch, token)
    if patched is not None:
        store_engine_settings(hass, entry, patched)
        return patched
    # Older engines: refresh first so a stale cache cannot wipe operator edits.
    current = await async_refresh_engine_settings(hass, entry, force=True)
    if not current:
        return None
    posted = await async_put_settings(hass, url, {**current, **patch}, token)
    if posted is not None:
        store_engine_settings(hass, entry, posted)
    return posted


async def async_sync_time_zone(hass: HomeAssistant, entry: ConfigEntry) -> None:
    """Keep the engine clock on Home Assistant's configured timezone."""
    zone = time_zone_name(getattr(getattr(hass, "config", None), "time_zone", None))
    if not time_zone_needs_sync(cached_engine_settings(hass, entry), zone):
        return
    await async_patch_engine_settings(hass, entry, {"time_zone": zone})


async def async_sync_default_language(hass: HomeAssistant, entry: ConfigEntry) -> None:
    """When the HA option is "system", set engine default_language from HA's language."""
    wanted = engine_default_language(
        entry.options.get(CONF_LANGUAGES),
        getattr(getattr(hass, "config", None), "language", None),
    )
    current = str(cached_engine_settings(hass, entry).get("default_language") or "")
    if current == wanted:
        return
    await async_patch_engine_settings(hass, entry, {"default_language": wanted})


async def async_sync_unit_system(hass: HomeAssistant, entry: ConfigEntry) -> None:
    """Copy Home Assistant's unit system onto the engine at setup (H-05)."""
    wanted = ha_unit_system(hass)
    current = str(cached_engine_settings(hass, entry).get("unit_system") or "").lower()
    if current == wanted:
        return
    await async_patch_engine_settings(hass, entry, {"unit_system": wanted})


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
    headers = engine_headers(token)
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
