"""Realize catalog cases as native sentences from a locale lexicon."""

from __future__ import annotations

from pathlib import Path

import yaml

from lex import color_word, floor_word, room

PHRASES = Path(__file__).resolve().parent / "phrases"
DEFAULTS = {
    "on_area": ["{on} {light} {room}", "{light} {room} {on}", "{on} {all} {light} {room}", "{all} {light} {room} {on}", "{on} {room} {light}", "{room} {light} {on}"],
    "off_area": ["{off} {light} {room}", "{light} {room} {off}", "{off} {all} {light} {room}", "{all} {light} {room} {off}", "{off} {room} {light}", "{room} {light} {off}"],
    "on_fixture": ["{on} {fixture} {room}", "{fixture} {room} {on}", "{on} {room} {fixture}", "{room} {fixture} {on}", "{on} {fixture}", "{fixture} {on}"],
    "off_fixture": ["{off} {fixture} {room}", "{fixture} {room} {off}", "{off} {room} {fixture}", "{room} {fixture} {off}", "{off} {fixture}", "{fixture} {off}"],
    "set_bright": ["{set} {fixture} {room} {n}", "{fixture} {room} {n}", "{room} {fixture} {n}", "{set} {room} {fixture} {n}", "{set} {fixture} {n}", "{room} {n} {fixture}"],
    "set_color": ["{set} {fixture} {room} {color}", "{set} {light} {room} {color}", "{light} {room} {color}", "{fixture} {room} {color}", "{set} {fixture} {color}", "{room} {light} {color}"],
    "set_temp": ["{set} {climate} {room} {n}", "{climate} {room} {n}", "{room} {climate} {n}", "{set} {room} {climate} {n}"],
    "get_temp": ["{query} {climate} {room}", "{climate} {room}", "{room} {climate}", "{query} {room} {climate}"],
    "set_temp_ac": ["{set} {ac} {room} {n}", "{ac} {room} {n}", "{set} {room} {ac} {n}"],
    "get_temp_ac": ["{query} {ac} {room}", "{ac} {room}", "{query} {room} {ac}"],
    "open_cover": ["{open} {cover} {room}", "{cover} {room} {open}", "{open} {room} {cover}", "{room} {cover} {open}", "{open} {cover}", "{cover} {open}"],
    "close_cover": ["{close} {cover} {room}", "{cover} {room} {close}", "{close} {room} {cover}", "{room} {cover} {close}", "{close} {cover}", "{cover} {close}"],
    "set_pos": ["{set} {cover} {room} {n}", "{cover} {room} {n}", "{set} {cover} {n}", "{room} {cover} {n}", "{set} {room} {cover} {n}", "{cover} {n}"],
    "lock": ["{lock_v} {lock} {room}", "{lock_v} {door}", "{lock} {room} {lock_v}", "{room} {lock} {lock_v}", "{lock_v} {lock}", "{door} {lock_v}"],
    "unlock": ["{unlock} {lock} {room}", "{unlock} {door}", "{lock} {room} {unlock}", "{room} {lock} {unlock}", "{unlock} {lock}", "{door} {unlock}"],
    "fan_on": ["{fan} {room} {on}", "{on} {fan} {room}", "{on} {room} {fan}", "{room} {fan} {on}", "{fan} {on}", "{on} {fan}"],
    "fan_off": ["{off} {fan} {room}", "{fan} {room} {off}", "{off} {room} {fan}", "{room} {fan} {off}", "{off} {fan}", "{fan} {off}"],
    "fan_speed": ["{set} {fan} {room} {n}", "{fan} {room} {n}", "{set} {fan} {n}", "{room} {fan} {n}", "{set} {room} {fan} {n}", "{fan} {n}"],
    "switch_on": ["{on} {appliance} {room}", "{appliance} {room} {on}", "{on} {room} {appliance}", "{room} {appliance} {on}", "{on} {appliance}", "{appliance} {on}"],
    "switch_off": ["{off} {appliance} {room}", "{appliance} {room} {off}", "{off} {room} {appliance}", "{room} {appliance} {off}", "{off} {appliance}", "{appliance} {off}"],
    "vac_start": ["{on} {vacuum}", "{vacuum} {on}", "{on} the {vacuum}", "{vacuum} {on} {room}", "{on} {vacuum} {room}", "{room} {vacuum} {on}"],
    "vac_dock": ["{off} {vacuum}", "{vacuum} {off}", "{dock} {vacuum}", "{vacuum} {dock}", "{vacuum} {home}", "{home} {vacuum}"],
    "media_on": ["{tv} {room} {on}", "{on} {tv} {room}", "{on} {room} {tv}", "{room} {tv} {on}", "{tv} {on}", "{on} {tv}"],
    "media_off": ["{off} {tv} {room}", "{tv} {room} {off}", "{off} {room} {tv}", "{room} {tv} {off}", "{off} {tv}", "{tv} {off}"],
    "media_pause": ["{pause} {tv} {room}", "{tv} {room} {pause}", "{pause} {music} {room}", "{music} {room} {pause}", "{pause} {tv} {room}", "{tv} {pause} {room}"],
    "media_vol": ["{set} {music} {room} {n}", "{music} {room} {n}", "{set} {tv} {room} {n}", "{tv} {room} {n}", "{set} {music} {n}", "{music} {n}"],
    "media_next": ["{next} {track} {room}", "{next} {track}"],
    "media_prev": ["{prev} {track} {room}", "{prev} {track}"],
    "play_search": ["{play} {search} {room}", "{play} {search} {music} {room}"],
    "play_album": ["{play} {album} {search} {by} {artist} {room}", "{play} {album} {search} {by} {artist}"],
    "play_radio": ["{play} {search} radio mode {room}", "{play} {search} radio mode"],
    "play_queue": ["{play} {search} queue {room}", "{play} {search} queue"],
    "now_playing": ["{query} {music} {room}", "{query} {track} {room}"],
    "scene": ["{on} {scene_name}", "{scene_name} {on}", "{on} the {scene_name}", "{scene_name}", "{play} {scene_name}", "{scene_name} {play}"],
    "script": ["{on} {scene_name}", "{scene_name} {on}", "{on} the {scene_name}", "{scene_name}", "{play} {scene_name}", "{scene_name} {play}"],
    "timer_start": ["{on} {timer} {n} {minutes}", "{timer} {n} {minutes}", "{on} {timer_name} {n}", "{timer} {timer_name} {n} {minutes}", "{set} {timer} {n}", "{timer} {n}"],
    "timer_cancel": ["{off} {timer}", "{timer} {off}", "{off} the {timer}", "{timer} {done}", "{off} {timer_name}", "{timer} {timer_name} {off}"],
    "list_add": ["{add} {item} {list}", "{item} {list}", "{add} {item}", "{list} {item}", "{add} {item} {to} {list}", "{item} {add} {list}"],
    "list_done": ["{done} {item}", "{item} {done}", "{done} {item} {list}", "{item} {list} {done}", "{off} {item}", "{item} {off}"],
    "floor_on": ["{on} {all} {light} {floor}", "{light} {floor} {on}", "{on} {light} {floor}", "{floor} {light} {on}", "{on} {all} {light} {floor}", "{all} {light} {floor} {on}"],
    "floor_off": ["{off} {all} {light} {floor}", "{light} {floor} {off}", "{off} {light} {floor}", "{floor} {light} {off}", "{off} {all} {light} {floor}", "{all} {light} {floor} {off}"],
    "all_except": ["{off} {all} {light} {except} {room}", "{all} {light} {off} {except} {room}", "{off} {light} {except} {room}", "{all} {off} {except} {room}", "{off} {all} {except} {room}", "{light} {off} {except} {room}"],
    "all_except_on": ["{on} {all} {light} {except} {room}", "{all} {light} {on} {except} {room}", "{on} {light} {except} {room}", "{all} {on} {except} {room}", "{on} {all} {except} {room}", "{light} {on} {except} {room}"],
    "except_fixture": ["{off} {all} {light} {except} {skip_fixture} {room}", "{all} {light} {off} {except} {skip_fixture} {room}"],
    "query_entity": ["{query} {target} {room}", "{query} {target}", "{target} {room}", "{room} {target}", "{query} {room} {target}", "{target} {query}"],
    "multi_and": ["{on} {light} {room} {and} {room2}", "{light} {room} {and} {room2} {on}", "{on} {room} {and} {room2}", "{room} {and} {room2} {on}", "{on} {light} {room} {and} {light} {room2}", "{room} {light} {and} {room2}"],
    "multi_off": ["{off} {light} {room} {and} {room2}", "{light} {room} {and} {room2} {off}", "{off} {room} {and} {room2}", "{room} {and} {room2} {off}", "{off} {light} {room} {and} {light} {room2}", "{room} {light} {and} {room2} {off}"],
    "multi_fixtures": ["{on} {room} {fixture} {and} {room2} {fixture2}", "{on} {fixture} {and} {fixture2}", "{room} {fixture} {and} {room2} {fixture2} {on}", "{fixture} {and} {fixture2} {on}", "{on} {fixture} {room} {and} {fixture2} {room2}", "{room} {fixture} {and} {fixture2} {on}"],
    "multi_fixtures_off": ["{off} {room} {fixture} {and} {room2} {fixture2}", "{off} {fixture} {and} {fixture2}", "{room} {fixture} {and} {room2} {fixture2} {off}", "{fixture} {and} {fixture2} {off}", "{off} {fixture} {room} {and} {fixture2} {room2}", "{room} {fixture} {and} {fixture2} {off}"],
    "multi_three": ["{on} {light} {room} {and} {room2} {and} {room3}", "{light} {room} {room2} {and} {room3} {on}", "{on} {room} {and} {room2} {and} {room3}", "{room} {and} {room2} {and} {room3} {on}", "{on} {light} {room} {room2} {room3}", "{room} {room2} {room3} {light} {on}"],
    "multi_three_off": ["{off} {light} {room} {and} {room2} {and} {room3}", "{light} {room} {room2} {and} {room3} {off}", "{off} {room} {and} {room2} {and} {room3}", "{room} {and} {room2} {and} {room3} {off}", "{off} {light} {room} {room2} {room3}", "{room} {room2} {room3} {light} {off}"],
    "multi_off_lock": ["{off} {light} {room} {and} {lock_v} {door}", "{off} {room} {and} {lock_v} {door}", "{light} {room} {off} {and} {lock_v} {lock}", "{off} {light} {room} {and} {lock_v} {lock}", "{room} {off} {and} {door} {lock_v}", "{off} {all} {light} {room} {and} {lock_v} {door}"],
    "except_in_area": ["{off} {light} {room} {except} {skip_fixture}", "{off} {all} {light} {room} {except} {skip_fixture}", "{light} {room} {off} {except} {skip_fixture}", "{off} {room} {except} {skip_fixture}", "{all} {light} {room} {off} {except} {skip_fixture}", "{room} {off} {except} {skip_fixture}"],
    "except_two": ["{off} {all} {light} {except} {room} {and} {room2}", "{all} {light} {off} {except} {room} {and} {room2}", "{off} {light} {except} {room} {and} {room2}", "{all} {off} {except} {room} {and} {room2}", "{off} {all} {except} {room} {and} {room2}", "{light} {off} {except} {room} {room2}"],
    "except_fixture_on": ["{on} {all} {light} {except} {skip_fixture} {room}", "{all} {light} {on} {except} {skip_fixture} {room}"],
    "floor_except": ["{off} {all} {light} {floor} {except} {room}", "{light} {floor} {off} {except} {room}", "{off} {floor} {except} {room}", "{all} {light} {floor} {off} {except} {room}", "{off} {light} {floor} {except} {room}", "{floor} {off} {except} {room}"],
    "multi_covers": ["{open} {cover} {room} {and} {room2}", "{cover} {room} {and} {room2} {open}", "{open} {room} {and} {room2}", "{room} {and} {room2} {cover} {open}", "{open} {cover} {room} {and} {cover} {room2}", "{cover} {room} {and} {room2} {open}"],
    "multi_covers_close": ["{close} {cover} {room} {and} {room2}", "{cover} {room} {and} {room2} {close}", "{close} {room} {and} {room2}", "{room} {and} {room2} {cover} {close}", "{close} {cover} {room} {and} {cover} {room2}", "{cover} {room} {and} {room2} {close}"],
    "multi_climate": ["{set} {climate} {room} {and} {room2} {n}", "{climate} {room} {and} {room2} {n}", "{set} {room} {and} {room2} {n}", "{room} {and} {room2} {climate} {n}", "{set} {climate} {room} {and} {climate} {room2} {n}", "{climate} {n} {room} {and} {room2}"],
    "multi_bright": ["{set} {light} {room} {and} {room2} {n}", "{light} {room} {and} {room2} {n}", "{set} {room} {and} {room2} {n}", "{room} {and} {room2} {n}", "{set} {light} {room} {and} {light} {room2} {n}", "{room} {and} {room2} {light} {n}"],
    "multi_color": ["{set} {light} {room} {and} {room2} {color}", "{light} {room} {and} {room2} {color}", "{room} {and} {room2} {color}", "{set} {room} {and} {room2} {color}", "{light} {room} {color} {and} {room2}", "{room} {and} {room2} {light} {color}"],
    "query_two": ["{query} {light} {room} {and} {room2}", "{query} {target} {room} {and} {room2}", "{light} {room} {and} {room2}", "{query} {room} {and} {room2}", "{status} {room} {and} {room2}", "{room} {and} {room2} {query}"],
    "query_and_off": ["{query} {climate} {room} {and} {off} {light} {room2}", "{off} {light} {room2} {and} {query} {climate} {room}", "{climate} {room} {and} {light} {room2} {off}", "{query} {room} {and} {off} {room2}", "{off} {room2} {and} {query} {climate}", "{light} {room2} {off} {and} {climate} {room}"],
    "multi_locks": ["{lock_v} {front_door} {and} {garage_door}", "{lock_v} {lock} {room} {and} {room2}", "{lock_v} {door} {and} {lock} {room2}", "{room} {and} {room2} {lock_v}", "{lock_v} {room} {and} {room2}", "{door} {room} {and} {room2} {lock_v}"],
    "scene_and_off": ["{on} {scene_name} {and} {off} {light} {room}", "{off} {light} {room} {and} {on} {scene_name}", "{scene_name} {and} {light} {room} {off}", "{on} {scene_name} {and} {room} {off}", "{scene_name} {on} {and} {off} {room}", "{off} {room} {and} {scene_name}"],
    "clock": ["{clock}", "{query} {clock}", "{clock} {query}", "{query} {clock} {clock}", "{clock} {clock}", "{query}"],
    "weather": ["{weather}", "{query} {weather}", "{weather} {query}", "{query} {weather} {weather}", "{weather} {weather}", "{query}"],
}

