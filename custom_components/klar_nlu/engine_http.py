"""POST to the Klar engine, trying Supervisor host aliases when needed."""

from __future__ import annotations

from typing import Any

import aiohttp

from .const import engine_url_candidates
from .contracts import validate_v2_payload


async def post_parse(
    session: aiohttp.ClientSession,
    url: str,
    body: dict[str, Any],
    headers: dict[str, str],
) -> tuple[dict[str, Any] | None, Exception | None]:
    last_err: Exception | None = None
    for base in engine_url_candidates(url):
        try:
            async with session.post(
                f"{base}/api/v2/parse",
                json=body,
                headers=headers,
                timeout=aiohttp.ClientTimeout(total=5),
            ) as resp:
                resp.raise_for_status()
                return validate_v2_payload(await resp.json()), None
        except Exception as err:  # noqa: BLE001 — boundary to the local engine
            last_err = err
    return None, last_err
