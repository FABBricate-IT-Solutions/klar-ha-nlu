"""Download and supervise the Klar binary next to Home Assistant."""

from __future__ import annotations

import asyncio
import hashlib
import logging
import platform
import secrets
import tarfile
from io import BytesIO
from pathlib import Path

from aiohttp import ClientError, ClientSession, ClientTimeout
from homeassistant.core import HomeAssistant
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .const import DEFAULT_URL, ENGINE_VERSION, GITHUB_REPO, PERSONALITIES

_LOGGER = logging.getLogger(__name__)

_ASSETS = {
    "x86_64": "klar-linux-x86_64.tar.gz",
    "amd64": "klar-linux-x86_64.tar.gz",
    "aarch64": "klar-linux-aarch64.tar.gz",
    "arm64": "klar-linux-aarch64.tar.gz",
    "armv7l": "klar-linux-armv7.tar.gz",
    "armv7": "klar-linux-armv7.tar.gz",
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

    def __init__(self, hass: HomeAssistant) -> None:
        self.hass = hass
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

    async def _ensure_binary(self) -> None:
        stamp = self.bindir / "version"
        if stamp.is_file() and stamp.read_text(encoding="utf-8").strip() != ENGINE_VERSION:
            self.binary.unlink(missing_ok=True)
        if self.binary.is_file():
            return
        machine = platform.machine().lower()
        asset = _ASSETS.get(machine)
        if asset is None:
            raise UnsupportedMachineError(
                f"No Klar build for {machine}. Run the engine yourself and pick the URL."
            )
        session = async_get_clientsession(self.hass)
        timeout = ClientTimeout(total=180)
        try:
            release = await self._fetch_release(session, timeout)
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
        self._check_digest(chosen.get("digest"), blob)
        await self.hass.async_add_executor_job(self._extract, blob)

    async def _fetch_release(
        self, session: ClientSession, timeout: ClientTimeout
    ) -> dict:
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

    def _check_digest(self, digest: object, blob: bytes) -> None:
        if not isinstance(digest, str) or not digest.startswith("sha256:"):
            return
        got = hashlib.sha256(blob).hexdigest()
        if got != digest.split(":", 1)[1]:
            raise RuntimeError("Klar archive checksum mismatch")

    def _extract(self, blob: bytes) -> None:
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
        (self.bindir / "version").write_text(ENGINE_VERSION, encoding="utf-8")

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
        self._proc = await asyncio.create_subprocess_exec(
            str(self.binary),
            "--config-dir",
            str(self.hass.config.config_dir),
            "--data-dir",
            str(self.bindir),
            "--token",
            self.token,
            "--http",
            "127.0.0.1:10520",
            "--wyoming",
            "127.0.0.1:10500",
        )
        _LOGGER.info("Started Klar engine pid=%s", self._proc.pid)

    async def _wait_ready(self) -> None:
        for _ in range(_READY_TRIES):
            if self._proc is not None and self._proc.returncode is not None:
                raise RuntimeError(
                    f"Klar engine exited with {self._proc.returncode}"
                )
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
    if personality not in PERSONALITIES:
        personality = "default"
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
