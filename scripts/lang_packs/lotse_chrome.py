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
    for_lane: str = "Lotse",
) -> dict[str, str]:
    return {
        "trainer": "Lotse",
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
        for key in KEYS:
            fields[key] = row[key]
