//! Localized weather frames, day names, units, and condition words.

use crate::types::UnitSystem;

/// now, today, tomorrow, rain_yes, rain_no, umbrella_yes, umbrella_no
#[rustfmt::skip]
const FRAMES: &[(&str, [&str; 7])] = &[
    ("af", ["{c}, {t} {u}.", "Vandag {c}, {t} {u}.", "Môre {c}, {t} {u}.", "Ja, reën word verwag.", "Nee, vandag geen reën.", "Ja, vat 'n sambreel.", "Nee, jy het nie 'n sambreel nodig."]),
    ("ar", ["{c}، {t} {u}.", "اليوم {c}، {t} {u}.", "غدا {c}، {t} {u}.", "نعم، يتوقع مطر.", "لا، لا مطر اليوم.", "نعم، خذ مظلة.", "لا، لست بحاجة إلى مظلة."]),
    ("bg", ["{c}, {t} {u}.", "Днес {c}, {t} {u}.", "Утре {c}, {t} {u}.", "Да, очаква се дъжд.", "Не, днес няма дъжд.", "Да, вземи чадър.", "Не, чадър не ти трябва."]),
    ("bn", ["{c}, {t} {u}.", "আজ {c}, {t} {u}.", "আগামীকাল {c}, {t} {u}.", "হ্যাঁ, বৃষ্টি হবে.", "না, আজ বৃষ্টি নেই.", "হ্যাঁ, ছাতা নাও.", "না, ছাতার দরকার নেই."]),
    ("ca", ["{c}, {t} {u}.", "Avui {c}, {t} {u}.", "Demà {c}, {t} {u}.", "Sí, es preveu pluja.", "No, avui no plou.", "Sí, agafa un paraigua.", "No, no et cal paraigua."]),
    ("cs", ["{c}, {t} {u}.", "Dnes {c}, {t} {u}.", "Zítra {c}, {t} {u}.", "Ano, je hlášen déšť.", "Ne, dnes neprší.", "Ano, vezmi si deštník.", "Ne, deštník nepotřebuješ."]),
    ("cy", ["{c}, {t} {u}.", "Heddiw {c}, {t} {u}.", "Yfory {c}, {t} {u}.", "Ie, disgwylir glaw.", "Na, dim glaw heddiw.", "Ie, cymer ymbarél.", "Na, does dim angen ymbarél."]),
    ("da", ["{c}, {t} {u}.", "I dag {c}, {t} {u}.", "I morgen {c}, {t} {u}.", "Ja, der er varslet regn.", "Nej, ingen regn i dag.", "Ja, tag en paraply med.", "Nej, du behøver ingen paraply."]),
    ("de", ["{c}, {t} {u}.", "Heute {c}, {t} {u}.", "Morgen {c}, {t} {u}.", "Ja, Regen ist gemeldet.", "Nein, heute kein Regen.", "Ja, nimm einen Schirm mit.", "Nein, du brauchst keinen Schirm."]),
    ("de-AT", ["{c}, {t} {u}.", "Heute {c}, {t} {u}.", "Morgen {c}, {t} {u}.", "Ja, Regen is gmeldet.", "Nein, heute kein Regen.", "Ja, nimm an Schirm mit.", "Nein, du brauchst keinen Schirm."]),
    ("de-CH", ["{c}, {t} {u}.", "Hüt {c}, {t} {u}.", "Morn {c}, {t} {u}.", "Ja, Regen isch gmeldet.", "Nei, hüt kein Regen.", "Ja, nimm en Schirm mit.", "Nei, du bruchsch kein Schirm."]),
    ("el", ["{c}, {t} {u}.", "Σήμερα {c}, {t} {u}.", "Αύριο {c}, {t} {u}.", "Ναι, αναμένεται βροχή.", "Όχι, σήμερα δεν βρέχει.", "Ναι, πάρε ομπρέλα.", "Όχι, δεν χρειάζεσαι ομπρέλα."]),
    ("en", ["{c}, {t} {u}.", "Today {c}, {t} {u}.", "Tomorrow {c}, {t} {u}.", "Yes, rain is expected.", "No rain today.", "Yes, take an umbrella.", "No, you do not need an umbrella."]),
    ("en-GB", ["{c}, {t} {u}.", "Today {c}, {t} {u}.", "Tomorrow {c}, {t} {u}.", "Yes, rain is expected.", "No rain today.", "Yes, take an umbrella.", "No, you do not need an umbrella."]),
    ("es", ["{c}, {t} {u}.", "Hoy {c}, {t} {u}.", "Mañana {c}, {t} {u}.", "Sí, se espera lluvia.", "No, hoy no llueve.", "Sí, lleva un paraguas.", "No, no necesitas paraguas."]),
    ("et", ["{c}, {t} {u}.", "Täna {c}, {t} {u}.", "Homme {c}, {t} {u}.", "Jah, vihma on oodata.", "Ei, täna vihma pole.", "Jah, võta vihmavari.", "Ei, vihmavarju pole vaja."]),
    ("eu", ["{c}, {t} {u}.", "Gaur {c}, {t} {u}.", "Bihar {c}, {t} {u}.", "Bai, euria espero da.", "Ez, gaur ez du euririk.", "Bai, hartu aterkia.", "Ez, ez duzu aterkirik behar."]),
    ("fa", ["{c}، {t} {u}.", "امروز {c}، {t} {u}.", "فردا {c}، {t} {u}.", "بله، باران پیش‌بینی شده.", "خیر، امروز باران نیست.", "بله، چتر ببر.", "خیر، به چتر نیاز نیست."]),
    ("fi", ["{c}, {t} {u}.", "Tänään {c}, {t} {u}.", "Huomenna {c}, {t} {u}.", "Kyllä, sadetta on tulossa.", "Ei sadetta tänään.", "Kyllä, ota sateenvarjo.", "Ei, et tarvitse sateenvarjoa."]),
    ("fr", ["{c}, {t} {u}.", "Aujourd'hui {c}, {t} {u}.", "Demain {c}, {t} {u}.", "Oui, de la pluie est prévue.", "Non, pas de pluie aujourd'hui.", "Oui, prends un parapluie.", "Non, tu n'as pas besoin de parapluie."]),
    ("ga", ["{c}, {t} {u}.", "Inniu {c}, {t} {u}.", "Amárach {c}, {t} {u}.", "Tá, tá báisteach ag teacht.", "Níl, níl báisteach inniu.", "Tá, tóg scáth.", "Níl, níl scáth de dhíth."]),
    ("gl", ["{c}, {t} {u}.", "Hoxe {c}, {t} {u}.", "Mañá {c}, {t} {u}.", "Si, agardase choiva.", "Non, hoxe non chove.", "Si, leva un paraugas.", "Non, non precisas paraugas."]),
    ("gu", ["{c}, {t} {u}.", "આજે {c}, {t} {u}.", "આવતીકાલે {c}, {t} {u}.", "હા, વરસાદ પડશે.", "ના, આજે વરસાદ નથી.", "હા, છત્રી લો.", "ના, છત્રીની જરૂર નથી."]),
    ("he", ["{c}, {t} {u}.", "היום {c}, {t} {u}.", "מחר {c}, {t} {u}.", "כן, צפוי גשם.", "לא, אין גשם היום.", "כן, קח מטריה.", "לא, אין צורך במטריה."]),
    ("hi", ["{c}, {t} {u}.", "आज {c}, {t} {u}.", "कल {c}, {t} {u}.", "हाँ, बारिश होगी.", "नहीं, आज बारिश नहीं.", "हाँ, छाता ले लो.", "नहीं, छाते की ज़रूरत नहीं."]),
    ("hr", ["{c}, {t} {u}.", "Danas {c}, {t} {u}.", "Sutra {c}, {t} {u}.", "Da, najavljena je kiša.", "Ne, danas nema kiše.", "Da, ponesi kišobran.", "Ne, kišobran ti ne treba."]),
    ("hu", ["{c}, {t} {u}.", "Ma {c}, {t} {u}.", "Holnap {c}, {t} {u}.", "Igen, esőt jeleznek.", "Nem, ma nincs eső.", "Igen, vigyél esernyőt.", "Nem, nincs szükséged esernyőre."]),
    ("hy", ["{c}, {t} {u}.", "Այսօր {c}, {t} {u}.", "Վաղը {c}, {t} {u}.", "Այո, անձրև է սպասվում.", "Ոչ, այսօր անձրև չկա.", "Այո, վերցրու հովանոց.", "Ոչ, հովանոց պետք չէ."]),
    ("id", ["{c}, {t} {u}.", "Hari ini {c}, {t} {u}.", "Besok {c}, {t} {u}.", "Ya, akan hujan.", "Tidak, hari ini tidak hujan.", "Ya, bawa payung.", "Tidak, payung tidak perlu."]),
    ("is", ["{c}, {t} {u}.", "Í dag {c}, {t} {u}.", "Á morgun {c}, {t} {u}.", "Já, rigning er áætluð.", "Nei, engin rigning í dag.", "Já, taktu regnhlíf.", "Nei, þú þarft ekki regnhlíf."]),
    ("it", ["{c}, {t} {u}.", "Oggi {c}, {t} {u}.", "Domani {c}, {t} {u}.", "Sì, è prevista pioggia.", "No, oggi non piove.", "Sì, porta un ombrello.", "No, non ti serve l'ombrello."]),
    ("ja", ["{c}、{t} {u}。", "今日は{c}、{t} {u}。", "明日は{c}、{t} {u}。", "はい、雨です。", "今日は雨ではありません。", "はい、傘を持って。", "傘は不要です。"]),
    ("ka", ["{c}, {t} {u}.", "დღეს {c}, {t} {u}.", "ხვალ {c}, {t} {u}.", "დიახ, წვიმაა მოსალოდნელი.", "არა, დღეს წვიმა არ არის.", "დიახ, აიღე ქოლგა.", "არა, ქოლგა არ გჭირდება."]),
    ("kn", ["{c}, {t} {u}.", "ಇಂದು {c}, {t} {u}.", "ನಾಳೆ {c}, {t} {u}.", "ಹೌದು, ಮಳೆ ಬರಲಿದೆ.", "ಇಲ್ಲ, ಇಂದು ಮಳೆ ಇಲ್ಲ.", "ಹೌದು, ಕೊಡೆ ತೆಗೆದುಕೊ.", "ಇಲ್ಲ, ಕೊಡೆ ಬೇಡ."]),
    ("ko", ["{c}, {t} {u}.", "오늘 {c}, {t} {u}.", "내일 {c}, {t} {u}.", "네, 비가 옵니다.", "아니요, 오늘 비 없습니다.", "네, 우산을 챙기세요.", "아니요, 우산은 필요 없습니다."]),
    ("kw", ["{c}, {t} {u}.", "Hedhyw {c}, {t} {u}.", "A-vorow {c}, {t} {u}.", "Ya, glaw yw gwaytys.", "Na, nyns eus glaw hedhyw.", "Ya, kemmer skoes.", "Na, nyns yw edhom a skoes."]),
    ("lb", ["{c}, {t} {u}.", "Haut {c}, {t} {u}.", "Muer {c}, {t} {u}.", "Jo, Reen ass gemellt.", "Nee, haut kee Reen.", "Jo, huel e Regeschierm.", "Nee, du brauchs kee Schirm."]),
    ("lt", ["{c}, {t} {u}.", "Šiandien {c}, {t} {u}.", "Rytoj {c}, {t} {u}.", "Taip, numatomas lietus.", "Ne, šiandien nelyja.", "Taip, pasiimk skėtį.", "Ne, skėčio nereikia."]),
    ("lv", ["{c}, {t} {u}.", "Šodien {c}, {t} {u}.", "Rīt {c}, {t} {u}.", "Jā, gaidāms lietus.", "Nē, šodien nelīst.", "Jā, ņem līdzi lietussargu.", "Nē, lietussargs nav vajadzīgs."]),
    ("ml", ["{c}, {t} {u}.", "ഇന്ന് {c}, {t} {u}.", "നാളെ {c}, {t} {u}.", "അതെ, മഴയുണ്ടാകും.", "ഇല്ല, ഇന്ന് മഴയില്ല.", "അതെ, കുടയെടുക്കൂ.", "ഇല്ല, കുട വേണ്ട."]),
    ("mn", ["{c}, {t} {u}.", "Өнөөдөр {c}, {t} {u}.", "Маргааш {c}, {t} {u}.", "Тийм, бороо орно.", "Үгүй, өнөөдөр бороогүй.", "Тийм, шүхэр ав.", "Үгүй, шүхэр хэрэггүй."]),
    ("mr", ["{c}, {t} {u}.", "आज {c}, {t} {u}.", "उद्या {c}, {t} {u}.", "हो, पाऊस पडेल.", "नाही, आज पाऊस नाही.", "हो, छत्री घे.", "नाही, छत्रीची गरज नाही."]),
    ("ms", ["{c}, {t} {u}.", "Hari ini {c}, {t} {u}.", "Esok {c}, {t} {u}.", "Ya, akan hujan.", "Tidak, hari ini tiada hujan.", "Ya, bawa payung.", "Tidak, payung tidak diperlukan."]),
    ("nb", ["{c}, {t} {u}.", "I dag {c}, {t} {u}.", "I morgen {c}, {t} {u}.", "Ja, det er varslet regn.", "Nei, ingen regn i dag.", "Ja, ta med en paraply.", "Nei, du trenger ingen paraply."]),
    ("ne", ["{c}, {t} {u}.", "आज {c}, {t} {u}.", "भोलि {c}, {t} {u}.", "हो, वर्षा हुन्छ.", "होइन, आज वर्षा छैन.", "हो, छाता लेऊ.", "होइन, छाता चाहिँदैन."]),
    ("nl", ["{c}, {t} {u}.", "Vandaag {c}, {t} {u}.", "Morgen {c}, {t} {u}.", "Ja, regen wordt verwacht.", "Nee, vandaag geen regen.", "Ja, neem een paraplu mee.", "Nee, je hebt geen paraplu nodig."]),
    ("pa", ["{c}, {t} {u}.", "ਅੱਜ {c}, {t} {u}.", "ਕੱਲ੍ਹ {c}, {t} {u}.", "ਹਾਂ, ਮੀਂਹ ਪਵੇਗਾ.", "ਨਹੀਂ, ਅੱਜ ਮੀਂਹ ਨਹੀਂ.", "ਹਾਂ, ਛਤਰੀ ਲੈ.", "ਨਹੀਂ, ਛਤਰੀ ਦੀ ਲੋੜ ਨਹੀਂ."]),
    ("pl", ["{c}, {t} {u}.", "Dziś {c}, {t} {u}.", "Jutro {c}, {t} {u}.", "Tak, zapowiadany jest deszcz.", "Nie, dziś bez deszczu.", "Tak, weź parasol.", "Nie, parasol nie jest potrzebny."]),
    ("pt", ["{c}, {t} {u}.", "Hoje {c}, {t} {u}.", "Amanhã {c}, {t} {u}.", "Sim, há chuva prevista.", "Não, hoje não chove.", "Sim, leva um guarda-chuva.", "Não, não precisas de guarda-chuva."]),
    ("pt-BR", ["{c}, {t} {u}.", "Hoje {c}, {t} {u}.", "Amanhã {c}, {t} {u}.", "Sim, há chuva prevista.", "Não, hoje não chove.", "Sim, leve um guarda-chuva.", "Não, você não precisa de guarda-chuva."]),
    ("ro", ["{c}, {t} {u}.", "Azi {c}, {t} {u}.", "Mâine {c}, {t} {u}.", "Da, este anunțată ploaie.", "Nu, azi nu plouă.", "Da, ia o umbrelă.", "Nu, nu ai nevoie de umbrelă."]),
    ("sk", ["{c}, {t} {u}.", "Dnes {c}, {t} {u}.", "Zajtra {c}, {t} {u}.", "Áno, hlásia dážď.", "Nie, dnes neprší.", "Áno, zober si dáždnik.", "Nie, dáždnik nepotrebuješ."]),
    ("sl", ["{c}, {t} {u}.", "Danes {c}, {t} {u}.", "Jutri {c}, {t} {u}.", "Da, napovedan je dež.", "Ne, danes ni dežja.", "Da, vzemi dežnik.", "Ne, dežnika ne potrebuješ."]),
    ("sr", ["{c}, {t} {u}.", "Данас {c}, {t} {u}.", "Сутра {c}, {t} {u}.", "Да, најављена је киша.", "Не, данас нема кише.", "Да, понеси кишобран.", "Не, кишобран ти не треба."]),
    ("sr-Latn", ["{c}, {t} {u}.", "Danas {c}, {t} {u}.", "Sutra {c}, {t} {u}.", "Da, najavljena je kiša.", "Ne, danas nema kiše.", "Da, ponesi kišobran.", "Ne, kišobran ti ne treba."]),
    ("sv", ["{c}, {t} {u}.", "I dag {c}, {t} {u}.", "I morgon {c}, {t} {u}.", "Ja, regn är väntat.", "Nej, inget regn i dag.", "Ja, ta med ett paraply.", "Nej, du behöver inget paraply."]),
    ("sw", ["{c}, {t} {u}.", "Leo {c}, {t} {u}.", "Kesho {c}, {t} {u}.", "Ndiyo, mvua inatarajiwa.", "Hapana, leo hakuna mvua.", "Ndiyo, chukua mwavuli.", "Hapana, huhitaji mwavuli."]),
    ("ta", ["{c}, {t} {u}.", "இன்று {c}, {t} {u}.", "நாளை {c}, {t} {u}.", "ஆம், மழை பெய்யும்.", "இல்லை, இன்று மழை இல்லை.", "ஆம், குடை எடு.", "இல்லை, குடை தேவையில்லை."]),
    ("te", ["{c}, {t} {u}.", "ఈరోజు {c}, {t} {u}.", "రేపు {c}, {t} {u}.", "అవును, వర్షం పడుతుంది.", "కాదు, ఈరోజు వర్షం లేదు.", "అవును, గొడుగు తీసుకో.", "కాదు, గొడుగు అవసరం లేదు."]),
    ("th", ["{c} {t} {u}.", "วันนี้ {c} {t} {u}.", "พรุ่งนี้ {c} {t} {u}.", "ใช่ จะมีฝน.", "วันนี้ไม่มีฝน.", "ใช่ พกร่มไป.", "ไม่ ไม่ต้องพกร่ม."]),
    ("tr", ["{c}, {t} {u}.", "Bugün {c}, {t} {u}.", "Yarın {c}, {t} {u}.", "Evet, yağmur bekleniyor.", "Hayır, bugün yağmur yok.", "Evet, şemsiye al.", "Hayır, şemsiyeye gerek yok."]),
    ("uk", ["{c}, {t} {u}.", "Сьогодні {c}, {t} {u}.", "Завтра {c}, {t} {u}.", "Так, очікується дощ.", "Ні, сьогодні без дощу.", "Так, візьми парасольку.", "Ні, парасолька не потрібна."]),
    ("ur", ["{c}، {t} {u}.", "آج {c}، {t} {u}.", "کل {c}، {t} {u}.", "ہاں، بارش ہوگی.", "نہیں، آج بارش نہیں.", "ہاں، چھتری لے لو.", "نہیں، چھتری کی ضرورت نہیں."]),
    ("vi", ["{c}, {t} {u}.", "Hôm nay {c}, {t} {u}.", "Ngày mai {c}, {t} {u}.", "Có, trời sẽ mưa.", "Không, hôm nay không mưa.", "Có, hãy mang ô.", "Không, bạn không cần ô."]),
    ("zh-CN", ["{c}，{t} {u}。", "今天{c}，{t} {u}。", "明天{c}，{t} {u}。", "是的，会下雨。", "今天不会下雨。", "是的，带伞。", "不用带伞。"]),
    ("zh-HK", ["{c}，{t} {u}。", "今日{c}，{t} {u}。", "聽日{c}，{t} {u}。", "係，會落雨。", "今日唔會落雨。", "係，帶遮。", "唔使帶遮。"]),
    ("zh-TW", ["{c}，{t} {u}。", "今天{c}，{t} {u}。", "明天{c}，{t} {u}。", "是的，會下雨。", "今天不會下雨。", "是的，帶傘。", "不用帶傘。"]),
];

