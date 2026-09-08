"""Lotse operator chrome overlaid onto every Assist locale."""

from __future__ import annotations

KEYS = (
    "trainer",
    "trainerForLane",
    "trainerHint",
    "trainerEmpty",
    "trainerEmptyHint",
    "trainerYou",
    "trainerPermit",
    "trainerPromptGaps",
    "trainerPromptNight",
    "trainerPromptMatchers",
    "trainerPromptPrecedence",
    "trainerPromptLexicon",
    "trainerPromptSlang",
    "trainerTool",
    "trainerClear",
    "trainerComposer",
    "trainerAllow",
    "trainerAllowOnce",
    "trainerYolo",
    "trainerDeny",
    "trainerAskAgain",
    "trainerSend",
    "trainerNeedLlm",
    "trainerOpenSettings",
    "trainerStreaming",
    "trainerOk",
    "trainerFail",
)

def lotse(
    *,
    hint: str,
    empty: str,
    empty_hint: str,
    you: str,
    permit: str,
    send: str,
    allow: str,
    allow_once: str,
    deny: str,
    ask: str,
    need: str,
    open_settings: str,
    streaming: str,
    ok: str,
    fail: str,
    tool: str,
    clear: str,
    composer: str,
    gaps: str,
    night: str,
    matchers: str,
    precedence: str,
    lexicon: str,
    slang: str,
    for_lane: str = "Guide",
    trainer: str = "Guide",
) -> dict[str, str]:
    return {
        "trainer": trainer,
        "trainerForLane": for_lane,
        "trainerHint": hint,
        "trainerEmpty": empty,
        "trainerEmptyHint": empty_hint,
        "trainerYou": you,
        "trainerPermit": permit,
        "trainerPromptGaps": gaps,
        "trainerPromptNight": night,
        "trainerPromptMatchers": matchers,
        "trainerPromptPrecedence": precedence,
        "trainerPromptLexicon": lexicon,
        "trainerPromptSlang": slang,
        "trainerTool": tool,
        "trainerClear": clear,
        "trainerComposer": composer,
        "trainerAllow": allow,
        "trainerAllowOnce": allow_once,
        "trainerYolo": "YOLO",
        "trainerDeny": deny,
        "trainerAskAgain": ask,
        "trainerSend": send,
        "trainerNeedLlm": need,
        "trainerOpenSettings": open_settings,
        "trainerStreaming": streaming,
        "trainerOk": ok,
        "trainerFail": fail,
    }


_ENGLISH_LOTSE = {
    "trainerHint": "Ask about Klar anytime. Writes wait for Allow in this chat.",
    "trainerEmpty": "Ask how Klar works — or about a gap.",
    "trainerEmptyHint": "Lotse writes nothing until you tap Allow.",
    "trainerNeedLlm": "Configure an LLM in Settings first. No other Home Assistant conversation agent.",
    "trainerPromptGaps": "Which devices have no room?",
    "trainerPromptNight": "Suggest a house rule for good night.",
    "trainerPromptMatchers": "Which matchers are on, and in what order?",
    "trainerPromptPrecedence": "Explain matcher precedence. Do not change it.",
    "trainerPromptLexicon": "Which lexicon paths exist for German?",
    "trainerPromptSlang": "Suggest slang for a lexicon path. Do not write yet.",
}

_LOTSE_FORMS = (
    "Lotsénak",
    "Lotsovi",
    "Lotselle",
    "Lotsile",
    "Lotsui",
    "Lotsem",
    "Lotsed",
    "Lotseri",
    "Lotsa",
    "Lotsu",
    "Lotsе",
    "Lotsen",
    "Lotse",
    "Lots",
)


def rewrite_lotse_name(text: str, name: str) -> str:
    for token in _LOTSE_FORMS:
        if token in text:
            text = text.replace(token, name)
    return text


def apply_lotse_chrome(packs: dict[str, dict[str, str]]) -> None:
    from lang_packs.lotse_chrome_east import PACKS as EAST
    from lang_packs.lotse_chrome_script import PACKS as SCRIPT
    from lang_packs.lotse_chrome_west import PACKS as WEST

    copy: dict[str, dict[str, str]] = {}
    copy.update(WEST)
    copy.update(EAST)
    copy.update(SCRIPT)
    missing = sorted(set(packs) - set(copy))
    if missing:
        raise SystemExit(f"lotse chrome missing locales: {missing}")
    extra = sorted(set(copy) - set(packs))
    if extra:
        raise SystemExit(f"lotse chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: lotse chrome missing keys {absent}")
        saved_trainer = fields.get("trainer")
        saved_lane = fields.get("trainerForLane")
        saved_native = {
            key: fields[key]
            for key in _ENGLISH_LOTSE
            if fields.get(key) and fields[key] not in _ENGLISH_LOTSE.values()
        }
        for key in KEYS:
            if key == "trainer":
                if code.startswith("de"):
                    fields[key] = row.get("trainer") or "Lotse"
                continue
            fields[key] = row[key]
        if code.startswith("de"):
            continue
        name = saved_trainer or row.get("trainer") or "Guide"
        for key in ("trainerEmptyHint", "trainerComposer", "trainerForLane", "trainerEmpty", "trainerHint"):
            fields[key] = rewrite_lotse_name(fields[key], name)
        if fields["trainerForLane"] == name and saved_lane and saved_lane != name:
            fields["trainerForLane"] = saved_lane
        for key, value in saved_native.items():
            if fields.get(key) in _ENGLISH_LOTSE.values():
                fields[key] = value
        allow = fields.get("effectAllow") or fields.get("trainerAllow") or ""
        empty_hint = fields.get("trainerEmptyHint") or ""
        if allow and ("writes nothing" in empty_hint or "until you tap" in empty_hint or empty_hint in _ENGLISH_LOTSE.values()):
            fields["trainerEmptyHint"] = f"{name} — {allow}."
        if fields.get("trainerEmpty") in _ENGLISH_LOTSE.values():
            native_empty = fields.get("emptyBundle") or fields.get("noGaps")
            if native_empty:
                fields["trainerEmpty"] = native_empty
