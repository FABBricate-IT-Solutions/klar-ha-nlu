"""Page-guide and LLM-model chrome overlaid onto every Assist locale."""

from __future__ import annotations

KEYS = (
    "llmModelsEmpty",
    "llmModelsFail",
    "llmModelsLoading",
    "llmThinkingHint",
    "conversationsEmptyHint",
    "evaluatorHint",
    "rulesRoutinesHint",
    "rulesSentencesHint",
    "rulesPoliciesHint",
    "labGuide",
)


def guides(
    *,
    empty: str,
    fail: str,
    loading: str,
    thinking: str,
    convos: str,
    evaluator: str,
    routines: str,
    sentences: str,
    policies: str,
    lab: str,
) -> dict[str, str]:
    return {
        "llmModelsEmpty": empty,
        "llmModelsFail": fail,
        "llmModelsLoading": loading,
        "llmThinkingHint": thinking,
        "conversationsEmptyHint": convos,
        "evaluatorHint": evaluator,
        "rulesRoutinesHint": routines,
        "rulesSentencesHint": sentences,
        "rulesPoliciesHint": policies,
        "labGuide": lab,
    }


def apply_guides_copy(packs: dict[str, dict[str, str]]) -> None:
    from lang_packs.web_ui_guides_asia import PACKS as ASIA
    from lang_packs.web_ui_guides_europe import PACKS as EUROPE
    from lang_packs.web_ui_guides_indic import PACKS as INDIC
    from lang_packs.web_ui_guides_mena import PACKS as MENA
    from lang_packs.web_ui_guides_nordic import PACKS as NORDIC
    from lang_packs.web_ui_guides_slavic import PACKS as SLAVIC
    from lang_packs.web_ui_guides_west import PACKS as WEST

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
        raise SystemExit(f"guides chrome missing locales: {missing}")
    extra = sorted(set(copy) - set(packs))
    if extra:
        raise SystemExit(f"guides chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: guides chrome missing keys {absent}")
        extra_keys = [key for key in row if key not in KEYS]
        if extra_keys:
            raise SystemExit(f"{code}: guides chrome extra keys {extra_keys}")
        fields.update(row)
