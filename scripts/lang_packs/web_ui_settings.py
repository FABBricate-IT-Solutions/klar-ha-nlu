"""Translated operator Settings chrome for every compiled Assist locale."""

from __future__ import annotations

import json
from pathlib import Path

from lang_packs.web_ui_keys import FALLBACKS

KEYS = (
    "engineHint",
    "languageHint",
    "personalityHa",
    "assistLanguages",
    "assistLanguagesHint",
    "allAssistLanguages",
    "pinLanguage",
    "voice",
    "voiceHint",
    "extraPrompt",
    "extraPromptHint",
    "refineSpeech",
    "refineSpeechHint",
    "quietAck",
    "quietAckHint",
    "calendarLlm",
    "calendarLlmHint",
    "allowLlmTools",
    "allowLlmToolsHint",
    "missTitle",
    "missHint",
    "operatorChrome",
    "operatorChromeHint",
    "haGlueHint",
    "settingsGuide",
    "settingsGuideVoice",
    "settingsGuideLlm",
    "settingsGuideLang",
    "appearanceDark",
    "appearanceLight",
    "setupReplay",
    "personalityDefault",
    "personalityButler",
    "personalityLocker",
    "personalityFuersorglich",
    "personalityParty",
    "personalityGrantig",
    "personalitySarkastisch",
    "personalityPirat",
    "personalityHippie",
    "personalityGollum",
    "personalityJarvis",
    "operatorLanguage",
    "operatorLanguageHint",
    "nluRag",
    "modeFull",
    "modeContext",
    "inLab",
    "undoLastCommand",
    "applyDone",
    "applyUndone",
    "applyUndoFailed",
    "llm",
    "llmHint",
    "llmBaseUrl",
    "llmModel",
    "llmApiKey",
    "llmApiKeyHint",
    "llmPresetOpenAi",
    "llmPresetOllama",
    "llmConfigured",
    "llmNotConfigured",
    "llmClear",
    "trainer",
    "trainerForLane",
    "trainerHint",
    "trainerContext",
    "trainerValidate",
    "trainerApply",
    "trainerProposal",
    "trainerOk",
    "trainerFail",
    "trainerSend",
    "trainerNeedLlm",
    "trainerOpenSettings",
    "trainerApplyHouse",
    "trainerApplyMatch",
    "trainerApplyLanguage",
    "trainerAdvanced",
    "trainerStreaming",
)

_COPY_PATH = Path(__file__).with_name("web_ui_settings_copy.json")


def _load_copy() -> dict[str, dict[str, str]]:
    if not _COPY_PATH.is_file():
        return {}
    payload = json.loads(_COPY_PATH.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise SystemExit("web_ui_settings_copy.json must be an object")
    return payload


def apply_settings_copy(packs: dict[str, dict[str, str]]) -> None:
    copy = _load_copy()
    missing_locales = sorted(set(packs) - set(copy))
    if missing_locales:
        raise SystemExit(f"settings chrome missing locales: {missing_locales}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: settings chrome missing keys {absent}")
        for key in KEYS:
            fields[key] = row[key] or FALLBACKS[key]
