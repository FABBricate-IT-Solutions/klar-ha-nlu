"""Except words, floor words, and extra family-home rooms for Assist packs.

FAMILY aliases ({bed}2, {entry}gang, {family}ess) belong in
tests/datasets/parity/rooms.yaml via scripts/parity/generate.py.
pack_extras() only copies spoken laundry/entry/except words into compiled packs.
"""

from __future__ import annotations

from lang_packs.convo import pack_words
from lang_packs.dock_play import dock_words, play_words

# code -> except tokens
EXCEPT = {
    "fr": ["sauf", "excepte"],
    "nl": ["behalve", "uitgezonderd"],
    "es": ["excepto", "salvo"],
    "it": ["eccetto", "tranne"],
    "pt": ["exceto", "excepto"],
    "ca": ["excepte", "tret"],
    "ro": ["exceptand", "fara"],
    "da": ["undtagen", "minus"],
    "nb": ["unntatt", "utenom"],
    "sv": ["utom", "forutom"],
    "fi": ["paitsi", "ilman"],
    "de-CH": ["ausser", "ohni"],
    "de-AT": ["ausser", "ohne"],
    "en-GB": ["except", "without"],
    "pt-BR": ["exceto", "menos"],
    "af": ["behalwe", "sonder"],
    "cs": ["krome", "mimo"],
    "sk": ["okrem", "mimo"],
    "pl": ["oprocz", "bez"],
    "hu": ["kiveve", "nelkul"],
    "hr": ["osim", "bez"],
    "sl": ["razen", "brez"],
    "bg": ["освен", "без"],
    "el": ["εκτος", "χωρις"],
    "sr": ["осим", "без"],
    "sr-Latn": ["osim", "bez"],
    "uk": ["крім", "без"],
    "zh-CN": ["除了", "除外"],
    "zh-TW": ["除了", "除外"],
    "zh-HK": ["除咗", "除外"],
    "ar": ["إلا", "ماعدا"],
    "he": ["חוץ", "למעט"],
    "fa": ["جز", "بدون"],
    "ur": ["سوا", "بغیر"],
    "tr": ["haric", "disinda"],
    "th": ["ยกเว้น", "นอกจาก"],
    "ko": ["빼고", "제외"],
    "ja": ["以外", "除く"],
    "cy": ["heblaw", "ac eithrio"],
    "et": ["peale", "valja"],
    "eu": ["izan", "ezik"],
    "ga": ["ach", "seachas"],
    "gl": ["excepto", "salvo"],
    "is": ["nema", "an"],
    "lb": ["ausser", "ouni"],
    "kw": ["marnas", "heb"],
    "lt": ["iskyrus", "be"],
    "lv": ["iznemot", "bez"],
    "id": ["kecuali", "selain"],
    "ms": ["kecuali", "selain"],
    "sw": ["isipokuwa", "bila"],
    "vi": ["tru", "ngoai"],
    "hi": ["sivay", "bina"],
    "bn": ["chara", "byatit"],
    "gu": ["sivay", "vagar"],
    "kn": ["hodake", "illade"],
    "ml": ["ozhike", "koodate"],
    "mr": ["vagal", "sivay"],
    "ta": ["thavira", "illaamal"],
    "te": ["tapp", "lekunda"],
    "pa": ["ilava", "bina"],
    "ne": ["bahek", "bina"],
    "hy": ["batsi", "aranc"],
    "ka": ["garda", "gareshe"],
    "mn": ["busad", "gui"],
}

