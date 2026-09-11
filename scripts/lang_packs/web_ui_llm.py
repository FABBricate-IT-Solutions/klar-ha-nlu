"""Dashboard LLM and Lab path chrome for every Assist locale."""

from __future__ import annotations

KEYS = (
    "llmCalls",
    "llmCallsCaption",
    "llmKindRefine",
    "llmKindAssist",
    "llmKindChat",
    "llmErrors",
    "llmAcceptRate",
    "llmLatency",
    "llmLatencyCaption",
    "llmTokens",
    "llmTokensCaption",
    "llmNoCalls",
    "labThisTurn",
)

# Keys that must not stay English (except en-GB). Short chips may stay Assist/chat.
CHECK_KEYS = (
    "llmCalls",
    "llmCallsCaption",
    "llmNoCalls",
    "labThisTurn",
    "llmLatency",
    "llmLatencyCaption",
    "llmTokensCaption",
    "llmAcceptRate",
)

INDIC = ("hi", "bn", "gu", "kn", "ml", "mr", "ta", "te", "pa", "ne", "sw")


def llm_row(**fields: str) -> dict[str, str]:
    missing = [key for key in KEYS if key not in fields]
    extra = [key for key in fields if key not in KEYS]
    if missing or extra:
        raise SystemExit(f"llm row missing={missing} extra={extra}")
    return {key: fields[key] for key in KEYS}


def _strip_llm(value: str) -> str:
    text = value.replace("LLM", "").replace("  ", " ").strip(" ·-–")
    return text or value


def compose_llm(fields: dict[str, str]) -> dict[str, str]:
    """Build Indic/Swahili LLM chrome from catalog strings already in that locale."""
    llm = fields["llm"]
    source = fields["source"]
    engine = fields["engineReady"].split()[0]
    hours = fields["journalHint"].replace("।", ".").split(".")[0].strip()
    window = fields["latencyCaption"].split(",", 1)[-1].strip()
    return llm_row(
        llmCalls=f"{llm} · {fields['recordings']}",
        llmCallsCaption=f"{source}: {engine} {llm}, {hours}",
        llmKindRefine=_strip_llm(fields["speechRefined"]),
        llmKindAssist="Assist",
        llmKindChat=_strip_llm(fields["speechChat"]),
        llmErrors=fields["needsWork"],
        llmAcceptRate=fields["accept"],
        llmLatency=f"{llm} · {fields['latency']}",
        llmLatencyCaption=f"{source}: {engine} {llm}, {window}",
        llmTokens=fields["stageTokens"],
        llmTokensCaption=f"{source}: {fields['stageTokens']}",
        llmNoCalls=fields["noConversations"],
        labThisTurn=fields["unitsTurns"],
    )


def apply_llm_copy(packs: dict[str, dict[str, str]]) -> None:
    # Regional packs import llm_row; load them here to avoid a cycle.
    from lang_packs.web_ui_llm_asia import PACKS as ASIA
    from lang_packs.web_ui_llm_europe import PACKS as EUROPE
    from lang_packs.web_ui_llm_mena import PACKS as MENA
    from lang_packs.web_ui_llm_nordic import PACKS as NORDIC
    from lang_packs.web_ui_llm_slavic import PACKS as SLAVIC
    from lang_packs.web_ui_llm_west import PACKS as WEST

    copy: dict[str, dict[str, str]] = {}
    copy.update(WEST)
    copy.update(EUROPE)
    copy.update(NORDIC)
    copy.update(SLAVIC)
    copy.update(ASIA)
    copy.update(MENA)
    for code in INDIC:
        copy[code] = compose_llm(packs[code])
    missing = sorted(set(packs) - set(copy))
    if missing:
        raise SystemExit(f"llm chrome missing locales: {missing}")
    extra = sorted(set(copy) - set(packs))
    if extra:
        raise SystemExit(f"llm chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: llm chrome missing keys {absent}")
        extra_keys = [key for key in row if key not in KEYS]
        if extra_keys:
            raise SystemExit(f"{code}: llm chrome extra keys {extra_keys}")
        fields.update(row)