DE_EXTRAS = {
    "on_area": ["alle lichter im {room} an", "lichter {room} an"],
    "off_area": ["alle lichter im {room} aus", "lichter {room} aus"],
    "on_fixture": ["{fixture} {room} an", "mach {fixture} im {room} an"],
    "off_fixture": ["{fixture} {room} aus"],
    "set_bright": ["stelle {fixture} {room} auf {n}", "dimme {room} auf {n}"],
    "set_color": ["{fixture} {room} {color}", "licht {room} {color}"],
    "set_temp": ["heizung {room} auf {n}", "stelle heizung {room} auf {n}"],
    "get_temp": ["wie warm ist es im {room}", "temperatur {room}"],
    "set_temp_ac": ["klima {room} auf {n}", "stelle klima {room} auf {n}"],
    "get_temp_ac": ["wie warm ist klima im {room}", "klima {room}"],
    "open_cover": ["oeffne rollo {room}", "rollo {room} auf"],
    "close_cover": ["schliesse rollo {room}", "rollo {room} zu"],
    "lock": ["{lock_v} {lock} {room}", "schliess die haustuer"],
    "unlock": ["{unlock} {lock} {room}", "oeffne die haustuer"],
    "play_search": ["spiel {search} im {room}", "spiele {search} im {room}"],
    "play_album": ["spiel das album {search} von {artist} im {room}"],
    "play_radio": ["spiel {search} radio modus im {room}"],
    "now_playing": ["was laeuft im {room}"],
    "all_except": ["alle lichter aus ausser {room}", "alle lichter aus ohne {room}"],
    "all_except_on": ["alle lichter an ausser {room}", "alle lichter an ohne {room}"],
    "except_fixture": ["alle lichter aus ausser {skip_fixture} im {room}"],
    "multi_off": ["licht {room} und {room2} aus"],
    "multi_fixtures": ["mach {fixture} im {room} an und {fixture2} im {room2} an"],
    "multi_fixtures_off": ["mach {fixture} im {room} aus und {fixture2} im {room2} aus"],
    "multi_three": ["licht {room} und {room2} und {room3} an"],
    "multi_three_off": ["licht {room} aus und licht {room2} aus und licht {room3} aus"],
    "clock": ["wie spaet ist es", "wie viel uhr"],
    "weather": ["wie ist das wetter", "wetter"],
    "vac_start": ["staubsauger starten", "sauger an"],
    "vac_dock": ["staubsauger zurueck", "sauger zurueck"],
    "scene": ["{scene_name} an"],
    "script": ["{scene_name} an"],
    "multi_and": ["licht {room} und {room2} an"],
    "multi_off_lock": ["alle lichter im {room} aus und schliess die haustuer"],
    "fan_on": ["luefter {room} an", "mach luefter im {room} an"],
    "fan_off": ["luefter {room} aus"],
    "fan_speed": ["stelle luefter {room} auf {n}"],
    "switch_on": ["{appliance} {room} an"],
    "switch_off": ["{appliance} {room} aus"],
    "set_pos": ["stelle rollo {room} auf {n}"],
    "media_on": ["tv {room} an"],
    "media_off": ["tv {room} aus"],
    "media_pause": ["pause tv {room}"],
    "media_vol": ["stelle lautstaerke im {room} auf {n}"],
    "media_next": ["naechster titel im {room}"],
    "media_prev": ["vorheriger titel im {room}"],
    "play_queue": ["queue {search} im {room}"],
    "timer_start": ["starte {timer_name} timer {n} minuten"],
    "timer_cancel": ["timer aus", "timer abbrechen"],
    "list_add": ["setz {item} auf die liste"],
    "list_done": ["{item} erledigt"],
    "floor_on": ["alle lichter {floor} an"],
    "floor_off": ["alle lichter {floor} aus"],
    "query_entity": ["status {target} {room}", "wie ist {target} {room}"],
    "except_in_area": ["alle lichter im {room} aus ausser {skip_fixture}"],
    "except_two": ["alle lichter aus ausser {room} und {room2}"],
    "except_fixture_on": ["alle lichter an ausser {skip_fixture} {room}"],
    "floor_except": ["alle lichter {floor} aus ausser {room}"],
    "multi_covers": ["oeffne rollo {room} und {room2}"],
    "multi_covers_close": ["schliesse rollo {room} und {room2}"],
    "multi_climate": ["heizung {room} und {room2} auf {n}"],
    "multi_bright": ["dimme {room} und {room2} auf {n}"],
    "multi_color": ["licht {room} und {room2} {color}"],
    "query_two": ["status licht {room} und {room2}"],
    "query_and_off": ["temperatur {room} und alle lichter im {room2} aus"],
    "multi_locks": ["schliess haustuer und garagentor", "{lock_v} {lock} {room} und {room2}"],
    "scene_and_off": ["{scene_name} an und licht {room} aus"],
}

