"""Mood word classes for generated packs: what closes a sentence as a question.

de and en carry the full classes by hand (negation, tense, time, condition). Every other
pack closes on a question mark, which the engine reads in any script, and on the question
particle of its language listed here. `requests` are the openings of a polite request,
which stays a command although it ends in a question mark.

Entries are written the way people write them; the emitter folds them like speech.
A phrase is words separated by one space, split where the tokenizer splits.
"""

from __future__ import annotations

from lang_packs.emit_rust import RUST_BANNER, rust_slice

MOOD: dict[str, dict[str, list[str]]] = {
    "fr": {
        "openers": ["est ce"],
        "requests": [
            "peux tu",
            "pourrais tu",
            "pouvez vous",
            "pourriez vous",
            "tu peux",
            "tu pourrais",
            "vous pouvez",
            "veux tu",
            "voulez vous",
            "est ce que tu peux",
            "est ce que tu pourrais",
            "est ce que vous pouvez",
        ],
    },
    "nl": {"requests": ["kun je", "kan je", "kun jij", "kunt u", "wil je", "zou je", "zou u", "kunnen we"]},
    "es": {"requests": ["puedes", "podrías", "puede", "podría", "me puedes", "me podrías"]},
    "it": {"requests": ["puoi", "potresti", "può", "potrebbe", "potete", "mi puoi", "mi potresti"]},
    "pt": {"requests": ["podes", "pode", "poderia", "poderias", "podia", "podias", "consegues", "você pode", "você poderia"]},
    "ca": {"requests": ["pots", "podries", "pot", "podria"]},
    "gl": {"requests": ["podes", "pode", "poderías", "podería"]},
    "ro": {"openers": ["oare"], "requests": ["poți", "ai putea", "puteți"]},
    "da": {"requests": ["kan du", "kunne du", "vil du", "gider du"]},
    "nb": {"requests": ["kan du", "kunne du", "vil du"]},
    "sv": {"requests": ["kan du", "kan ni", "kunde du", "skulle du kunna"]},
    "is": {"requests": ["geturðu", "getur þú", "gætirðu", "viltu"]},
    "af": {"requests": ["kan jy", "sal jy", "kan u"]},
    "lb": {"requests": ["kanns du", "kënns du", "géifs du"]},
    "fi": {"openers": ["onko", "ovatko", "oliko", "eikö"], "requests": ["voitko", "voisitko", "voitteko", "viitsitkö"]},
    "et": {"openers": ["kas"], "requests": ["kas sa saad", "kas sa saaksid", "kas saaksid", "saaksid"]},
    "hu": {"openers": ["vajon"], "requests": ["tudnád", "tudnál"]},
    "cs": {"requests": ["můžeš", "můžete", "mohl bys", "mohla bys", "mohli byste"]},
    "sk": {"openers": ["či"], "requests": ["môžeš", "môžete", "mohol by si", "mohla by si"]},
    "pl": {
        "openers": ["czy"],
        "requests": ["czy możesz", "możesz", "czy mógłbyś", "mógłbyś", "czy mogłabyś", "mogłabyś", "czy możecie"],
    },
    "hr": {"openers": ["je li", "da li", "jel"], "requests": ["možeš li", "možeš", "možete li", "bi li mogao", "bi li mogla"]},
    "sl": {"openers": ["ali"], "requests": ["ali lahko", "a lahko", "lahko"]},
    "sr": {"openers": ["да ли", "је ли"], "requests": ["можеш ли", "можете ли", "да ли можеш", "да ли можете"]},
    "sr-Latn": {
        "openers": ["da li", "je li", "jel"],
        "requests": ["možeš li", "možete li", "da li možeš", "da li možete"],
    },
    "bg": {"particles": ["ли"], "requests": ["можеш ли", "може ли", "бихте ли", "би ли"]},
    "uk": {"openers": ["чи"], "requests": ["чи можеш", "чи можете", "можеш", "можете"]},
    "lt": {"openers": ["ar"], "requests": ["ar gali", "ar galite", "ar galėtum", "gal gali"]},
    "lv": {"openers": ["vai"], "requests": ["vai vari", "vai tu vari", "vai varētu", "vai tu varētu", "vai jūs varētu"]},
    "el": {"openers": ["μήπως"], "endings": [";"], "requests": ["μπορείς να", "μπορείτε να", "θα μπορούσες να"]},
    "tr": {"particles": ["mi", "mı", "mu", "mü", "midir", "mıdır", "mudur", "müdür"]},
    "cy": {
        "openers": ["ydy", "ydyr", "ydi", "ydir", "yw", "ywr", "oes"],
        "requests": ["allwch chi", "elli di", "wnei di", "wnewch chi"],
    },
    "ga": {
        "openers": ["an bhfuil", "an raibh", "nach bhfuil", "nach raibh"],
        "requests": ["an féidir leat", "an bhféadfá", "ar mhiste leat"],
    },
    "eu": {"particles": ["al"]},
    "id": {
        "openers": ["apakah", "apa"],
        "requests": [
            "bisakah",
            "bisakah kamu",
            "bisakah anda",
            "dapatkah",
            "dapatkah kamu",
            "dapatkah anda",
            "bisa tolong",
            "boleh tolong",
            "bisa",
        ],
    },
    "ms": {
        "openers": ["adakah", "apakah"],
        "requests": ["bolehkah", "bolehkah awak", "bolehkah anda", "boleh tolong", "boleh awak", "boleh"],
    },
    "sw": {"openers": ["je"], "requests": ["je unaweza", "unaweza"]},
    "vi": {"endings": ["không", "chưa", "hả", "nhỉ"]},
    "ar": {"openers": ["هل"], "requests": ["هل يمكنك", "هل تستطيع", "هل يمكن", "ممكن"]},
    "he": {
        "openers": ["האם"],
        "requests": ["אתה יכול", "את יכולה", "אפשר", "תוכל", "תוכלי", "האם אתה יכול", "האם את יכולה", "האם אפשר"],
    },
    "fa": {
        "openers": ["آیا"],
        "requests": ["می‌توانی", "میتوانی", "می‌تونی", "میتونی", "می‌شود", "می‌شه", "میشه", "آیا می‌توانی", "آیا می‌شود"],
    },
    "ur": {"openers": ["کیا"]},
    "hi": {"openers": ["क्या"]},
    "bn": {"particles": ["কি"]},
    "mr": {"particles": ["का"]},
    "gu": {"openers": ["શું"]},
    "pa": {"openers": ["ਕੀ"]},
    "ne": {"openers": ["के"]},
    "hy": {"openers": ["արդյոք"]},
    "ka": {"openers": ["ნუთუ"]},
    "mn": {"endings": ["уу", "үү", "юу", "юү", "бэ", "вэ"]},
    "ja": {"endings": ["か", "かな", "かしら", "かい"]},
    "ko": {"endings": ["까", "까요", "나요"]},
    "th": {
        "endings": [
            "ไหม",
            "มั้ย",
            "มั๊ย",
            "หรือเปล่า",
            "รึเปล่า",
            "หรือยัง",
            "ไหมครับ",
            "ไหมคะ",
            "ไหมค่ะ",
            "มั้ยครับ",
            "มั้ยคะ",
            "หรือเปล่าครับ",
            "หรือเปล่าคะ",
        ]
    },
    "zh-CN": {"endings": ["吗", "嗎", "呢", "么"]},
    "zh-TW": {"endings": ["嗎", "吗", "呢"]},
    "zh-HK": {"endings": ["嗎", "吗", "呢", "咩", "未"]},
}


