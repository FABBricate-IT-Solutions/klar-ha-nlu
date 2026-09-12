//! Multilingual weather stems. Token match for ASCII; contains for other scripts.

#[rustfmt::skip]
pub(super) const RAIN: &[&str] = &[
    "regen", "regnen", "regnerisch", "niederschlag", "rain", "raining", "rainy", "rainfall", "pluie", "pleut", "pleuvoir", "regenen",
    "regent", "lluvia", "llueve", "llover", "pioggia", "piove", "piovere", "chuva", "chove", "chover", "pluja", "plou", "ploaie", "ploua",
    "regn", "regner", "regne", "regnar", "regna", "sataa", "sadetta", "prset", "dazd", "prsat", "deszcz", "esik", "dezuje", "дождь", "дъжд",
    "вали", "βροχη", "βρεχει", "βροχή", "киша", "дощ", "дощить", "下雨", "会下雨", "會下雨", "下雨吗", "下雨嗎", "雨が", "雨降", "비가", "مطر", "تمطر", "גשם",
    "باران", "بارش", "yagmur", "yagacak", "yağmur", "ฝน", "glaw", "vihm", "sajab", "euri", "baisteach", "choiva", "rigning", "rignir",
    "lietus", "lyja", "hujan", "mvua", "बारिश", "वर्षा", "বৃষ্টি", "ಮಳೆ", "മഴ", "पाऊस", "மழை", "వర్షం", "ਮੀਂਹ", "વરસાદ", "անձրև", "წვიმა",
    "бороо", "deigh", "bwrw glaw", "līst", "reën", "prší", "kiša", "dež", "mưa", "hujan"
];

#[rustfmt::skip]
pub(super) const UMBRELLA: &[&str] = &[
    "regenschirm", "umbrella", "brolly", "parapluie", "paraplu", "paraguas", "ombrello", "guarda-chuva", "guarda chuva", "paraigua",
    "umbrela", "paraply", "sateenvarjo", "sambreel", "destnik", "dazdnik", "esernyo", "kisetobran", "deznik", "чадър", "ομπρελα", "кишобран",
    "парасолька", "парасоля", "雨伞", "雨傘", "傘", "우산", "مظلة", "מטריה", "چتر", "چھتری", "semsiye", "şemsiye", "ร่ม", "vihmavari", "aterki",
    "paraugas", "regnhlif", "regeschierm", "sketis", "lietussargs", "payung", "mwavuli", "छाता", "ছাতা", "छत्री", "குடை", "గొడుగు", "ਛਤਰੀ",
    "છત્રી", "ಕೊಡೆ", "കുട", "հովանոց", "ქოლგა", "шүхэр", "ymbare", "scath", "cysgod"
];

#[rustfmt::skip]
pub(super) const AMBIG_UMBRELLA: &[&str] = &[
    "parasol", "schirm"
];

#[rustfmt::skip]
pub(super) const NEED: &[&str] = &[
    "brauche", "brauchen", "brauch", "nimm", "mitnehmen", "need", "take", "nehmen", "besoin", "necesito", "preciso", "precisa", "trzeba",
    "potrzebuje", "treba", "χρειάζομαι", "нужен", "надо", "لازم", "צריך", "いる", "필요", "cần", "perlu", "butuh", "nodig", "behoefte",
    "bisogna"
];

#[rustfmt::skip]
pub(super) const WEATHER: &[&str] = &[
    "wetter", "wetterbericht", "weather", "forecast", "meteo", "weerbericht", "het weer", "el tiempo", "que tiempo", "il tempo", "che tempo",
    "o tempo", "pronostico", "previsioni", "previsao", "el temps", "vremea", "vejret", "vaeret", "vadret", "pocasi", "pocasie", "pogoda",
    "idojaras", "vrijeme", "καιρος", "погода", "天气", "天氣", "天気予報", "天気", "날씨", "طقس", "موسم", "hava durumu", "tywydd", "eguraldi", "aimsir",
    "vedur", "cuaca", "thoi tiet", "मौसम", "আবহাওয়া", "हवामान", "வானிலை", "వాతావరణం", "ਮੌਸਮ", "હવામાન", "ಹವಾಮಾನ", "കാലാവസ്ഥ", "եղանակ",
    "ამინდი", "цаг агаар", "laika", "oras", "hali ya hewa", "weersvoorspelling", "weerbericht", "สภาพอากาศ", "هوا"
];

