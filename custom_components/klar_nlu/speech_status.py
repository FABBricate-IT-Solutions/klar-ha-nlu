"""Localized floor-status clauses: area, then named devices, presence, sensors."""

from __future__ import annotations

from pathlib import Path
from typing import Any

try:
    from .speech_status_device import _DEVICE, _DEVICE_KEYS
except ImportError:
    from speech_status_device import _DEVICE, _DEVICE_KEYS


def _infra_needles() -> tuple[str, ...]:
    path = Path(__file__).with_name("infra_needles.txt")
    return tuple(
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    )


_INFRA = _infra_needles()


def _infra_state(state: Any) -> bool:
    entity_id = str(getattr(state, "entity_id", "") or "").lower()
    name = str(getattr(state, "name", "") or "").lower()
    attrs = getattr(state, "attributes", None) or {}
    if isinstance(attrs, dict):
        name = str(attrs.get("friendly_name") or name).lower()
        tags = attrs.get("tags") or []
        if isinstance(tags, list) and any(str(tag).lower() == "infra" for tag in tags):
            return True
    blob = f"{entity_id} {name}"
    return any(needle in blob for needle in _INFRA)

# on, off, light, lights, socket, sockets, present, absent, temp, lux
_WORDS: dict[str, tuple[str, str, str, str, str, str, str, str, str, str]] = {
    "de": ("an", "aus", "Licht", "Lichter", "Steckdose", "Steckdosen", "jemand da", "niemand da", "{n} Grad", "{n} Lux"),
    "en": ("on", "off", "light", "lights", "socket", "sockets", "occupied", "empty", "{n} degrees", "{n} lux"),
    "fr": ("allumée", "éteinte", "lumière", "lumières", "prise", "prises", "occupé", "vide", "{n} degrés", "{n} lux"),
    "nl": ("aan", "uit", "licht", "lichten", "stopcontact", "stopcontacten", "bezet", "leeg", "{n} graden", "{n} lux"),
    "es": ("encendida", "apagada", "luz", "luces", "enchufe", "enchufes", "ocupado", "vacío", "{n} grados", "{n} lux"),
    "it": ("accesa", "spenta", "luce", "luci", "presa", "prese", "occupato", "vuoto", "{n} gradi", "{n} lux"),
    "pt": ("acesa", "apagada", "luz", "luzes", "tomada", "tomadas", "ocupado", "vazio", "{n} graus", "{n} lux"),
    "ca": ("encesa", "apagada", "llum", "llums", "endoll", "endolls", "ocupat", "buit", "{n} graus", "{n} lux"),
    "ro": ("aprinsă", "stinsă", "lumină", "lumini", "priză", "prize", "ocupat", "gol", "{n} grade", "{n} lux"),
    "da": ("tændt", "slukket", "lys", "lys", "stikkontakt", "stikkontakter", "optaget", "tomt", "{n} grader", "{n} lux"),
    "nb": ("på", "av", "lys", "lys", "stikkontakt", "stikkontakter", "opptatt", "tomt", "{n} grader", "{n} lux"),
    "sv": ("på", "av", "ljus", "ljus", "uttag", "uttag", "upptaget", "tomt", "{n} grader", "{n} lux"),
    "fi": ("päällä", "pois", "valo", "valot", "pistorasia", "pistorasiat", "varattu", "tyhjä", "{n} astetta", "{n} luksia"),
    "af": ("aan", "af", "lig", "ligte", "prop", "proppe", "beset", "leeg", "{n} grade", "{n} lux"),
    "cs": ("zapnuto", "vypnuto", "světlo", "světla", "zásuvka", "zásuvky", "obsazeno", "prázdno", "{n} stupňů", "{n} lux"),
    "sk": ("zapnuté", "vypnuté", "svetlo", "svetlá", "zásuvka", "zásuvky", "obsadené", "prázdne", "{n} stupňov", "{n} lux"),
    "pl": ("włączone", "wyłączone", "światło", "światła", "gniazdko", "gniazdka", "zajęte", "pusto", "{n} stopni", "{n} luksów"),
    "hu": ("be", "ki", "lámpa", "lámpák", "konnektor", "konnektorok", "foglalt", "üres", "{n} fok", "{n} lux"),
    "hr": ("uključeno", "isključeno", "svjetlo", "svjetla", "utikač", "utikači", "zauzeto", "prazno", "{n} stupnjeva", "{n} lux"),
    "sl": ("prižgano", "ugasnjeno", "luč", "luči", "vtičnica", "vtičnice", "zasedeno", "prazno", "{n} stopinj", "{n} lux"),
    "bg": ("включена", "изключена", "светлина", "светлини", "контакт", "контакти", "заето", "празно", "{n} градуса", "{n} лукса"),
    "el": ("αναμμένο", "σβηστό", "φως", "φώτα", "πρίζα", "πρίζες", "κατειλημμένο", "άδειο", "{n} βαθμοί", "{n} lux"),
    "sr": ("укључено", "искључено", "светло", "светла", "утичница", "утичнице", "заузето", "празно", "{n} степени", "{n} лукса"),
    "uk": ("увімкнено", "вимкнено", "світло", "світла", "розетка", "розетки", "зайнято", "порожньо", "{n} градусів", "{n} люкс"),
    "zh-CN": ("开", "关", "灯", "灯", "插座", "插座", "有人", "没人", "{n}度", "{n}勒克斯"),
    "ar": ("تشغيل", "إيقاف", "ضوء", "أضواء", "مقبس", "مقابس", "مشغول", "فارغ", "{n} درجة", "{n} لكس"),
    "he": ("דלוק", "כבוי", "אור", "אורות", "שקע", "שקעים", "תפוס", "ריק", "{n} מעלות", "{n} לוקס"),
    "fa": ("روشن", "خاموش", "چراغ", "چراغ‌ها", "پریز", "پریزها", "اشغال", "خالی", "{n} درجه", "{n} لوکس"),
    "ur": ("آن", "آف", "روشنی", "روشنیاں", "ساکٹ", "ساکٹس", "موجود", "خالی", "{n} ڈگری", "{n} لکس"),
    "tr": ("açık", "kapalı", "ışık", "ışıklar", "priz", "prizler", "dolu", "boş", "{n} derece", "{n} lüks"),
    "th": ("เปิด", "ปิด", "ไฟ", "ไฟ", "ปลั๊ก", "ปลั๊ก", "มีคน", "ว่าง", "{n} องศา", "{n} ลักซ์"),
    "ko": ("켜짐", "꺼짐", "조명", "조명", "콘센트", "콘센트", "있음", "없음", "{n}도", "{n}럭스"),
    "ja": ("点灯", "消灯", "照明", "照明", "コンセント", "コンセント", "在室", "不在", "{n}度", "{n}ルクス"),
    "cy": ("ymlaen", "i ffwrdd", "golau", "goleuadau", "soced", "socedi", "meddiannwyd", "gwag", "{n} gradd", "{n} lwcs"),
    "et": ("sees", "väljas", "valgus", "tuled", "pistik", "pistikud", "hõivatud", "tühi", "{n} kraadi", "{n} luks"),
    "eu": ("piztuta", "itzalita", "argia", "argiak", "entxufea", "entxufeak", "okupatuta", "hutsik", "{n} gradu", "{n} lux"),
    "ga": ("ann", "as", "solas", "soilse", "soicéad", "soicéid", "áitithe", "folamh", "{n} céim", "{n} lux"),
    "gl": ("acesa", "apagada", "luz", "luces", "enchufe", "enchufes", "ocupado", "baleiro", "{n} graos", "{n} lux"),
    "is": ("kveikt", "slökkt", "ljós", "ljós", "innstunga", "innstungur", "upptekið", "autt", "{n} gráður", "{n} lux"),
    "lb": ("un", "aus", "Luucht", "Luuchten", "Steckdos", "Steckdosen", "een do", "keen do", "{n} Grad", "{n} Lux"),
    "kw": ("yn", "mes", "golow", "golowow", "soket", "sokettys", "prenys", "gwag", "{n} gradh", "{n} lux"),
    "lt": ("įjungta", "išjungta", "šviesa", "šviesos", "lizdas", "lizdai", "užimta", "tuščia", "{n} laipsnių", "{n} liuksų"),
    "lv": ("ieslēgts", "izslēgts", "gaisma", "gaismas", "rozetes", "rozetes", "aizņemts", "tukšs", "{n} grādi", "{n} luksi"),
    "id": ("nyala", "mati", "lampu", "lampu", "stopkontak", "stopkontak", "ada orang", "kosong", "{n} derajat", "{n} lux"),
    "ms": ("hidup", "mati", "lampu", "lampu", "soket", "soket", "ada orang", "kosong", "{n} darjah", "{n} lux"),
    "sw": ("washa", "zima", "taa", "taa", "soketi", "soketi", "kuna mtu", "tupu", "{n} digrii", "{n} lux"),
    "vi": ("bật", "tắt", "đèn", "đèn", "ổ cắm", "ổ cắm", "có người", "trống", "{n} độ", "{n} lux"),
    "hi": ("चालू", "बंद", "बत्ती", "बत्तियाँ", "सॉकेट", "सॉकेट", "कोई है", "खाली", "{n} डिग्री", "{n} लक्स"),
    "bn": ("চালু", "বন্ধ", "আলো", "আলো", "সকেট", "সকেট", "কেউ আছে", "খালি", "{n} ডিগ্রি", "{n} লাক্স"),
    "gu": ("ચાલુ", "બંધ", "પ્રકાશ", "પ્રકાશ", "સોકેટ", "સોકેટ", "કોઈ છે", "ખાલી", "{n} ડિગ્રી", "{n} લક્સ"),
    "kn": ("ಆನ್", "ಆಫ್", "ಬೆಳಕು", "ದೀಪಗಳು", "ಸಾಕೆಟ್", "ಸಾಕೆಟ್‌ಗಳು", "ಯಾರೋ ಇದ್ದಾರೆ", "ಖಾಲಿ", "{n} ಡಿಗ್ರಿ", "{n} ಲಕ್ಸ್"),
    "ml": ("ഓൺ", "ഓഫ്", "വിളക്ക്", "വിളക്കുകള്‍", "സോക്കറ്റ്", "സോക്കറ്റുകള്‍", "ആരെങ്കിലും ഉണ്ട്", "ശൂന്യം", "{n} ഡിഗ്രി", "{n} ലക്സ്"),
    "mr": ("चालू", "बंद", "दिवा", "दिवे", "सॉकेट", "सॉकेट", "कोणीतरी आहे", "रिकामे", "{n} अंश", "{n} लक्स"),
    "ta": ("ஆன்", "ஆஃப்", "விளக்கு", "விளக்குகள்", "சாக்கெட்", "சாக்கெட்", "யாரோ உள்ளனர்", "காலி", "{n} டிகிரி", "{n} லக்ஸ்"),
    "te": ("ఆన్", "ఆఫ్", "వెలుగు", "లైట్లు", "సాకెట్", "సాకెట్", "ఎవరో ఉన్నారు", "ఖాళీ", "{n} డిగ్రీలు", "{n} లక్స్"),
    "pa": ("ਚਾਲੂ", "ਬੰਦ", "ਰੋਸ਼ਨੀ", "ਰੋਸ਼ਨੀਆਂ", "ਸਾਕਟ", "ਸਾਕਟ", "ਕੋਈ ਹੈ", "ਖਾਲੀ", "{n} ਡਿਗਰੀ", "{n} ਲਕਸ"),
    "ne": ("अन", "अफ", "बत्ती", "बत्तीहरू", "सकेट", "सकेट", "कोही छ", "खाली", "{n} डिग्री", "{n} लक्स"),
    "hy": ("միացված", "անջատված", "լույս", "լույսեր", "վարդակ", "վարդակներ", "զբաղված", "դատարկ", "{n} աստիճան", "{n} լյուքս"),
    "ka": ("ჩართული", "გამორთული", "სინათლე", "სინათლეები", "როზეტი", "როზეტები", "დაკავებული", "ცარიელი", "{n} გრადუსი", "{n} ლუქსი"),
    "mn": ("асаалттай", "унтраалттай", "гэрэл", "гэрлүүд", "розетка", "розетка", "хүн байна", "хоосон", "{n} градус", "{n} люкс"),
}