#[cfg(test)]
const PACKS: &[&str] = &[
    "af", "ar", "bg", "bn", "ca", "cs", "cy", "da", "de", "de-AT", "de-CH", "el", "en", "en-GB", "es", "et", "eu", "fa", "fi", "fr", "ga",
    "gl", "gu", "he", "hi", "hr", "hu", "hy", "id", "is", "it", "ja", "ka", "kn", "ko", "kw", "lb", "lt", "lv", "ml", "mn", "mr", "ms",
    "nb", "ne", "nl", "pa", "pl", "pt", "pt-BR", "ro", "sk", "sl", "sr", "sr-Latn", "sv", "sw", "ta", "te", "th", "tr", "uk", "ur", "vi",
    "zh-CN", "zh-HK", "zh-TW",
];

/// mon..sun, weekend
#[rustfmt::skip]
const DAYS: &[(&str, [&str; 8])] = &[
    ("af", ["Maandag", "Dinsdag", "Woensdag", "Donderdag", "Vrydag", "Saterdag", "Sondag", "Hierdie naweek"]),
    ("ar", ["الاثنين", "الثلاثاء", "الأربعاء", "الخميس", "الجمعة", "السبت", "الأحد", "نهاية الأسبوع"]),
    ("bg", ["Понеделник", "Вторник", "Сряда", "Четвъртък", "Петък", "Събота", "Неделя", "През уикенда"]),
    ("ca", ["Dilluns", "Dimarts", "Dimecres", "Dijous", "Divendres", "Dissabte", "Diumenge", "Aquest cap de setmana"]),
    ("cs", ["Pondělí", "Úterý", "Středa", "Čtvrtek", "Pátek", "Sobota", "Neděle", "O víkendu"]),
    ("da", ["Mandag", "Tirsdag", "Onsdag", "Torsdag", "Fredag", "Lørdag", "Søndag", "I weekenden"]),
    ("de", ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag", "Am Wochenende"]),
    ("el", ["Δευτέρα", "Τρίτη", "Τετάρτη", "Πέμπτη", "Παρασκευή", "Σάββατο", "Κυριακή", "Το Σαββατοκύριακο"]),
    ("en", ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday", "This weekend"]),
    ("es", ["Lunes", "Martes", "Miércoles", "Jueves", "Viernes", "Sábado", "Domingo", "Este fin de semana"]),
    ("et", ["Esmaspäev", "Teisipäev", "Kolmapäev", "Neljapäev", "Reede", "Laupäev", "Pühapäev", "Sel nädalavahetusel"]),
    ("fi", ["Maanantai", "Tiistai", "Keskiviikko", "Torstai", "Perjantai", "Lauantai", "Sunnuntai", "Tänä viikonloppuna"]),
    ("fr", ["Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi", "Dimanche", "Ce week-end"]),
    ("he", ["יום שני", "יום שלישי", "יום רביעי", "יום חמישי", "יום שישי", "שבת", "יום ראשון", "סוף השבוע"]),
    ("hi", ["सोमवार", "मंगलवार", "बुधवार", "गुरुवार", "शुक्रवार", "शनिवार", "रविवार", "सप्ताहांत"]),
    ("hr", ["Ponedjeljak", "Utorak", "Srijeda", "Četvrtak", "Petak", "Subota", "Nedjelja", "Vikendom"]),
    ("hu", ["Hétfő", "Kedd", "Szerda", "Csütörtök", "Péntek", "Szombat", "Vasárnap", "Hétvégén"]),
    ("id", ["Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu", "Minggu", "Akhir pekan ini"]),
    ("it", ["Lunedì", "Martedì", "Mercoledì", "Giovedì", "Venerdì", "Sabato", "Domenica", "Questo fine settimana"]),
    ("ja", ["月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日", "日曜日", "今週末"]),
    ("ko", ["월요일", "화요일", "수요일", "목요일", "금요일", "토요일", "일요일", "이번 주말"]),
    ("nl", ["Maandag", "Dinsdag", "Woensdag", "Donderdag", "Vrijdag", "Zaterdag", "Zondag", "Dit weekend"]),
    ("nb", ["Mandag", "Tirsdag", "Onsdag", "Torsdag", "Fredag", "Lørdag", "Søndag", "I helgen"]),
    ("pl", ["Poniedziałek", "Wtorek", "Środa", "Czwartek", "Piątek", "Sobota", "Niedziela", "W weekend"]),
    ("pt", ["Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado", "Domingo", "Neste fim de semana"]),
    ("ro", ["Luni", "Marți", "Miercuri", "Joi", "Vineri", "Sâmbătă", "Duminică", "În weekend"]),
    ("ru", ["Понедельник", "Вторник", "Среда", "Четверг", "Пятница", "Суббота", "Воскресенье", "На выходных"]),
    ("sk", ["Pondelok", "Utorok", "Streda", "Štvrtok", "Piatok", "Sobota", "Nedeľa", "Cez víkend"]),
    ("sv", ["Måndag", "Tisdag", "Onsdag", "Torsdag", "Fredag", "Lördag", "Söndag", "I helgen"]),
    ("tr", ["Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi", "Pazar", "Hafta sonu"]),
    ("uk", ["Понеділок", "Вівторок", "Середа", "Четвер", "П’ятниця", "Субота", "Неділя", "На вихідних"]),
    ("vi", ["Thứ hai", "Thứ ba", "Thứ tư", "Thứ năm", "Thứ sáu", "Thứ bảy", "Chủ nhật", "Cuối tuần này"]),
    ("zh-CN", ["星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日", "这周末"]),
    ("zh-HK", ["星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日", "今個週末"]),
    ("zh-TW", ["星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日", "這週末"]),
    ("bn", ["সোমবার", "মঙ্গলবার", "বুধবার", "বৃহস্পতিবার", "শুক্রবার", "শনিবার", "রবিবার", "এই সপ্তাহান্তে"]),
    ("cy", ["Dydd Llun", "Dydd Mawrth", "Dydd Mercher", "Dydd Iau", "Dydd Gwener", "Dydd Sadwrn", "Dydd Sul", "Y penwythnos hwn"]),
    ("eu", ["Astelehena", "Asteartea", "Asteazkena", "Osteguna", "Ostirala", "Larunbata", "Igandea", "Asteburu honetan"]),
    ("fa", ["دوشنبه", "سه‌شنبه", "چهارشنبه", "پنجشنبه", "جمعه", "شنبه", "یکشنبه", "این آخر هفته"]),
    ("ga", ["Dé Luain", "Dé Máirt", "Dé Céadaoin", "Déardaoin", "Dé hAoine", "Dé Sathairn", "Dé Domhnaigh", "An deireadh seachtaine seo"]),
    ("gl", ["Luns", "Martes", "Mércores", "Xoves", "Venres", "Sábado", "Domingo", "Neste fin de semana"]),
    ("is", ["Mánudagur", "Þriðjudagur", "Miðvikudagur", "Fimmtudagur", "Föstudagur", "Laugardagur", "Sunnudagur", "Um helgina"]),
    ("lb", ["Méindeg", "Dënschdeg", "Mëttwoch", "Donneschdeg", "Freideg", "Samschdeg", "Sonndeg", "Dëse Weekend"]),
    ("lt", ["Pirmadienis", "Antradienis", "Trečiadienis", "Ketvirtadienis", "Penktadienis", "Šeštadienis", "Sekmadienis", "Šį savaitgalį"]),
    ("lv", ["Pirmdiena", "Otrdiena", "Trešdiena", "Ceturtdiena", "Piektdiena", "Sestdiena", "Svētdiena", "Šajā nedēļas nogalē"]),
    ("ms", ["Isnin", "Selasa", "Rabu", "Khamis", "Jumaat", "Sabtu", "Ahad", "Hujung minggu ini"]),
    ("sl", ["Ponedeljek", "Torek", "Sreda", "Četrtek", "Petek", "Sobota", "Nedelja", "Ta vikend"]),
    ("sr", ["Понедељак", "Уторак", "Среда", "Четвртак", "Петак", "Субота", "Недеља", "Овог викенда"]),
    ("sr-Latn", ["Ponedeljak", "Utorak", "Sreda", "Četvrtak", "Petak", "Subota", "Nedelja", "Ovog vikenda"]),
    ("sw", ["Jumatatu", "Jumanne", "Jumatano", "Alhamisi", "Ijumaa", "Jumamosi", "Jumapili", "Wikendi hii"]),
    ("th", ["วันจันทร์", "วันอังคาร", "วันพุธ", "วันพฤหัสบดี", "วันศุกร์", "วันเสาร์", "วันอาทิตย์", "สุดสัปดาห์นี้"]),
    ("ur", ["پیر", "منگل", "بدھ", "جمعرات", "جمعہ", "ہفتہ", "اتوار", "اس ہفتے کے آخر"]),
];

const COND: &[(&str, &[(&str, &str)])] = &[
    (
        "de",
        &[
            ("clear-night", "klar"),
            ("cloudy", "bewölkt"),
            ("exceptional", "ungewöhnlich"),
            ("fog", "neblig"),
            ("hail", "Hagel"),
            ("lightning", "Gewitter"),
            ("lightning-rainy", "Gewitter mit Regen"),
            ("partlycloudy", "teilweise bewölkt"),
            ("pouring", "Starkregen"),
            ("rainy", "regnerisch"),
            ("snowy", "Schnee"),
            ("snowy-rainy", "Schneeregen"),
            ("sunny", "sonnig"),
            ("windy", "windig"),
            ("windy-variant", "stürmisch"),
            ("clear", "klar"),
        ],
    ),
    (
        "fr",
        &[
            ("cloudy", "nuageux"),
            ("partlycloudy", "partiellement nuageux"),
            ("rainy", "pluvieux"),
            ("pouring", "forte pluie"),
            ("sunny", "ensoleillé"),
            ("snowy", "neige"),
            ("fog", "brouillard"),
            ("windy", "venteux"),
            ("clear", "clair"),
        ],
    ),
    (
        "nl",
        &[
            ("cloudy", "bewolkt"),
            ("partlycloudy", "licht bewolkt"),
            ("rainy", "regenachtig"),
            ("sunny", "zonnig"),
            ("snowy", "sneeuw"),
            ("fog", "mistig"),
            ("windy", "winderig"),
            ("clear", "helder"),
        ],
    ),
    (
        "es",
        &[
            ("cloudy", "nublado"),
            ("partlycloudy", "parcialmente nublado"),
            ("rainy", "lluvioso"),
            ("sunny", "soleado"),
            ("snowy", "nieve"),
            ("fog", "niebla"),
            ("windy", "ventoso"),
            ("clear", "despejado"),
        ],
    ),
    (
        "it",
        &[
            ("cloudy", "nuvoloso"),
            ("partlycloudy", "parzialmente nuvoloso"),
            ("rainy", "piovoso"),
            ("sunny", "soleggiato"),
            ("snowy", "neve"),
            ("fog", "nebbia"),
            ("windy", "ventoso"),
            ("clear", "sereno"),
        ],
    ),
    (
        "pt",
        &[
            ("cloudy", "nublado"),
            ("partlycloudy", "parcialmente nublado"),
            ("rainy", "chuvoso"),
            ("sunny", "ensolarado"),
            ("snowy", "neve"),
            ("fog", "nevoeiro"),
            ("clear", "limpo"),
        ],
    ),
    (
        "pl",
        &[
            ("cloudy", "pochmurno"),
            ("rainy", "deszczowo"),
            ("sunny", "słonecznie"),
            ("snowy", "śnieg"),
            ("fog", "mgła"),
            ("clear", "bezchmurnie"),
        ],
    ),
    ("zh-CN", &[("cloudy", "多云"), ("rainy", "有雨"), ("sunny", "晴"), ("snowy", "雪"), ("fog", "雾"), ("clear", "晴")]),
    ("ja", &[("cloudy", "曇り"), ("rainy", "雨"), ("sunny", "晴れ"), ("snowy", "雪"), ("fog", "霧"), ("clear", "快晴")]),
    ("ko", &[("cloudy", "흐림"), ("rainy", "비"), ("sunny", "맑음"), ("snowy", "눈"), ("clear", "맑음")]),
    ("ar", &[("cloudy", "غائم"), ("rainy", "ممطر"), ("sunny", "مشمس"), ("clear", "صاف")]),
    ("hi", &[("cloudy", "बादल"), ("rainy", "बारिश"), ("sunny", "धूप"), ("clear", "साफ़")]),
];

const EN_COND: &[(&str, &str)] = &[
    ("clear-night", "clear"),
    ("cloudy", "cloudy"),
    ("exceptional", "unusual"),
    ("fog", "foggy"),
    ("hail", "hail"),
    ("lightning", "thunderstorms"),
    ("lightning-rainy", "thunderstorms"),
    ("partlycloudy", "partly cloudy"),
    ("pouring", "pouring rain"),
    ("rainy", "rainy"),
    ("snowy", "snow"),
    ("snowy-rainy", "sleet"),
    ("sunny", "sunny"),
    ("windy", "windy"),
    ("windy-variant", "windy"),
    ("clear", "clear"),
];

pub(super) fn frames_for(pack: &str) -> [&'static str; 7] {
    lookup(FRAMES, pack).unwrap_or_else(|| lookup(FRAMES, "en").unwrap_or(FRAMES[0].1))
}

pub(super) fn weekday_label(pack: &str, key: &str) -> Option<String> {
    let idx = match key {
        "mon" => 0,
        "tue" => 1,
        "wed" => 2,
        "thu" => 3,
        "fri" => 4,
        "sat" => 5,
        "sun" => 6,
        "weekend" => 7,
        _ => return None,
    };
    lookup(DAYS, pack).or_else(|| lookup(DAYS, "en")).map(|row| row[idx].to_string())
}

pub(super) fn speak_condition(raw: &str, pack: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }
    if let Some((_, spoken)) = cond_row(pack).and_then(|row| row.iter().find(|(name, _)| *name == raw)) {
        return (*spoken).to_string();
    }
    EN_COND.iter().find(|(name, _)| *name == raw).map(|(_, spoken)| (*spoken).to_string()).unwrap_or_else(|| raw.replace('-', " "))
}

pub(super) fn unit_word(pack: &str, system: UnitSystem) -> &'static str {
    if matches!(system, UnitSystem::Imperial) {
        return "Fahrenheit";
    }
    match pack.split(['-', '_']).next().unwrap_or(pack) {
        "de" | "lb" => "Grad",
        "fr" => "degrés",
        "nl" | "af" => "graden",
        "es" | "ca" | "gl" => "grados",
        "it" => "gradi",
        "pt" => "graus",
        "pl" => "stopni",
        "cs" | "sk" => "stupňů",
        "da" | "nb" | "sv" => "grader",
        "fi" | "et" => "astetta",
        "hu" => "fok",
        "tr" => "derece",
        "ar" => "درجة",
        "he" => "מעלות",
        "hi" | "mr" | "ne" => "डिग्री",
        "zh" | "ja" | "ko" => "度",
        "vi" => "độ",
        "id" | "ms" => "derajat",
        "th" => "องศา",
        "uk" | "bg" | "sr" => "градусів",
        "el" => "βαθμοί",
        _ => "degrees",
    }
}