# code -> (upper, ground)
FLOORS = {
    "fr": (["etage", "haut"], ["rez", "bas"]),
    "nl": (["boven", "verdieping"], ["beneden", "begane"]),
    "es": (["piso", "arriba"], ["planta", "abajo"]),
    "it": (["piano", "sopra"], ["terra", "sotto"]),
    "pt": (["andar", "cima"], ["terreo", "baixo"]),
    "ca": (["pis", "dalt"], ["baix", "terra"]),
    "ro": (["etaj", "sus"], ["parter", "jos"]),
    "da": (["overetage", "oppe"], ["stuenetage", "nede"]),
    "nb": (["overetasje", "oppe"], ["forste", "nede"]),
    "sv": (["ovanvaning", "uppe"], ["botten", "nere"]),
    "fi": (["yla", "kerros"], ["ala", "pohja"]),
    "de-CH": (["obergeschoss", "obe"], ["erdgeschoss", "unte"]),
    "de-AT": (["obergeschoss", "oben"], ["erdgeschoss", "unten"]),
    "en-GB": (["upstairs", "upper"], ["downstairs", "ground"]),
    "pt-BR": (["andar", "cima"], ["terreo", "baixo"]),
    "af": (["boonste", "bo"], ["onderste", "onder"]),
    "cs": (["patro", "nahore"], ["prizemi", "dole"]),
    "sk": (["poschodie", "hore"], ["prizemie", "dole"]),
    "pl": (["pietro", "gora"], ["parter", "dol"]),
    "hu": (["emelet", "fent"], ["foldszint", "lent"]),
    "hr": (["kat", "gore"], ["prizemlje", "dolje"]),
    "sl": (["nadstropje", "zgoraj"], ["pritlicje", "spodaj"]),
    "bg": (["етаж", "горе"], ["партер", "долу"]),
    "el": (["οροφος", "πανω"], ["ισογειο", "κατω"]),
    "sr": (["спрат", "горе"], ["приземље", "доле"]),
    "sr-Latn": (["sprat", "gore"], ["prizemlje", "dole"]),
    "uk": (["поверх", "вгорі"], ["перший", "внизу"]),
    "zh-CN": (["loushang", "shangceng"], ["louxia", "diceng"]),
    "zh-TW": (["loushang", "shangceng"], ["louxia", "diceng"]),
    "zh-HK": (["loushang", "shangceng"], ["louxia", "diceng"]),
    "ar": (["فوق", "طابق"], ["ارض", "تحت"]),
    "he": (["קומה", "למעלה"], ["קרקע", "למטה"]),
    "fa": (["بالا", "طبقه"], ["پایین", "همکف"]),
    "ur": (["اوپر", "منزل"], ["نیچے", "زمین"]),
    "tr": (["ust", "kat"], ["alt", "zemin"]),
    "th": (["chanbon", "bon"], ["chanlang", "lang"]),
    "ko": (["wicheung", "wi"], ["araecheung", "arae"]),
    "ja": (["joukai", "ue"], ["kakai", "shita"]),
    "cy": (["llawr", "uwch"], ["gwaelod", "is"]),
    "et": (["korrus", "uleval"], ["alumine", "allkorrus"]),
    "eu": (["solairu", "goian"], ["behe", "behean"]),
    "ga": (["urlár", "thuas"], ["bun", "thios"]),
    "gl": (["andar", "arriba"], ["baixo", "terra"]),
    "is": (["haed", "upp"], ["nedri", "nidri"]),
    "lb": (["iwwer", "uewen"], ["ierd", "ënnen"]),
    "kw": (["leur", "ugh"], ["dor", "is"]),
    "lt": (["aukstas", "virsuj"], ["pirmas", "apacioje"]),
    "lv": (["stavs", "augsa"], ["pirmais", "apaks"]),
    "id": (["atas", "lantai"], ["bawah", "dasar"]),
    "ms": (["atas", "tingkat"], ["bawah", "dasar"]),
    "sw": (["juu", "orofa"], ["chini", "chini"]),
    "vi": (["tang", "tren"], ["duoi", "tret"]),
    "hi": (["upar", "manzil"], ["neeche", "bhumi"]),
    "bn": (["upor", "tala"], ["nich", "nich"]),
    "gu": (["upar", "majla"], ["niche", "zamin"]),
    "kn": (["mele", "antastu"], ["kelage", "nelada"]),
    "ml": (["mukalil", "nela"], ["thazhe", "bhumi"]),
    "mr": (["var", "majla"], ["khali", "bhumi"]),
    "ta": (["mel", "thattu"], ["kizh", "nilam"]),
    "te": (["paina", "anta"], ["kinda", "bhumi"]),
    "pa": (["uppar", "manzil"], ["hethan", "zamin"]),
    "ne": (["mathi", "talla"], ["tala", "bhumi"]),
    "hy": (["verev", "hark"], ["nerqev", "getin"]),
    "ka": (["zeda", "sartuli"], ["qveda", "pirveli"]),
    "mn": (["deed", "davhar"], ["dood", "gazar"]),
}

