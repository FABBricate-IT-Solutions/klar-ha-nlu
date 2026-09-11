"""Rust source templates for generated Assist packs."""

from __future__ import annotations

from lang_packs.emit_rust import RUST_BANNER, rust_list, rust_pairs, rust_personality, rust_str
from lang_packs.fold import fold_latin


def pairs2(rows: list[list[str]]) -> str:
    return ", ".join(f'["{a}", "{b}"]' for a, b in rows)


def triples(rows: list[list[str]]) -> str:
    return ", ".join(f'["{a}", "{b}", "{c}"]' for a, b, c in rows)

def speech_rs(lang: dict) -> str:
    s = lang["speech"]
    rooms = rust_pairs(lang.get("room_names", []), "    ")
    pers = rust_personality(lang.get("personality", []), "    ")
    der = rust_list(lang.get("loc_der_rooms", []), "    ")
    return f"""{RUST_BANNER}use crate::lang::speech::Speech;

pub(super) const SPEECH: Speech = Speech {{
    unknown: {rust_str(s["unknown"])},
    need_on: {rust_str(s["need_on"])},
    need_off: {rust_str(s["need_off"])},
    need_which: {rust_str(s["need_which"])},
    correction: {rust_str(s["correction"])},
    clarify: {rust_str(s["clarify"])},
    clarify_or: {rust_str(s["clarify_or"])},
    and_join: {rust_str(s["and_join"])},
    group_on: {rust_str(s["group_on"])},
    group_off: {rust_str(s["group_off"])},
    turn_on: {rust_str(s["turn_on"])},
    turn_on_scene: {rust_str(s["turn_on_scene"])},
    turn_off: {rust_str(s["turn_off"])},
    toggle: {rust_str(s["toggle"])},
    light_set: {rust_str(s["light_set"])},
    light_color: {rust_str(s["light_color"])},
    climate_set: {rust_str(s["climate_set"])},
    heat_noun: {rust_str(s["heat_noun"])},
    cool_noun: {rust_str(s["cool_noun"])},
    get_temp: {rust_str(s["get_temp"])},
    get_state: {rust_str(s["get_state"])},
    media_pause: {rust_str(s["media_pause"])},
    media_play: {rust_str(s["media_play"])},
    media_next: {rust_str(s["media_next"])},
    media_previous: {rust_str(s["media_previous"])},
    media_mute: {rust_str(s["media_mute"])},
    media_unmute: {rust_str(s["media_unmute"])},
    media_volume: {rust_str(s["media_volume"])},
    media_search: {rust_str(s["media_search"])},
    media_transfer: {rust_str(s["media_transfer"])},
    media_favorite: {rust_str(s["media_favorite"])},
    fan_set: {rust_str(s["fan_set"])},
    vacuum_start: {rust_str(s["vacuum_start"])},
    vacuum_dock: {rust_str(s["vacuum_dock"])},
    vacuum_default: {rust_str(s["vacuum_default"])},
    timer_start: {rust_str(s["timer_start"])},
    timer_cancel: {rust_str(s["timer_cancel"])},
    timer_pause: {rust_str(s["timer_pause"])},
    timer_how_long: {rust_str(s.get("timer_how_long", "How long?"))},
    list_add: {rust_str(s["list_add"])},
    calendar_list: {rust_str(s.get("calendar_list", "{items}"))},
    calendar_empty: {rust_str(s.get("calendar_empty", s["unknown"]))},
    calendar_none: {rust_str(s.get("calendar_none", s["unknown"]))},
    calendar_created: {rust_str(s.get("calendar_created", "{summary} {when}"))},
    calendar_need_title: {rust_str(s.get("calendar_need_title", s["unknown"]))},
    calendar_need_when: {rust_str(s.get("calendar_need_when", s["unknown"]))},
    calendar_readonly: {rust_str(s.get("calendar_readonly", s["unknown"]))},
    calendar_deleted: {rust_str(s.get("calendar_deleted", s["unknown"]))},
    calendar_moved: {rust_str(s.get("calendar_moved", "{summary} {when}"))},
    calendar_which: {rust_str(s.get("calendar_which", s["unknown"]))},
    calendar_no_uid: {rust_str(s.get("calendar_no_uid", s["unknown"]))},
    no_music_player: {rust_str(s.get("no_music_player", s["unknown"]))},
    done: {rust_str(s["done"])},
    light_suffix: {rust_str(s["light_suffix"])},
    area_light: {rust_str(s["area_light"])},
    loc_in: {rust_str(s["loc_in"])},
    loc_in_der: {rust_str(s["loc_in_der"])},
    loc_home: {rust_str(s["loc_home"])},
    or_home: {rust_str(s["or_home"])},
    room_names: {rooms},
    loc_der_rooms: {der},
    personality: {pers},
    confirm: {rust_str(s["confirm"])},
}};
"""