#[rustfmt::skip]
pub(super) const FORECAST: &[&str] = &[
    "vorhersage", "wetterbericht", "forecast", "previsions", "pronostico", "previsioni", "voorspelling", "prognose", "prognoza", "ennuste",
    "预报", "預報", "予報", "예보", "توقعات", "tahmin", "พยากรณ์", "ramalan", "previsao", "előrejelzes"
];

#[rustfmt::skip]
pub(super) const TODAY: &[&str] = &[
    "heute", "huet", "today", "aujourd", "aujourdhui", "vandaag", "hoy", "oggi", "hoje", "avui", "azi", "idag", "i dag", "tanaan", "vandag",
    "dnes", "dneska", "dzis", "danas", "danes", "днес", "σημερα", "данас", "сьогодні", "今天", "今日", "오늘", "اليوم", "היום", "امروز", "bugun",
    "bugün", "วันนี้", "heddiw", "inniu", "hoxe", "hari ini", "hom nay", "आज", "আজ", "આજે", "ಇಂದು", "ഇന്ന്", "இன்று", "ఈరోజు", "ਅੱਜ",
    "այսօր", "დღეს", "өнөөдөр", "gaur", "dzisiaj", "sodien", "tana"
];

#[rustfmt::skip]
pub(super) const TOMORROW: &[&str] = &[
    "morgen", "tomorrow", "demain", "manana", "domani", "amanha", "dema", "imorgen", "imorgon", "i morgen", "huomenna", "zitra", "zajtra",
    "jutro", "holnap", "sutra", "jutri", "утре", "αυριο", "сутра", "завтра", "明天", "明日", "내일", "غدا", "מחר", "فردا", "yarin", "yarın",
    "พรุ่งนี้", "yfory", "ngay mai", "कल", "আগামীকাল", "ನಾಳೆ", "നാളെ", "उद्या", "நாளை", "రేపు", "ਕੱਲ੍ਹ", "भोलि", "վաղը", "ხვალ", "маргааш",
    "besok", "esok", "kesho"
];

#[rustfmt::skip]
pub(super) const FUTURE: &[&str] = &[
    "wird", "werden", "going to", "gonna", "will be", "va t il", "va-t-il", "va il", "sera t il", "wordt", "blir", "bude", "bedzie", "lesz",
    "будет", "会怎样", "なる", "될까", "akan"
];

#[rustfmt::skip]
pub(super) const WEEKEND: &[&str] = &[
    "wochenende", "weekend", "week-end", "fin de semana", "fine settimana", "fim de semana", "vikend", "vikaend", "savaitgal", "helgen",
    "viikonloppu", "σαββατοκυριακο", "выходн", "周末", "週末", "주말", "نهاية الأسبوع", "סוף שבוע", "akhir pekan"
];

#[rustfmt::skip]
pub(super) const MORNING: &[&str] = &[
    "vormittag", "frueh", "früh", "morning", "matin", "ochtend", "mattina", "manha", "dopoledne", "rano", "reggel", "πρωι", "утро", "早上",
    "午前", "아침", "صباح", "בוקר", "sabah", "เช้า", "buoi sang", "pagi", "vanochtend", "vanmorgen"
];

#[rustfmt::skip]
pub(super) const AFTERNOON: &[&str] = &[
    "nachmittag", "afternoon", "apres midi", "apres-midi", "middag", "tarde", "pomeriggio", "odpoledne", "popoludnie", "delutan", "μεσημερι",
    "день", "下午", "午後", "오후", "بعد الظهر", "öğleden", "บ่าย", "chieu", "sore"
];

#[rustfmt::skip]
pub(super) const EVENING: &[&str] = &[
    "abend", "evening", "soir", "avond", "noche", "noite", "vecer", "wieczor", "βραδυ", "вечер", "晚上", "저녁", "مساء", "ערב", "aksam", "เย็น",
    "malam"
];

#[rustfmt::skip]
pub(super) const NIGHT: &[&str] = &[
    "nacht", "night", "nuit", "notte", "noc", "νυχτα", "ночь", "夜里", "夜中", "밤", "ليل", "לילה", "gece"
];

#[rustfmt::skip]
pub(super) const GREET_STRIP: &[&str] = &[
    "guten morgen", "gueten morgen", "guete morge", "good morning", "goedemorgen", "goedenmorgen", "goede morgen", "god morgen",
    "god morgon", "godmorgon", "bonjour", "buenos dias", "buongiorno", "bom dia", "goeie more", "goeiemore", "dobre rano", "dzien dobry",
    "jo reggelt", "heute morgen", "this morning", "ce matin", "vanochtend", "esta manana", "stamattina", "vanmorgen", "kalimera",
    "καλημερα"
];