def _family(entry: str, family: str, laundry: str, powder: str, bed: str, bath: str, *, de_compounds: bool = False) -> dict:
    compact = bed.replace(" ", "")
    rooms = {
        "entryway": [entry],
        "family_room": [family],
        "laundry": [laundry],
        "powder_room": [powder],
        "garage": ["garage"],
        "bedroom_2": [f"{compact}2"],
        "bedroom_3": [f"{compact}3"],
        "bedroom_4": [f"{compact}4"],
        "master_bath": [bath],
        "kids": [f"{compact}kids"],
        "dining": [f"{family}ess"],
        "master_bedroom": [bed],
        "hallway": [f"{entry}gang"],
    }
    if de_compounds:
        rooms["schlafzimmer"] = [bed]
        rooms["flur"] = [f"{entry}gang"]
        rooms["hallway"] = [f"{entry}gang"]
        rooms["esszimmer"] = [f"{family}ess"]
        rooms["dining"] = [f"{family}ess"]
        rooms["badezimmer"] = [f"{bath}main"]
        rooms["main_bath"] = [f"{bath}main"]
    return rooms


FAMILY = {
    "fr": _family("entree", "famille", "buanderie", "toilettes", "chambre", "bainmaster"),
    "nl": _family("hal", "huiskamer", "wasruimte", "toilet", "slaapkamer", "badmaster"),
    "es": _family("entrada", "estancia", "lavadero", "aseo", "dormitorio", "banomaster"),
    "it": _family("ingresso", "soggiornofam", "lavanderia", "bagnetto", "camera", "bagnomaster"),
    "pt": _family("entrada", "salaestar", "lavandaria", "wc", "quarto", "banhomaster"),
    "ca": _family("entrada", "salaestar", "bugaderia", "lavabo", "habitacio", "banymaster"),
    "ro": _family("hol", "sufrageriefam", "spalatorie", "toaleta", "dormitor", "baiemaster"),
    "da": _family("entree", "alrum", "vaskerum", "toilet", "sovevaerelse", "badmaster"),
    "nb": _family("entree", "stuefam", "vaskerom", "toalett", "soverom", "badmaster"),
    "sv": _family("entre", "allrum", "tvattstuga", "toalett", "sovrum", "badmaster"),
    "fi": _family("eteinen", "olohuonefam", "kodinhoito", "wc", "makuuhuone", "kylpymaster"),
    "de-CH": _family("eingang", "stube", "waschruum", "toilette", "schlafzimmer", "badmaster", de_compounds=True),
    "de-AT": _family("vorzimmer", "wohnstube", "waschraum", "klosett", "schlafzimmer", "badmaster", de_compounds=True),
    "en-GB": _family("porch", "loungefam", "utility", "loo", "bedroom", "ensuite"),
    "pt-BR": _family("hallentrada", "salafam", "lavanderia", "lavabo", "quarto", "suitemaster"),
    "af": _family("ingang", "sitkamerfam", "waskamer", "toilet", "slaapkamer", "badmaster"),
    "cs": _family("predsin", "obyvakfam", "pradelna", "wc", "loznice", "koupelmaster"),
    "sk": _family("predsien", "obyvackafam", "pradelna", "wc", "spalna", "kupelmaster"),
    "pl": _family("przedpokoj", "salonfam", "pralnia", "toaleta", "sypialnia", "lazmaster"),
    "hu": _family("eloter", "nappalifam", "mosokonyha", "wc", "haloszoba", "furdomaster"),
    "hr": _family("hodnik", "dnevnifam", "perionica", "wc", "spavaca", "kupaonicamaster"),
    "sl": _family("predsoba", "dnevnafam", "pralnica", "wc", "spalnica", "kopalmaster"),
    "bg": _family("антре", "дневнаfam", "пералня", "тоалетна", "спалня", "баняmaster"),
    "el": _family("εισοδος", "σαλονιfam", "πλυσταριο", "wc", "υπνοδωματο", "μπανιοmaster"),
    "sr": _family("улаз", "дневнаfam", "вешерница", "wc", "спавача", "купатилоmaster"),
    "sr-Latn": _family("ulaz", "dnevnafam", "vesernica", "wc", "spavaca", "kupatilomaster"),
    "uk": _family("передпокій", "вітальняfam", "пральня", "туалет", "спальня", "ваннаmaster"),
    "zh-CN": _family("xuanguan", "jiatingfang", "xiyifang", "kewei", "woshi", "zhuwei"),
    "zh-TW": _family("xuanguan", "jiatingfang", "xiyifang", "kewei", "woshi", "zhuwei"),
    "zh-HK": _family("xuanguan", "jiatingfang", "xiyifang", "kewei", "woshi", "zhuwei"),
    "ar": _family("مدخل", "عائلي", "غسيل", "حمامضيف", "غرفة", "حمامرئيسي"),
    "he": _family("כניסה", "משפחה", "כביסה", "שירותים", "חדרשינה", "חדררחצהראשי"),
    "fa": _family("ورودی", "خانواده", "رختشویی", "سرویس", "اتاقخواب", "حماماصلی"),
    "ur": _family("داخلہ", "خاندانی", "لانڈری", "غسلخانہ", "bedroom", "ماسٹرباتھ"),
    "tr": _family("giris", "aile", "camasir", "wc", "yatakodasi", "ebatmaster"),
    "th": _family("thangkhau", "hongkhropkhrua", "hongsakpha", "hongnamkhaek", "hongnon", "hongnamlak"),
    "ko": _family("hyeongwan", "gajoksil", "setaksil", "hwajangsil", "chimsil", "anbangyoksil"),
    "ja": _family("genkan", "famirii", "sentakushitsu", "toire", "shinshitsu", "shuyokushitsu"),
    "cy": _family("cyntedd", "ystafellteulu", "ystafellolchi", "toiled", "llofft", "bathmaster"),
    "et": _family("esik", "peredetuba", "pesuruum", "wc", "magamistuba", "vannmaster"),
    "eu": _family("sarrera", "familiagela", "garbiketa", "wc", "logela", "bainumaster"),
    "ga": _family("halla", "seomrateaghlaigh", "seomraniochta", "leithreas", "seomraleong", "folcadanmaster"),
    "gl": _family("entrada", "salafam", "lavandaria", "aseo", "cuarto", "banomaster"),
    "is": _family("anddyri", "stofafam", "thvottahus", "klósett", "svefnherbergi", "badmaster"),
    "lb": _family("entrada", "wunnsall", "waschraum", "toilette", "schlofzëmmer", "bademaster"),
    "kw": _family("porth", "stevellteylu", "stevellgolhi", "tybach", "chambour", "kewndhavas"),
    "lt": _family("prieskambaris", "svetainefam", "skalbykla", "wc", "miegamasis", "voniamaster"),
    "lv": _family("prieksnams", "dzivojamafam", "velas", "wc", "gulamistaba", "vannamaster"),
    "id": _family("lorong", "ruangkeluarga", "binatu", "wc", "kamar", "kamarmandimaster"),
    "ms": _family("lorong", "ruankeluarga", "dobi", "wc", "bilik", "bilikmandimaster"),
    "sw": _family("kiingilio", "sefula", "chumbayaosha", "choo", "chumbakulala", "bafumaster"),
    "vi": _family("loivao", "phonggiadinh", "phonggiat", "wc", "phongngu", "phongtamchinh"),
    "hi": _family("pravesh", "parivarik", "dhobi", "shauchalay", "shayankaksh", "mukhyasnan"),
    "bn": _family("prabesh", "paribarik", "dhoa", "shauchagar", "shobarghor", "masterbath"),
    "gu": _family("pravesh", "kutumb", "dhovanu", "shauchalay", "shayankhand", "mukhyasnan"),
    "kn": _family("pravesha", "kutumba", "ogeyuva", "shauchalaya", "shayanakone", "mukhyasnana"),
    "ml": _family("praveshanam", "kutumbam", "alakku", "toilet", "kitappumuri", "masterbath"),
    "mr": _family("pravesh", "kutumb", "dhune", "swachhata", "shayankaksh", "mukhyasnan"),
    "ta": _family("nuzhaivu", "kutumbam", "salavai", "kazhivarai", "padukkaiyarai", "muthanmaikuliyal"),
    "te": _family("pravesham", "kutumbam", "laundry", "toilet", "padakagadi", "masterbath"),
    "pa": _family("dakhla", "parivarak", "dhon", "toilet", "bedroom", "masterbath"),
    "ne": _family("pravesh", "parivarik", "dhune", "shauchalay", "sutnekotha", "mukhyasnan"),
    "hy": _family("mutq", "yntaniq", "lvacqaran", "wc", "qnakaran", "lvacaranmaster"),
    "ka": _family("shemosasvleli", "ojakhisotakhi", "saretskhave", "wc", "sadzinebeli", "abazanamaster"),
    "mn": _family("orol", "gerbul", "ugaalga", "oroon", "untlagiin", "untlagiinugasalgach"),
}



