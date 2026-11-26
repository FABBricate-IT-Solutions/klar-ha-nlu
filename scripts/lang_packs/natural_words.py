"""Everyday words the first lexicon pass missed, found by the natural gold sentences.

`tests/datasets/gold/{code}.yaml` holds hand-written sentences. Where one of them made a
pack do something dangerous (play a search for "pause the music", turn a light off for
"close the blinds", set a light to 2 % for "bujha do"), the missing everyday word is added
here instead of loosening the gold. Words are merged into the pack's own lists; they are
folded and de-duplicated like every other lexicon word.
"""

from __future__ import annotations

NATURAL_WORDS: dict[str, dict[str, list[str]]] = {
    # "Pauzeer de muziek" must pause, never start a search for "pauzeer muziek".
    "nl": {"pause": ["pauzeer"]},
    "fi": {"pause": ["keskeytä"], "add": ["lisää"], "lock_v": ["lukitse"], "on": ["päälle"]},
    "af": {"pause": ["onderbreek", "pouseer"], "add": ["voeg"], "lock_v": ["sluit"], "on": ["sit"]},
    "el": {"pause": ["παύση", "παύσε"]},
    "fa": {"pause": ["متوقف"], "add": ["اضافه"], "lock_v": ["قفل"]},
    "ur": {"pause": ["روکو"], "lock_v": ["لاک"]},
    "et": {"pause": ["pausile", "peata"], "add": ["lisa"], "lock_v": ["lukusta"], "on": ["süüta"], "light": ["lamp", "valgus"], "on_particles": ["sisse"], "off_particles": ["välja"], "switch_verbs": ["lülita"]},
    "ga": {"pause": ["sos"], "on": ["las", "siúl"]},
    "kw": {"pause": ["powes"]},
    "lt": {"pause": ["sustabdyk", "pristabdyk"], "add": ["pridėk"], "lock_v": ["užrakink"]},
    # lb: "Maach d'Rolllueden zou" must close the blinds, not open them; "Pauséier d'Musek" must pause.
    "lb": {"pause": ["paus", "pauséier"], "cover": ["rolllueden"], "close": ["zou"], "add": ["setz"], "lock_v": ["spär"], "on": ["schalt"]},
    # Definite plurals: "jaluzelele", "תריסים" are the blinds, never the bedroom light.
    "ro": {"cover": ["jaluzele", "jaluzelele", "perdele", "perdelele"], "lock_v": ["încuie"]},
    "he": {"cover": ["תריס", "תריסים", "וילונות"], "pause": ["השהה"], "add": ["הוסף"], "lock_v": ["נעל"], "on": ["תדליק"], "off": ["תכבה"]},
    # "रसोई की बत्ती बुझा दो": "दो" is the helper verb here, not the number two.
    "hi": {"off": ["बुझा"], "lock_v": ["लॉक"]},
    # Everyday "add to the list", "lock the door" and "pause the music" verbs, by locale.
    "da": {"add": ["tilføj"], "lock_v": ["lås"]},
    "nb": {"add": ["legg"], "lock_v": ["lås"]},
    "sv": {"pause": ["pausa"], "add": ["lägg"]},
    "sk": {"pause": ["pozastav"], "add": ["pridaj"], "lock_v": ["zamkni"], "on": ["rozsviet"]},
    "cs": {"pause": ["pozastav"], "on": ["rozsviť"], "off": ["zhasni"]},
    "pl": {"pause": ["wstrzymaj"], "add": ["dodaj"], "on": ["zapal"], "off": ["zgaś"]},
    "hu": {"pause": ["szüneteltesd"]},
    "hr": {"pause": ["pauziraj"], "add": ["dodaj"], "lock_v": ["zaključaj"], "on": ["uključi"], "off": ["isključi"]},
    "sl": {"pause": ["ustavi"], "lock_v": ["zakleni"], "on": ["vklopi"]},
    "bg": {"pause": ["паузирай"], "add": ["добави"], "lock_v": ["заключи"], "on": ["светни"]},
    "sr": {"pause": ["паузирај"], "add": ["додај"], "lock_v": ["закључај"], "on": ["упали"], "off": ["угаси"]},
    "sr-Latn": {"pause": ["pauziraj"], "add": ["dodaj"], "lock_v": ["zaključaj"], "on": ["upali"], "off": ["ugasi"]},
    "uk": {"pause": ["паузу"], "add": ["додай"], "lock_v": ["замкни"]},
    "ar": {"pause": ["أوقف"], "lock_v": ["اقفل"]},
    "tr": {"pause": ["duraklat"], "add": ["ekle"], "lock_v": ["kilitle"]},
    "th": {"add": ["เพิ่ม"], "lock_v": ["ล็อก"]},
    "ko": {"lock_v": ["잠가"]},
    "cy": {"pause": ["oedi"], "lock_v": ["cloi"], "on": ["ymlaen"], "off": ["diffodd"]},
    "eu": {"pause": ["pausan"], "add": ["gehitu"], "lock_v": ["giltzapetu"]},
    "gl": {"pause": ["pausa"], "on": ["prende"]},
    "is": {"pause": ["pásu"], "add": ["bættu"], "lock_v": ["læstu"]},
    "lv": {"pause": ["pauzē"], "add": ["pievieno"], "lock_v": ["aizslēdz"], "on": ["iededz"]},
    "id": {"pause": ["jeda"], "on": ["hidupkan"]},
    "ms": {"pause": ["jeda"], "lock_v": ["kunci"], "on": ["pasang"]},
    "sw": {"pause": ["sitisha"]},
    "vi": {"pause": ["tạm"], "add": ["thêm"]},
    "bn": {"add": ["যোগ"], "on": ["চালু"], "open": ["খোলো"]},
    "gu": {"pause": ["થોભાવો"], "add": ["ઉમેરો"], "lock_v": ["લોક"]},
    "kn": {"pause": ["ವಿರಾಮಗೊಳಿಸಿ"], "add": ["ಸೇರಿಸಿ"], "lock_v": ["ಲಾಕ್"], "on": ["ಆನ್", "ಹಚ್ಚಿ"], "off": ["ಆಫ್"], "open": ["ತೆರೆಯಿರಿ"]},
    "ml": {"pause": ["നിർത്തൂ"], "add": ["ചേർക്കൂ"], "lock_v": ["പൂട്ടൂ"], "on": ["ഓൺ", "ഇടൂ"], "off": ["ഓഫ്"], "open": ["തുറക്കൂ"]},
    "mr": {"pause": ["थांबवा"], "add": ["जोडा"], "lock_v": ["लॉक"], "on": ["लावा"], "open": ["उघडा"]},
    "ta": {"pause": ["இடைநிறுத்து"], "add": ["சேர்"], "lock_v": ["பூட்டு"], "on": ["ஆன்", "எரியவிடு"]},
    "te": {"pause": ["పాజ్"], "add": ["చేర్చు"], "lock_v": ["లాక్"], "on": ["ఆన్"], "off": ["ఆఫ్"]},
    "pa": {"lock_v": ["ਲਾਕ"], "on": ["ਜਗਾਓ"], "open": ["ਖੋਲ੍ਹੋ"]},
    "ne": {"add": ["थप"], "on": ["अन"]},
    "hy": {"pause": ["դադարեցրու"], "add": ["ավելացրու"], "lock_v": ["կողպիր"], "on": ["վառիր"]},
    "ka": {"pause": ["დააპაუზე"], "add": ["დაამატე"], "lock_v": ["ჩაკეტე"], "open": ["გააღე"]},
    "mn": {"add": ["нэм"], "lock_v": ["цоожло"], "on": ["асаагаарай"]},
    "ca": {"on": ["engega"], "open": ["obre"]},
    "zh-CN": {"add": ["加"]},
    "zh-TW": {"add": ["加"]},
    "zh-HK": {"add": ["加"]},
}


def merge_natural_words(code: str, w: dict) -> None:
    for key, words in NATURAL_WORDS.get(code, {}).items():
        # `lock_v` replaces the lock nouns as lock verbs once present, so start from them.
        base = w.get("lock_v") or w.get("lock") if key == "lock_v" else w.get(key)
        merged = list(base or [])
        for word in words:
            if word not in merged:
                merged.append(word)
        w[key] = merged
