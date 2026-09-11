"""Speech slots: Slavic, Greek, Hungarian, Ukrainian, Bulgarian."""

from lang_packs.speech_slots import S

EAST = {
    "cs": S(
        "Časovač běží.", "Časovač je vypnutý.", "Časovač je pozastavený.",
        "Co mám přehrát?", "Hudba přesunuta.", "Přidáno do oblíbených.",
        "{target} vysává.", "{target} jede do stanice.", "Vysavač",
        "topení", "klimatizace", "doma", "Ventilátor na {n} procent.",
        jarvis=("Samozřejmě, pane. ", "Hned. ", ""),
    ),
    "sk": S(
        "Časovač beží.", "Časovač je vypnutý.", "Časovač je pozastavený.",
        "Čo mám prehrať?", "Hudba presunutá.", "Pridané do obľúbených.",
        "{target} vysáva.", "{target} ide do stanice.", "Vysávač",
        "kúrenie", "klimatizácia", "doma", "Ventilátor na {n} percent.",
        jarvis=("Samozrejme, pán. ", "Hneď. ", ""),
    ),
    "pl": S(
        "Minutnik działa.", "Minutnik wyłączony.", "Minutnik wstrzymany.",
        "Co mam odtworzyć?", "Muzyka przeniesiona.", "Dodano do ulubionych.",
        "{target} odkurza.", "{target} wraca do stacji.", "Odkurzacz",
        "ogrzewanie", "klimatyzacja", "w domu", "Wentylator na {n} procent.",
        jarvis=("Oczywiście, panie. ", "Natychmiast. ", ""),
    ),
    "hu": S(
        "Az időzítő fut.", "Az időzítő leállt.", "Az időzítő szünetel.",
        "Mit játsszak?", "Zene áthelyezve.", "Hozzáadva a kedvencekhez.",
        "{target} porszívózik.", "{target} megy a dokkolóra.", "A porszívó",
        "fűtés", "klíma", "otthon", "Ventilátor {n} százalékon.",
        jarvis=("Persze, uram. ", "Azonnal. ", ""),
    ),
    "hr": S(
        "Timer radi.", "Timer je ugašen.", "Timer je pauziran.",
        "Što da pustim?", "Glazba je premještena.", "Dodano u favorite.",
        "{target} usisava.", "{target} ide na stanicu.", "Usisavač",
        "grijanje", "klima", "kod kuće", "Ventilator na {n} posto.",
        jarvis=("Naravno, gospodine. ", "Odmah. ", ""),
    ),
    "sl": S(
        "Časovnik teče.", "Časovnik je ugasnjen.", "Časovnik je pavziran.",
        "Kaj naj predvajam?", "Glasba prestavljena.", "Dodano med priljubljene.",
        "{target} sesa.", "{target} gre na postajo.", "Sesalnik",
        "ogrevanje", "klima", "doma", "Ventilator na {n} odstotkov.",
        jarvis=("Seveda, gospod. ", "Takoj. ", ""),
    ),
    "bg": S(
        "Таймерът работи.", "Таймерът е спрян.", "Таймерът е на пауза.",
        "Какво да пусна?", "Музиката е преместена.", "Добавено в любими.",
        "{target} почиства.", "{target} се връща към станцията.", "Прахосмукачката",
        "отопление", "климатик", "у дома", "Вентилатор на {n} процента.",
        jarvis=("Разбира се, сър. ", "Веднага. ", ""),
    ),
    "el": S(
        "Το χρονόμετρο τρέχει.", "Το χρονόμετρο σταμάτησε.", "Το χρονόμετρο είναι σε παύση.",
        "Τι να παίξω;", "Η μουσική μεταφέρθηκε.", "Προστέθηκε στα αγαπημένα.",
        "{target} σκουπίζει.", "{target} πάει στη βάση.", "Η σκούπα",
        "θέρμανση", "κλιματισμός", "στο σπίτι", "Ανεμιστήρας στο {n} τοις εκατό.",
        jarvis=("Βεβαίως, κύριε. ", "Αμέσως. ", ""),
    ),
    "sr": S(
        "Тајмер ради.", "Тајмер је искључен.", "Тајмер је паузиран.",
        "Шта да пустим?", "Музика је премештена.", "Додато у омиљене.",
        "{target} усисава.", "{target} иде на станицу.", "Усисивач",
        "грејање", "клима", "код куће", "Вентилатор на {n} посто.",
        jarvis=("Наравно, господине. ", "Одмах. ", ""),
    ),
    "sr-Latn": S(
        "Tajmer radi.", "Tajmer je isključen.", "Tajmer je pauziran.",
        "Šta da pustim?", "Muzika je premeštena.", "Dodato u omiljene.",
        "{target} usisava.", "{target} ide na stanicu.", "Usisivač",
        "grejanje", "klima", "kod kuće", "Ventilator na {n} posto.",
        jarvis=("Naravno, gospodine. ", "Odmah. ", ""),
    ),
    "uk": S(
        "Таймер працює.", "Таймер вимкнено.", "Таймер на паузі.",
        "Що відтворити?", "Музику переміщено.", "Додано в улюблене.",
        "{target} прибирає.", "{target} їде на станцію.", "Пилосос",
        "опалення", "кондиціонер", "вдома", "Вентилятор на {n} відсотків.",
        jarvis=("Звичайно, сер. ", "Зараз. ", ""),
    ),
}
