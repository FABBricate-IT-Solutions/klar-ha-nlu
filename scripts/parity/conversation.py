"""Phrase table for conversation overlays: auch/too form, one line per locale."""

from __future__ import annotations

from lang_packs.calendar_lex import calendar_for
from lex import color_word, room


def conversation_overlay(lex: dict) -> dict[str, list]:
    living = room(lex, "wohnzimmer")
    kitchen = room(lex, "kuche")
    bed = room(lex, "schlafzimmer")
    on, off, light, sett = lex["on"], lex["off"], lex["light"], lex["set"]
    too, anda, query = lex["too"], lex["and"], lex["query"]
    red, warm = color_word(lex, "red"), lex["warm"]
    play, pause, music, tv = lex["play"], lex["pause"], lex["music"], lex["tv"]
    climate, dim, medium = lex["climate"], lex["dim"], lex["medium"]
    nxt, quiet, percent = lex["next"], lex["quiet"], lex["percent"]
    clock = lex["clock"] if lex["clock"] != query else f"{query} {lex['hours']}".strip()
    weather = lex["weather"] if lex["weather"] != "weather" else f"{query} {climate}".strip()
    cal = (calendar_for(lex["code"]).get("nouns") or ["calendar"])[0]
    calendar = f"{query} {cal}".strip()
    on_living = f"{on} {light} {living}".strip()
    kitchen_too = f"{anda} {kitchen} {too}".strip()
    return {
        "conversation/followup::followup_same_action_other_room": [[on_living, kitchen_too]],
        "conversation/followup::followup_not_poisoned_by_prior_off": [
            [f"{off} {light} {bed}".strip(), on_living, kitchen_too]
        ],
        "conversation/lights::brightness_percent": [f"{sett} {light} {living} 30 {percent}".strip()],
        "conversation/lights::relative_dim": [[on_living, dim]],
        "conversation/lights::dim_then_level": [[f"{dim} {light} {living}".strip(), medium]],
        "conversation/lights::color_red": [[on_living, red]],
        "conversation/lights::warm_white": [[on_living, warm]],
        "conversation/lights::which_lights_on": [
            [on_living, f"{on} {light} {kitchen}".strip(), f"{query} {light}".strip()]
        ],
        "conversation/lights::ambiguous_light": [f"{on} {light}".strip()],
        "conversation/climate::temperature_query": [f"{query} {climate} {living}".strip()],
        "conversation/climate::heat_setpoint": [f"{sett} {climate} {living} 21".strip()],
        "conversation/climate::heat_on_then_set": [[f"{on} {climate} {living}".strip(), "21"]],
        "conversation/house::scene_all_off": [f"{on} alloff"],
        "conversation/house::clock_calendar_weather": [clock, calendar, weather],
        "conversation/house::false_done_forbidden": ["capital france"],
        "conversation/media::tv_not_light": [f"{on} {tv} {living}".strip()],
        "conversation/media::music_play_in_room": [f"{play} {music} {kitchen}".strip()],
        "conversation/media::music_artist_then_now_playing": [
            [f"{play} queen {kitchen}".strip(), f"{query} {music}".strip()]
        ],
        "conversation/media::volume_down": [f"{quiet} {music}".strip()],
        "conversation/media::next_and_pause": [[f"{play} {music} {kitchen}".strip(), nxt, pause]],
    }
