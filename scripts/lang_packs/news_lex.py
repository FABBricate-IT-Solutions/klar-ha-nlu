"""News briefing words and spoken lines for generated Assist packs.

de and en stay hand-written. Every other pack code must be here so the
generator cannot fall back to an off-verb or the "I did not understand" line.
Tokens are already folded the way speech is tokenized.
"""

from __future__ import annotations


def _row(news: list[str], intro: str, nudge: str, done: str, dismiss: list[str] | None = None) -> dict:
    return {
        "news": news,
        "news_dismiss": dismiss or ["stop", "nein", "no"],
        "news_intro": intro,
        "news_nudge": nudge,
        "news_done": done,
    }


_DE = ["nachrichten", "nachricht", "news", "meldungen", "meldung", "schlagzeilen", "headlines", "neuigkeiten", "neuigkeit", "neues", "briefing", "flash"]
_EN = ["news", "headlines", "headline", "bulletin", "bulletins", "briefing", "flash", "what new", "whats new"]

NEWS: dict[str, dict] = {
    "fr": _row(["actu", "infos", "actualites", "nouvelles", "briefing"], "Je regarde les dernieres actualites.", "Tu veux en savoir plus sur l'une d'elles ?", "D'accord.", ["stop", "assez", "non"]),
    "nl": _row(["nieuws", "berichten", "koppen", "briefing"], "Ik zoek het laatste nieuws even op.", "Wil je over een bericht meer horen?", "Ok.", ["stop", "genoeg", "nee"]),
    "es": _row(["noticias", "noticia", "actualidad", "briefing"], "Miro las noticias de ahora.", "Quieres saber mas de alguna?", "De acuerdo.", ["stop", "basta", "no"]),
    "it": _row(["notizie", "notizia", "attualita", "briefing"], "Cerco le notizie di adesso.", "Vuoi saperne di piu su una?", "Va bene.", ["stop", "basta", "no"]),
    "pt": _row(["noticias", "noticia", "atualidades", "briefing"], "Vou ver as noticias de agora.", "Queres saber mais de alguma?", "Esta bem.", ["stop", "chega", "nao"]),
    "ca": _row(["noticies", "noticia", "actualitat", "briefing"], "Miro les noticies d'ara.", "En vols saber mes d'alguna?", "D'acord.", ["stop", "prou", "no"]),
    "ro": _row(["stiri", "stire", "actualitati", "briefing"], "Caut stirile de acum.", "Vrei mai multe despre una?", "Bine.", ["stop", "ajunge", "nu"]),
    "da": _row(["nyheder", "nyhed", "overskrifter", "briefing"], "Jeg finder de seneste nyheder.", "Vil du hore mere om en af dem?", "Ok.", ["stop", "nok", "nej"]),
    "nb": _row(["nyheter", "nyhet", "overskrifter", "briefing"], "Jeg henter de siste nyhetene.", "Vil du hore mer om en av dem?", "Ok.", ["stop", "nok", "nei"]),
    "sv": _row(["nyheter", "nyhet", "rubriker", "briefing"], "Jag hamtar de senaste nyheterna.", "Vill du hora mer om nagon?", "Ok.", ["stop", "nog", "nej"]),
    "fi": _row(["uutiset", "uutinen", "briefing"], "Haen tuoreet uutiset.", "Haluatko kuulla yhdesta lisaa?", "Selva.", ["stop", "riittaa", "ei"]),
    "de-CH": _row(_DE, "Ich lueg kurz d Nachrichten noche.", "Wotsch zu einere Meldig meh ghoere?", "Passt.", ["stop", "stopp", "reicht", "genug", "nei", "nein"]),
    "de-AT": _row(_DE, "Passt, ich schau kurz die aktuellen Nachrichten an.", "Magst zu einer Meldung mehr hoeren?", "Passt.", ["stop", "stopp", "reicht", "genug", "nein"]),
    "en-GB": _row(_EN, "Sure, I'll look up the latest news.", "Would you like to hear more about any of those?", "All right.", ["stop", "enough", "no"]),
    "pt-BR": _row(["noticias", "noticia", "atualidades", "briefing"], "Vou ver as noticias de agora.", "Quer saber mais de alguma?", "Certo.", ["stop", "chega", "nao"]),
    "af": _row(["nuus", "nuusberig", "opskrifte", "briefing"], "Ek soek gou die nuutste nuus.", "Wil jy meer van een hoor?", "Reg so.", ["stop", "genoeg", "nee"]),
    "cs": _row(["zpravy", "zprava", "aktuality", "briefing"], "Podivam se na aktualni zpravy.", "Chces o nejake vedet vic?", "Dobre.", ["stop", "dost", "ne"]),
    "sk": _row(["spravy", "sprava", "aktuality", "briefing"], "Pozriem aktualne spravy.", "Chces o nejakej viac?", "Dobre.", ["stop", "dost", "nie"]),
    "pl": _row(["wiadomosci", "wiadomosc", "aktualnosci", "briefing"], "Sprawdzam aktualne wiadomosci.", "Chcesz wiecej o ktorejs?", "Dobrze.", ["stop", "wystarczy", "nie"]),
    "hu": _row(["hirek", "hir", "briefing"], "Megnezem a friss hireket.", "Akarsz valamelyikrol tobbet?", "Rendben.", ["stop", "eleg", "nem"]),
    "hr": _row(["vijesti", "vijest", "briefing"], "Gledam aktualne vijesti.", "Zelis li vise o jednoj?", "U redu.", ["stop", "dosta", "ne"]),
    "sl": _row(["novice", "novica", "briefing"], "Pogledam aktualne novice.", "Hoces vec o kateri?", "V redu.", ["stop", "dovolj", "ne"]),
    "bg": _row(["новини", "новина", "briefing"], "Поглеждам актуалните новини.", "Искаш ли повече за някоя?", "Добре.", ["stop", "стига", "не"]),
    "el": _row(["ειδήσεις", "ειδησεις", "briefing"], "Κοιτάζω τα νέα τώρα.", "Θέλεις περισσότερα για κάποιο;", "Εντάξει.", ["stop", "αρκετά", "όχι"]),
    "sr": _row(["вести", "новости", "briefing"], "Гледам актуелне вести.", "Желиш ли више о једној?", "У реду.", ["stop", "доста", "не"]),
    "sr-Latn": _row(["vesti", "novosti", "briefing"], "Gledam aktuelne vesti.", "Zelis li vise o jednoj?", "U redu.", ["stop", "dosta", "ne"]),
    "uk": _row(["новини", "новина", "briefing"], "Дивлюсь актуальні новини.", "Хочеш більше про одну?", "Добре.", ["stop", "досить", "ні"]),
    "zh-CN": _row(["新闻", "消息", "头条"], "我去看一下最新消息。", "想再听其中一条吗？", "好。", ["stop", "够了", "不用"]),
    "zh-TW": _row(["新聞", "消息", "頭條"], "我看一下最新消息。", "想再聽其中一則嗎？", "好。", ["stop", "夠了", "不用"]),
    "zh-HK": _row(["新聞", "消息", "頭條"], "我睇下最新消息。", "想再聽其中一則嗎？", "好。", ["stop", "夠了", "不用"]),
    "ar": _row(["أخبار", "اخبار", "خبر"], "أبحث عن آخر الأخبار.", "تريد المزيد عن خبر؟", "حسنا.", ["stop", "يكفي", "لا"]),
    "he": _row(["חדשות", "ידיעות"], "אני בודק את החדשות.", "לשמוע עוד על אחת?", "בסדר.", ["stop", "מספיק", "לא"]),
    "fa": _row(["اخبار", "خبر"], "خبرهای تازه را نگاه می‌کنم.", "یکی را بیشتر بشنوی؟", "باشد.", ["stop", "کافیست", "نه"]),
    "ur": _row(["خبریں", "خبر", "اخبار"], "تازہ خبریں دیکھتا ہوں۔", "کسی خبر کے بارے میں اور؟", "ٹھیک ہے۔", ["stop", "بس", "نہیں"]),
    "tr": _row(["haberler", "haber", "gundem", "briefing"], "Guncel haberlere bakiyorum.", "Birinden daha fazla ister misin?", "Tamam.", ["stop", "yeter", "hayir"]),
    "th": _row(["ข่าว", "ข่าวสาร"], "ขอดูข่าวล่าสุดก่อน", "อยากฟังข่าวไหนเพิ่มไหม", "ได้", ["stop", "พอ", "ไม่"]),
    "ko": _row(["뉴스", "소식", "헤드라인"], "최신 뉴스를 찾아볼게.", "그중 더 들을 소식 있어?", "알겠어.", ["stop", "그만", "아니"]),
    "ja": _row(["ニュース", "報道"], "最新のニュースを見るね。", "どれか詳しく聞く？", "わかった。", ["stop", "十分", "いいえ"]),
    "cy": _row(["newyddion", "briefing"], "Edrychaf ar y newyddion diweddaraf.", "Eisiau mwy am un?", "Iawn.", ["stop", "digon", "na"]),
    "et": _row(["uudised", "uudis", "briefing"], "Vaatan varskeid uudiseid.", "Tahad uhe kohta rohkem?", "Selge.", ["stop", "aitab", "ei"]),
    "eu": _row(["albisteak", "albiste", "berriak", "briefing"], "Albiste berriak begiratzen ditut.", "Bati buruz gehiago nahi?", "Ados.", ["stop", "nahikoa", "ez"]),
    "ga": _row(["nuacht", "nuachta", "briefing"], "Feachfaidh me ar an nuacht.", "An dteastaionn nios mo faoi cheann?", "Ceart go leor.", ["stop", "leor", "ni"]),
    "gl": _row(["novas", "noticias", "briefing"], "Miro as novas de agora.", "Queres saber mais dunha?", "De acordo.", ["stop", "abonda", "non"]),
    "is": _row(["frettir", "frett", "briefing"], "Eg skoda nyjustu frettir.", "Viltu heyra meira um eina?", "Allt i lagi.", ["stop", "nog", "nei"]),
    "lb": _row(["neiegkeeten", "noriichten", "news", "briefing"], "Ech kucken d'Neiegkeeten.", "Wells du mei iwwer eng?", "Ok.", ["stop", "genuch", "neen"]),
    "kw": _row(["nowodhow", "nowodhow", "briefing"], "My a wra mires orth an nowodhow.", "Mynnes moy a unn?", "Da.", ["stop", "lowr", "na"]),
    "lt": _row(["naujienos", "naujiena", "zinios", "briefing"], "Ziuriu naujienas.", "Nori daugiau apie viena?", "Gerai.", ["stop", "gana", "ne"]),
    "lv": _row(["zinas", "jaunumi", "briefing"], "Apskatu jaunakas zinas.", "Gribi vairak par kadu?", "Labi.", ["stop", "diezgan", "ne"]),
    "id": _row(["berita", "kabar", "warta", "briefing"], "Saya cari berita terbaru.", "Mau dengar lebih satu berita?", "Baik.", ["stop", "cukup", "tidak"]),
    "ms": _row(["berita", "khabar", "briefing"], "Saya cari berita terkini.", "Nak dengar lagi satu?", "Baik.", ["stop", "cukup", "tidak"]),
    "sw": _row(["habari", "taarifa", "briefing"], "Ninaangalia habari za sasa.", "Unataka zaidi kuhusu moja?", "Sawa.", ["stop", "basi", "hapana"]),
    "vi": _row(["tintuc", "tin tuc", "briefing"], "Toi xem tin moi.", "Ban muon nghe them mot tin?", "Duoc.", ["stop", "du", "khong"]),
    "hi": _row(["समाचार", "खबर", "खबरें", "news"], "मैं ताज़ा खबरें देखता हूँ।", "किसी खबर के बारे में और?", "ठीक।", ["stop", "बस", "नहीं"]),
    "bn": _row(["সংবাদ", "খবর", "news"], "আমি তাজা খবর দেখছি।", "একটা খবর আর শুনবে?", "ঠিক আছে।", ["stop", "যথেষ্ট", "না"]),
    "gu": _row(["સમાચાર", "ખબર", "news"], "હું તાજા સમાચાર જોઉં છું.", "એક વિશે વધુ?", "બરાબર.", ["stop", "બસ", "ના"]),
    "kn": _row(["ಸುದ್ದಿ", "ವಾರ್ತೆ", "news"], "ಇತ್ತೀಚಿನ ಸುದ್ದಿ ನೋಡುತ್ತೇನೆ.", "ಒಂದರ ಬಗ್ಗೆ ಇನ್ನಷ್ಟು?", "ಸರಿ.", ["stop", "ಸಾಕು", "ಇಲ್ಲ"]),
    "ml": _row(["വാർത്ത", "വാര്ത്ത", "news"], "പുതിയ വാർത്ത നോക്കുന്നു.", "ഒന്നിനെക്കുറിച്ച് കൂടുതൽ?", "ശരി.", ["stop", "മതി", "വേണ്ട"]),
    "mr": _row(["बातम्या", "बातमी", "news"], "ताज्या बातम्या बघतो.", "एकाबद्दल आणखी?", "ठीक.", ["stop", "पुरे", "नाही"]),
    "ta": _row(["செய்தி", "செய்திகள்", "news"], "புதிய செய்தி பார்க்கிறேன்.", "ஒன்றை இன்னும் கேட்கவா?", "சரி.", ["stop", "போதும்", "வேண்டாம்"]),
    "te": _row(["వార్తలు", "వార్త", "news"], "తాజా వార్తలు చూస్తాను.", "ఒకదాని గురించి ఇంకా?", "సరే.", ["stop", "చాలు", "వద్దు"]),
    "pa": _row(["ਖਬਰਾਂ", "ਖਬਰ", "news"], "ਮੈਂ ਤਾਜ਼ਾ ਖ਼ਬਰਾਂ ਵੇਖਦਾ ਹਾਂ।", "ਕਿਸੇ ਇੱਕ ਬਾਰੇ ਹੋਰ?", "ਠੀਕ।", ["stop", "ਬਸ", "ਨਹੀਂ"]),
    "ne": _row(["समाचार", "खबर", "news"], "ताजा समाचार हेर्छु।", "एउटाबारे थप?", "ठीक।", ["stop", "पुग्यो", "होइन"]),
    "hy": _row(["նորություններ", "լուրեր", "լուր"], "Նայում եմ թարմ լուրերը.", "Մեկի մասին ավելին?", "Լավ.", ["stop", "բավական", "ոչ"]),
    "ka": _row(["სიახლეები", "ამბები", "სიახლე"], "ახალ ამბებს ვნახულობ.", "ერთზე მეტი გინდა?", "კარგი.", ["stop", "კმარა", "არა"]),
    "mn": _row(["мэдээ", "мэдээлэл"], "Шинэ мэдээ харж байна.", "Нэгийг нь дэлгэрэнгүй?", "За.", ["stop", "болно", "үгүй"]),
}


def apply_news(code: str, chat: dict) -> None:
    row = NEWS.get(code)
    if row is None:
        raise KeyError(f"news lexicon missing for {code}")
    chat.update(row)
