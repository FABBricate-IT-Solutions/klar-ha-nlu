"""Spoken calendar lines the way a person would say them."""

from __future__ import annotations

from datetime import date, datetime, timedelta
from typing import Any
from zoneinfo import ZoneInfo

# item, created, deleted, moved, empty, today, tomorrow, all_day, at, clock
Row = tuple[str, str, str, str, str, str, str, str, str, str]

SAY: dict[str, Row] = {
    "de": ("{when} steht {summary} an.", "{summary} ist für {when} eingetragen.", "{summary} ist gelöscht.", "{summary} ist jetzt {when}.", "In nächster Zeit steht nichts an.", "heute", "morgen", "den ganzen Tag", "um {time}", "de"),
    "de-CH": ("{when} het's {summary}.", "{summary} isch für {when} ytrait.", "{summary} isch glöscht.", "{summary} isch jetzt {when}.", "S het nüüt astehends.", "hüt", "morn", "de ganz Tag", "um {time}", "de"),
    "de-AT": ("{when} steht {summary} an.", "{summary} ist für {when} eingetragen.", "{summary} ist gelöscht.", "{summary} ist jetzt {when}.", "In nächster Zeit steht nichts an.", "heute", "morgen", "den ganzen Tag", "um {time}", "de"),
    "en": ("{summary} is {when}.", "I added {summary} for {when}.", "{summary} is gone.", "{summary} is now {when}.", "Nothing coming up.", "today", "tomorrow", "all day", "at {time}", "12"),
    "en-GB": ("{summary} is {when}.", "I have put {summary} on for {when}.", "{summary} is gone.", "{summary} is now {when}.", "Nothing coming up.", "today", "tomorrow", "all day", "at {time}", "12"),
    "fr": ("{summary} est prévu {when}.", "J'ai noté {summary} pour {when}.", "{summary} est supprimé.", "{summary} est maintenant {when}.", "Rien de prévu.", "aujourd'hui", "demain", "toute la journée", "à {time}", "fr"),
    "nl": ("{when} staat {summary}.", "{summary} staat voor {when}.", "{summary} is verwijderd.", "{summary} is nu {when}.", "Er staat niets gepland.", "vandaag", "morgen", "de hele dag", "om {time}", "nl"),
    "es": ("Tienes {summary} {when}.", "He apuntado {summary} para {when}.", "He borrado {summary}.", "{summary} es ahora {when}.", "No hay nada próximo.", "hoy", "mañana", "todo el día", "a las {time}", "es"),
    "it": ("Hai {summary} {when}.", "Ho messo {summary} per {when}.", "Ho cancellato {summary}.", "{summary} è ora {when}.", "Non c'è nulla in programma.", "oggi", "domani", "tutto il giorno", "alle {time}", "it"),
    "pt": ("Tens {summary} {when}.", "Marquei {summary} para {when}.", "Apaguei {summary}.", "{summary} é agora {when}.", "Não há nada a seguir.", "hoje", "amanhã", "o dia todo", "às {time}", "pt"),
    "pt-BR": ("Você tem {summary} {when}.", "Marquei {summary} para {when}.", "Apaguei {summary}.", "{summary} agora é {when}.", "Nada pela frente.", "hoje", "amanhã", "o dia todo", "às {time}", "pt"),
    "ca": ("Tens {summary} {when}.", "He apuntat {summary} per {when}.", "He esborrat {summary}.", "{summary} ara és {when}.", "No hi ha res previst.", "avui", "demà", "tot el dia", "a les {time}", "es"),
    "ro": ("Ai {summary} {when}.", "Am notat {summary} pentru {when}.", "Am șters {summary}.", "{summary} este acum {when}.", "Nu e nimic programat.", "astăzi", "mâine", "toată ziua", "la {time}", "24"),
    "da": ("{summary} står {when}.", "Jeg har sat {summary} til {when}.", "{summary} er slettet.", "{summary} er nu {when}.", "Der står ikke noget.", "i dag", "i morgen", "hele dagen", "kl. {time}", "nord"),
    "nb": ("{summary} står {when}.", "Jeg har satt opp {summary} {when}.", "{summary} er slettet.", "{summary} er nå {when}.", "Ingenting kommer.", "i dag", "i morgen", "hele dagen", "kl. {time}", "nord"),
    "sv": ("{summary} är {when}.", "Jag har lagt in {summary} {when}.", "{summary} är borttagen.", "{summary} är nu {when}.", "Inget på gång.", "i dag", "i morgon", "hela dagen", "kl. {time}", "nord"),
    "fi": ("{summary} on {when}.", "Lisäsin kohteen {summary} ajalle {when}.", "{summary} on poistettu.", "{summary} on nyt {when}.", "Ei tulevia tapahtumia.", "tänään", "huomenna", "koko päivän", "klo {time}", "24"),
    "af": ("{summary} is {when}.", "Ek het {summary} geskeduleer vir {when}.", "{summary} is verwyder.", "{summary} is nou {when}.", "Niks kom aan nie.", "vandag", "môre", "die hele dag", "om {time}", "nl"),
    "cs": ("{summary} je {when}.", "Přidal jsem {summary} na {when}.", "{summary} je smazáno.", "{summary} je teď {when}.", "Nic se nechystá.", "dnes", "zítra", "celý den", "v {time}", "24"),
    "sk": ("{summary} je {when}.", "Pridal som {summary} na {when}.", "{summary} je zmazané.", "{summary} je teraz {when}.", "Nič sa nechystá.", "dnes", "zajtra", "celý deň", "o {time}", "24"),
    "pl": ("{summary} jest {when}.", "Wpisałem {summary} na {when}.", "Usunąłem {summary}.", "{summary} jest teraz {when}.", "Nic się nie szykuje.", "dzisiaj", "jutro", "przez cały dzień", "o {time}", "24"),
    "hu": ("{summary} {when} van.", "Felvettem: {summary}, {when}.", "{summary} törölve.", "{summary} most {when} van.", "Nincs közelgő esemény.", "ma", "holnap", "egész nap", "{time}-kor", "24"),
    "hr": ("{summary} je {when}.", "Dodao sam {summary} za {when}.", "{summary} je obrisano.", "{summary} je sada {when}.", "Nema predstojećih termina.", "danas", "sutra", "cijeli dan", "u {time}", "24"),
    "sl": ("{summary} je {when}.", "Dodat sem {summary} za {when}.", "{summary} je izbrisano.", "{summary} je zdaj {when}.", "Ni prihajajočih dogodkov.", "danes", "jutri", "cel dan", "ob {time}", "24"),
    "bg": ("{summary} е {when}.", "Добавих {summary} за {when}.", "{summary} е изтрито.", "{summary} вече е {when}.", "Няма предстоящи събития.", "днес", "утре", "целия ден", "в {time}", "24"),
    "el": ("{summary} είναι {when}.", "Έβαλα {summary} για {when}.", "Διέγραψα το {summary}.", "{summary} είναι πλέον {when}.", "Δεν υπάρχει κάτι προσεχές.", "σήμερα", "αύριο", "όλη μέρα", "στις {time}", "24"),
    "sr": ("{summary} је {when}.", "Додао сам {summary} за {when}.", "{summary} је обрисано.", "{summary} је сада {when}.", "Нема предстојећих догађаја.", "данас", "сутра", "цео дан", "у {time}", "24"),
    "sr-Latn": ("{summary} je {when}.", "Dodao sam {summary} za {when}.", "{summary} je obrisano.", "{summary} je sada {when}.", "Nema predstojećih događaja.", "danas", "sutra", "ceo dan", "u {time}", "24"),
    "uk": ("{summary} — {when}.", "Я додав {summary} на {when}.", "{summary} видалено.", "{summary} тепер {when}.", "Найближчим часом нічого немає.", "сьогодні", "завтра", "весь день", "о {time}", "24"),
    "zh-CN": ("{when}有{summary}。", "已把{summary}记在{when}。", "已删掉{summary}。", "{summary}改到{when}了。", "最近没有安排。", "今天", "明天", "全天", "{time}", "zh"),
    "zh-TW": ("{when}有{summary}。", "已把{summary}記在{when}。", "已刪掉{summary}。", "{summary}改到{when}了。", "最近沒有行程。", "今天", "明天", "整天", "{time}", "zh"),
    "zh-HK": ("{when}有{summary}。", "已經將{summary}記喺{when}。", "刪咗{summary}。", "{summary}改到{when}。", "近期冇行程。", "今日", "聽日", "全日", "{time}", "zh"),
    "ar": ("لديك {summary} {when}.", "أضفت {summary} في {when}.", "حذفت {summary}.", "{summary} أصبح {when}.", "لا يوجد شيء قريب.", "اليوم", "غدًا", "طوال اليوم", "الساعة {time}", "24"),
    "he": ("יש {summary} {when}.", "הוספתי את {summary} ל{when}.", "מחקתי את {summary}.", "{summary} עכשיו {when}.", "אין כלום בקרוב.", "היום", "מחר", "כל היום", "בשעה {time}", "24"),
    "fa": ("{summary} {when} است.", "{summary} را برای {when} گذاشتم.", "{summary} حذف شد.", "{summary} الان {when} است.", "چیزی در پیش نیست.", "امروز", "فردا", "تمام روز", "ساعت {time}", "24"),
    "ur": ("{when} {summary} ہے۔", "میں نے {summary} {when} لکھ دیا۔", "{summary} حذف ہو گیا۔", "{summary} اب {when} ہے۔", "قریب کچھ نہیں۔", "آج", "کل", "پورے دن", "{time} بجے", "24"),
    "tr": ("{when} {summary} var.", "{summary} etkinliğini {when} ekledim.", "{summary} silindi.", "{summary} artık {when}.", "Yakında bir şey yok.", "bugün", "yarın", "tüm gün", "saat {time}", "24"),
    "th": ("มี{summary}{when}", "ใส่{summary}ไว้{when}แล้ว", "ลบ{summary}แล้ว", "{summary}ย้ายเป็น{when}", "ไม่มีนัดใกล้ ๆ นี้", "วันนี้", "พรุ่งนี้", "ทั้งวัน", "เวลา {time}", "th"),
    "ko": ("{when} {summary} 있어요.", "{summary}을 {when}에 넣었어요.", "{summary} 지웠어요.", "{summary}은 이제 {when}이에요.", "곧 있을 일정이 없어요.", "오늘", "내일", "하루 종일", "{time}", "ko"),
    "ja": ("{when}、{summary}があります。", "{summary}を{when}に入れました。", "{summary}を消しました。", "{summary}は{when}になりました。", "この先の予定はありません。", "今日", "明日", "終日", "{time}", "ja"),
    "cy": ("Mae {summary} {when}.", "Rydw i wedi rhoi {summary} ar gyfer {when}.", "Mae {summary} wedi'i dileu.", "Mae {summary} nawr {when}.", "Dim byd ar y gweill.", "heddiw", "yfory", "drwy'r dydd", "am {time}", "24"),
    "et": ("{summary} on {when}.", "Lisasin {summary} ajale {when}.", "{summary} on kustutatud.", "{summary} on nüüd {when}.", "Midagi ees ei ole.", "täna", "homme", "kogu päev", "kell {time}", "24"),
    "eu": ("{summary} daukazu {when}.", "{summary} jarri dut {when}.", "{summary} ezabatu dut.", "{summary} orain {when} da.", "Ez dago ezer hurbil.", "gaur", "bihar", "egun osoan", "{time}etan", "24"),
    "ga": ("Tá {summary} {when}.", "Chuir mé {summary} síos do {when}.", "Scrios mé {summary}.", "Tá {summary} {when} anois.", "Níl aon rud ag teacht.", "inniu", "amárach", "an lá ar fad", "ag {time}", "24"),
    "gl": ("Tes {summary} {when}.", "Anotei {summary} para {when}.", "Borrei {summary}.", "{summary} agora é {when}.", "Non hai nada próximo.", "hoxe", "mañá", "todo o día", "ás {time}", "es"),
    "is": ("{summary} er {when}.", "Ég setti {summary} á {when}.", "{summary} er eytt.", "{summary} er nú {when}.", "Ekkert á döfinni.", "í dag", "á morgun", "allan daginn", "klukkan {time}", "nord"),
    "lb": ("{when} steet {summary}.", "{summary} ass fir {when} agedroen.", "{summary} ass geläscht.", "{summary} ass elo {when}.", "Näischt steet un.", "haut", "muer", "de ganzen Dag", "um {time}", "de"),
    "kw": ("Yma {summary} {when}.", "My a worras {summary} rag {when}.", "{summary} yw diles.", "{summary} yw {when} lemmyn.", "Nyns eus travyth a-dheu.", "hedhyw", "avorow", "dres an jydh", "dhe {time}", "24"),
    "lt": ("{summary} yra {when}.", "Įrašiau {summary} {when}.", "{summary} ištrinta.", "{summary} dabar {when}.", "Nieko artimiausiu metu nėra.", "šiandien", "rytoj", "visą dieną", "{time}", "24"),
    "lv": ("{summary} ir {when}.", "Ieliku {summary} uz {when}.", "{summary} ir dzēsts.", "{summary} tagad ir {when}.", "Tuvākajā laikā nekā nav.", "šodien", "rīt", "visu dienu", "plkst. {time}", "24"),
    "id": ("{summary} ada {when}.", "Saya masukkan {summary} untuk {when}.", "{summary} sudah dihapus.", "{summary} sekarang {when}.", "Tidak ada yang segera.", "hari ini", "besok", "sehari penuh", "pukul {time}", "24"),
    "ms": ("{summary} ada {when}.", "Saya letak {summary} untuk {when}.", "{summary} sudah dipadam.", "{summary} sekarang {when}.", "Tiada apa-apa terdekat.", "hari ini", "esok", "sehari penuh", "pukul {time}", "24"),
    "sw": ("{summary} iko {when}.", "Nimeweka {summary} kwa {when}.", "{summary} imefutwa.", "{summary} sasa ni {when}.", "Hakuna kinachokuja.", "leo", "kesho", "siku nzima", "saa {time}", "24"),
    "vi": ("Bạn có {summary} {when}.", "Tôi đã ghi {summary} vào {when}.", "Đã xóa {summary}.", "{summary} giờ là {when}.", "Không có gì sắp tới.", "hôm nay", "ngày mai", "cả ngày", "lúc {time}", "24"),
    "hi": ("{when} {summary} है।", "मैंने {summary} {when} लिख दिया।", "{summary} हटा दिया।", "{summary} अब {when} है।", "आगे कुछ नहीं है।", "आज", "कल", "पूरे दिन", "{time} बजे", "24"),
    "bn": ("{when} {summary} আছে।", "আমি {summary} {when} লিখেছি।", "{summary} মুছেছি।", "{summary} এখন {when}।", "কাছাকাছি কিছু নেই।", "আজ", "কাল", "সারাদিন", "{time}টায়", "24"),
    "gu": ("{when} {summary} છે.", "મેં {summary} {when} લખ્યું.", "{summary} કાઢી નાખ્યું.", "{summary} હવે {when} છે.", "નજીકમાં કંઈ નથી.", "આજે", "કાલે", "આખો દિવસ", "{time} વાગ્યે", "24"),
    "kn": ("{when} {summary} ಇದೆ.", "ನಾನು {summary} {when} ಹಾಕಿದೆ.", "{summary} ತೆಗೆದಿದೆ.", "{summary} ಈಗ {when}.", "ಮುಂದೆ ಏನು ಇಲ್ಲ.", "ಇಂದು", "ನಾಳೆ", "ಇದೀ ದಿನ", "{time}kke", "24"),
    "ml": ("{when} {summary} ഉണ്ട്.", "ഞാന് {summary} {when} ചേർത്തു.", "{summary} മാറ്റി.", "{summary} ഇപ്പോൾ {when}.", "അടുത്ത് ഒന്നുമില്ല.", "ഇന്നു", "നാളെ", "ദിവസം മുഴുവന്", "{time}nu", "24"),
    "mr": ("{when} {summary} आहे.", "मी {summary} {when} लिहिले.", "{summary} काढले.", "{summary} आता {when} आहे.", "पुढे काही नाही.", "आज", "उद्या", "दिवसभर", "{time} vajta", "24"),
    "ta": ("{when} {summary} உள்ளது.", "நான் {summary} {when} போட்டேன்.", "{summary} நீக்கப்பட்டது.", "{summary} இப்போது {when}.", "அருகில் ஒன்றுமில்லை.", "இன்று", "நாளை", "நாள் முழுவதும்", "{time}kku", "24"),
    "te": ("{when} {summary} ఉంది.", "నేను {summary} {when} పెట్టాను.", "{summary} తీసాను.", "{summary} ఇప్పుడు {when}.", "ముందు ఏమి లేదు.", "ఈరోజు", "రేపు", "రోజంతా", "{time}ki", "24"),
    "pa": ("{when} {summary} ਹੈ.", "ਮੈਂ {summary} {when} ਲਿਖਿਆ.", "{summary} ਹਟਾ ਦਿੱਤਾ.", "{summary} ਹੁਣ {when} ਹੈ.", "ਨੇੜੇ ਕੁਝ ਨਹੀ.", "ਅੱਜ", "ਕੱਲ", "ਸਾਰਾ ਦਿਨ", "{time} vaje", "24"),
    "ne": ("{when} {summary} छ.", "मैले {summary} {when} लेखे.", "{summary} हटायो.", "{summary} अब {when} हो.", "नजिक केही छैन.", "आज", "भोली", "दिनभरी", "{time} baje", "24"),
    "hy": ("{summary} {when} է.", "Ես գրանտեցի {summary} {when}.", "{summary} ջնջվեց.", "{summary} հիմա {when} է.", "Մոտ ոչինչ չկա.", "այսօր", "վաղչ", "ամբողջոր օրը", "{time}-in", "24"),
    "ka": ("{summary} არის {when}.", "ჩავწერე {summary} {when}.", "{summary} წაიშალა.", "{summary} ახლა {when} არის.", "ახლო არაფერი არის.", "დღეს", "ხვალ", "მთელი დღე", "{time}-ze", "24"),
    "mn": ("{when} {summary} байна.", "Би {summary}-г {when} нэмсэн.", "{summary} устгасан.", "{summary} одоо {when}.", "Ойрын үед юу ч алга.", "өнөөдөр", "маргааш", "өдөржин", "{time} цагт", "24"),
}

