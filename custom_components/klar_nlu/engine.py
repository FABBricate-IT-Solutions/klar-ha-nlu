"""Download and supervise the Klar binary next to Home Assistant."""

from __future__ import annotations

import asyncio
import logging
import platform
import tarfile
from io import BytesIO
from pathlib import Path

from aiohttp import ClientError, ClientTimeout
from homeassistant.core import HomeAssistant
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .const import DEFAULT_URL, GITHUB_REPO

_LOGGER = logging.getLogger(__name__)

_ASSETS = {
    "x86_64": "klar-linux-x86_64.tar.gz",
    "amd64": "klar-linux-x86_64.tar.gz",
    "aarch64": "klar-linux-aarch64.tar.gz",
    "arm64": "klar-linux-aarch64.tar.gz",
    "armv7l": "klar-linux-armv7.tar.gz",
    "armv7": "klar-linux-armv7.tar.gz",
}

_RELEASES = f"https://api.github.com/repos/{GITHUB_REPO}/releases/latest"
_READY_TRIES = 30


class UnsupportedMachineError(RuntimeError):
    """Home Assistant is running on an architecture we do not ship."""


class KlarEngine:
    """Klar child process started by the integration."""

    def __init__(self, hass: HomeAssistant) -> None:
        self.hass = hass
        self._proc: asyncio.subprocess.Process | None = None

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
            async with session.get(_RELEASES, timeout=timeout) as resp:
                if resp.status == 404:
                    raise RuntimeError(
                        "No GitHub release yet. Tag v0.1.0 or start Klar yourself."
                    )
                resp.raise_for_status()
                release = await resp.json()
            url = next(
                (
                    item["browser_download_url"]
                    for item in release.get("assets") or []
                    if item.get("name") == asset
                ),
                None,
            )
            if not url:
                raise RuntimeError(f"Release has no asset {asset}")
            async with session.get(url, timeout=timeout) as resp:
                resp.raise_for_status()
                blob = await resp.read()
        except ClientError as err:
            raise RuntimeError(f"Could not download Klar: {err}") from err
        await self.hass.async_add_executor_job(self._extract, blob)

    def _extract(self, blob: bytes) -> None:
        self.bindir.mkdir(parents=True, exist_ok=True)
        with tarfile.open(fileobj=BytesIO(blob), mode="r:gz") as tar:
            member = next(
                (item for item in tar.getmembers() if item.isfile()),
                None,
            )
            if member is None:
                raise RuntimeError("Klar archive is empty")
            extracted = tar.extractfile(member)
            if extracted is None:
                raise RuntimeError("Klar archive could not be read")
            self.binary.write_bytes(extracted.read())
        self.binary.chmod(0o755)

    async def _spawn(self) -> None:
        self._proc = await asyncio.create_subprocess_exec(
            str(self.binary),
            "--config-dir",
            str(self.hass.config.config_dir),
            "--http",
            "127.0.0.1:10520",
            "--wyoming",
            "127.0.0.1:10500",
            stdout=asyncio.subprocess.DEVNULL,
            stderr=asyncio.subprocess.DEVNULL,
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
