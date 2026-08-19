"""Download and supervise the Klar binary next to Home Assistant."""

from __future__ import annotations

import asyncio
import logging
import os
import platform
import secrets
import tarfile
from io import BytesIO
from pathlib import Path

from aiohttp import ClientError, ClientSession, ClientTimeout
from homeassistant.core import HomeAssistant
from homeassistant.helpers import issue_registry as ir
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .archive import require_sha256
from .const import (
    CHANNEL_STAGING,
    DEFAULT_URL,
    DOMAIN,
    ENGINE_VERSION,
    GITHUB_REPO,
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
        return Path(self.hass.config.path(".storage", "klar_nlu"))

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
        current = stamp.read_text(encoding="utf-8").strip() if stamp.is_file() else ""
        session = async_get_clientsession(self.hass)
        timeout = ClientTimeout(total=180)
        if self.channel != CHANNEL_STAGING:
            wanted = self._stamp_for(ENGINE_VERSION)
            if self.binary.is_file() and current in {wanted, ENGINE_VERSION}:
                return
        try:
            release = await self._fetch_release(session, timeout)
            version = str(release.get("tag_name") or ENGINE_VERSION).lstrip("v")
            wanted = self._stamp_for(version)
            if self.binary.is_file() and current == wanted:
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
        self.bindir.mkdir(parents=True, exist_ok=True)
        with tarfile.open(fileobj=BytesIO(blob), mode="r:gz") as tar:
            tar.extraction_filter = getattr(tarfile, "data_filter", tarfile.tar_filter)
            member = next(
                (
                    item
                    for item in tar.getmembers()
                    if item.isfile()
                    and Path(item.name).name == "klar"
                    and ".." not in Path(item.name).parts
                ),
                None,
            )
            if member is None:
                raise RuntimeError("Klar archive has no klar binary")
            extracted = tar.extractfile(member)
            if extracted is None:
                raise RuntimeError("Klar archive could not be read")
            self.binary.write_bytes(extracted.read())
        self.binary.chmod(0o755)
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
        self.token = self._ensure_token()
        env = os.environ.copy()
        env["KLAR_TOKEN"] = self.token
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


async def async_push_personality(
    hass: HomeAssistant, url: str, personality: str, token: str | None = None
) -> None:
    """Write the HA personality onto the engine so the Klar UI matches Assist."""
    personality = resolve_personality(personality)
    session = async_get_clientsession(hass)
    settings_url = f"{url.rstrip('/')}/api/settings"
    headers = {"X-Klar-Token": token} if token else {}
    try:
        async with session.get(
            settings_url, headers=headers, timeout=ClientTimeout(total=3)
        ) as resp:
            resp.raise_for_status()
            data = await resp.json()
        if not isinstance(data, dict):
            return
        data["personality"] = personality
        async with session.post(
            settings_url, json=data, headers=headers, timeout=ClientTimeout(total=3)
        ) as resp:
            resp.raise_for_status()
    except (ClientError, TimeoutError, OSError) as err:
        _LOGGER.debug("Klar personality not synced: %s", err)
