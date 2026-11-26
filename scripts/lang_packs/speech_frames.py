"""Post-execute speech frames that used to live as DE/EN branches in the engine.

Filled for every generated locale after speech_slots. de/en are handwritten.
"""

from __future__ import annotations

import json
from pathlib import Path

from lang_packs.native_apply import is_script_pack

_EMPTY = json.loads((Path(__file__).with_name("empty_place.json")).read_text(encoding="utf-8"))

# HA state → spoken. Locales may overlay; missing keys fall through to the raw HA token.
_STATES = {
    "en": [
        ("on", "on"), ("off", "off"), ("unavailable", "unavailable"), ("unknown", "unknown"),
        ("open", "open"), ("closed", "closed"), ("locked", "locked"), ("unlocked", "unlocked"),
        ("playing", "playing"), ("paused", "paused"), ("idle", "idle"),
        ("heat", "heating"), ("cool", "cooling"),
        ("cloudy", "cloudy"), ("partlycloudy", "partly cloudy"), ("rainy", "rainy"),
        ("sunny", "sunny"), ("clear", "clear"),
    ],
    "de": [
        ("on", "an"), ("off", "aus"), ("unavailable", "nicht da"), ("unknown", "unbekannt"),
        ("open", "offen"), ("closed", "zu"), ("locked", "zu"), ("unlocked", "offen"),
        ("playing", "spielt"), ("paused", "pausiert"), ("idle", "bereit"),
        ("heat", "heizt"), ("cool", "kühlt"),
        ("cloudy", "bewölkt"), ("partlycloudy", "teilweise bewölkt"), ("rainy", "regnerisch"),
        ("sunny", "sonnig"), ("clear", "klar"),
    ],
    "fr": [
        ("on", "allumée"), ("off", "éteinte"), ("open", "ouvert"), ("closed", "ferme"),
        ("locked", "verrouille"), ("unlocked", "deverrouille"),
        ("playing", "en lecture"), ("paused", "en pause"), ("idle", "inactif"),
        ("heat", "chauffe"), ("cool", "refroidit"),
    ],
    "nl": [
        ("on", "aan"), ("off", "uit"), ("open", "open"), ("closed", "dicht"),
        ("playing", "speelt"), ("paused", "gepauzeerd"), ("idle", "inactief"),
    ],
    "es": [
        ("on", "encendido"), ("off", "apagado"), ("open", "abierto"), ("closed", "cerrado"),
        ("playing", "reproduciendo"), ("paused", "en pausa"),
    ],
    "it": [
        ("on", "acceso"), ("off", "spento"), ("open", "aperto"), ("closed", "chiuso"),
        ("playing", "in riproduzione"), ("paused", "in pausa"),
    ],
    "ja": [
        ("on", "オン"), ("off", "オフ"), ("open", "開"), ("closed", "閉"),
        ("playing", "再生中"), ("paused", "一時停止"), ("idle", "待機"),
        ("heat", "暖房"), ("cool", "冷房"), ("locked", "施錠"), ("unlocked", "解錠"),
    ],
    "zh-CN": [
        ("on", "开"), ("off", "关"), ("open", "开"), ("closed", "关"),
        ("playing", "播放中"), ("paused", "已暂停"), ("idle", "空闲"),
        ("heat", "制热"), ("cool", "制冷"), ("locked", "已锁"), ("unlocked", "已开锁"),
    ],
    "ar": [
        ("on", "يعمل"), ("off", "مطفأ"), ("open", "مفتوح"), ("closed", "مغلق"),
        ("playing", "يشغل"), ("paused", "متوقف"), ("locked", "مقفل"), ("unlocked", "مفتوح"),
    ],
    "he": [
        ("on", "דולק"), ("off", "כבוי"), ("open", "פתוח"), ("closed", "סגור"),
        ("playing", "מתנגן"), ("paused", "מושהה"),
    ],
    "ko": [
        ("on", "켜짐"), ("off", "꺼짐"), ("open", "열림"), ("closed", "닫힘"),
        ("playing", "재생 중"), ("paused", "일시정지"),
    ],
    "hi": [
        ("on", "चालू"), ("off", "बंद"), ("open", "खुला"), ("closed", "बंद"),
        ("playing", "चल रहा"), ("paused", "रुका"),
    ],
}