def parent(code: str) -> str:
    return code.split("-")[0]


def regional_mood_rs(lang: dict) -> str:
    """A regional pack is bound together with its parent pack, which brings the word classes.
    Its own file holds the regional words only ("ned" for "nicht")."""
    rows = sorted((name, values) for name, values in (lang.get("mood") or {}).items() if values)
    if not rows:
        return f"""{RUST_BANNER}use crate::lang::mood::MoodWords;

/// The parent pack is bound together with this one and brings its word classes.
pub(super) const MOOD: MoodWords = MoodWords::EMPTY;
"""
    body = "".join(f"    {name}: {rust_slice(values, '    ')},\n" for name, values in rows)
    return f"""{RUST_BANNER}use crate::lang::mood::MoodWords;

/// Regional words only. The parent pack is bound together with this one.
pub(super) const MOOD: MoodWords = MoodWords {{
{body}    ..MoodWords::EMPTY
}};
"""


def mood_rs(lang: dict) -> str:
    code = lang["code"]
    if lang.get("parent"):
        return regional_mood_rs(lang)
    row = MOOD.get(code) or MOOD.get(parent(code)) or {}
    fields = [
        ("request_frames", row.get("requests", [])),
        ("question_openers", row.get("openers", [])),
        ("question_particles", row.get("particles", [])),
        ("question_endings", row.get("endings", [])),
    ]
    body = "".join(f"    {name}: {rust_slice(values, '    ')},\n" for name, values in fields if values)
    if not body:
        return f"""{RUST_BANNER}use crate::lang::mood::MoodWords;

/// No question particle listed: this pack closes on a question mark.
pub(super) const MOOD: MoodWords = MoodWords::EMPTY;
"""
    return f"""{RUST_BANNER}use crate::lang::mood::MoodWords;

pub(super) const MOOD: MoodWords = MoodWords {{
{body}    ..MoodWords::EMPTY
}};
"""