def except_words(code: str) -> list[str]:
    return list(EXCEPT.get(code, []))


def floors(code: str) -> dict[str, list[str]]:
    upper, ground = FLOORS.get(code, (["upper"], ["ground"]))
    return {"upper": list(upper), "ground": list(ground)}


HOUSEHOLD = {
    "de-CH": {
        "teach": ["nenn das ", "nenne das ", "das heisst "],
        "explain": ["was hesch ghoert", "was hast du gehoert", "warum hast du gestoppt"],
        "undo": ["rueckgaengig", "nimm das zurueck"],
        "clock": ["wie spaet", "wie viel uhr", "d uhrzit"],
        "weather": ["wie isch s wetter", "wetterbericht", "wetter"],
        "clock_skip": ["timer", "wecker"],
    },
    "de-AT": {
        "teach": ["nenn das ", "nenne das ", "das heisst "],
        "explain": ["was hast du gehoert", "was hast du verstanden", "warum hast du gestoppt"],
        "undo": ["rueckgaengig", "nimm das zurueck"],
        "clock": ["wie spaet", "wie viel uhr", "die uhrzeit"],
        "weather": ["wie ist das wetter", "wetterbericht", "wetter"],
        "clock_skip": ["timer", "wecker"],
    },
    "en-GB": {
        "teach": ["call this ", "call it ", "name this "],
        "explain": ["what did you hear", "what did you understand", "why did you stop"],
        "undo": ["undo that", "undo", "take that back"],
        "clock": ["what time", "whats the time"],
        "weather": ["whats the weather", "what is the weather", "weather forecast"],
        "clock_skip": ["timer", "alarm"],
    },
}