_KIND = {
    "en": ("window", "door", "motion", "smoke"),
    "de": ("Fenstersensoren", "Türsensoren", "Bewegungsmelder", "Rauchmelder"),
    "fr": ("fenetres", "portes", "detecteurs de mouvement", "detecteurs de fumee"),
    "nl": ("raamsensoren", "deursensoren", "bewegingssensoren", "rookmelders"),
    "ja": ("窓", "ドア", "人感センサー", "火災報知器"),
    "zh-CN": ("窗户", "门", "移动传感器", "烟雾报警器"),
    "ar": ("نوافذ", "أبواب", "مستشعرات حركة", "أجهزة دخان"),
    "he": ("חלונות", "דלתות", "חיישני תנועה", "גלאי עשן"),
    "ko": ("창문", "문", "동작 센서", "연기 감지기"),
    "hi": ("खिड़कियाँ", "दरवाज़े", "मोशन सेंसर", "स्मोक अलार्म"),
}

_FRAMES = {
    "en": dict(
        entity_state="{target} is {state}.",
        climate_at="{target} is at {n} {unit}.",
        temp_reading="{target} is {n} {unit}.",
        light_noun="light",
        lights_on="{n} lights on",
        lights_off="{n} lights off",
        unit_c="degrees",
        unit_f="Fahrenheit",
        reminder="Reminder: {task}.",
        presence_yes="occupied",
        presence_no="empty",
        list_none_found="No {kind} sensors were found.",
        list_none_state="No windows are {state}.",
        list_state="These windows are {state}: {names}.",
        list_all="Window sensors: {names}.",
        state_open="open",
        state_closed="closed",
        media_volume_set="{target} volume is set to {n} percent.",
        media_volume_up="{target} volume was raised.",
        media_volume_down="{target} volume was lowered.",
        media_volume_query="Volume is {n} percent.",
        media_playing="Playing {title}.",
        media_playing_by="Playing {title} by {artist}.",
        queue_empty="The queue is empty.",
    ),
    "de": dict(
        entity_state="{target} ist {state}.",
        climate_at="{target} auf {n} {unit}.",
        temp_reading="{target} {n} {unit}.",
        light_noun="Licht",
        lights_on="{n} Lichter an",
        lights_off="{n} Lichter aus",
        unit_c="Grad",
        unit_f="Fahrenheit",
        reminder="Erinnerung: {task}.",
        presence_yes="jemand da",
        presence_no="niemand da",
        list_none_found="Es wurden keine {kind} gefunden.",
        list_none_state="Es sind keine Fenster {state}.",
        list_state="Diese Fenster sind {state}: {names}.",
        list_all="Fenstersensoren: {names}.",
        state_open="offen",
        state_closed="geschlossen",
        media_volume_set="Die Lautstärke von {target} ist auf {n} Prozent.",
        media_volume_up="Die Lautstärke von {target} wurde erhöht.",
        media_volume_down="Die Lautstärke von {target} wurde verringert.",
        media_volume_query="Lautstärke ist {n} Prozent.",
        media_playing="Ich spiele {title}.",
        media_playing_by="Ich spiele {title} von {artist}.",
        queue_empty="Die Warteschlange ist leer.",
    ),
    "fr": dict(
        entity_state="{target} est {state}.",
        climate_at="{target} est a {n} {unit}.",
        temp_reading="{target} {n} {unit}.",
        light_noun="lumiere",
        lights_on="{n} lumieres allumees",
        lights_off="{n} lumieres eteintes",
        unit_c="degres",
        unit_f="Fahrenheit",
        reminder="Rappel: {task}.",
        presence_yes="occupe",
        presence_no="vide",
        list_none_found="Aucun capteur {kind} trouve.",
        list_none_state="Aucune {kind} {state}.",
        list_state="Ces {kind} sont {state}: {names}.",
        state_open="ouvertes",
        state_closed="fermees",
        media_volume_set="Le volume de {target} est a {n} pour cent.",
        media_volume_up="Le volume de {target} a ete augmente.",
        media_volume_down="Le volume de {target} a ete diminue.",
        media_volume_query="Le volume est a {n} pour cent.",
        media_playing="Je joue {title}.",
        media_playing_by="Je joue {title} de {artist}.",
        queue_empty="La file est vide.",
    ),
    "nl": dict(
        entity_state="{target} is {state}.",
        climate_at="{target} staat op {n} {unit}.",
        temp_reading="{target} {n} {unit}.",
        light_noun="licht",
        lights_on="{n} lichten aan",
        lights_off="{n} lichten uit",
        unit_c="graden",
        unit_f="Fahrenheit",
        reminder="Herinnering: {task}.",
        presence_yes="bezet",
        presence_no="leeg",
        list_none_found="Er zijn geen {kind} gevonden.",
        list_none_state="Er staan geen ramen {state}.",
        list_state="Deze ramen staan {state}: {names}.",
        list_all="Raamsensoren: {names}.",
        state_open="open",
        state_closed="gesloten",
        media_volume_set="Het volume van {target} staat op {n} procent.",
        media_volume_up="Het volume van {target} is verhoogd.",
        media_volume_down="Het volume van {target} is verlaagd.",
        media_volume_query="Volume is {n} procent.",
        media_playing="Ik speel {title}.",
        media_playing_by="Ik speel {title} van {artist}.",
        queue_empty="De wachtrij is leeg.",
    ),
    "ja": dict(
        entity_state="{target}は{state}。",
        climate_at="{target}は{n}{unit}。",
        temp_reading="{target} {n}{unit}。",
        light_noun="ライト",
        lights_on="ライト{n}個オン",
        lights_off="ライト{n}個オフ",
        unit_c="度",
        unit_f="華氏",
        reminder="リマインダー: {task}。",
        presence_yes="在室",
        presence_no="不在",
        list_none_found="{kind}センサーがありません。",
        list_none_state="{kind}は{state}ではありません。",
        list_state="{kind}が{state}: {names}。",
        state_open="開",
        state_closed="閉",
        media_volume_set="{target}の音量は{n}パーセント。",
        media_volume_up="{target}の音量を上げました。",
        media_volume_down="{target}の音量を下げました。",
        media_volume_query="音量は{n}パーセント。",
        media_playing="{title}を再生中。",
        media_playing_by="{artist}の{title}を再生中。",
        queue_empty="キューは空です。",
    ),
    "zh-CN": dict(
        entity_state="{target}是{state}。",
        climate_at="{target}设为{n}{unit}。",
        temp_reading="{target} {n}{unit}。",
        light_noun="灯",
        lights_on="{n}盏灯开着",
        lights_off="{n}盏灯关着",
        unit_c="度",
        unit_f="华氏度",
        reminder="提醒：{task}。",
        presence_yes="有人",
        presence_no="无人",
        list_none_found="没有找到{kind}传感器。",
        list_none_state="没有{kind}处于{state}。",
        list_state="这些{kind}是{state}：{names}。",
        state_open="打开",
        state_closed="关闭",
        media_volume_set="{target}音量设为{n}%。",
        media_volume_up="{target}音量已调高。",
        media_volume_down="{target}音量已调低。",
        media_volume_query="音量是{n}%。",
        media_playing="正在播放{title}。",
        media_playing_by="正在播放{artist}的{title}。",
        queue_empty="队列为空。",
    ),
    "ar": dict(
        entity_state="{target} {state}.",
        climate_at="{target} على {n} {unit}.",
        temp_reading="{target} {n} {unit}.",
        light_noun="ضوء",
        lights_on="{n} أضواء تعمل",
        lights_off="{n} أضواء مطفأة",
        unit_c="درجة",
        unit_f="فهرنهايت",
        reminder="تذكير: {task}.",
        presence_yes="مشغول",
        presence_no="فارغ",
        list_none_found="لا توجد مستشعرات {kind}.",
        list_none_state="لا {kind} {state}.",
        list_state="هذه ال{kind} {state}: {names}.",
        state_open="مفتوحة",
        state_closed="مغلقة",
        media_volume_set="مستوى صوت {target} على {n} بالمئة.",
        media_volume_up="رفع صوت {target}.",
        media_volume_down="خفض صوت {target}.",
        media_volume_query="المستوى {n} بالمئة.",
        media_playing="تشغيل {title}.",
        media_playing_by="تشغيل {title} لـ {artist}.",
        queue_empty="قائمة الانتظار فارغة.",
    ),
    "he": dict(
        entity_state="{target} {state}.",
        climate_at="{target} על {n} {unit}.",
        temp_reading="{target} {n} {unit}.",
        light_noun="אור",
        lights_on="{n} אורות דולקים",
        lights_off="{n} אורות כבויים",
        unit_c="מעלות",
        unit_f="פרנהייט",
        reminder="תזכורת: {task}.",
        presence_yes="תפוס",
        presence_no="ריק",
        list_none_found="לא נמצאו חיישני {kind}.",
        list_none_state="אין {kind} {state}.",
        list_state="{kind} {state}: {names}.",
        state_open="פתוחים",
        state_closed="סגורים",
        media_volume_set="עוצמת {target} על {n} אחוז.",
        media_volume_up="עוצמת {target} הוגברה.",
        media_volume_down="עוצמת {target} הונמכה.",
        media_volume_query="העוצמה {n} אחוז.",
        media_playing="מנגן {title}.",
        media_playing_by="מנגן {title} של {artist}.",
        queue_empty="התור ריק.",
    ),
    "ko": dict(
        entity_state="{target} {state}.",
        climate_at="{target} {n}{unit}.",
        temp_reading="{target} {n}{unit}.",
        light_noun="조명",
        lights_on="조명 {n}개 켜짐",
        lights_off="조명 {n}개 꺼짐",
        unit_c="도",
        unit_f="화씨",
        reminder="알림: {task}.",
        presence_yes="있음",
        presence_no="없음",
        list_none_found="{kind} 센서가 없습니다.",
        list_none_state="{kind}이(가) {state} 상태가 아닙니다.",
        list_state="{kind} {state}: {names}.",
        state_open="열림",
        state_closed="닫힘",
        media_volume_set="{target} 볼륨 {n}퍼센트.",
        media_volume_up="{target} 볼륨을 올렸습니다.",
        media_volume_down="{target} 볼륨을 낮췄습니다.",
        media_volume_query="볼륨 {n}퍼센트.",
        media_playing="{title} 재생 중.",
        media_playing_by="{artist}의 {title} 재생 중.",
        queue_empty="대기열이 비어 있습니다.",
    ),
    "hi": dict(
        entity_state="{target} {state} है।",
        climate_at="{target} {n} {unit} पर।",
        temp_reading="{target} {n} {unit}।",
        light_noun="लाइट",
        lights_on="{n} लाइट चालू",
        lights_off="{n} लाइट बंद",
        unit_c="डिग्री",
        unit_f="फ़ारेनहाइट",
        reminder="याद दिलाना: {task}।",
        presence_yes="भरा",
        presence_no="खाली",
        list_none_found="कोई {kind} सेंसर नहीं।",
        list_none_state="कोई {kind} {state} नहीं।",
        list_state="ये {kind} {state}: {names}।",
        state_open="खुले",
        state_closed="बंद",
        media_volume_set="{target} वॉल्यूम {n} प्रतिशत।",
        media_volume_up="{target} वॉल्यूम बढ़ाया।",
        media_volume_down="{target} वॉल्यूम घटाया।",
        media_volume_query="वॉल्यूम {n} प्रतिशत।",
        media_playing="{title} चल रहा है।",
        media_playing_by="{artist} का {title} चल रहा है।",
        queue_empty="कतार खाली है।",
    ),
}

