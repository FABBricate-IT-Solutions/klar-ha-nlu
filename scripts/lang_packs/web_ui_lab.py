"""Lab, mapping, and settings-backup chrome overlaid onto every Assist locale."""

from __future__ import annotations

KEYS = (
    "parseHint",
    "triggerFirst",
    "labPipeline",
    "labChipContextOnly",
    "labChipNluRag",
    "labChipSemantic",
    "labChipNoConfirm",
    "labChipLlmRefine",
    "labChipCalendarLlm",
    "labChipQuietAck",
    "labChipLlmTools",
    "labChipLlmChat",
    "labChipConfirmRisky",
    "labDecisionExecute",
    "labDecisionBriefing",
    "labParse",
    "reasonMissingArea",
    "reasonWeakName",
    "reasonReady",
    "reasonMatch",
    "sentencesEmpty",
    "policiesEmpty",
    "setupAgainWhere",
    "whyThisBand",
    "rememberAsPhrase",
    "evidence",
    "names",
    "settingsBackup",
    "settingsBackupHint",
    "settingsBackupDownload",
    "settingsBackupIncludeKey",
    "settingsBackupIncludeKeyHint",
    "settingsBackupIncludeKeyConfirm",
    "settingsBackupRestore",
    "settingsBackupRestoreConfirm",
    "settingsBackupRestoreOk",
    "settingsBackupRestoreFail",
    "settingsBackupPickFile",
    "speechRefined",
    "speechChat",
    "refineRejected",
)


def lab_row(**fields: str) -> dict[str, str]:
    missing = [key for key in KEYS if key not in fields]
    extra = [key for key in fields if key not in KEYS]
    if missing or extra:
        raise SystemExit(f"lab row missing={missing} extra={extra}")
    return {key: fields[key] for key in KEYS}


def apply_lab_copy(packs: dict[str, dict[str, str]]) -> None:
    from lang_packs.web_ui_lab_asia import PACKS as ASIA
    from lang_packs.web_ui_lab_europe import PACKS as EUROPE
    from lang_packs.web_ui_lab_indic import PACKS as INDIC
    from lang_packs.web_ui_lab_mena import PACKS as MENA
    from lang_packs.web_ui_lab_nordic import PACKS as NORDIC
    from lang_packs.web_ui_lab_slavic import PACKS as SLAVIC
    from lang_packs.web_ui_lab_west import PACKS as WEST

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
        raise SystemExit(f"lab chrome missing locales: {missing}")
    extra = sorted(set(copy) - set(packs))
    if extra:
        raise SystemExit(f"lab chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: lab chrome missing keys {absent}")
        extra_keys = [key for key in row if key not in KEYS]
        if extra_keys:
            raise SystemExit(f"{code}: lab chrome extra keys {extra_keys}")
        fields.update(row)