# island, ceiling, globe, bedside, device, film, night, leave
WORDS = {
    "fr": ("ilot", "plafond", "globe", "chevet", "appareil", "film", "nuit", "partir"),
    "nl": ("eiland", "plafond", "bol", "nachtkast", "apparaat", "film", "nacht", "vertrek"),
    "es": ("isla", "techo", "bola", "mesilla", "aparato", "cine", "noche", "salida"),
    "it": ("isola", "soffitto", "sfera", "comodino", "apparecchio", "film", "notte", "uscita"),
    "pt": ("ilha", "teto", "globo", "cabeceira", "aparelho", "filme", "noite", "saida"),
    "ca": ("illa", "sostre", "bola", "tauleta", "aparell", "film", "nit", "sortida"),
    "ro": ("insula", "tavan", "globe", "noptiera", "aparat", "film", "noapte", "plecare"),
    "da": ("oe", "loft", "kugle", "natbord", "apparat", "film", "nat", "afgang"),
    "nb": ("oy", "tak", "kule", "nattbord", "apparat", "film", "natt", "avgang"),
    "sv": ("o", "tak", "kula", "sanglampa", "apparat", "film", "natt", "avgang"),
    "fi": ("saareke", "katto", "pallo", "yopoyta", "laite", "elokuva", "yo", "lahto"),
    "de-CH": ("insel", "decke", "kugel", "nachttisch", "geraet", "filmabend", "nacht", "verlassen"),
    "de-AT": ("insel", "decke", "kugel", "nachttisch", "geraet", "filmabend", "nacht", "verlassen"),
    "en-GB": ("island", "ceiling", "globe", "bedside", "device", "film", "night", "leaving"),
    "pt-BR": ("ilha", "teto", "globo", "cabeceira", "aparelho", "filme", "noite", "saida"),
    "af": ("eiland", "plafon", "bol", "nagus", "toestel", "film", "nag", "vertrek"),
    "cs": ("ostrov", "strop", "koule", "nocnik", "pristroj", "film", "vecer", "odchod"),
    "sk": ("ostrov", "strop", "gula", "nocny", "pristroj", "film", "noc", "odchod"),
    "pl": ("wyspa", "sufit", "kula", "noca", "urzadzenie", "film", "noc", "wyjscie"),
    "hu": ("sziget", "mennyezet", "gomb", "ejjeliszekreny", "keszulek", "film", "ejjel", "tavozas"),
    "hr": ("otok", "strop", "kugla", "nocni", "uredaj", "film", "noc", "odlazak"),
    "sl": ("otok", "strop", "krogla", "nocna", "naprava", "film", "noc", "odhod"),
    "bg": ("остров", "таван", "топка", "нощно", "уред", "филм", "нощ", "изход"),
    "el": ("νησι", "ταβανι", "σφαιρα", "κομοδινο", "συσκευη", "ταινια", "νυχτα", "εξοδος"),
    "sr": ("острво", "плафон", "кугла", "ноћни", "уређај", "филм", "ноћ", "излазак"),
    "sr-Latn": ("ostrvo", "plafon", "kugla", "nocni", "uredjaj", "film", "noc", "izlazak"),
    "uk": ("острів", "стеля", "куля", "нічний", "пристрій", "фільм", "ніч", "вихід"),
    "zh-CN": ("daotai", "tianhuaban", "qiudeng", "chuangtoudeng", "shebei", "dianying", "wanshang", "likai"),
    "zh-TW": ("daotai", "tianhuaban", "qiudeng", "chuangtoudeng", "shebei", "dianying", "wanshang", "likai"),
    "zh-HK": ("daotai", "tianhuaban", "qiudeng", "chuangtoudeng", "shebei", "dianying", "wanshang", "likai"),
    "ar": ("جزيرة", "سقف", "كرة", "جانبسرير", "جهاز", "فيلم", "ليل", "خروج"),
    "he": ("אי", "תקרה", "כדור", "לידמיטה", "מכשיר", "סרט", "לילה", "יציאה"),
    "fa": ("جزیره", "سقف", "گوی", "کنارتخت", "دستگاه", "فیلم", "شب", "خروج"),
    "ur": ("جزیرہ", "چھت", "گولا", "بسترکنار", "آلہ", "فلم", "رات", "خروج"),
    "tr": ("ada", "tavan", "kure", "komodin", "cihaz", "film", "gece", "cikis"),
    "th": ("ko", "phalang", "luk", "khangtiang", "khrueang", "nang", "khuen", "ok"),
    "ko": ("seom", "cheonjang", "gong", "chimdae", "jangchi", "yeonghwa", "bam", "chulbal"),
    "ja": ("shima", "tenjo", "tama", "beddowaki", "kiki", "eiga", "yoru", "taishutsu"),
    "cy": ("ynys", "nenfwd", "pelen", "gwely", "dyfais", "ffilm", "nos", "gadael"),
    "et": ("saar", "lagi", "kera", "ooekapp", "seade", "film", "oo", "lahkumine"),
    "eu": ("uharte", "sabai", "bola", "gaua", "gailu", "filma", "gau", "irteera"),
    "ga": ("oilean", "uasteorainn", "liathroid", "leaba", "gléas", "scannan", "oiche", "imeacht"),
    "gl": ("illa", "teito", "bola", "mesilla", "aparello", "filme", "noite", "saida"),
    "is": ("eyja", "loft", "kula", "nattbord", "taeki", "kvikmynd", "nott", "brottfor"),
    "lb": ("eiland", "decken", "kugel", "nuettsdesch", "apparat", "film", "nuets", "fortgoen"),
    "kw": ("enys", "nen", "pel", "gweli", "tol", "film", "nos", "mos"),
    "lt": ("sala", "lubos", "rutulys", "naktinis", "prietaisas", "filmas", "naktis", "isejimas"),
    "lv": ("sala", "griesti", "lode", "nakts", "ierice", "filma", "nakts", "izeja"),
    "id": ("pulau", "langit", "bola", "nakas", "alat", "film", "malam", "pergi"),
    "ms": ("pulau", "siling", "bola", "katil", "alat", "filem", "malam", "pergi"),
    "sw": ("kisiwa", "dari", "tufe", "kitanda", "kifaa", "filamu", "usiku", "ondoka"),
    "vi": ("dao", "tran", "cau", "dau giuong", "thietbi", "phim", "dem", "ra ve"),
    "hi": ("dweep", "chhat", "gola", "palang", "upakaran", "film", "raat", "jaana"),
    "bn": ("dip", "chhad", "gola", "bichana", "jantro", "chobi", "rat", "ber"),
    "gu": ("tapu", "chhat", "gol", "palang", "upkaran", "film", "raat", "javu"),
    "kn": ("dweepa", "chavani", "goli", "mancha", "upakarana", "cinema", "ratri", "horatu"),
    "ml": ("dweep", "tosh", "golam", "kattil", "upakaranam", "cinema", "rathri", "poku"),
    "mr": ("dweep", "chhat", "gol", "palang", "upkaran", "chitrapat", "ratra", "jane"),
    "ta": ("theevu", "meetai", "undai", "kattil", "karuvi", "padam", "iravu", "veli"),
    "te": ("dweepam", "kappu", "burra", "mancham", "yantram", "cinema", "ratri", "vellu"),
    "pa": ("tapu", "chhat", "gola", "palang", "upkaran", "film", "raat", "jana"),
    "ne": ("dweep", "chhana", "gola", "khat", "upakaran", "chalchitra", "raat", "jane"),
    "hy": ("kghzi", "stalat", "gundayin", "gisherayin", "sark", "film", "gisher", "elq"),
    "ka": ("kuntkhuli", "potoloni", "burtuli", "sagamise", "mokobiloba", "filmi", "ghame", "gasvla"),
    "mn": ("aran", "taaz", "bomboo", "oronii", "togkhoromj", "kino", "shono", "garah"),
}