_ALIAS = {
    "de-CH": "de",
    "de-AT": "de",
    "en-GB": "en",
    "pt-BR": "pt",
    "zh-TW": "zh-CN",
    "zh-HK": "zh-CN",
    "sr-Latn": "hr",
}
_COMMA = {
    "de", "fr", "nl", "es", "it", "pt", "ca", "ro", "da", "nb", "sv", "fi",
    "cs", "sk", "pl", "hu", "hr", "sl", "tr", "id", "vi", "de-CH", "de-AT", "pt-BR",
}
_IDEO = {"zh-CN", "zh-TW", "zh-HK", "ja"}
_PRESENCE = {"occupancy", "motion", "presence"}
_ON = {"on", "home", "detected", "open", "unlocked", "playing", "cleaning"}
_OFF = {"off", "not_home", "clear", "closed", "locked", "idle", "docked", "paused"}
_EMPTY = {
    "de": "Keine Geräte.",
    "en": "No devices.",
    "fr": "Aucun appareil.",
    "nl": "Geen apparaten.",
    "es": "Ningún aparato.",
    "it": "Nessun dispositivo.",
    "pt": "Nenhum aparelho.",
    "ca": "Cap aparell.",
    "ro": "Niciun aparat.",
    "da": "Ingen enheder.",
    "nb": "Ingen enheter.",
    "sv": "Inga enheter.",
    "fi": "Ei laitteita.",
    "af": "Geen toestelle.",
    "cs": "Žádná zařízení.",
    "sk": "Žiadne zariadenia.",
    "pl": "Brak urządzeń.",
    "hu": "Nincs eszköz.",
    "hr": "Nema uređaja.",
    "sl": "Ni naprav.",
    "bg": "Няма устройства.",
    "el": "Κανένα συσκευή.",
    "sr": "Нема уређаја.",
    "uk": "Немає пристроїв.",
    "zh-CN": "没有设备。",
    "ar": "لا أجهزة.",
    "he": "אין מכשירים.",
    "fa": "دستگاهی نیست.",
    "ur": "کوئی آلہ نہیں.",
    "tr": "Cihaz yok.",
    "th": "ไม่มีอุปกรณ์",
    "ko": "기기 없음.",
    "ja": "機器はありません。",
    "cy": "Dim dyfeisiau.",
    "et": "Seadmeid pole.",
    "eu": "Ez dago gailurik.",
    "ga": "Níl aon ghléas.",
    "gl": "Ningún aparello.",
    "is": "Engin tæki.",
    "lb": "Keng Geräter.",
    "kw": "Ny vyjy.",
    "lt": "Nėra įrenginių.",
    "lv": "Nav ierīču.",
    "id": "Tidak ada perangkat.",
    "ms": "Tiada peranti.",
    "sw": "Hakuna vifaa.",
    "vi": "Không có thiết bị.",
    "hi": "कोई उपकरण नहीं.",
    "bn": "কোনো যন্ত্র নেই.",
    "gu": "કોઈ ઉપકરણ નથી.",
    "kn": "ಯಾವುದೇ ಸಾಧನವಿಲ್ಲ.",
    "ml": "ഉപകരണങ്ങളില്ല.",
    "mr": "साधने नाहीत.",
    "ta": "சாதனங்கள் இல்லை.",
    "te": "పరికరాలు లేవు.",
    "pa": "ਕੋਈ ਯੰਤਰ ਨਹੀਂ.",
    "ne": "कुनै उपकरण छैन.",
    "hy": "Սարքեր չկան.",
    "ka": "მოწყობილობა არ არის.",
    "mn": "Төхөөрөмж байхгүй.",
}