EN_EXTRAS = {
    "on_area": ["turn on the {room} lights", "activate the {room} lights"],
    "off_area": ["turn off the {room} lights", "turn off the lights in the {room}"],
    "on_fixture": ["turn on the {room} {fixture}", "{room} {fixture} on"],
    "off_fixture": ["turn off the {room} {fixture}", "{room} {fixture} off"],
    "set_bright": ["set the {room} {fixture} to {n}", "{room} brightness {n}"],
    "set_color": ["set the {room} {fixture} to {color}", "{room} lights {color}"],
    "set_temp": ["set the {room} thermostat to {n}", "{room} temperature {n}"],
    "get_temp": ["what's the temperature in the {room}", "{room} thermostat temperature"],
    "set_temp_ac": ["set the {room} ac to {n}", "{room} ac {n}"],
    "get_temp_ac": ["what's the {room} ac temperature", "{room} ac temperature"],
    "open_cover": ["open the {room} blinds", "{room} blinds open"],
    "close_cover": ["close the {room} blinds", "{room} blinds closed"],
    "lock": ["lock the {room} {lock}", "lock the {room} door", "lock the front door"],
    "unlock": ["unlock the {room} {lock}", "unlock the {room} door", "unlock the front door"],
    "play_search": ["play {search} in the {room}", "play {search} in {room}"],
    "play_album": ["play the album {search} by {artist} in the {room}"],
    "play_radio": ["play {search} using radio mode in the {room}"],
    "now_playing": ["what's playing in the {room}"],
    "all_except": ["turn off all lights except the {room}", "all lights off except the {room}"],
    "all_except_on": ["turn on all lights except the {room}", "all lights on except the {room}"],
    "except_fixture": ["turn off all lights except the {skip_fixture} in the {room}"],
    "multi_off": ["turn off the {room} and {room2} lights"],
    "multi_fixtures": ["turn on the {room} {fixture} light and turn on the {room2} {fixture2} light"],
    "multi_fixtures_off": ["turn off the {room} {fixture} light and turn off the {room2} {fixture2} light"],
    "multi_three": ["turn on the {room} and {room2} and {room3} lights"],
    "multi_three_off": ["turn off the {room} lights and turn off the {room2} lights and turn off the {room3} lights"],
    "clock": ["what time is it", "what's the time"],
    "weather": ["what's the weather", "weather forecast"],
    "vac_start": ["start the vacuum", "vacuum on"],
    "vac_dock": ["dock the vacuum"],
    "scene": ["turn on {scene_name}", "activate {scene_name}"],
    "script": ["run {scene_name}", "activate {scene_name}"],
    "multi_and": ["turn on the {room} and {room2} lights"],
    "multi_off_lock": ["turn off the {room} lights and lock the front door"],
    "fan_on": ["turn on the {room} fan", "{room} fan on"],
    "fan_off": ["turn off the {room} fan"],
    "fan_speed": ["set the {room} fan to {n}"],
    "switch_on": ["turn on the {room} {appliance}"],
    "switch_off": ["turn off the {room} {appliance}"],
    "set_pos": ["set the {room} blinds to {n}"],
    "media_on": ["turn on the {room} tv"],
    "media_off": ["turn off the {room} tv"],
    "media_pause": ["pause the {room} tv"],
    "media_vol": ["set the volume in the {room} to {n}"],
    "media_next": ["next track in the {room}"],
    "media_prev": ["previous track in the {room}"],
    "play_queue": ["queue {search} in the {room}"],
    "timer_start": ["start the {timer_name} timer for {n} minutes"],
    "timer_cancel": ["cancel the timer", "stop the timer"],
    "list_add": ["add {item} to the list"],
    "list_done": ["{item} done"],
    "floor_on": ["turn on the {floor} lights"],
    "floor_off": ["turn off the {floor} lights"],
    "query_entity": ["what's the {target} in the {room}", "{room} {target} status"],
    "except_in_area": ["turn off all lights in the {room} except the {skip_fixture}"],
    "except_two": ["turn off all lights except the {room} and {room2}"],
    "except_fixture_on": ["turn on all lights except the {room} {skip_fixture}"],
    "floor_except": ["all lights {floor} off except the {room}"],
    "multi_covers": ["open the {room} and {room2} blinds"],
    "multi_covers_close": ["close the {room} and {room2} blinds"],
    "multi_climate": ["set the {room} and {room2} thermostats to {n}"],
    "multi_bright": ["set the {room} and {room2} lights to {n}"],
    "multi_color": ["set the {room} and {room2} lights to {color}"],
    "query_two": ["what's the {room} and {room2} lights"],
    "query_and_off": ["what's the temperature in the {room} and turn off the {room2} lights"],
    "multi_locks": ["lock the front door and the garage door", "lock the {room} and {room2} doors"],
    "scene_and_off": ["turn on {scene_name} and turn off the {room} lights"],
}


