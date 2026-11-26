"""Real household and chat trigger phrases (package 4.4 / L-17, L-22).

Phrases are never built from on/off/set verbs. Empty means the pack must not
claim that skill; clock and weather stay required by check_lang_packs.
"""

from __future__ import annotations

# code -> (clock|…, weekday|…, weather|…, explain|…, greet|…)
_ROW: dict[str, tuple[str, str, str, str, str]] = {
    "fr": ("quelle heure|quelle heure est il", "quel jour|quel jour est il", "quel temps|meteo", "qu as tu entendu|pourquoi as tu arrete", "bonjour|salut|coucou|bonsoir"),
    "nl": ("hoe laat|hoe laat is het", "welke dag|welke weekdag", "hoe is het weer|weerbericht", "wat heb je gehoord|waarom gestopt", "hallo|hoi"),
    "es": ("que hora|que hora es", "que dia|que dia es", "que tiempo|el tiempo", "que has oido|por que paraste", "hola|buenas"),
    "it": ("che ore|che ore sono", "che giorno|che giorno e", "che tempo|il tempo", "cosa hai sentito|perche hai fermato", "ciao|salve"),
    "pt": ("que horas|que horas sao", "que dia|que dia e", "que tempo|o tempo", "o que ouviste|porque paraste", "ola|oi"),
    "ca": ("quina hora|quina hora es", "quin dia|quin dia es", "quin temps|el temps", "que has sentit|per que has parat", "hola|bon dia"),
    "ro": ("ce ora|cat este ceasul", "ce zi|ce zi este", "ce vreme|vremea", "ce ai auzit|de ce te-ai oprit", "salut|buna"),
    "da": ("hvad er klokken|hvilken tid", "hvilken dag|hvad er dagen", "hvordan er vejret|vejret", "hvad horte du|hvorfor stoppede du", "hej|goddag"),
    "nb": ("hva er klokka|hvilken tid", "hvilken dag|hva er dagen", "hvordan er været|været", "hva horte du|hvorfor stoppet du", "hei|god dag"),
    "sv": ("vad ar klockan|vilken tid", "vilken dag|vad ar dagen", "hur ar vadret|vadret", "vad horde du|varfor stoppade du", "hej|god dag"),
    "fi": ("mita kello on|paljonko kello", "mika paiva|mika viikonpaiva", "millainen saa|saa", "mita kuulit|miksi pysahdyit", "hei|moi"),
    "de-CH": ("wie spaet|wie viel uhr", "welcher tag|welcher wochentag", "wie isch s wetter|wetter", "was hesch ghoert|warum gestoppt", "hallo|grüezi"),
    "de-AT": ("wie spaet|wie viel uhr", "welcher tag|welcher wochentag", "wie ist das wetter|wetter", "was hast du gehoert|warum gestoppt", "hallo|servus"),
    "en-GB": ("what time|whats the time", "what day is|what day of the week", "whats the weather|weather forecast", "what did you hear|why did you stop", "hello|hi"),
    "pt-BR": ("que horas|que horas sao", "que dia da semana|qual o dia", "como esta o tempo|previsao do tempo", "o que voce ouviu|por que parou", "ola|oi"),
    "af": ("hoe laat|watter tyd", "watter dag|watter weekdag", "hoe is die weer|weer", "wat het jy gehoor|hoekom gestop", "hallo|hi"),
    "cs": ("kolik je hodin|jaky cas", "jaky den|ktery den", "jake je pocasi|pocasi", "co jsi slysel|proc jsi zastavil", "ahoj|dobry den"),
    "sk": ("kolko je hodin|aky cas", "aky den|ktory den", "ake je pocasie|pocasie", "co si pocul|preco si zastavil", "ahoj|dobry den"),
    "pl": ("ktora godzina|jaki czas", "jaki dzien|ktory dzien", "jaka pogoda|pogoda", "co slyszales|dlaczego zatrzymales", "czesc|dzien dobry"),
    "hu": ("mennyi az ido|hány óra", "milyen nap|melyik nap", "milyen az ido|idojaras", "mit hallottal|miert alltal le", "szia|jo napot"),
    "hr": ("koliko je sati|koje je vrijeme", "koji dan|koji je dan", "kakvo je vrijeme|vrijeme", "sto si cuo|zasto si stao", "bok|dobar dan"),
    "sl": ("koliko je ura|katera ura", "kateri dan|kaksen dan", "kakšno je vreme|vreme", "kaj si slišal|zakaj si ustavil", "zivjo|dober dan"),
    "bg": ("колко е часът|какъв час", "кой ден|какъв ден", "какво е времето|времето", "какво чу|защо спря", "здравей|здрасти"),
    "el": ("τι ωρα ειναι|ποια ωρα", "ποια μερα|τι μερα", "τι καιρος|ο καιρος", "τι ακουσες|γιατι σταματησες", "γεια|καλημερα"),
    "sr": ("колико је сати|које је време", "који дан|какав дан", "какво је време|време", "шта си чуо|зашто си стао", "здраво|ћao"),
    "sr-Latn": ("koliko je sati|koje je vreme", "koji dan|kakav dan", "kakvo je vreme|vreme", "sta si cuo|zasto si stao", "zdravo|cao"),
    "uk": ("котра година|який час", "який день|котрий день", "яка погода|погода", "що ти чув|чому зупинився", "привіт|добрий день"),
    "zh-CN": ("几点了|现在几点", "星期几|今天星期几", "天气怎么样|天气预报", "你听到了什么|为什么停下", "你好|您好"),
    "zh-TW": ("幾點了|現在幾點", "星期幾|今天星期幾", "天氣怎麼樣|天氣預報", "你聽到了什麼|為什麼停下", "你好|您好"),
    "zh-HK": ("幾點呀|而家幾點", "星期幾|今日星期幾", "天氣點呀|天氣預報", "你聽到咩|點解停", "你好|哈囉"),
    "ar": ("كم الساعة|ما الوقت", "اي يوم|ما اليوم", "كيف الطقس|الطقس", "ماذا سمعت|لماذا توقفت", "مرحبا|السلام عليكم"),
    "he": ("מה השעה|איזו שעה", "איזה יום|מה היום", "מה מזג האוויר|מזג האוויר", "מה שמעת|למה עצרת", "שלום|היי"),
    "fa": ("ساعت چنده|چه ساعتی", "چه روزی|امروز چه روزی", "هوا چطوره|آب و هوا", "چه شنیدی|چرا ایستادی", "سلام|درود"),
    "ur": ("کتنے بجے|وقت کیا ہے", "کون سا دن|آج کون سا دن", "موسم کیسا ہے|موسم", "کیا سنا|کیوں رکے", "سلام|ہیلو"),
    "tr": ("saat kac|saat nedir", "hangi gun|bugun hangi gun", "hava nasil|hava durumu", "ne duydun|neden durdun", "merhaba|selam"),
    "th": ("กี่โมง|ตอนนี้กี่โมง", "วันอะไร|วันนี้วันอะไร", "อากาศเป็นยังไง|พยากรณ์อากาศ", "ได้ยินอะไร|ทำไมหยุด", "สวัสดี|หวัดดี"),
    "ko": ("몇 시야|지금 몇 시", "무슨 요일|오늘 무슨 요일", "날씨 어때|날씨", "뭐 들었어|왜 멈췄어", "안녕|안녕하세요"),
    "ja": ("何時|今何時", "何曜日|今日は何曜日", "天気はどう|天気予報", "何を聞いた|なぜ止めた", "こんにちは|やあ"),
    "cy": ("faint o'r gloch|pa amser", "pa ddiwrnod|pa ddydd", "sut mae'r tywydd|tywydd", "beth glywaist ti|pam wnaethost ti stopio", "helo|shwmae"),
    "et": ("mis kell on|kui palju on kell", "mis paev|mis nadalapaev", "kuidas on ilm|ilm", "mida sa kuulsid|miks sa peatusid", "tere|tsau"),
    "eu": ("zer ordu da|zenbat dira", "zer egun|zein egun", "zer eguraldi|eguraldia", "zer entzun duzu|zergatik gelditu zara", "kaixo|agur"),
    "ga": ("cen t-am e|cad e an t-am", "cen la e|cad e an la", "cen chuma ata ar an aimsir|aimsir", "cad a chuala tu|cen fath ar stop tu", "dia duit|haigh"),
    "gl": ("que hora e|que horas son", "que dia e|que dia", "que tempo fai|o tempo", "que escoitaches|por que paraches", "ola|boas"),
    "is": ("hvad er klukkan|hvada timi", "hvada dagur|hvad er dagurinn", "hvernig er vedurid|vedur", "hvad heyrdir thu|af hverju stoppadir thu", "hallo|saell"),
    "lb": ("wéi spéit|wéi vill auer", "wéi en dag|wéi e wochendag", "wéi ass d wetter|wetter", "wat hues de héieren|firwat hues de gestoppt", "moien|hallo"),
    "kw": ("py eur yw|py dermyn", "py jydh|py dydh", "py awel|awel", "pyth a glewsys|prag y hedhys", "dydh da|lowena"),
    "lt": ("kiek valandu|koks laikas", "kokia diena|kuri diena", "koks oras|oras", "ka girdėjai|kodel sustojai", "labas|sveiki"),
    "lv": ("cik ir pulkstenis|kads laiks", "kada diena|kura diena", "kads ir laiks|laikapstakli", "ko tu dzirdeji|kapec tu apstajas", "sveiki|labdien"),
    "id": ("jam berapa|pukul berapa", "hari apa|hari apa sekarang", "cuaca bagaimana|cuaca", "apa yang kau dengar|kenapa berhenti", "halo|hai"),
    "ms": ("pukul berapa|jam berapa", "hari apa|hari apakah", "cuaca bagaimana|cuaca", "apa yang kau dengar|kenapa berhenti", "halo|hai"),
    "sw": ("saa ngapi|ni saa ngapi", "siku gani|ni siku gani", "hali ya hewa|hali ya anga", "ulisikia nini|kwa nini ulisimama", "habari|jambo"),
    "vi": ("may gio|bay gio la may gio", "thu may|hom nay thu may", "thoi tiet the nao|thoi tiet", "ban da nghe gi|tai sao dung", "xin chao|chao"),
    "hi": ("क्या समय है|अब कितने बजे", "कौन सा दिन|आज कौन सा दिन", "मौसम कैसा है|मौसम", "क्या सुना|क्यों रुके", "नमस्ते|हैलो"),
    "bn": ("কয়টা বাজে|সময় কত", "কোন দিন|আজ কোন দিন", "আবহাওয়া কেমন|আবহাওয়া", "কী শুনলে|কেন থামলে", "নমস্কার|হ্যালো"),
    "gu": ("કેટલા વાગ્યા|સમય શું છે", "કયો દિવસ|આજે કયો દિવસ", "હવામાન કેવું|હવામાન", "શું સાંભળ્યું|શા માટે અટક્યા", "નમસ્તે|હેલો"),
    "kn": ("ಎಷ್ಟು ಗಂಟೆ|ಸಮಯ ಎಷ್ಟು", "ಯಾವ ದಿನ|ಇಂದು ಯಾವ ದಿನ", "ಹವಾಮಾನ ಹೇಗಿದೆ|ಹವಾಮಾನ", "ಏನು ಕೇಳಿದೆ|ಯಾಕೆ ನಿಂತೆ", "ನಮಸ್ಕಾರ|ಹಲೋ"),
    "ml": ("എന്ത് സമയം|ഇപ്പോൾ എത്ര മണി", "ഏത് ദിവസം|ഇന്ന് ഏത് ദിവസം", "കാലാവസ്ഥ എങ്ങനെ|കാലാവസ്ഥ", "എന്ത് കേട്ടു|എന്തിന് നിർത്തി", "നമസ്കാരം|ഹലോ"),
    "mr": ("किती वाजले|वेळ काय आहे", "कोणता दिवस|आज कोणता दिवस", "हवामान कसे आहे|हवामान", "काय ऐकले|का थांबलात", "नमस्कार|हॅलो"),
    "ta": ("என்ன நேரம்|இப்போது எத்தனை மணி", "என்ன நாள்|இன்று என்ன நாள்", "வானிலை எப்படி|வானிலை", "என்ன கேட்டாய்|ஏன் நின்றாய்", "வணக்கம்|ஹலோ"),
    "te": ("ఎంత సమయం|ఇప్పుడు ఎన్ని గంటలు", "ఏ రోజు|ఈరోజు ఏ రోజు", "వాతావరణం ఎలా ఉంది|వాతావరణం", "ఏమి విన్నావు|ఎందుకు ఆపావు", "నమస్కారం|హలో"),
    "pa": ("ਕਿੰਨੇ ਵਜੇ|ਸਮਾਂ ਕੀ ਹੈ", "ਕਿਹੜਾ ਦਿਨ|ਅੱਜ ਕਿਹੜਾ ਦਿਨ", "ਮੌਸਮ ਕਿਵੇਂ ਹੈ|ਮੌਸਮ", "ਕੀ ਸੁਣਿਆ|ਕਿਉਂ ਰੁਕੇ", "ਸਤ ਸ੍ਰੀ ਅਕਾਲ|ਹੈਲੋ"),
    "ne": ("कति बज्यो|समय कति भयो", "कुन दिन|आज कुन दिन", "मौसम कस्तो छ|मौसम", "के सुन्यो|किन रोकियो", "नमस्ते|हेलो"),
    "hy": ("ժամը քանիսն է|ինչ ժամ է", "ինչ օր է|այսօր ինչ օր է", "ինչպիսին է եղանակը|եղանակ", "ինչ լսեցիր|ինչու կանգ առար", "բարև|ողջույն"),
    "ka": ("რომელი საათია|რა დროა", "რომელი დღეა|დღეს რომელი დღეა", "როგორია ამინდი|ამინდი", "რა გაიგე|რატომ გაჩერდი", "გამარჯობა|სალამი"),
    "mn": ("хэдэн цаг болж байна|цаг хэд вэ", "ямар өдөр|өнөөдөр ямар өдөр", "цаг агаар ямар байна|цаг агаар", "юу сонсов|яагаад зогсов", "сайн байна уу|сайн уу"),
}