# washer, dishwasher, tv
APPLIANCES = {
    "fr": ("lave", "lavevaisselle", "tele"),
    "nl": ("wasmachine", "vaatwasser", "tv"),
    "es": ("lavadora", "lavavajillas", "tele"),
    "it": ("lavatrice", "lavastoviglie", "tv"),
    "pt": ("maquina", "loiça", "tv"),
    "ca": ("rentadora", "eixugadora", "tele"),
    "ro": ("masina", "vase", "tv"),
    "da": ("vaskemaskine", "opvask", "tv"),
    "nb": ("vaskemaskin", "oppvask", "tv"),
    "sv": ("tvattmaskin", "disk", "tv"),
    "fi": ("pesukone", "astianpesu", "tv"),
    "de-CH": ("waschmaschine", "spuelmaschine", "tv"),
    "de-AT": ("waschmaschine", "spuelmaschine", "tv"),
    "en-GB": ("washer", "dishwasher", "telly"),
    "pt-BR": ("maquina", "louca", "tv"),
    "af": ("wasmasjien", "skottelgoed", "tv"),
    "cs": ("pracka", "mycka", "televize"),
    "sk": ("pracka", "umyvacka", "televizor"),
    "pl": ("pralka", "zmywarka", "telewizor"),
    "hu": ("mosogep", "mosogatogep", "tv"),
    "hr": ("perilica", "posuda", "tv"),
    "sl": ("pralni", "pomivalni", "tv"),
    "bg": ("пералня", "съдомиялна", "тв"),
    "el": ("πλυντηριο", "πιατα", "τηλεοραση"),
    "sr": ("веш", "судови", "тв"),
    "sr-Latn": ("ves", "sudovi", "tv"),
    "uk": ("пральна", "посудомийка", "тв"),
    "zh-CN": ("xiyiji", "xiwanji", "dianshi"),
    "zh-TW": ("xiyiji", "xiwanji", "dianshi"),
    "zh-HK": ("xiyiji", "xiwanji", "dianshi"),
    "ar": ("غسالة", "جلاية", "تلفاز"),
    "he": ("מכונתכביסה", "מדיח", "טלוויזיה"),
    "fa": ("لباسشویی", "ظرفشویی", "تلویزیون"),
    "ur": ("واشنگ", "برتن", "ٹی وی"),
    "tr": ("camasir", "bulasik", "tv"),
    "th": ("sakpha", "langchan", "thorathat"),
    "ko": ("setakgi", "sikgi", "tv"),
    "ja": ("sentakuki", "shokkaiki", "terebi"),
    "cy": ("peiriannolchi", "llestri", "teledu"),
    "et": ("pesumasin", "noudepesu", "tv"),
    "eu": ("garbigailu", "ontzigailu", "tb"),
    "ga": ("meaisinniocadh", "soitheach", "teilifis"),
    "gl": ("lavadora", "lavavajillas", "tv"),
    "is": ("thvottavel", "uppvthvott", "sjónvarp"),
    "lb": ("waschmaschinn", "spuelmaschinn", "tv"),
    "kw": ("jynnwolhi", "lestri", "pellwolok"),
    "lt": ("skalbimo", "indaplove", "tv"),
    "lv": ("veļasmašīna", "trauku", "tv"),
    "id": ("pencuci", "piring", "tv"),
    "ms": ("mesinbasuh", "pinggan", "tv"),
    "sw": ("mashine", "vyombo", "tv"),
    "vi": ("maygiat", "chen", "tivi"),
    "hi": ("washing", "bartan", "tv"),
    "bn": ("washing", "basan", "tv"),
    "gu": ("washing", "vasan", "tv"),
    "kn": ("washing", "patre", "tv"),
    "ml": ("washing", "pathram", "tv"),
    "mr": ("washing", "bhandi", "tv"),
    "ta": ("washing", "pattiram", "tv"),
    "te": ("washing", "ginjalu", "tv"),
    "pa": ("washing", "bartan", "tv"),
    "ne": ("washing", "basna", "tv"),
    "hy": ("lvacqimekena", "aman", "tv"),
    "ka": ("saretskhave", "jamtchebi", "tv"),
    "mn": ("ugaalgyn", "sav", "tv"),
}