def _phrase_path(code: str) -> Path | None:
    wanted = code.lower().replace("_", "-")
    for path in (
        PHRASES / f"{code}.yaml",
        PHRASES / f"{code.replace('-', '_')}.yaml",
        PHRASES / f"{code.lower()}.yaml",
        PHRASES / f"{code.replace('-', '_').lower()}.yaml",
    ):
        if path.exists():
            return path
    if not PHRASES.is_dir():
        return None
    for path in PHRASES.glob("*.yaml"):
        if path.stem.lower().replace("_", "-") == wanted:
            return path
    return None

_YAML_CACHE: dict[str, dict[str, list[str]]] = {}


def _load_extras(code: str) -> dict[str, list[str]]:
    if code in {"de", "en"}:
        return {}
    if code in _YAML_CACHE:
        return _YAML_CACHE[code]
    path = _phrase_path(code)
    if path is None:
        _YAML_CACHE[code] = {}
        return {}
    raw = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    loaded = {key: list(val) for key, val in raw.items() if isinstance(val, list)}
    _YAML_CACHE[code] = loaded
    return loaded


_NEED = {
    "on_area": ("{on}", "{light}", "{room}"),
    "off_area": ("{off}", "{light}", "{room}"),
    "on_fixture": ("{on}", "{fixture}"),
    "off_fixture": ("{off}", "{fixture}"),
    "set_bright": ("{n}", "{room}", "{fixture}"),
    "media_next": ("{next}", "{room}"),
    "media_prev": ("{prev}", "{room}"),
    "media_vol": ("{n}", "{room}"),
    "play_album": ("{play}", "{album}", "{search}", "{room}"),
    "play_radio": ("{play}", "{search}", "{room}"),
    "play_queue": ("{play}", "{search}", "{room}"),
    "now_playing": ("{query}", "{room}"),
    "set_color": ("{color}", "{room}"),
    "set_temp": ("{climate}", "{room}", "{n}"),
    "get_temp": ("{climate}", "{room}"),
    "set_temp_ac": ("{ac}", "{room}", "{n}"),
    "get_temp_ac": ("{ac}", "{room}"),
    "open_cover": ("{open}", "{cover}", "{room}"),
    "close_cover": ("{close}", "{cover}", "{room}"),
    "set_pos": ("{n}", "{room}"),
    "lock": ("{lock_v}", "{room}"),
    "unlock": ("{unlock}", "{room}"),
    "play_search": ("{play}", "{search}", "{room}"),
    "list_add": ("{add}", "{item}", "{list}"),
    "all_except": ("{off}", "{all}", "{light}", "{except}", "{room}"),
    "all_except_on": ("{on}", "{all}", "{light}", "{except}", "{room}"),
    "except_fixture": ("{all}", "{light}", "{except}", "{skip_fixture}", "{room}"),
    "except_fixture_on": ("{on}", "{all}", "{light}", "{except}", "{skip_fixture}", "{room}"),
    "multi_fixtures": ("{and}", "{room}", "{fixture}", "{room2}", "{fixture2}"),
    "multi_fixtures_off": ("{and}", "{room}", "{fixture}", "{room2}", "{fixture2}"),
    "except_in_area": ("{all}", "{except}", "{room}", "{skip_fixture}"),
    "except_two": ("{all}", "{except}", "{room}", "{room2}"),
    "floor_except": ("{all}", "{except}", "{floor}", "{room}"),
    "query_entity": ("{query}", "{room}"),
    "multi_and": ("{and}", "{room}", "{room2}"),
    "multi_off": ("{and}", "{room}", "{room2}"),
    "multi_climate": ("{n}", "{room}", "{room2}"),
}