#[rustfmt::skip]
pub(super) const DAYS: &[(&str, u8)] = &[
    ("montag", 0), ("monday", 0), ("lundi", 0), ("maandag", 0), ("lunes", 0), ("lunedi", 0), ("segunda feira", 0), ("pondeli", 0),
    ("pondelok", 0), ("poniedzialek", 0), ("hetfo", 0), ("ponedjeljak", 0), ("понеделник", 0), ("δευτερα", 0), ("понедельник", 0), ("周一", 0),
    ("月曜", 0), ("월요일", 0), ("الاثنين", 0), ("יום שני", 0), ("pazartesi", 0), ("maanantai", 0), ("mandag", 0), ("luni", 0), ("hétfő", 0),
    ("dienstag", 1), ("tuesday", 1), ("mardi", 1), ("dinsdag", 1), ("martes", 1), ("martedi", 1), ("terca feira", 1), ("utery", 1),
    ("utorok", 1), ("wtorek", 1), ("kedd", 1), ("utorak", 1), ("вторник", 1), ("τριτη", 1), ("周二", 1), ("火曜", 1), ("화요일", 1),
    ("الثلاثاء", 1), ("tiistai", 1), ("tisdag", 1), ("tirsdag", 1), ("mittwoch", 2), ("wednesday", 2), ("mercredi", 2), ("woensdag", 2),
    ("miercoles", 2), ("mercoledi", 2), ("quarta feira", 2), ("streda", 2), ("sroda", 2), ("szerda", 2), ("srijeda", 2), ("сряда", 2),
    ("τεταρτη", 2), ("среда", 2), ("周三", 2), ("水曜", 2), ("수요일", 2), ("الاربعاء", 2), ("carsamba", 2), ("keskiviikko", 2), ("onsdag", 2),
    ("donnerstag", 3), ("thursday", 3), ("jeudi", 3), ("donderdag", 3), ("jueves", 3), ("giovedi", 3), ("quinta feira", 3), ("ctvrtek", 3),
    ("stvrtok", 3), ("czwartek", 3), ("csutortok", 3), ("cetvrtak", 3), ("четвъртък", 3), ("πεμπτη", 3), ("четверг", 3), ("周四", 3),
    ("木曜", 3), ("목요일", 3), ("الخميس", 3), ("persembe", 3), ("torstai", 3), ("torsdag", 3), ("freitag", 4), ("friday", 4), ("vendredi", 4),
    ("vrijdag", 4), ("viernes", 4), ("venerdi", 4), ("sexta feira", 4), ("patek", 4), ("piatok", 4), ("piatek", 4), ("pentek", 4),
    ("petak", 4), ("петък", 4), ("παρασκευη", 4), ("пятница", 4), ("周五", 4), ("金曜", 4), ("금요일", 4), ("الجمعة", 4), ("cuma", 4),
    ("perjantai", 4), ("fredag", 4), ("vineri", 4), ("samstag", 5), ("sonnabend", 5), ("saturday", 5), ("samedi", 5), ("zaterdag", 5),
    ("sabado", 5), ("sabato", 5), ("sobota", 5), ("szombat", 5), ("subota", 5), ("събота", 5), ("σαββατο", 5), ("суббота", 5), ("周六", 5),
    ("土曜", 5), ("토요일", 5), ("السبت", 5), ("cumartesi", 5), ("lauantai", 5), ("lordag", 5), ("sambata", 5), ("sonntag", 6), ("sunday", 6),
    ("dimanche", 6), ("zondag", 6), ("domingo", 6), ("domenica", 6), ("nedele", 6), ("nedela", 6), ("niedziela", 6), ("vasarnap", 6),
    ("nedjelja", 6), ("неделя", 6), ("κυριακη", 6), ("воскресенье", 6), ("周日", 6), ("日曜", 6), ("일요일", 6), ("الاحد", 6), ("pazar", 6),
    ("sunnuntai", 6), ("sondag", 6), ("duminica", 6), ("jumatatu", 0), ("jumanne", 1), ("jumatano", 2), ("alhamisi", 3), ("ijumaa", 4),
    ("jumamosi", 5), ("jumapili", 6), ("astelehena", 0), ("asteartea", 1), ("asteazkena", 2), ("osteguna", 3), ("ostirala", 4),
    ("larunbata", 5), ("igandea", 6)
];