def empty_status_speech(pack: str) -> str:
    return _EMPTY.get(_base(pack)) or _EMPTY["en"]


def rooms_status_speech(rooms: list[tuple[str, list[Any]]], pack: str) -> str:
    stop = "。" if _base(pack) in _IDEO else " "
    parts = [area_status_speech(name, states, pack) for name, states in rooms]
    return stop.join(part for part in parts if part)


def area_status_speech(name: str, states: list[Any], pack: str) -> str:
    words = _words(pack)
    visible = [state for state in states if not _infra_state(state) and _usable(state)]
    facts = _facts(visible, pack, words)
    if not facts:
        return ""
    pretty = _title(name)
    if _base(pack) in _IDEO:
        return f"{pretty}。{'，'.join(facts)}"
    return f"{pretty}. {'. '.join(facts)}."


def _facts(states: list[Any], pack: str, words: dict[str, str]) -> list[str]:
    lights = [state for state in states if _domain(state) == "light"]
    sockets = [state for state in states if _is_socket(state)]
    presence = [state for state in states if _class_of(state) in _PRESENCE]
    temps = [state for state in states if _is_temp(state)]
    luxes = [state for state in states if _class_of(state) == "illuminance"]
    silent = set(map(_eid, lights + sockets + presence + luxes))
    for state in temps:
        if _domain(state) != "climate":
            silent.add(_eid(state))
    others = [state for state in states if _eid(state) not in silent]
    facts: list[str] = []
    for state in lights + sockets:
        spoken = _other(state, pack, words)
        if spoken:
            facts.append(spoken)
    if presence:
        facts.append(words["present"] if any(_is_on(state) for state in presence) else words["absent"])
    sensors = [state for state in temps if _class_of(state) == "temperature"]
    temp = _first_number(sensors, None)
    if temp == "":
        temp = _first_number(temps, "current_temperature")
    if temp != "":
        facts.append(words["temp"].replace("{n}", _num(temp, pack)))
    lux = _first_number(luxes, None)
    if lux != "":
        facts.append(words["lux"].replace("{n}", _num(lux, pack, digits=0)))
    for state in others:
        spoken = _other(state, pack, words)
        if spoken:
            facts.append(spoken)
    return facts