def _has_need(kind: str, tmpl: str) -> bool:
    if kind in {"play_search", "play_queue"} and "{on}" in tmpl:
        return False
    if kind == "vac_dock" and "dock" in tmpl and "{dock}" not in tmpl and "{off}" not in tmpl:
        return False
    need = _NEED.get(kind)
    return True if not need else all(token in tmpl for token in need)


def _glues_slot_suffix(tmpl: str) -> bool:
    pos = 0
    while True:
        start = tmpl.find("{", pos)
        if start < 0:
            return False
        end = tmpl.find("}", start)
        if end < 0:
            return False
        if end + 1 < len(tmpl) and tmpl[end + 1].isalpha():
            return True
        pos = end + 1


def _hand_extras(code: str) -> dict[str, list[str]]:
    return {"de": DE_EXTRAS, "en": EN_EXTRAS}.get(code, {})


def _yaml_full(code: str, kind: str) -> list[str]:
    if code in {"de", "en"}:
        return []
    return [item for item in (_load_extras(code).get(kind) or []) if _has_need(kind, item) and not _glues_slot_suffix(item)]


def _templates(lex: dict, kind: str, merge_defaults: bool = False) -> list[str]:
    code = lex.get("code", "")
    extra = _hand_extras(code)
    found = list(extra.get(kind) or [])
    if kind == "play_search" and not merge_defaults:
        found = [item for item in found if "{room}" in item and "{on}" not in item] or found
    fallback = [item for item in (DEFAULTS.get(kind) or ["{on} {light}"]) if _has_need(kind, item)]
    if merge_defaults:
        if extra.get(kind):
            found.extend(_yaml_full(code, kind))
        else:
            found = list(fallback)
    if not found:
        found.extend(item for item in fallback if item not in found)
    if not found:
        found = fallback or ["{on} {light}"]
    seen: set[str] = set()
    out = []
    for item in found:
        if item not in seen:
            seen.add(item)
            out.append(item)
    return out