# instruction, events heading — system prompt for calendar-query LLM
LLM: dict[str, tuple[str, str]] = {
    "de": ("Der Nutzer fragt nach seinem Kalender. Formuliere die folgenden Termine natürlich und knapp. Erfinde keine Termine. Wenn die Liste leer ist, sag das klar. Steuere keine Geräte.", "Termine"),
    "de-CH": ("De Nutzer fragt nach sim Kalender. Sag d folgende Termin natürlich und churz. Erfind kei Termin. Wenn d Liste leer isch, säg das klar. Stüür kei Grät.", "Termin"),
    "de-AT": ("Der Nutzer fragt nach seinem Kalender. Formuliere die folgenden Termine natürlich und knapp. Erfinde keine Termine. Wenn die Liste leer ist, sag das klar. Steuere keine Geräte.", "Termine"),
    "en": ("The user is asking about their calendar. Say the following events naturally and briefly. Do not invent events. If the list is empty, say so clearly. Do not control devices.", "Events"),
    "en-GB": ("The user is asking about their calendar. Say the following events naturally and briefly. Do not invent events. If the list is empty, say so clearly. Do not control devices.", "Events"),
    "fr": ("L'utilisateur demande son agenda. Dis les événements suivants naturellement et brièvement. N'invente aucun événement. Si la liste est vide, dis-le clairement. Ne commande aucun appareil.", "Événements"),
    "nl": ("De gebruiker vraagt naar de agenda. Zeg de volgende afspraken natuurlijk en kort. Verzin geen afspraken. Als de lijst leeg is, zeg dat duidelijk. Bedien geen apparaten.", "Afspraken"),
    "es": ("El usuario pregunta por su calendario. Di los siguientes eventos con naturalidad y brevedad. No inventes eventos. Si la lista está vacía, dilo claro. No controles dispositivos.", "Eventos"),
    "it": ("L'utente chiede del calendario. Di' i seguenti eventi in modo naturale e breve. Non inventare eventi. Se l'elenco è vuoto, dillo chiaramente. Non controllare dispositivi.", "Eventi"),
    "pt": ("O utilizador pergunta pelo calendário. Diz os seguintes eventos de forma natural e breve. Não inventes eventos. Se a lista estiver vazia, diz-lo claramente. Não controles aparelhos.", "Eventos"),
    "pt-BR": ("O usuário pergunta pelo calendário. Diga os eventos a seguir de forma natural e breve. Não invente eventos. Se a lista estiver vazia, diga isso claramente. Não controle aparelhos.", "Eventos"),
    "ca": ("L'usuari pregunta pel calendari. Digues els esdeveniments següents de manera natural i breu. No t'inventis res. Si la llista és buida, digues-ho clar. No controlis aparells.", "Esdeveniments"),
    "ro": ("Utilizatorul întreabă de calendar. Spune următoarele evenimente natural și scurt. Nu inventa evenimente. Dacă lista e goală, spune-o clar. Nu controla dispozitive.", "Evenimente"),
    "da": ("Brugeren spørger til kalenderen. Sig de følgende aftaler naturligt og kort. Find ikke på aftaler. Hvis listen er tom, så sig det tydeligt. Styr ingen enheder.", "Aftaler"),
    "nb": ("Brukeren spør om kalenderen. Si de følgende avtalene naturlig og kort. Ikke finn på avtaler. Hvis listen er tom, si det tydelig. Ikke styr enheter.", "Avtaler"),
    "sv": ("Användaren frågar om kalendern. Säg följande händelser naturligt och kort. Hitta inte på händelser. Om listan är tom, säg det tydligt. Styr inga enheter.", "Händelser"),
    "fi": ("Käyttäjä kysyy kalenterista. Kerro seuraavat tapahtumat luonnollisesti ja lyhyesti. Älä keksi tapahtumia. Jos lista on tyhjä, sano se selvästi. Älä ohjaa laitteita.", "Tapahtumat"),
    "af": ("Die gebruiker vra oor die kalender. Sê die volgende afsprake natuurlik en kort. Moenie afsprake versin nie. As die lys leeg is, sê dit duidelik. Moenie toestelle beheer nie.", "Afsprake"),
    "cs": ("Uživatel se ptá na kalendář. Řekni následující události přirozeně a stručně. Nevymýšlej události. Je-li seznam prázdný, řekni to jasně. Neovládej zařízení.", "Události"),
    "sk": ("Používateľ sa pýta na kalendár. Povedz nasledujúce udalosti prirodzene a stručne. Nevymýšľaj udalosti. Ak je zoznam prázdny, povedz to jasne. Neovládaj zariadenia.", "Udalosti"),
    "pl": ("Użytkownik pyta o kalendarz. Powiedz następujące wydarzenia naturalnie i krótko. Nie zmyślaj wydarzeń. Jeśli lista jest pusta, powiedz to jasno. Nie steruj urządzeniami.", "Wydarzenia"),
    "hu": ("A felhasználó a naptáráról kérdez. Mondd el a következő eseményeket természetesen és röviden. Ne találj ki eseményeket. Ha a lista üres, mondd el világosan. Ne irányíts eszközöket.", "Események"),
    "hr": ("Korisnik pita za kalendar. Reci sljedeće događaje prirodno i kratko. Ne izmišljaj događaje. Ako je popis prazan, reci to jasno. Ne upravljaj uređajima.", "Događaji"),
    "sl": ("Uporabnik sprašuje o koledarju. Povej naslednje dogodke naravno in na kratko. Ne izmišljaj dogodkov. Če je seznam prazen, to jasno povej. Ne upravljaj naprav.", "Dogodki"),
    "bg": ("Потребителят пита за календара. Кажи следващите събития естествено и кратко. Не измисляй събития. Ако списъкът е празен, кажи го ясно. Не управлявай устройства.", "Събития"),
    "el": ("Ο χρήστης ρωτά για το ημερολόγιο. Πες τα παρακάτω γεγονότα φυσικά και σύντομα. Μην επινοείς γεγονότα. Αν η λίστα είναι άδεια, πες το καθαρά. Μην ελέγχεις συσκευές.", "Γεγονότα"),
    "sr": ("Корисник пита за календар. Реци следеће догађаје природно и кратко. Не измишљај догађаје. Ако је списак празан, реци то јасно. Не управљај уређајима.", "Догађаји"),
    "sr-Latn": ("Korisnik pita za kalendar. Reci sledeće događaje prirodno i kratko. Ne izmišljaj događaje. Ako je spisak prazan, reci to jasno. Ne upravljaj uređajima.", "Događaji"),
    "uk": ("Користувач питає про календар. Скажи наступні події природно і коротко. Не вигадуй подій. Якщо список порожній, скажи це чітко. Не керуй пристроями.", "Події"),
    "zh-CN": ("用户在问日历。用自然、简短的话念出下面的日程。不要编造日程。如果列表是空的，就直接说没有。不要控制设备。", "日程"),
    "zh-TW": ("使用者在問日曆。用自然、簡短的話唸出下面的行程。不要編造行程。如果清單是空的，就直接說沒有。不要控制裝置。", "行程"),
    "zh-HK": ("用戶喺問日曆。用自然、簡短嘅話講下面嘅行程。唔好捏造行程。如果清單係空，就直認冇。唔好控制裝置。", "行程"),
    "ar": ("يسأل المستخدم عن تقويمه. قل المواعيد التالية بشكل طبيعي ومختصر. لا تخترع مواعيد. إذا كانت القائمة فارغة فقل ذلك بوضوح. لا تتحكم في الأجهزة.", "المواعيد"),
    "he": ("המשתמש שואל על היומן. אמור את האירועים הבאים באופן טבעי וקצר. אל תמציא אירועים. אם הרשימה ריקה, אמור זאת בבירור. אל תשלוט במכשירים.", "אירועים"),
    "fa": ("کاربر از تقویمش می‌پرسد. رویدادهای زیر را طبیعی و کوتاه بگو. رویدادی نساز. اگر فهرست خالی است واضح بگو. دستگاه‌ها را کنترل نکن.", "رویدادها"),
    "ur": ("صارف اپنے کیلنڈر کے بارے میں پوچھ رہا ہے۔ درج ذیل تقریبات قدرتی اور مختصر کہو۔ کوئی تقریب مت بناؤ۔ اگر فہرست خالی ہو تو واضح کہو۔ آلات مت چلاؤ۔", "تقریبات"),
    "tr": ("Kullanıcı takvimini soruyor. Aşağıdaki etkinlikleri doğal ve kısa söyle. Etkinlik uydurma. Liste boşsa bunu açık söyle. Cihazları kontrol etme.", "Etkinlikler"),
    "th": ("ผู้ใช้ถามเรื่องปฏิทิน พูดนัดต่อไปนี้อย่างเป็นธรรมชาติและสั้น อย่าแต่งนัด ถ้าไม่มีนัดให้พูดชัด อย่าควบคุมอุปกรณ์", "นัดหมาย"),
    "ko": ("사용자가 일정을 묻습니다. 아래 일정을 자연스럽고 짧게 말하세요. 일정을 지어내지 마세요. 목록이 비어 있으면 분명히 말하세요. 기기를 제어하지 마세요.", "일정"),
    "ja": ("利用者はカレンダーを聞いています。次の予定を自然に、短く話してください。予定を作らないでください。一覧が空なら、はっきりそう言ってください。機器は操作しないでください。", "予定"),
    "cy": ("Mae'r defnyddiwr yn gofyn am y calendr. Dywedwch y digwyddiadau canlynol yn naturiol ac yn fyr. Peidiwch â dyfeisio digwyddiadau. Os yw'r rhestr yn wag, dywedwch hynny'n glir. Peidiwch â rheoli dyfeisiau.", "Digwyddiadau"),
    "et": ("Kasutaja küsib kalendri kohta. Ütle järgmised sündmused loomulikult ja lühidalt. Ära leiuta sündmusi. Kui nimekiri on tühi, ütle seda selgelt. Ära juhi seadmeid.", "Sündmused"),
    "eu": ("Erabiltzaileak egutegia galdetzen du. Esan hurrengo gertaerak era naturalean eta labur. Ez asmatu gertaerarik. Zerrenda hutsik badago, esan argi. Ez kontrolatu gailurik.", "Gertaerak"),
    "ga": ("Tá an t-úsáideoir ag fiafraí faoin bhféilire. Abair na himeachtaí seo a leanas go nádúrtha agus go gairid. Ná cum imeachtaí. Más folamh an liosta, abair é sin go soiléir. Ná rialaigh gléasanna.", "Imeachtaí"),
    "gl": ("O usuario pregunta polo calendario. Di os seguintes eventos con naturalidade e brevedade. Non inventes eventos. Se a lista está baleira, dilo claro. Non controles aparellos.", "Eventos"),
    "is": ("Notandinn spyr um dagatalið. Segðu eftirfarandi atburði eðlilega og stutt. Ekki búa til atburði. Ef listinn er tómur, segðu það skýrt. Ekki stjórna tækjum.", "Atburðir"),
    "lb": ("De Benotzer freet no sengem Kalenner. So déi folgend Terminer natierlech a kuerz. Erfann keng Terminer. Wann d'Lëscht eidel ass, so dat kloer. Stéier keng Apparater.", "Terminer"),
    "kw": ("An devnydhyer a wovyn orth y galender. Lavar an hwarvosow a sew yn naturel hag yn kott. Na vriw hwarvosow. Mars yw gwag an rol, lavar henna yn kler. Na rewle devisys.", "Hwarvosow"),
    "lt": ("Naudotojas klausia apie kalendorių. Pasakyk šiuos įvykius natūraliai ir trumpai. Neišgalvok įvykių. Jei sąrašas tuščias, pasakyk tai aiškiai. Nevaldyk įrenginių.", "Įvykiai"),
    "lv": ("Lietotājs jautā par kalendāru. Pasaki šos notikumus dabiski un īsi. Neizdomā notikumus. Ja saraksts ir tukšs, pasaki to skaidri. Nestūrē ierīces.", "Notikumi"),
    "id": ("Pengguna bertanya tentang kalender. Sampaikan acara berikut secara alami dan singkat. Jangan mengarang acara. Jika daftar kosong, katakan dengan jelas. Jangan kendalikan perangkat.", "Acara"),
    "ms": ("Pengguna bertanya tentang kalendar. Sebut acara berikut secara semula jadi dan ringkas. Jangan cipta acara. Jika senarai kosong, sebut dengan jelas. Jangan kawal peranti.", "Acara"),
    "sw": ("Mtumiaji anauliza kuhusu kalenda. Sema matukio yafuatayo kwa asili na ufupi. Usibuni matukio. Orodha ikiwa tupu, sema wazi. Usidhibiti vifaa.", "Matukio"),
    "vi": ("Người dùng hỏi về lịch. Nói các sự kiện sau một cách tự nhiên và ngắn. Đừng bịa sự kiện. Nếu danh sách trống, nói rõ. Đừng điều khiển thiết bị.", "Sự kiện"),
    "hi": ("उपयोगकर्ता अपने कैलेंडर के बारे में पूछ रहा है। नीचे दिए कार्यक्रम स्वाभाविक और संक्षेप में कहो। कोई कार्यक्रम मत गढ़ो। सूची खाली हो तो साफ़ कहो। उपकरण मत चलाओ।", "कार्यक्रम"),
    "bn": ("ব্যবহারকারী ক্যালেন্ডার নিয়ে জিজ্ঞাসা করছে। নিচের অনুষ্ঠানগুলো স্বাভাবিক ও সংক্ষেপে বলো। অনুষ্ঠান বানিও না। তালিকা খালি হলে স্পষ্ট করে বলো। যন্ত্র চালাবে না।", "অনুষ্ঠান"),
    "gu": ("વપરાકર્તા કેલેન્ડર વિશે પૂછે છે. નીચે ઘટનાઓ સ્વાભાવિક અને ટૂંકામા કહો. ઘટના ઘડશો નહી. યાદી ખાલી હોય તો સ્પષ્ટ કહો. ઉપકરણો ચલાવશો નહી.", "ઘટનાઓ"),
    "kn": ("ಬಳಕೆದಾರ ಕ್ಯಾಲೆಂಡರ ಕುರಿತು ಇದ್ದಾರೆ. ಕೆಳಗಿನ ಘಟನೆಗಳನ್ನು ಸಹಜವಾಗಿ, ಚಿಕ್ಕದಾಗಿ ಹೇಳಿ. ಘಟನೆ ಕಲ್પಿಸಬೇಡಿ. ಪಟ್ಟಿ ಖಾಲಿ ಇದ್ದರೆ ಸ್ಪಷ್ಟವಾಗಿ ಹೇಳಿ. ಸಾಧನಗಳನ್ನು ನಿಯಂತ್ರಿಸಬೇಡಿ.", "ಘಟನೆಗಳು"),
    "ml": ("ഉപയോക്താവ് കലണ്ടറിനെക്കുറിച്ച് ചോദിക്കുന്നു. താഴെയുള്ള ഇവന്റുകൾ സ്വാഭാവികമായും ചുരുക്കിയും പറയുക. ഇവന്റ് ഉണ്ടാക്കരുത്. ലിസ്റ്റ് ശൂന്യമെങ്കിൽ വ്യക്തമായി പറയുക. ഉപകരണങ്ങൾ നിയന്ത്രിക്കരുത്.", "ഇവന്റുകൾ"),
    "mr": ("वापरकर्ता दिनदर्शिकेबद्दल विचारतो. खालील कार्यक्रम नैसर्गिक व थोडक्यात सांगा. कार्यक्रम रचू नका. यादी रिकामी असेल तर स्पष्ट सांगा. उपकरणे चालवू नका.", "कार्यक्रम"),
    "ta": ("பயனர் நாட்காட்டி பற்றி கேட்கிறார். பின்வரும் நிகழ்வுகளை இயல்பாகவும் சுருக்கமாகவும் சொல்லுங்கள். நிகழ்வை உருவாக்க வேண்டாம். பட்டியல் காலியாக இருந்தால் தெளிவாகச் சொல்லுங்கள். சாதனங்களைக் கட்டுப்படுத்த வேண்டாம்.", "நிகழ்வுகள்"),
    "te": ("వాడకుడు క్యాలెండర్ గురించి అడుగుతున్నారు. కింది సంఘటనాలను సహజంగా, సంక్షిప్తంగా చెప్పండి. సంఘటనాలను కల్పించవద్దు. జాబితా ఖాళిగా ఉంటే స్పష్టంగా చెప్పండి. పరికరాలను నియంత్రించవద్దు.", "సంఘటనాలు"),
    "pa": ("ਵਰਤੋਂ ਅਪਣੇ ਕੈਲੰਡਰ ਬਾਰੇ ਪੁੱਛ ਰਿਹਾ ਹੈ. ਹੇਠਲੀਆਂ ਘਟਨਾਵਾਂ ਕੁਦਰਤੀ ਤੇ ਛੋਟੀਆਂ ਦੱਸੋ. ਘਟਨਾ ਨਾ ਘੜੋ. ਸੂਚੀ ਖਾਲੀ ਹੋਵੇ ਤਾਂ ਸਾਫ ਦੱਸੋ. ਜੰਤਰ ਨਾ ਚਲਾਓ.", "ਘਟਨਾਵਾਂ"),
    "ne": ("प्रयोगकर्ताले पात्रोबारे सोध्दै छ। तलका कार्यक्रम स्वाभाविक र छोटो भन्नुहोस्। कार्यक्रम नबनाउनुहोस्। सूची खाली भए स्पष्ट भन्नुहोस्। उपकरण नचलाउनुहोस्।", "कार्यक्रम"),
    "hy": ("Օգտատերը հարցնում է օրացույցի մասին։ Հաջորդ իրադարձությունները ասա բնական ու կարճ։ Իրադարձություններ մի հորինիր։ Եթե ցուցակը դատարկ էՌ ասա հստակ։ Սարքեր մի կառավարիր։", "Իրադարձություններ"),
    "ka": ("მომხმარებელი კალენდარს ეკითხება. თქვი შემდეგი მოვლენები ბუნებრივად და მოკლედ. ნუ მოიგონებ მოვლენებს. თუ სია ცარიელია, თქვი მკაფიოდ. ნუ მართავ მოწყობილობებს.", "მოვლენები"),
    "mn": ("Хэрэглэгч хуанлигаа асууж байна. Дараах үйл явдлуудыг байгалийн, товч хэл. Үйл явдал зохиож болохгүй. Жагсаалт хоосон бол тодорхой хэл. Төхөөрөмж бүү удирд.", "Үйл явдал"),
}