DRYERS = {
    "fr": "sechelinge",
    "cs": "susicka",
    "sk": "susicka",
    "ja": "kansouki",
    "nl": "droger",
    "es": "secadora",
    "it": "asciugatrice",
    "de-CH": "trockner",
    "de-AT": "trockner",
    "pl": "suszarka",
}


def pack_extras(code: str) -> dict[str, list[str]]:
    extra: dict[str, list[str]] = {}
    words = except_words(code)
    if words:
        extra["except"] = words
    family = FAMILY.get(code)
    if family:
        extra["laundry"] = list(family["laundry"])
        extra["entry"] = list(family["entryway"])
    extra.update(HOUSEHOLD.get(code, {}))
    if code in WORDS:
        island, ceiling, globe, bedside, device, film, night, leave = WORDS[code]
        extra.setdefault("island", [island])
        extra.setdefault("ceiling", [ceiling])
        extra.setdefault("globe", [globe])
        extra.setdefault("bedside", [bedside])
        extra.setdefault("device", [device])
        extra.setdefault("named", [globe])
        extra.setdefault("scenes", [film])
        extra.setdefault("good_night", [night])
        extra.setdefault("leaving", [leave])
        extra.setdefault("switch", [device])
        if code in APPLIANCES:
            washer, dish, tv = APPLIANCES[code]
            extra.setdefault("washer", [washer])
            extra.setdefault("dishwasher", [dish])
            extra.setdefault("tv", [tv])
            extra.setdefault("dryer", [DRYERS.get(code, "dryer")])
            extra["named"] = list(dict.fromkeys(extra.get("named", []) + [globe, washer, dish, tv]))
    if dock := dock_words(code):
        extra.setdefault("dock", dock)
    if play := play_words(code):
        extra.setdefault("play", play)
    for key, words in pack_words(code).items():
        extra.setdefault(key, words)
    return extra