def pack_rs(lang: dict) -> str:
    t = lang["talk"]
    n = lang["nouns"]
    f = lang["fixtures"]
    c = lang["cues"]
    m = lang["maps"]
    ch = lang["chat"]
    s = lang["speech"]
    code = lang["code"]
    seen_alias: set[str] = set()
    alias_rows: list[str] = []
    for key, vals in lang.get("fixture_aliases", []):
        folded = fold_latin(key)
        if not folded or folded in seen_alias:
            continue
        seen_alias.add(folded)
        alias_rows.append(f'            ({rust_str(folded)}, {rust_list(vals, "            ")})')
    aliases = ",\n".join(alias_rows)
    alias_block = f"&[\n{aliases},\n        ]" if aliases else "&[]"
    numbers = ",\n".join(
        f'            ({rust_str(fold_latin(word))}, {num})' for word, num in m["numbers"] if fold_latin(word)
    )
    colors = rust_pairs(m["colors"], "            ")
    domains = rust_pairs(m["domain_map"], "            ")
    style = m.get("number_style", "ListedOnly")
    return f"""{RUST_BANNER}use crate::lang::groups::{{Chat, Cues, Fixtures, GroupClarify, Household, LanguagePack, Maps, Nouns, NumberStyle, Talk}};
use crate::lang::morphology::PackMorphology;
use crate::lang::LangId;

use super::speech::SPEECH;
use super::verbs::VERBS;

pub const PACK: LanguagePack = LanguagePack {{
    id: LangId::new("{code}"),
    verbs: VERBS,
    talk: Talk {{
        fillers: {rust_list(t["fillers"])},
        action_keep: {rust_list(t["action_keep"])},
        conjunctions: {rust_list(t["conjunctions"])},
        particles: {rust_list(t["particles"])},
        affirm: {rust_list(t["affirm"])},
        or_words: {rust_list(t["or_words"])},
        except_words: {rust_list(t["except_words"])},
        all_words: {rust_list(t["all_words"])},
        query_hint: {rust_list(t["query_hint"])},
        question_starts: {rust_list(t["question_starts"])},
        question_words: {rust_list(t["question_words"])},
        correction: {rust_list(t["correction"])},
        correction_phrases: {rust_list(t.get("correction_phrases", []))},
        clarify_pick: {rust_list(t["clarify_pick"])},
    }},
    nouns: Nouns {{
        light_nouns: {rust_list(n["light_nouns"])},
        light_singular: {rust_list(n["light_singular"])},
        light_plural: {rust_list(n["light_plural"])},
        cover_nouns: {rust_list(n["cover_nouns"])},
        curtain_nouns: {rust_list(n["curtain_nouns"])},
        fan_nouns: {rust_list(n["fan_nouns"])},
        climate_nouns: {rust_list(n["climate_nouns"])},
        media_nouns: {rust_list(n["media_nouns"])},
        lock_nouns: {rust_list(n["lock_nouns"])},
        door_nouns: {rust_list(n["door_nouns"])},
        garage_words: {rust_list(n["garage_words"])},
        garage_cover: {rust_list(n["garage_cover"])},
        timer_nouns: {rust_list(n["timer_nouns"])},
        list_nouns: {rust_list(n["list_nouns"])},
        calendar_nouns: {rust_list(n.get("calendar_nouns", []))},
        vacuum_nouns: {rust_list(n["vacuum_nouns"])},
        scene_nouns: {rust_list(n["scene_nouns"])},
        script_words: {rust_list(n["script_words"])},
        switch_plural: {rust_list(n["switch_plural"])},
        device_side: {rust_list(n["device_side"])},
        named_device: {rust_list(n["named_device"])},
    }},
    fixtures: Fixtures {{
        island: {rust_list(f["island"])},
        ceiling: {rust_list(f["ceiling"])},
        lamp_fixture: {rust_list(f["lamp_fixture"])},
        pendant: {rust_list(f["pendant"])},
        bedside: {rust_list(f["bedside"])},
        left: {rust_list(f["left"])},
        right: {rust_list(f["right"])},
        sides: {rust_list(f["sides"])},
        fixture_aliases: {alias_block},
        group_clarify: Some(GroupClarify {{
            trigger: {rust_list(f["clarify_trigger"])},
            pairs: &[{pairs2(f["clarify_pairs"])}],
            triples: &[{triples(f.get("clarify_triples", []))}],
        }}),
        singular_lamp: {rust_list(f["singular_lamp"])},
        singular_lamp_block: {rust_list(f["singular_lamp_block"])},
    }},
    cues: Cues {{
        power_words: {rust_list(c["power_words"])},
        command_hedges: {rust_list(c["command_hedges"])},
        skip_light: {rust_list(c["skip_light"])},
        laundry_area: {rust_list(c["laundry_area"])},
        laundry_machines: {rust_list(c["laundry_machines"])},
        kitchen: {rust_list(c["kitchen"])},
        open_words: {rust_list(c["open_words"])},
        close_words: {rust_list(c["close_words"])},
        roll_close: {rust_list(c["roll_close"])},
        unlock_follow: {rust_list(c["unlock_follow"])},
        cover_open_follow: {rust_list(c["cover_open_follow"])},
        garage_lock_block: {rust_list(c["garage_lock_block"])},
        on_words: {rust_list(c["on_words"])},
        off_words: {rust_list(c["off_words"])},
        scene_named: {rust_list(c["scene_named"])},
        temp_query: {rust_list(c["temp_query"])},
        timer_query: {rust_list(c["timer_query"])},
        brightness: {rust_list(c["brightness"])},
        start_words: {rust_list(c["start_words"])},
        replay_on_off: {rust_list(c["replay_on_off"])},
        replay_off: {rust_list(c["replay_off"])},
        sensor_words: {rust_list(c["sensor_words"])},
        lock_verbs: {rust_list(c["lock_verbs"])},
        entry_words: {rust_list(c["entry_words"])},
        oven: {rust_list(c["oven"])},
        laundry_timer: {rust_list(c["laundry_timer"])},
        illuminate: {rust_list(c["illuminate"])},
        list_down: {rust_list(c["list_down"])},
        chores: {rust_list(c["chores"])},
        weak_scene: {rust_list(c["weak_scene"])},
        timer_cancel: {rust_list(c["timer_cancel"])},
        timer_pause: {rust_list(c["timer_pause"])},
        timer_add: {rust_list(c["timer_add"])},
        timer_remove: {rust_list(c.get("timer_remove", []))},
        list_complete: {rust_list(c["list_complete"])},
        playback_resume: {rust_list(c["playback_resume"])},
        calendar_query: {rust_list(c.get("calendar_query", []))},
        calendar_create: {rust_list(c.get("calendar_create", []))},
        calendar_today: {rust_list(c.get("calendar_today", []))},
        calendar_tomorrow: {rust_list(c.get("calendar_tomorrow", []))},
        calendar_when: {rust_list(c.get("calendar_when", []))},
        calendar_delete: {rust_list(c.get("calendar_delete", []))},
        calendar_move: {rust_list(c.get("calendar_move", []))},
        vacuum_start: {rust_list(c["vacuum_start"])},
        hours: {rust_list(c["hours"])},
        minutes: {rust_list(c["minutes"])},
        seconds: {rust_list(c["seconds"])},
        list_skip: {rust_list(c["list_skip"])},
        shopping_names: {rust_list(c["shopping_names"])},
        status_words: {rust_list(c["status_words"])},
        window_words: {rust_list(c["window_words"])},
        open_close: {rust_list(c["open_close"])},
        laundry_hint: {rust_list(c["laundry_hint"])},
        bare_switch: {rust_list(c["bare_switch"])},
        outlet_words: {rust_list(c["outlet_words"])},
        tv_words: {rust_list(c["tv_words"])},
        climate_cool: {rust_list(c["climate_cool"])},
        climate_heat: {rust_list(c["climate_heat"])},
        role_light: {rust_list(c["role_light"])},
        role_climate: {rust_list(c["role_climate"])},
        role_media: {rust_list(c["role_media"])},
        role_fan: {rust_list(c["role_fan"])},
        generic: {rust_list(c["generic"])},
        room_level: {rust_list(c["room_level"])},
        extra_device_nouns: {rust_list(c.get("extra_device_nouns", []))},
        synonym_pairs: {rust_pairs(c["synonym_pairs"])},
        scene_synonyms: {rust_pairs(c.get("scene_synonyms", []))},
        article_one: {rust_list(c["article_one"])},
        strip_pairs: {rust_pairs(c.get("strip_pairs", []))},
        keep_after: &[],
    }},
    maps: Maps {{
        domain_map: {domains},
        colors: {colors},
        numbers: &[
{numbers},
        ],
        number_style: NumberStyle::{style},
        room_index_nouns: {rust_list(m["room_index_nouns"])},
    }},
    chat: Chat {{
        greet: {rust_list(ch["greet"])},
        thanks: {rust_list(ch["thanks"])},
        feeling: {rust_list(ch["feeling"])},
        identity: {rust_list(ch["identity"])},
        tell: {rust_list(ch["tell"])},
        yarn: {rust_list(ch["yarn"])},
        world: {rust_list(ch["world"])},
        advice: {rust_list(ch["advice"])},
        open: {rust_list(ch["open"])},
        news: {rust_list(ch["news"])},
        news_dismiss: {rust_list(ch["news_dismiss"])},
        news_intro: {rust_str(ch["news_intro"])},
        news_nudge: {rust_str(ch["news_nudge"])},
        news_done: {rust_str(ch["news_done"])},
    }},
    household: Household {{
        teach: {rust_list(lang.get("household", {}).get("teach", []))},
        explain: {rust_list(lang.get("household", {}).get("explain", []))},
        undo: {rust_list(lang.get("household", {}).get("undo", []))},
        clock: {rust_list(lang.get("household", {}).get("clock", []))},
        weather: {rust_list(lang.get("household", {}).get("weather", []))},
        clock_skip: {rust_list(lang.get("household", {}).get("clock_skip", []))},
        heard_nothing: {rust_str(lang.get("household", {}).get("heard_nothing") or s.get("unknown", ""))},
        heard: {rust_str(lang.get("household", {}).get("heard") or "{{text}}")},
        executed: {rust_str(lang.get("household", {}).get("executed") or "{{names}}")},
        asked_risky: {rust_str(lang.get("household", {}).get("asked_risky") or s.get("confirm", ""))},
        unclear_device: {rust_str(lang.get("household", {}).get("unclear_device") or s.get("need_which", ""))},
        stopped: {rust_str(lang.get("household", {}).get("stopped") or "{{reason}}")},
        no_match: {rust_str(lang.get("household", {}).get("no_match") or s.get("unknown", ""))},
        was_chat: {rust_str(lang.get("household", {}).get("was_chat") or s.get("done", ""))},
        decision: {rust_str(lang.get("household", {}).get("decision") or "{{decision}}")},
        in_area: {rust_str(lang.get("household", {}).get("in_area") or "{{area}}")},
        nothing_undo: {rust_str(lang.get("household", {}).get("nothing_undo") or s.get("unknown", ""))},
        teach_which: {rust_str(lang.get("household", {}).get("teach_which") or s.get("need_which", ""))},
        teach_invalid: {rust_str(lang.get("household", {}).get("teach_invalid") or s.get("unknown", ""))},
        teach_ok: {rust_str(lang.get("household", {}).get("teach_ok") or s.get("done", ""))},
        clock_ok: {rust_str(lang.get("household", {}).get("clock_ok") or "{{time}}")},
        clock_missing: {rust_str(lang.get("household", {}).get("clock_missing") or s.get("unknown", ""))},
        no_weather: {rust_str(lang.get("household", {}).get("no_weather") or s.get("unknown", ""))},
    }},
    speech: SPEECH,
    morphology: PackMorphology::EMPTY,
}};
"""

