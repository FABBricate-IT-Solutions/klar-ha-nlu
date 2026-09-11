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
    "personalityPrompt",
    "personalityPromptHint",
    "refineSpeech",
    "refineSpeechHint",
    "refineBandStatus",
    "refineBandStatusHint",
    "refineBandCommand",
    "refineBandCommandHint",
    "refineBandPrompt",
    "refineBandPromptHint",
    "refineBandReject",
    "refineBandRejectHint",
    "refineBandsHint",
    "quietAck",
    "quietAckHint",
    "unitSystem",
    "unitSystemHint",
    "unitMetric",
    "unitImperial",
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
    "settingsNavLlm",
    "settingsNavVoice",
    "settingsNavLanguages",
    "settingsNavEngine",
    "settingsNavBackup",
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
    "modeHint",
    "confirmRiskyHint",
    "recordProtocolHint",
    "includeRawTextHint",
    "semanticAdaptersHint",
    "tokenHint",
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
    "llmProvider",
    "llmPresetOpenAi",
    "llmPresetOllama",
    "llmPresetAnthropic",
    "llmPresetGoogle",
    "llmPresetLemonade",
    "llmPresetLlamaCpp",
    "llmPresetCustom",
    "llmConfigured",
    "llmNotConfigured",
    "llmClear",
    "saveOk",
    "saveFail",
    "saveUnauthorized",
    "llmThinking",
    "llmThinkingHint",
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
        absent = [key for key in KEYS if key not in row and key not in FALLBACKS]
        if absent:
            raise SystemExit(f"{code}: settings chrome missing keys {absent}")
        for key in KEYS:
            fields[key] = row.get(key) or FALLBACKS[key]
        if code.startswith("de"):
            fields["saveOk"] = row.get("saveOk") or "Gespeichert."
            fields["saveFail"] = row.get("saveFail") or "Speichern fehlgeschlagen."
            fields["saveUnauthorized"] = row.get("saveUnauthorized") or "Nicht gespeichert. Schreib-Token unter Engine eintragen."
            fields["refineSpeechHint"] = row.get("refineSpeechHint") or "Klar steuert weiter ohne LLM. Hier nur, welche fertigen Sätze die Stimme bekommen."
            fields["refineBandStatus"] = row.get("refineBandStatus") or "Status"
            fields["refineBandCommand"] = row.get("refineBandCommand") or "Befehle"
            fields["refineBandPrompt"] = row.get("refineBandPrompt") or "Rückfragen"
            fields["refineBandReject"] = row.get("refineBandReject") or "Ablehnung"
            fields["refineBandsHint"] = row.get("refineBandsHint") or "Chat und Kalender-LLM bleiben unangetastet — kein zweites Refine."
            fields["refineBandStatusHint"] = row.get("refineBandStatusHint") or "Antworten wie Raum- oder Gerätestatus. An: das LLM schreibt den fertigen Satz um."
            fields["refineBandCommandHint"] = row.get("refineBandCommandHint") or "Bestätigung, nachdem Klar den Befehl schon ausgeführt hat. An: „Licht ist an“ bekommt diese Stimme."
            fields["refineBandPromptHint"] = row.get("refineBandPromptHint") or "Rückfragen zu Raum oder Gerät. An: Klars Frage wird umgeschrieben."
            fields["refineBandRejectHint"] = row.get("refineBandRejectHint") or "Sätze wie „Das habe ich nicht verstanden“. An: auch die bekommen diese Stimme."
            fields["modeHint"] = row.get("modeHint") or "Geräte auflösen: Klar wählt das Entity. Nur Räume: es bleibt bei der Area."
            fields["confirmRiskyHint"] = row.get("confirmRiskyHint") or "An: Schlösser, schließende Rollläden und große oder raumweite riskante Befehle fragen einmal nach. Aus: sofort ausführen."
            fields["recordProtocolHint"] = row.get("recordProtocolHint") or "An: Klar behält die letzten 200 Turns 24 Stunden. Braucht es für die Downloads darunter."
            fields["includeRawTextHint"] = row.get("includeRawTextHint") or "An: Downloads enthalten den gesprochenen Satz. Aus: nur geschwärzt."
            fields["semanticAdaptersHint"] = row.get("semanticAdaptersHint") or "Standard aus. An: nach einem Miss kann ein lokaler Bedeutungsabgleich den Intent noch finden. Kein LLM."
            fields["tokenHint"] = row.get("tokenHint") or "Zum Speichern auf diesem Bildschirm. Derselbe Wert wie das App-Token."