# Regional / sibling aliases → base frame table.
_ALIAS = {
    "de-AT": "de", "de-CH": "de", "en-GB": "en", "pt-BR": "pt",
    "zh-TW": "zh-CN", "zh-HK": "zh-CN", "pt": "es", "ca": "es", "gl": "es",
    "sv": "en", "da": "en", "nb": "en", "fi": "en", "af": "en",
    "cs": "en", "sk": "en", "pl": "en", "hu": "en", "hr": "en", "sl": "en",
    "ro": "en", "tr": "en", "id": "en", "ms": "en", "sw": "en", "vi": "en",
    "cy": "en", "et": "en", "eu": "en", "ga": "en", "is": "en", "lb": "de",
    "kw": "en", "lt": "en", "lv": "en", "bg": "en", "el": "en", "sr": "en",
    "sr-Latn": "en", "uk": "en", "fa": "ar", "ur": "ar", "th": "en",
    "bn": "hi", "gu": "hi", "kn": "hi", "ml": "hi", "mr": "hi",
    "ta": "hi", "te": "hi", "pa": "hi", "ne": "hi", "hy": "en", "ka": "en", "mn": "en",
}


def _base(code: str) -> str:
    if code in _FRAMES:
        return code
    return _ALIAS.get(code, "en")


def _states_for(code: str) -> list[tuple[str, str]]:
    if code in _STATES:
        return list(_STATES[code])
    base = _base(code)
    if base in _STATES:
        return list(_STATES[base])
    return list(_STATES["en"])


