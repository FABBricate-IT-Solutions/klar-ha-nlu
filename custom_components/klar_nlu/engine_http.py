"""POST to the Klar engine, trying Supervisor host aliases when needed."""

from __future__ import annotations

from typing import Any

import aiohttp

from .const import engine_url_candidates
from .contracts import validate_v2_payload


class SchemaError(ValueError):
    """The engine answered, but the V2 body failed the contract (H-13)."""


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
                timeout=aiohttp.ClientTimeout(total=30),
            ) as resp:
                resp.raise_for_status()
                raw = await resp.json()
            try:
                return validate_v2_payload(raw), None
            except ValueError as err:
                # Schema failures are not "unreachable" — the engine did answer.
                return None, SchemaError(str(err))
        except Exception as err:  # noqa: BLE001 — boundary to the local engine
            # TimeoutError stringifies to "" in Python — keep a readable hint.
            last_err = TimeoutError("Klar parse timed out") if isinstance(err, TimeoutError) else err
    return None, last_err