def _other(state: Any, pack: str, words: dict[str, str]) -> str:
    attrs = getattr(state, "attributes", None) or {}
    name = _title(str(attrs.get("friendly_name") or getattr(state, "name", "") or _eid(state)))
    if not name:
        return ""
    raw = str(getattr(state, "state", "") or "")
    if _numeric(raw):
        spoken = _num(raw, pack)
    else:
        spoken = _device_state(raw, pack, words)
    return f"{name} {spoken}".strip()


def _device_state(raw: str, pack: str, words: dict[str, str]) -> str:
    key = raw.lower().replace("-", "_")
    aliases = {
        "return_to_base": "returning",
        "returning_to_base": "returning",
        "heating": "heat",
        "cooling": "cool",
        "fanonly": "fan_only",
        "fan": "fan_only",
        "drying": "dry",
        "automatic": "auto",
    }
    key = aliases.get(key, key)
    if key in {"on", "off", "open", "closed", "locked", "unlocked"}:
        return words["on"] if key in {"on", "open", "unlocked"} else words["off"]
    row = _DEVICE.get(_base(pack)) or _DEVICE["en"]
    spoken = dict(zip(_DEVICE_KEYS, row, strict=True)).get(key)
    return spoken or raw.replace("_", " ")


def _words(pack: str) -> dict[str, str]:
    row = _WORDS.get(_base(pack)) or _WORDS["en"]
    names = ("on", "off", "light", "lights", "socket", "sockets", "present", "absent", "temp", "lux")
    return dict(zip(names, row, strict=True))