def llm_copy(pack: str) -> tuple[str, str]:
    return LLM.get(pack) or LLM["en"]


_KEYS = ("calendar_item", "calendar_created", "calendar_deleted", "calendar_moved", "calendar_empty", "calendar_today", "calendar_tomorrow", "calendar_all_day", "calendar_at")


def templates(pack: str) -> dict[str, str]:
    row = SAY.get(pack) or SAY.get("en")
    out = {key: row[index] for index, key in enumerate(_KEYS)}
    out["calendar_clock"] = row[9]
    return out


def overlay(pack: str, base: dict[str, str]) -> dict[str, str]:
    merged = dict(base)
    merged.update(templates(pack))
    return merged


def fill(pack: str, key: str, **slots: str) -> str:
    template = templates(pack).get(key) or ""
    for name, value in slots.items():
        template = template.replace(f"{{{name}}}", value)
    return template.strip()


def _zone(hass: Any) -> ZoneInfo:
    name = str(getattr(getattr(hass, "config", None), "time_zone", None) or "UTC")
    try:
        return ZoneInfo(name)
    except Exception:  # noqa: BLE001
        return ZoneInfo("UTC")


def _as_start(event: dict[str, Any]) -> datetime | date | None:
    raw = event.get("start") or event.get("start_date_time") or event.get("start_date")
    if isinstance(raw, dict):
        raw = raw.get("dateTime") or raw.get("date")
    if isinstance(raw, (datetime, date)):
        return raw
    text = str(raw or "")
    if not text:
        return None
    try:
        return datetime.fromisoformat(text.replace("Z", "+00:00"))
    except ValueError:
        try:
            return date.fromisoformat(text[:10])
        except ValueError:
            return None


