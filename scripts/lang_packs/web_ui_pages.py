"""Rules/house guides and leftover Settings chrome for every Assist locale."""

from __future__ import annotations

KEYS = (
    "guideRoutinesSay",
    "guideRoutinesSayHint",
    "guideRoutinesScript",
    "guideRoutinesScriptHint",
    "guideSentencesPhrase",
    "guideSentencesPhraseHint",
    "guideSentencesIntent",
    "guideSentencesIntentHint",
    "guideSentencesTest",
    "guideSentencesTestHint",
    "guidePoliciesMatch",
    "guidePoliciesMatchHint",
    "guidePoliciesLang",
    "guidePoliciesLangHint",
    "guidePoliciesHouse",
    "guidePoliciesHouseHint",
    "guidePoliciesTest",
    "guidePoliciesTestHint",
    "houseGuideGraph",
    "houseGuideDevices",
    "houseGuideMap",
    "menu",
    "ragModeShort",
    "llmThinking",
    "unitSystem",
    "unitSystemHint",
    "unitMetric",
    "unitImperial",
    "settingsNavBackup",
    "settingsNavEngine",
    "settingsNavLanguages",
    "settingsNavVoice",
    "llmProvider",
    "llmPresetCustom",
    "personalityPrompt",
)


def pages(**fields: str) -> dict[str, str]:
    missing = [key for key in KEYS if key not in fields]
    extra = [key for key in fields if key not in KEYS]
    if missing or extra:
        raise SystemExit(f"pages row missing={missing} extra={extra}")
    return {key: fields[key] for key in KEYS}


def apply_pages_copy(packs: dict[str, dict[str, str]]) -> None:
    from lang_packs.web_ui_pages_asia import PACKS as ASIA
    from lang_packs.web_ui_pages_europe import PACKS as EUROPE
    from lang_packs.web_ui_pages_indic import PACKS as INDIC
    from lang_packs.web_ui_pages_mena import PACKS as MENA
    from lang_packs.web_ui_pages_nordic import PACKS as NORDIC
    from lang_packs.web_ui_pages_slavic import PACKS as SLAVIC
    from lang_packs.web_ui_pages_west import PACKS as WEST

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
        raise SystemExit(f"pages chrome missing locales: {missing}")
    extra = sorted(set(copy) - set(packs))
    if extra:
        raise SystemExit(f"pages chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: pages chrome missing keys {absent}")
        extra_keys = [key for key in row if key not in KEYS]
        if extra_keys:
            raise SystemExit(f"{code}: pages chrome extra keys {extra_keys}")
        fields.update(row)