def _fixture(lex: dict, name: str | None) -> str:
    if not name or name == "light":
        return lex["light"]
    word = lex.get(name, name)
    return name if word == lex["light"] else word


def _appliance(lex: dict, name: str | None) -> str:
    return lex.get(name or "switch", lex["switch"])


def _target(lex: dict, case: dict) -> str:
    domain = case.get("domain") or ""
    entity = case.get("entity") or ""
    fixture = case.get("fixture")
    if fixture:
        return _fixture(lex, fixture)
    if "ensuite" in entity:
        return lex.get("ensuite", lex["light"])
    if "window" in entity:
        return lex.get("windowsensor", lex.get("window", "window"))
    if "front_door_sensor" in entity or "doorsensor" in entity:
        return lex.get("doorsensor", lex.get("door", lex["lock"]))
    if "motion" in entity:
        return lex.get("motion", lex.get("door", lex["lock"]))
    if "humidity" in entity:
        return lex.get("humidity", lex["climate"])
    if entity.startswith("sensor.") and "temperature" in entity:
        return lex.get("tempsensor", lex["climate"])
    if "vacuum" in entity:
        return lex["vacuum"]
    if "music" in entity:
        return lex.get("music", lex["tv"])
    if "ac" in entity:
        return lex.get("ac", "ac")
    mapping = {
        "light": lex["light"],
        "cover": lex["cover"],
        "climate": lex["climate"],
        "lock": lex["lock"],
        "media_player": lex["tv"],
        "binary_sensor": lex.get("door", lex["lock"]),
        "sensor": lex["climate"],
        "fan": lex["fan"],
        "vacuum": lex["vacuum"],
    }
    return mapping.get(domain, lex["light"])