def is_all_day(event: dict[str, Any], start: datetime | date | None) -> bool:
    raw = event.get("start")
    if isinstance(raw, dict) and raw.get("date") and not raw.get("dateTime"):
        return True
    if isinstance(start, date) and not isinstance(start, datetime):
        return True
    stamp = str(event.get("start") or "")
    return "T" not in stamp and len(stamp) >= 8 and isinstance(start, date)


def clock(when: datetime, pack: str) -> str:
    kind = templates(pack)["calendar_clock"]
    hour24 = when.hour
    minute = when.minute
    if kind == "12":
        hour = when.strftime("%I").lstrip("0") or "12"
        suffix = when.strftime("%p").lstrip()
        return f"{hour} {suffix}" if minute == 0 else f"{hour}:{when:%M} {suffix}"
    if kind == "de":
        return f"{hour24} Uhr" if minute == 0 else f"{hour24}:{when:%M} Uhr"
    if kind == "nl":
        return f"{hour24} uur" if minute == 0 else f"{hour24}:{when:%M}"
    if kind == "nord":
        return f"{hour24}" if minute == 0 else f"{hour24}:{when:%M}"
    if kind == "fr":
        return f"{hour24} heures" if minute == 0 else f"{hour24} h {when:%M}"
    if kind in {"es", "it", "pt"}:
        return f"{hour24}" if minute == 0 else f"{hour24}:{when:%M}"
    if kind == "ja":
        return f"{hour24}時" if minute == 0 else f"{hour24}時{minute}分"
    if kind == "zh":
        return f"{hour24}点" if minute == 0 else f"{hour24}点{minute}分"
    if kind == "ko":
        return f"{hour24}시" if minute == 0 else f"{hour24}시 {minute}분"
    if kind == "th":
        return f"{hour24} น." if minute == 0 else f"{hour24}:{when:%M} น."
    return f"{hour24}:{when:%M}"


