"""Conversation-suite tokens: too, percent, dim, warm white, quieter."""

from __future__ import annotations

# too, percent, dim-words, medium, warmwhite, quiet
_ROW: dict[str, tuple[str, str, str, str, str, str]] = {
    "fr": ("aussi", "pourcent", "sombre moins", "moyen", "blancchaud", "moinsfort"),
    "nl": ("ook", "procent", "donkerder dimmen", "medium", "warmwit", "zachter"),
    "es": ("tambien", "porciento", "masoscuro atenuar", "medio", "blancocalido", "masbajo"),
    "it": ("anche", "percento", "piuoscuro attenua", "medio", "biancocaldo", "piubasso"),
    "pt": ("tambem", "porcento", "maisescuro escurecer", "medio", "brancquente", "maisbaixo"),
    "ca": ("tambe", "percent", "mesfosc atenuar", "mitja", "blanccalent", "mesbaix"),
    "ro": ("deasemenea", "la suta", "maiintunecat atenueaza", "mediu", "albcald", "maincet"),
    "da": ("ogsaa", "procent", "morkere dæmp", "medium", "varmhvid", "lavere"),
    "nb": ("ogsaa", "prosent", "morkere demp", "medium", "varmhvit", "lavere"),
    "sv": ("ocksa", "procent", "morkare dimra", "medium", "varmvit", "lagre"),
    "fi": ("myos", "prosenttia", "tummempi himmenna", "keskitaso", "lamminvalkoinen", "hiljaisempi"),
    "de-CH": ("auch", "prozent", "dunkler dimme", "mittelhell", "warmweiss", "leiser"),
    "de-AT": ("auch", "prozent", "dunkler dimme", "mittelhell", "warmweiss", "leiser"),
    "en-GB": ("too", "percent", "dimmer darker", "medium", "warmwhite", "quieter"),
    "pt-BR": ("tambem", "porcento", "maisescuro escurecer", "medio", "brancquente", "maisbaixo"),
    "af": ("ook", "persent", "donkerder dim", "medium", "warmwit", "sagter"),
    "cs": ("taky", "procent", "tmavsi stmiv", "stredni", "teplabila", "tiseji"),
    "sk": ("tiez", "percent", "tmavsi stmievaj", "stredne", "teplabela", "tissie"),
    "pl": ("tez", "procent", "ciemniej sciemniaj", "srednio", "cieplabiala", "ciszej"),
    "hu": ("is", "szazalek", "sotetebb dimmel", "kozepes", "melegfeher", "halkabban"),
    "hr": ("takodjer", "posto", "tamnije prigusiti", "srednje", "toplabijela", "tise"),
    "sl": ("tudi", "odstotkov", "temneje zatemniti", "srednje", "toplabela", "tise"),
    "bg": ("също", "процент", "по-тъмно затъмни", "средно", "топлобяло", "по-тихо"),
    "el": ("επισης", "τοις εκατο", "πιο σκοτεινα", "μεσαίο", "ζεστολευκο", "πιοσιγα"),
    "sr": ("такође", "проценат", "тамније пригуши", "средње", "топлобело", "тише"),
    "sr-Latn": ("takodje", "procenat", "tamnije prigusi", "srednje", "toplobelo", "tise"),
    "uk": ("також", "відсоток", "темніше затемни", "середньо", "теплийбілий", "тихіше"),
    "zh-CN": ("也", "百分之", "暗一点 调暗", "中等", "暖白", "小点声"),
    "zh-TW": ("也", "百分之", "暗一点 調暗", "中等", "暖白", "小點聲"),
    "zh-HK": ("都", "百分之", "暗啲 調暗", "中等", "暖白", "細聲啲"),
    "ar": ("ايضا", "بالمئة", "اغمق اخفت", "متوسط", "ابيضدافئ", "اخفض"),
    "he": ("גם", "אחוז", "כהה יותר עמעם", "בינוני", "לבןחם", "שקטיותר"),
    "fa": ("هم", "درصد", "تیره‌تر کم‌نور", "متوسط", "سفیدگرم", "آرامتر"),
    "ur": ("بھی", "فیصد", "گہرا دھندلا", "درمیانہ", "گرم سفید", "آہستہ"),
    "tr": ("dahi", "yuzde", "daha karanlik kis", "orta", "sicakbeyaz", "kisik"),
    "th": ("duay", "เปอร์เซ็นต์", "มืดลง หรี่", "ปานกลาง", "ขาวนวล", "เบาลง"),
    "ko": ("도", "퍼센트", "더어둡게 어둡게", "중간", "따뜻한흰색", "더작게"),
    "ja": ("も", "パーセント", "暗く 絞る", "中くらい", "暖色", "小さく"),
    "cy": ("hefyd", "y cant", "tywyllach pylu", "canolig", "gwyncynnes", "tawelach"),
    "et": ("ka", "protsenti", "tumedam hämarda", "keskmine", "soe valge", "vaiksemalt"),
    "eu": ("ere", "ehuneko", "ilunago lausotu", "ertain", "zuri bero", "isilago"),
    "ga": ("freisin", "faoin gcead", "nios dorcha", "meánach", "ban te", "nios ciúine"),
    "gl": ("tamén", "por cento", "maisescuro atenuar", "medio", "brancocálido", "maisbaixo"),
    "is": ("lika", "prosent", "dekkra dimma", "midlungs", "heitthvitt", "hlaera"),
    "lb": ("genausou", "prozent", "dunkel dimm", "mëttel", "waarmwäiss", "méi lues"),
    "kw": ("ynwedh", "kans", "tewldir", "kres", "gwynn toemm", "kosel"),
    "lt": ("irgi", "procentu", "tamsiau pritemdyk", "vidutinis", "silta balta", "tyliau"),
    "lv": ("ari", "procenti", "tumšak aptumšo", "videji", "silti balts", "klusak"),
    "id": ("juga", "persen", "lebihgelap redupkan", "sedang", "putihhangat", "lebihpelan"),
    "ms": ("juga", "peratus", "lebihgelap redup", "sederhana", "putihhangat", "lebihperlahan"),
    "sw": ("pia", "asilimia", "gizazaidi", "wastani", "nyeupejoto", "chinitoni"),
    "vi": ("nua", "phantram", "toihon giamden", "trungbinh", "trangam", "nhohon"),
    "hi": ("bhi", "pratishat", "zyadagehra dim", "madhyam", "garam safed", "ahista"),
    "bn": ("o", "shatakara", "aroandhar", "madhyam", "goromsada", "asthe"),
    "gu": ("pan", "takka", "vadhuneandharu", "madhyam", "garam safed", "dhime"),
    "kn": ("kooda", "shatamsha", "heccu kappu", "madhya", "bisi bili", "nidhanavagi"),
    "ml": ("koode", "shatamanam", "kooduthal iruttu", "madhyamam", "chusukulla vella", "shabdhamkuraykku"),
    "mr": ("suddha", "takke", "jahal andhar", "madhyam", "garam pandhra", "halu"),
    "ta": ("um", "satham", "irundal dim", "nadu", "velukku vellai", "meduva"),
    "te": ("kuda", "satam", "ekkuva nallaga", "madhyam", "vedi telupu", "mellaga"),
    "pa": ("vi", "satkar", "zyada hanera", "vichkar", "garam chitta", "hauli"),
    "ne": ("pani", "pratishat", "badhi andhyaro", "madhyam", "nyano seto", "bistari"),
    "hy": ("el", "tokos", "aveli mut", "mijn", "jag spitak", "aveli tsatsr"),
    "ka": ("c", "protsenti", "upro udaro", "shuaguli", "tbili tetri", "unda"),
    "mn": ("ch", "huvi", "ilh haranhui", "dund", "dulaan tsagaan", "suu"),
}

_FALLBACK = ("too", "percent", "dimmer darker", "medium", "warmwhite", "quieter")


def _row(code: str) -> tuple[str, str, str, str, str, str]:
    return _ROW.get(code, _FALLBACK)


def pack_words(code: str) -> dict[str, list[str]]:
    too, percent, dim, medium, warm, quiet = _row(code)
    return {
        "too": [too],
        "percent": [percent],
        "dim": dim.split(),
        "medium": [medium],
        "warmwhite": [warm],
        "quiet": [quiet],
    }


def with_warm_white(code: str, colors: list[tuple[str, str]]) -> list[tuple[str, str]]:
    if any(canon == "warmwhite" for _word, canon in colors):
        return colors
    warm = _row(code)[4]
    return list(colors) + [(warm, "warmwhite")]


def also_tokens() -> list[str]:
    seen: set[str] = set()
    out = ["auch", "too", "also", "well"]
    for too, *_rest in list(_ROW.values()) + [_FALLBACK]:
        if too not in seen:
            seen.add(too)
            out.append(too)
    return out
