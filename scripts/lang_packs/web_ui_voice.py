"""Custom-voice and interview chrome overlaid onto every Assist locale."""

from __future__ import annotations

KEYS = (
    "personalityCustom",
    "customVoice",
    "customVoiceHint",
    "customVoiceMake",
    "customVoiceFail",
    "customVoiceName",
    "customVoiceNameHint",
    "customVoiceSeed",
    "customVoiceSeedHint",
    "interviewTraitsHint",
    "interviewWarmth",
    "interviewSarcasm",
    "interviewFormality",
    "interviewVerbosity",
    "interviewEnergy",
    "interviewAddress",
    "interviewAddressDu",
    "interviewAddressSie",
    "interviewAddressName",
    "interviewName",
    "interviewTone",
    "interviewToneShort",
    "interviewToneWarm",
    "interviewToneDry",
    "interviewHumor",
    "interviewHumorNone",
    "interviewHumorLight",
    "interviewHumorSharp",
    "interviewLength",
    "interviewLengthOne",
    "interviewLengthMore",
    "interviewTaboo",
)


def voice(**fields: str) -> dict[str, str]:
    missing = [key for key in KEYS if key not in fields]
    extra = [key for key in fields if key not in KEYS]
    if missing or extra:
        raise SystemExit(f"voice row missing={missing} extra={extra}")
    return {key: fields[key] for key in KEYS}


def apply_voice_copy(packs: dict[str, dict[str, str]]) -> None:
    from lang_packs.web_ui_voice_asia import PACKS as ASIA
    from lang_packs.web_ui_voice_europe import PACKS as EUROPE
    from lang_packs.web_ui_voice_indic import PACKS as INDIC
    from lang_packs.web_ui_voice_mena import PACKS as MENA
    from lang_packs.web_ui_voice_nordic import PACKS as NORDIC
    from lang_packs.web_ui_voice_slavic import PACKS as SLAVIC
    from lang_packs.web_ui_voice_west import PACKS as WEST

    copy: dict[str, dict[str, str]] = {}
    copy.update(WEST)
    copy.update(EUROPE)
    copy.update(NORDIC)
    copy.update(SLAVIC)
    copy.update(ASIA)
    copy.update(MENA)
    copy.update(INDIC)
    missing = sorted(set(packs) - set(copy))
    if missing:
        raise SystemExit(f"voice chrome missing locales: {missing}")
    extra = sorted(set(copy) - set(packs))
    if extra:
        raise SystemExit(f"voice chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: voice chrome missing keys {absent}")
        extra_keys = [key for key in row if key not in KEYS]
        if extra_keys:
            raise SystemExit(f"{code}: voice chrome extra keys {extra_keys}")
        fields.update(row)