pub(super) fn locale_temp(temp: String, pack: &str) -> String {
    match pack.split(['-', '_']).next().unwrap_or(pack) {
        "de" | "fr" | "nl" | "es" | "it" | "pt" | "pl" | "cs" | "sk" | "hu" | "da" | "nb" | "sv" | "fi" | "tr" | "ro" | "hr" | "sl"
        | "af" | "ca" | "lb" => temp.replace('.', ","),
        _ => temp,
    }
}

fn cond_row(pack: &str) -> Option<&'static [(&'static str, &'static str)]> {
    COND.iter()
        .find(|(code, _)| *code == pack)
        .or_else(|| {
            let base = pack.split(['-', '_']).next().unwrap_or(pack);
            COND.iter().find(|(code, _)| *code == base)
        })
        .map(|(_, row)| *row)
}

fn lookup<T: Copy>(table: &[(&str, T)], pack: &str) -> Option<T> {
    table
        .iter()
        .find(|(code, _)| *code == pack)
        .or_else(|| {
            let base = pack.split(['-', '_']).next().unwrap_or(pack);
            table.iter().find(|(code, _)| *code == base)
        })
        .map(|(_, row)| *row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pack_has_own_frames() {
        assert_eq!(FRAMES.len(), PACKS.len());
        for code in PACKS {
            assert!(FRAMES.iter().any(|(row, _)| row == code), "{code}");
            let row = frames_for(code);
            assert_eq!(row.len(), 7, "{code}");
            assert!(row[3].chars().count() > 2, "{code} rain yes");
            assert!(!row[3].contains('\u{FFFD}'), "{code} replacement char");
        }
    }

    #[test]
    fn dutch_and_japanese_are_not_english() {
        assert!(frames_for("nl")[3].contains("regen"));
        assert!(frames_for("ja")[3].contains("雨"));
        assert_eq!(speak_condition("cloudy", "ja"), "曇り");
        assert_eq!(weekday_label("nl", "mon").as_deref(), Some("Maandag"));
        assert_eq!(weekday_label("th", "mon").as_deref(), Some("วันจันทร์"));
        assert_eq!(weekday_label("sw", "weekend").as_deref(), Some("Wikendi hii"));
    }
}
