"""Tab-separated operator UI catalogs, one column per locale."""

from __future__ import annotations

from lang_packs.web_ui_keys import CATALOG_KEYS, FALLBACKS


def parse_table(codes: list[str], table: str) -> dict[str, dict[str, str]]:
    packs = {code: {} for code in codes}
    seen: list[str] = []
    for raw in table.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        expected = 1 + len(codes)
        if len(parts) != expected:
            raise SystemExit(f"{codes}: expected {expected} columns, got {len(parts)} in {line[:96]!r}")
        key, *values = parts
        seen.append(key)
        for code, value in zip(codes, values):
            packs[code][key] = value.replace("\\n", "\n")
    extra = [key for key in seen if key not in CATALOG_KEYS]
    if extra:
        raise SystemExit(f"{codes}: extra={extra}")
    missing = [key for key in CATALOG_KEYS if key not in seen]
    unknown = [key for key in missing if key not in FALLBACKS]
    if unknown:
        raise SystemExit(f"{codes}: missing={unknown}")
    for key in missing:
        for code in codes:
            packs[code][key] = FALLBACKS[key]
    return packs