def _split(cell: str) -> list[str]:
    return [part.strip() for part in cell.split("|") if part.strip()]


EXTRA = {
    "de-CH": {
        "teach": ["nenn das ", "nenne das ", "das heisst "],
        "explain": ["was hesch ghoert", "was hast du gehoert", "warum hast du gestoppt"],
        "undo": ["rueckgaengig", "nimm das zurueck"],
        "clock": ["wie spaet", "wie viel uhr", "d uhrzit"],
        "date": ["welches datum", "der wievielte"],
        "weekday": ["welcher tag", "welcher wochentag"],
        "weather": ["wie isch s wetter", "wetterbericht", "wetter"],
        "clock_skip": ["timer", "wecker"],
        "date_ok": "Heute ist {weekday}, der {date}.",
        "weekday_ok": "Heute ist {weekday}.",
        "today_ok": "Heute ist {weekday}, der {date}.",
    },
    "de-AT": {
        "teach": ["nenn das ", "nenne das ", "das heisst "],
        "explain": ["was hast du gehoert", "was hast du verstanden", "warum hast du gestoppt"],
        "undo": ["rueckgaengig", "nimm das zurueck"],
        "clock": ["wie spaet", "wie viel uhr", "die uhrzeit"],
        "date": ["welches datum", "der wievielte"],
        "weekday": ["welcher tag", "welcher wochentag"],
        "weather": ["wie ist das wetter", "wetterbericht", "wetter"],
        "clock_skip": ["timer", "wecker"],
        "date_ok": "Heute ist {weekday}, der {date}.",
        "weekday_ok": "Heute ist {weekday}.",
        "today_ok": "Heute ist {weekday}, der {date}.",
    },
    "en-GB": {
        "teach": ["call this ", "call it ", "name this "],
        "explain": ["what did you hear", "what did you understand", "why did you stop"],
        "undo": ["undo that", "undo", "take that back"],
        "clock": ["what time", "whats the time"],
        "date": ["what date", "whats the date", "what is the date"],
        "weekday": ["what day is", "what day of the week"],
        "today": ["whats today", "what is today"],
        "weather": ["whats the weather", "what is the weather", "weather forecast"],
        "clock_skip": ["timer", "alarm"],
    },
    "nl": {
        "date": ["welke datum", "wat is de datum"],
        "weekday": ["welke dag", "welke weekdag", "welke dag is het"],
        "today": ["wat is vandaag"],
        "date_ok": "Vandaag is {weekday}, {date}.",
        "weekday_ok": "Vandaag is {weekday}.",
        "today_ok": "Vandaag is {weekday}, {date}.",
    },
    "fr": {
        "date": ["quelle date", "c est quelle date"],
        "weekday": ["quel jour", "quel jour sommes", "quel jour est il"],
        "today": ["on est quel jour"],
        "date_ok": "Nous sommes {weekday}, {date}.",
        "weekday_ok": "Nous sommes {weekday}.",
        "today_ok": "Nous sommes {weekday}, {date}.",
    },
    "it": {
        "date": ["che data", "qual e la data"],
        "weekday": ["che giorno", "che giorno e", "che giorno e oggi"],
        "today": ["che giorno e oggi"],
        "date_ok": "Oggi e {weekday}, {date}.",
        "weekday_ok": "Oggi e {weekday}.",
        "today_ok": "Oggi e {weekday}, {date}.",
    },
}

def household_for(code: str) -> dict:
    out: dict = {}
    row = _ROW.get(code) or _ROW.get(code.split("-")[0])
    if row:
        clock, weekday, weather, explain, greet = row
        out = {
            "clock": _split(clock),
            "weekday": _split(weekday),
            "weather": _split(weather),
            "explain": _split(explain),
            "greet": _split(greet),
        }
    for key, val in (EXTRA.get(code) or {}).items():
        if isinstance(val, list):
            out[key] = list(dict.fromkeys(list(val) + list(out.get(key) or [])))
        else:
            out[key] = val
    return out