def when_label(start: datetime | date, all_day: bool, pack: str, hass: Any) -> str:
    today = datetime.now(_zone(hass)).date()
    day = start.date() if isinstance(start, datetime) else start
    if day == today:
        day_word = fill(pack, "calendar_today")
    elif day == today + timedelta(days=1):
        day_word = fill(pack, "calendar_tomorrow")
    else:
        day_word = day.isoformat()
    if all_day:
        whole = fill(pack, "calendar_all_day")
        if pack.startswith("zh") or pack in {"ja", "th"}:
            return f"{day_word}{whole}"
        return f"{day_word} {whole}".strip()
    stamp = start if isinstance(start, datetime) else datetime.combine(start, datetime.min.time(), tzinfo=_zone(hass))
    if stamp.tzinfo is None:
        stamp = stamp.replace(tzinfo=_zone(hass))
    else:
        stamp = stamp.astimezone(_zone(hass))
    at = fill(pack, "calendar_at", time=clock(stamp, pack))
    return f"{day_word} {at}".strip()


def event_line(event: dict[str, Any], pack: str, hass: Any) -> str:
    summary = str(event.get("summary") or event.get("title") or "").strip()
    start = _as_start(event)
    if start is None:
        return summary
    all_day = is_all_day(event, start)
    when = when_label(start, all_day, pack, hass)
    if not summary:
        return when
    return fill(pack, "calendar_item", summary=summary, when=when)


def list_speech(events: list[dict[str, Any]], pack: str, hass: Any) -> str:
    if not events:
        return fill(pack, "calendar_empty")
    lines = [event_line(event, pack, hass) for event in events[:8]]
    return ". ".join(line.rstrip(".") for line in lines if line) + "."


def when_from_bounds(start: datetime, all_day: bool, pack: str, hass: Any) -> str:
    return when_label(start, all_day, pack, hass)
