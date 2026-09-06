"""Lane / path / lexicon chrome keys overlaid onto every Assist locale."""

from __future__ import annotations

KEYS = (
    "processPath",
    "pathUnchecked",
    "pathMatch",
    "pathSeed",
    "pathHouse",
    "pathBand",
    "laneMatch",
    "laneMatchEngine",
    "laneLanguage",
    "laneHouse",
    "laneTabs",
    "lexiconOverlay",
    "lexiconOverlayPlus",
    "lexiconEmpty",
    "governSeed",
    "governEmpty",
    "matchCatalog",
    "matchReadOnly",
    "matchEnabled",
    "matchDisabled",
    "matchPrecedence",
    "matchDisableWarning",
    "lexiconPath",
    "lexiconToken",
    "lexiconAdd",
    "lexiconRemove",
    "originEngine",
    "originOperator",
    "originSeed",
    "originTrainer",
    "compiledFloor",
    "seedOn",
    "seedOff",
)


def lane(
    *,
    process_path: str,
    path_match: str,
    path_seed: str,
    path_house: str,
    path_band: str,
    lane_match: str,
    lane_match_engine: str,
    lane_language: str,
    lane_house: str,
    lane_tabs: str,
    lexicon_overlay: str,
    lexicon_overlay_plus: str,
    lexicon_empty: str,
    govern_seed: str,
    govern_empty: str,
    match_catalog: str,
    match_read_only: str,
    match_enabled: str,
    match_disabled: str,
    match_precedence: str,
    match_disable_warning: str,
    lexicon_path: str,
    lexicon_token: str,
    lexicon_add: str,
    lexicon_remove: str,
    origin_engine: str,
    origin_operator: str,
    origin_seed: str,
    origin_trainer: str,
    compiled_floor: str,
    seed_on: str,
    seed_off: str,
    path_unchecked: str = "—",
) -> dict[str, str]:
    return {
        "processPath": process_path,
        "pathUnchecked": path_unchecked,
        "pathMatch": path_match,
        "pathSeed": path_seed,
        "pathHouse": path_house,
        "pathBand": path_band,
        "laneMatch": lane_match,
        "laneMatchEngine": lane_match_engine,
        "laneLanguage": lane_language,
        "laneHouse": lane_house,
        "laneTabs": lane_tabs,
        "lexiconOverlay": lexicon_overlay,
        "lexiconOverlayPlus": lexicon_overlay_plus,
        "lexiconEmpty": lexicon_empty,
        "governSeed": govern_seed,
        "governEmpty": govern_empty,
        "matchCatalog": match_catalog,
        "matchReadOnly": match_read_only,
        "matchEnabled": match_enabled,
        "matchDisabled": match_disabled,
        "matchPrecedence": match_precedence,
        "matchDisableWarning": match_disable_warning,
        "lexiconPath": lexicon_path,
        "lexiconToken": lexicon_token,
        "lexiconAdd": lexicon_add,
        "lexiconRemove": lexicon_remove,
        "originEngine": origin_engine,
        "originOperator": origin_operator,
        "originSeed": origin_seed,
        "originTrainer": origin_trainer,
        "compiledFloor": compiled_floor,
        "seedOn": seed_on,
        "seedOff": seed_off,
    }


def apply_lanes_copy(packs: dict[str, dict[str, str]], copy: dict[str, dict[str, str]]) -> None:
    missing_locales = sorted(set(packs) - set(copy))
    if missing_locales:
        raise SystemExit(f"lane chrome missing locales: {missing_locales}")
    extra = sorted(set(copy) - set(packs))
    if extra:
        raise SystemExit(f"lane chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = copy[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: lane chrome missing keys {absent}")
        for key in KEYS:
            fields[key] = row[key]