def _base(pack: str) -> str:
    if pack in _WORDS:
        return pack
    if pack in _ALIAS:
        return _ALIAS[pack]
    root = pack.split("-", 1)[0]
    return root if root in _WORDS else "en"


def _title(raw: str) -> str:
    text = " ".join(str(raw).replace("_", " ").split())
    return text[:1].upper() + text[1:] if text else ""


def _num(value: Any, pack: str, digits: int | None = 1) -> str:
    try:
        number = float(value)
    except (TypeError, ValueError):
        return str(value)
    text = str(int(round(number))) if digits == 0 else f"{number:.{digits}f}".rstrip("0").rstrip(".")
    if _base(pack) in _COMMA:
        text = text.replace(".", ",")
    return text


def _first_number(states: list[Any], attr: str | None) -> Any:
    for state in states:
        attrs = getattr(state, "attributes", None) or {}
        raw = attrs.get(attr) if attr else None
        if raw in (None, ""):
            raw = getattr(state, "state", None)
        if _numeric(raw):
            return raw
    return ""


def _numeric(value: Any) -> bool:
    try:
        float(value)
    except (TypeError, ValueError):
        return False
    return True


def _is_socket(state: Any) -> bool:
    return _class_of(state) == "outlet"


def _is_temp(state: Any) -> bool:
    if _class_of(state) == "temperature":
        return True
    if _domain(state) != "climate":
        return False
    attrs = getattr(state, "attributes", None) or {}
    return attrs.get("current_temperature") not in (None, "")


def _class_of(state: Any) -> str:
    attrs = getattr(state, "attributes", None) or {}
    return str(attrs.get("device_class") or "").lower()


def _domain(state: Any) -> str:
    return _eid(state).split(".", 1)[0]


def _eid(state: Any) -> str:
    return str(getattr(state, "entity_id", "") or "")


def _usable(state: Any) -> bool:
    return str(getattr(state, "state", "") or "").lower() not in {"unavailable", "unknown"}


def _is_on(state: Any) -> bool:
    return str(getattr(state, "state", "") or "").lower() in _ON


def _is_off(state: Any) -> bool:
    return str(getattr(state, "state", "") or "").lower() in _OFF
