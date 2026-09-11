"""Shared row helper for per-locale Settings hint chrome."""

from __future__ import annotations

KEYS = (
    "modeHint",
    "confirmRiskyHint",
    "recordProtocolHint",
    "includeRawTextHint",
    "semanticAdaptersHint",
    "tokenHint",
    "refineBandStatus",
    "refineBandStatusHint",
    "refineBandCommand",
    "refineBandCommandHint",
    "refineBandPrompt",
    "refineBandPromptHint",
    "refineBandReject",
    "refineBandRejectHint",
    "refineBandsHint",
    "saveOk",
    "saveFail",
    "saveUnauthorized",
)


def row(*values: str) -> dict[str, str]:
    if len(values) != len(KEYS):
        raise SystemExit(f"settings hint row expected {len(KEYS)}, got {len(values)}")
    return dict(zip(KEYS, values, strict=True))