def _kinds(code: str) -> tuple[str, str, str, str]:
    if code in _KIND:
        return _KIND[code]
    base = _base(code)
    return _KIND.get(base, _KIND["en"])


def _frames(code: str) -> dict:
    base = _base(code)
    return dict(_FRAMES.get(base, _FRAMES["en"]))


def apply_speech_frames(code: str, speech: dict, core: dict) -> dict:
    """Fill pack-owned post-execute frames. Never leaves DE/EN engine branches as the source."""
    frames = _frames(code)
    window, door, motion, smoke = _kinds(code)
    light = (speech.get("light_noun")
             or ((core.get("w") or {}).get("light") or [None])[0]
             or frames["light_noun"])
    out = {
        "entity_state": frames["entity_state"],
        "climate_at": frames["climate_at"],
        "temp_reading": frames["temp_reading"],
        "light_noun": light,
        "lights_on": frames["lights_on"],
        "lights_off": frames["lights_off"],
        "unit_c": frames["unit_c"],
        "unit_f": frames["unit_f"],
        "reminder": frames["reminder"],
        "presence_yes": frames["presence_yes"],
        "presence_no": frames["presence_no"],
        "list_none_found": frames["list_none_found"],
        "list_none_state": frames["list_none_state"],
        "list_state": frames["list_state"],
        "list_all": frames.get("list_all") or f"{{kind}}: {{names}}.",
        "kind_window": window,
        "kind_door": door,
        "kind_motion": motion,
        "kind_smoke": smoke,
        "state_open": frames["state_open"],
        "state_closed": frames["state_closed"],
        "media_volume_set": frames["media_volume_set"],
        "media_volume_up": frames["media_volume_up"],
        "media_volume_down": frames["media_volume_down"],
        "media_volume_query": frames["media_volume_query"],
        "media_playing": frames["media_playing"],
        "media_playing_by": frames["media_playing_by"],
        "queue_empty": frames["queue_empty"],
        "empty_place": _EMPTY.get(code) or _EMPTY.get(code.split("-")[0]) or frames.get("empty_place") or "—",
        "states": _states_for(code),
    }
    # Prefer climate_set with {unit} when the locale already has a natural set line.
    climate = speech.get("climate_set") or ""
    if "{n}" in climate and "{unit}" not in climate:
        for old in (" degrees", " Degrees", " Grad", " degres", " grados", " gradi", " grader", "度", "도"):
            if old in climate:
                speech["climate_set"] = climate.replace(old, " {unit}").replace("  {unit}", " {unit}")
                break
        else:
            if climate.endswith("."):
                speech["climate_set"] = climate[:-1] + " {unit}."
            else:
                speech["climate_set"] = climate + " {unit}"
    for key, value in out.items():
        if key == "states":
            speech[key] = value
            continue
        if not speech.get(key):
            speech[key] = value
    if is_script_pack(core):
        # Script packs must not keep English frame leftovers for these keys.
        for key in ("entity_state", "climate_at", "reminder", "queue_empty",
                    "media_playing", "media_playing_by", "media_volume_set", "media_volume_query",
                    "list_none_found", "list_none_state", "list_state", "lights_on", "lights_off"):
            val = speech.get(key) or ""
            if _ascii_heavy(val) and key in out:
                speech[key] = out[key]
    return speech


def _ascii_heavy(text: str) -> bool:
    letters = [ch for ch in text if ch.isalpha()]
    if not letters:
        return False
    return all(ord(ch) < 128 for ch in letters) and any(word in text.lower() for word in (
        " is ", "the ", "volume", "playing", "queue", "reminder", "lights", "degrees", "open", "closed",
    ))