def _ctx(lex: dict, case: dict) -> dict[str, str]:
    area = case.get("area") or ""
    scene_key = case.get("scene") or "scene"
    return {
        "on": lex["on"],
        "off": lex["off"],
        "set": lex["set"],
        "query": lex["query"],
        "open": lex["open"],
        "close": lex["close"],
        "light": lex["light"],
        "cover": lex["cover"],
        "climate": lex["climate"],
        "ac": lex.get("ac", "ac"),
        "lock": lex["lock"],
        "lock_v": lex["lock_v"],
        "unlock": lex["unlock"],
        "fan": lex["fan"],
        "vacuum": lex["vacuum"],
        "tv": lex["tv"],
        "music": lex.get("music", lex["media"]),
        "media": lex["media"],
        "timer": lex["timer"],
        "list": lex["list"],
        "and": lex["and"],
        "all": lex["all"],
        "except": lex["except"],
        "add": lex["add"],
        "done": lex["done"],
        "pause": lex["pause"],
        "play": lex["play"] if lex.get("play") and lex.get("play") != lex.get("on") else "play",
        "dock": lex.get("dock") or lex["off"],
        "home": lex.get("vac_home") or lex.get("dock") or lex["off"],
        "next": lex.get("next", "next"),
        "prev": lex.get("prev", "previous"),
        "track": lex.get("track", "track"),
        "minutes": lex["minutes"],
        "door": lex.get("door", lex["lock"]),
        "front_door": lex.get("front_door") or f"front {lex.get('door', lex['lock'])}",
        "garage_door": lex.get("garage_door") or f"garage {lex.get('door', lex['lock'])}",
        "radio": lex.get("radio", "radio"),
        "album": lex.get("album", "album"),
        "by": lex.get("by", "by"),
        "clock": lex.get("clock", lex["query"]),
        "weather": lex.get("weather", "weather"),
        "room": room(lex, area) if area else "",
        "room2": room(lex, case["area2"]) if case.get("area2") else "",
        "room3": room(lex, case["area3"]) if case.get("area3") else "",
        "fixture2": _fixture(lex, case.get("fixture2")),
        "skip_fixture": _fixture(lex, case.get("skip_fixture")) if case.get("skip_fixture") and case.get("skip_fixture") != "light" else (room(lex, case["area"]) if case.get("area") else lex["light"]),
        "floor": floor_word(lex, case["floor"]) if case.get("floor") else "",
        "fixture": _fixture(lex, case.get("fixture")),
        "appliance": _appliance(lex, case.get("appliance")),
        "color": color_word(lex, case["color"]) if case.get("color") else "",
        "n": str(case.get("n", "")),
        "search": case.get("query") or "",
        "artist": case.get("artist") or "",
        "item": case.get("item") or "",
        "scene_name": lex.get(scene_key, scene_key),
        "target": _target(lex, case),
        "timer_name": case.get("timer") or "",
    }


def _usable(case: dict, tmpl: str) -> bool:
    if case.get("kind") in {"multi_off_lock", "multi_locks"}:
        return True
    area = case.get("area") or ""
    return area in {"entryway", "entry", ""} or not any(token in tmpl.lower() for token in ("front", "haustuer", "haustür"))


def _kind(case: dict) -> str:
    kind = case["kind"]
    if "ac" in (case.get("entity") or "") and kind in {"set_temp", "get_temp"}:
        return f"{kind}_ac"
    return kind


def render(lex: dict, case: dict, limit: int, merge_defaults: bool = False) -> list[str]:
    ctx = _ctx(lex, case)
    sentences = []
    for tmpl in _templates(lex, _kind(case), merge_defaults):
        if not _usable(case, tmpl):
            continue
        try:
            text = " ".join(tmpl.format(**ctx).split())
        except KeyError:
            continue
        if text and text not in sentences:
            sentences.append(text)
        if len(sentences) >= limit:
            break
    return sentences or [f"{lex['on']} {lex['light']}"]
