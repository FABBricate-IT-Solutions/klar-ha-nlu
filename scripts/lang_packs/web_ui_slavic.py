"""Operator UI chrome for Slavic and Hungarian Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_table

CODES = ["cs", "sk", "pl", "hu", "hr", "sl", "bg", "sr", "sr-Latn", "uk"]

TABLE = """
home	Domov	Domov	Start	Kezdőlap	Početna	Domov	Начало	Почетна	Početna	Головна
conversations	Konverzace	Konverzácie	Rozmowy	Beszélgetések	Razgovori	Pogovori	Разговори	Разговори	Razgovori	Розмови
rules	Pravidla	Pravidlá	Reguły	Szabályok	Pravila	Pravila	Правила	Правила	Pravila	Правила
house	Dům	Dom	Dom	Ház	Kuća	Hiša	Къща	Кућа	Kuća	Дім
lab	Labor	Labor	Lab	Labor	Lab	Lab	Лаб	Лаб	Lab	Лаб
graph	Graf	Graf	Graf	Graf	Graf	Graf	Граф	Граф	Graf	Граф
calibrate	Mapování	Mapovanie	Mapowanie	Leképezés	Mapiranje	Preslikava	Съответствия	Мапирање	Mapiranje	Відповідності
entities	Zařízení	Zariadenia	Urządzenia	Eszközök	Uređaji	Naprave	Устройства	Уређаји	Uređaji	Пристрої
custom	Fráze	Frázy	Frazy	Mondatok	Fraze	Fraze	Фрази	Фразе	Fraze	Фрази
settings	Nastavení	Nastavenia	Ustawienia	Beállítások	Postavke	Nastavitve	Настройки	Подешавања	Podešavanja	Налаштування
open	otevřeno	otvorené	otwarte	nyitva	otvoreno	odprto	отворено	отворено	otvoreno	відкрито
bundleOn	Bundle zapnuto	Bundle zapnuté	Bundle włączony	Bundle be	Bundle uključeno	Bundle vklopljen	Bundle включен	Bundle укључен	Bundle uključen	Bundle увімкнено
bundleOff	Bundle vypnuto	Bundle vypnuté	Bundle wyłączony	Bundle ki	Bundle isključeno	Bundle izklopljen	Bundle изключен	Bundle искључен	Bundle isključen	Bundle вимкнено
engineReady	Engine připraven	Engine pripravený	Engine gotowy	Engine kész	Engine spreman	Engine pripravljen	Engine готов	Engine спреман	Engine spreman	Engine готовий
understandsHome	Proč Klar provedl, potvrdil nebo zastavil	Prečo Klar vykonal, potvrdil alebo zastavil	Dlaczego Klar wykonał, potwierdził albo zatrzymał	Miért hajtott végre, erősített meg vagy állt meg a Klar	Zašto je Klar izvršio, potvrdio ili zaustavio	Zakaj je Klar izvedel, potrdil ali ustavil	Защо Klar изпълни, потвърди или спря	Зашто је Klar извршио, потврдио или зауставио	Zašto je Klar izvršio, potvrdio ili zaustavio	Чому Klar виконав, підтвердив або зупинив
assistVisible	Assist viditelný	Assist viditeľný	Assist widoczny	Assist látható	Assist vidljiv	Assist viden	Assist видим	Assist видљив	Assist vidljiv	Assist видимий
certain	jisté	isté	pewne	biztos	sigurno	zanesljivo	сигурно	сигурно	sigurno	певно
needsWork	potřebuje práci	potrebuje prácu	wymaga pracy	javítani kell	treba rad	potrebuje delo	трябва работа	треба рад	treba rad	потребує роботи
recordings	Záznamy	Záznamy	Nagrania	Felvételek	Snimke	Posnetki	Записи	Снимци	Snimci	Записи
processed	zpracováno	spracované	przetworzone	feldolgozva	obrađeno	obdelano	обработено	обрађено	obrađeno	оброблено
coverage	Pokrytí	Pokrytie	Pokrycie	Lefedettség	Pokrivenost	Pokritost	Покритие	Покривеност	Pokrivenost	Покриття
confidence	Jistota	Istota	Pewność	Bizonyosság	Pouzdanost	Zanesljivost	Сигурност	Поузданост	Pouzdanost	Впевненість
domains	Domény	Domény	Domeny	Tartományok	Domene	Domene	Домейни	Домени	Domeni	Домени
rooms	Místnosti	Miestnosti	Pomieszczenia	Szobák	Prostorije	Prostori	Стаи	Просторије	Prostorije	Кімнати
recent	Poslední věty	Posledné vety	Ostatnie zdania	Legutóbbi mondatok	Nedavne rečenice	Zadnji stavki	Последни изречения	Недавне реченице	Nedavne rečenice	Останні речення
replay	Přehrát	Prehrať	Odtwórz	Újrajátszás	Ponovi	Predvajaj	Повтори	Понови	Ponovi	Повторити
applyAll	Použít návrhy	Použiť návrhy	Zastosuj sugestie	Javaslatok alkalmazása	Primijeni prijedloge	Uporabi predloge	Приложи предложения	Примени предлоге	Primeni predloge	Застосувати пропозиції
undo	Zpět	Späť	Cofnij	Visszavonás	Poništi	Razveljavi	Отмени	Опозови	Opozovi	Скасувати
accept	Přijmout	Prijať	Zaakceptuj	Elfogadás	Prihvati	Sprejmi	Приеми	Прихвати	Prihvati	Прийняти
otherRoom	Jiná místnost	Iná miestnosť	Inny pokój	Másik szoba	Druga prostorija	Drug prostor	Друга стая	Друга просторија	Druga prostorija	Інша кімната
dismiss	Zahodit	Zahodiť	Odrzuć	Elvetés	Odbaci	Opusti	Отхвърли	Одбаци	Odbaci	Відхилити
noGaps	Žádná otevřená mapování.	Žiadne otvorené mapovania.	Brak otwartych mapowań.	Nincs nyitott leképezés.	Nema otvorenih mapiranja.	Ni odprtih preslikav.	Няма отворени съответствия.	Нема отворених мапирања.	Nema otvorenih mapiranja.	Немає відкритих відповідностей.
unmapped	Bez místnosti	Bez miestnosti	Bez pomieszczenia	Nincs szoba	Bez prostorije	Brez prostora	Без стая	Без просторије	Bez prostorije	Без кімнати
parseHint	Spouštěče vět běží v Home Assistant před tímto parse. conversation.process: trigger, pak Klar, pak intent_script.	Spúšťače viet bežia v Home Assistant pred týmto parse. conversation.process: trigger, potom Klar, potom intent_script.	Wyzwalacze zdań działają w Home Assistant przed tym parse. conversation.process: trigger, potem Klar, potem intent_script.	A mondatindítók a Home Assistantben futnak ezen parse előtt. conversation.process: trigger, aztán Klar, aztán intent_script.	Okidači rečenica rade u Home Assistant prije ovog parse. conversation.process: trigger, zatim Klar, zatim intent_script.	Sprožilci stavkov tečejo v Home Assistant pred tem parse. conversation.process: trigger, nato Klar, nato intent_script.	Тригерите за изречения се изпълняват в Home Assistant преди този parse. conversation.process: trigger, после Klar, после intent_script.	Окидачи реченица раде у Home Assistant пре овог parse. conversation.process: trigger, затим Klar, затим intent_script.	Okidači rečenica rade u Home Assistant pre ovog parse. conversation.process: trigger, zatim Klar, zatim intent_script.	Тригери речень виконуються в Home Assistant перед цим parse. conversation.process: trigger, потім Klar, потім intent_script.
command	Příkaz	Príkaz	Polecenie	Parancs	Naredba	Ukaz	Команда	Команда	Komanda	Команда
analyze	Analyzovat	Analyzovať	Analizuj	Elemzés	Analiziraj	Analiziraj	Анализирай	Анализирај	Analiziraj	Аналізувати
raw	Holé	Holé	Surowe	Nyers	Sirovo	Surovo	Сурово	Сирово	Sirovo	Сире
speech	Řeč	Reč	Mowa	Beszéd	Govor	Govor	Реч	Говор	Govor	Мовлення
intent	Intent	Intent	Intent	Intent	Intent	Intent	Intent	Intent	Intent	Intent
slots	Slots	Slots	Slots	Slots	Slots	Slots	Slots	Slots	Slots	Slots
searchDevice	Jak se tomu říká?	Ako sa tomu hovorí?	Jak to nazywasz?	Minek hívnád?	Kako bi to zvali?	Kako bi temu rekli?	Как бихте го нарекли?	Како бисте то назвали?	Kako biste to nazvali?	Як би ви це назвали?
alias	Alias	Alias	Alias	Alias	Alias	Alias	Alias	Alias	Alias	Alias
room	Místnost	Miestnosť	Pokój	Szoba	Prostorija	Prostor	Стая	Просторија	Prostorija	Кімната
preferred	Výchozí světlo	Predvolené svetlo	Domyślne światło	Alapértelmezett fény	Zadano svjetlo	Privzeta luč	Стандартна светлина	Подразумевано светло	Podrazumevano svetlo	Типове світло
save	Uložit	Uložiť	Zapisz	Mentés	Spremi	Shrani	Запази	Сачувај	Sačuvaj	Зберегти
personality	Osobnost	Osobnosť	Osobowość	Személyiség	Osobnost	Osebnost	Личност	Личност	Ličnost	Особистість
mode	Režim	Režim	Tryb	Mód	Način	Način	Режим	Режим	Režim	Режим
supportBundle	Support bundle	Support bundle	Support bundle	Support bundle	Support bundle	Support bundle	Support bundle	Support bundle	Support bundle	Support bundle
recordProtocol	Nahrávat protokol	Nahrávať protokol	Nagrywaj protokół	Protokoll rögzítése	Snimi protokol	Snemaj protokol	Записвай протокол	Снимај протокол	Snimaj protokol	Записувати протокол
includeRawText	Zahrnout surový text do stažení	Zahrnúť surový text do sťahovaní	Dołącz surowy tekst do pobrań	Nyers szöveg a letöltésekben	Uključi sirovi tekst u preuzimanja	Vključi surovo besedilo v prenose	Включи суровия текст в изтеглянията	Укључи сирови текст у преузимања	Uključi sirovi tekst u preuzimanja	Включати сирий текст у завантаження
semanticAdapters	Lokální sémantické adaptéry	Lokálne sémantické adaptéry	Lokalne adaptery semantyczne	Helyi szemantikai adapterek	Lokalni semantički adapteri	Lokalni semantični adapterji	Локални семантични адаптери	Локални семантички адаптери	Lokalni semantički adapteri	Локальні семантичні адаптери
downloadDataset	Stáhnout dataset	Stiahnuť dataset	Pobierz dataset	Dataset letöltése	Preuzmi dataset	Prenesi dataset	Изтегли dataset	Преузми dataset	Preuzmi dataset	Завантажити dataset
downloadProtocol	Stáhnout protokol	Stiahnuť protokol	Pobierz protokół	Protokoll letöltése	Preuzmi protokol	Prenesi protokol	Изтегли протокол	Преузми протокол	Preuzmi protokol	Завантажити протокол
deleteSelected	Smazat výběr	Zmazať výber	Usuń zaznaczenie	Kijelölés törlése	Izbriši odabir	Izbriši izbor	Изтрий избраното	Обриши избор	Obriši izbor	Видалити вибране
clearAll	Smazat vše	Zmazať všetko	Usuń wszystko	Összes törlése	Izbriši sve	Izbriši vse	Изтрий всички	Обриши све	Obriši sve	Видалити все
token	Zapisovací token (LAN)	Zapisovací token (LAN)	Token zapisu (LAN)	Írási token (LAN)	Token za pisanje (LAN)	Žeton za pisanje (LAN)	Токен за запис (LAN)	Токен за упис (LAN)	Token za upis (LAN)	Токен запису (LAN)
customJson	Vlastní fráze jako JSON	Vlastné frázy ako JSON	Własne frazy jako JSON	Egyéni mondatok JSON-ként	Vlastite fraze kao JSON	Lastne fraze kot JSON	Собствени фрази като JSON	Сопствене фразе као JSON	Sopstvene fraze kao JSON	Власні фрази як JSON
customHint	Fráze na známý intent. Policies jsou vedle, ne jako HA automatizace.	Fráza na známy intent. Policies sú vedľa, nie ako HA automatizácie.	Fraza na znany intent. Policies są obok, nie jako automatyzacje HA.	Mondat egy ismert intentre. A Policies mellettük vannak, nem HA automatizálásként.	Fraza na poznati intent. Policies su pokraj, ne kao HA automatizacije.	Fraza na znan intent. Policies so zraven, ne kot HA avtomatizacije.	Фраза към известен intent. Policies са до това, не като HA автоматизации.	Фраза на познати intent. Policies су поред, не као HA аутоматизације.	Fraza na poznati intent. Policies su pored, ne kao HA automatizacije.	Фраза на відомий intent. Policies поруч, не як автоматизації HA.
addPhrase	Přidat frázi	Pridať frázu	Dodaj frazę	Mondat hozzáadása	Dodaj frazu	Dodaj frazo	Добави фраза	Додај фразу	Dodaj frazu	Додати фразу
previewRule	Náhled	Náhľad	Podgląd	Előnézet	Pregled	Predogled	Преглед	Преглед	Pregled	Перегляд
explainRule	Vysvětlit	Vysvetliť	Wyjaśnij	Magyarázat	Objasni	Razloži	Обясни	Објасни	Objasni	Пояснити
rollback	Vrátit zpět	Vrátiť späť	Wycofaj	Visszagörgetés	Vrati	Povrni	Върни назад	Врати	Vrati	Відкотити
noRules	Zatím žádné vlastní fráze.	Zatiaľ žiadne vlastné frázy.	Brak własnych fraz.	Még nincsenek egyéni mondatok.	Još nema vlastitih fraza.	Še ni lastnih fraz.	Все още няма собствени фрази.	Још нема сопствених фраза.	Još nema sopstvenih fraza.	Ще немає власних фраз.
engineOffline	Engine je nedostupný. Seznam je prázdný, dokud se nepodaří živé načtení.	Engine je nedostupný. Zoznam je prázdny, kým sa nepodarí živé načítanie.	Silnik jest nieosiągalny. Lista jest pusta, dopóki nie uda się wczytać na żywo.	A motor nem elérhető. A lista üres, amíg egy élő betöltés sikerül.	Motor je nedostupan. Popis je prazan dok se živo učitavanje ne uspije.	Motor je nedosegljiv. Seznam je prazen, dokler živo nalaganje ne uspe.	Двигателят е недостъпен. Списъкът е празен, докато живото зареждане не успее.	Мотор је недоступан. Списак је празан док се живо учитавање не успе.	Motor je nedostupan. Spisak je prazan dok se živo učitavanje ne uspe.	Рушій недоступний. Список порожній, поки не вдасться живе завантаження.
emptyBundle	Zatím žádné záznamy. Zapněte bundle a zkuste větu.	Zatiaľ žiadne záznamy. Zapnite bundle a skúste vetu.	Brak nagrań. Włącz bundle i spróbuj zdania.	Még nincs felvétel. Kapcsold be a bundle-t, és próbálj egy mondatot.	Još nema snimki. Uključite bundle i isprobajte rečenicu.	Še ni posnetkov. Vklopite bundle in poskusite stavek.	Все още няма записи. Включете bundle и опитайте изречение.	Још нема снимака. Укључите bundle и пробајте реченицу.	Još nema snimaka. Uključite bundle i probajte rečenicu.	Ще немає записів. Увімкніть bundle і спробуйте речення.
confirmApply	Použít tyto návrhy?	Použiť tieto návrhy?	Zastosować te sugestie?	Alkalmazod ezeket a javaslatokat?	Primijeniti ove prijedloge?	Uporabim te predloge?	Да се приложат тези предложения?	Применити ове предлоге?	Primeniti ove predloge?	Застосувати ці пропозиції?
cancel	Zrušit	Zrušiť	Anuluj	Mégse	Odustani	Prekliči	Отказ	Откажи	Otkaži	Скасувати
apply	Použít	Použiť	Zastosuj	Alkalmaz	Primijeni	Uporabi	Приложи	Примени	Primeni	Застосувати
close	Zavřít	Zavrieť	Zamknij	Bezárás	Zatvori	Zapri	Затвори	Затвори	Zatvori	Закрити
low	nízká	nízka	niski	alacsony	nisko	nizko	ниско	ниско	nisko	низька
medium	střední	stredná	średni	közepes	srednje	srednje	средно	средње	srednje	середня
high	vysoká	vysoká	wysoki	magas	visoko	visoko	високо	високо	visoko	висока
source	Zdroj	Zdroj	Źródło	Forrás	Izvor	Vir	Източник	Извор	Izvor	Джерело
language	Jazyk	Jazyk	Język	Nyelv	Jezik	Jezik	Език	Језик	Jezik	Мова
time	Čas	Čas	Czas	Idő	Vrijeme	Čas	Време	Време	Vreme	Час
text	Věta	Veta	Zdanie	Mondat	Rečenica	Stavek	Изречение	Реченица	Rečenica	Речення
answer	Odpověď	Odpoveď	Odpowiedź	Válasz	Odgovor	Odgovor	Отговор	Одговор	Odgovor	Відповідь
graphHint	Místnosti jako shluky, zařízení podle jistoty.	Miestnosti ako zhluky, zariadenia podľa istoty.	Pomieszczenia jako klastry, urządzenia według pewności.	Szobák klaszterként, eszközök a bizonyosság szerint.	Prostorije kao skupine, uređaji po pouzdanosti.	Prostori kot gruče, naprave po zanesljivosti.	Стаи като клъстери, устройства по сигурност.	Просторије као кластери, уређаји по поузданости.	Prostorije kao klasteri, uređaji po pouzdanosti.	Кімнати як кластери, пристрої за впевненістю.
resetLayout	Obnovit rozložení	Obnoviť rozloženie	Resetuj układ	Elrendezés visszaállítása	Resetiraj raspored	Ponastavi razporeditev	Нулирай подредбата	Ресетуј распоред	Resetuj raspored	Скинути розташування
score	Score	Score	Score	Score	Score	Score	Score	Score	Score	Score
noIntent	Žádný intent	Žiadny intent	Brak intent	Nincs intent	Nema intenta	Ni intenta	Няма intent	Нема intent-а	Nema intent-a	Немає intent
loading	Načítání Klar...	Načítava sa Klar...	Wczytywanie Klar...	Klar betöltése...	Učitavanje Klar...	Nalaganje Klar...	Зареждане на Klar...	Учитавање Klar...	Učitavanje Klar...	Завантаження Klar...
nluRagHint	Ve výchozím stavu vypnuto. Jen shodný úsek, nikdy nástroje Assist.	Predvolene vypnuté. Len zhodný úsek, nikdy nástroje Assist.	Domyślnie wyłączone. Tylko dopasowany fragment, nigdy narzędzia Assist.	Alapból ki. Csak az illeszkedő szelet, soha Assist-eszközök.	Zadano isključeno. Samo podudarni odsječak, nikad alati Assist.	Privzeto izklopljeno. Samo ujemajoči se del, nikoli orodja Assist.	По подразбиране изключено. Само съвпадналият отрязък, никога инструменти на Assist.	Подразумевано искључено. Само подударни одсечак, никад алати Assist.	Podrazumevano isključeno. Samo podudarni odsečak, nikad alati Assist.	Типово вимкнено. Лише збіжний фрагмент, ніколи інструменти Assist.
confirmRisky	Potvrzovat rizikové akce	Potvrdzovať rizikové akcie	Potwierdzaj ryzykowne działania	Kockázatos műveletek megerősítése	Potvrdi rizične radnje	Potrdi tvegana dejanja	Потвърждавай рискови действия	Потврди ризичне радње	Potvrdi rizične radnje	Підтверджувати ризиковані дії
languages	Jazyky	Jazyky	Języki	Nyelvek	Jezici	Jeziki	Езици	Језици	Jezici	Мови
languageSearch	Hledat jazyky	Hľadať jazyky	Szukaj języków	Nyelvek keresése	Pretraži jezike	Iskanje jezikov	Търси езици	Претражи језике	Pretraži jezike	Шукати мови
allLanguages	Všechny jazyky	Všetky jazyky	Wszystkie języki	Minden nyelv	Svi jezici	Vsi jeziki	Всички езици	Сви језици	Svi jezici	Усі мови
noLanguageMatch	Žádný jazyk nenalezen	Nenašiel sa žiadny jazyk	Nie znaleziono języka	Nincs ilyen nyelv	Jezik nije pronađen	Jezik ni najden	Няма намерен език	Језик није пронађен	Jezik nije pronađen	Мову не знайдено
languageHint	Hledejte a vyberte locales. Všechny jazyky nechá každý zkompilovaný pack zapnutý.	Hľadajte a vyberte locales. Všetky jazyky nechá každý skompilovaný pack zapnutý.	Szukaj i wybierz locales. Wszystkie języki zostawia każdy skompilowany pack włączony.	Keress és válassz locales. Minden nyelv minden lefordított packot bekapcsolva tart.	Tražite i odaberite locales. Svi jezici ostavljaju svaki kompajlirani pack uključen.	Iščite in izberite locales. Vsi jeziki ohranijo vsak preveden pack vklopljen.	Търсете и избирайте locales. Всички езици оставят всеки компилиран pack включен.	Тражите и изаберите locales. Сви језици остављају сваки компајлирани pack укључен.	Tražite i izaberite locales. Svi jezici ostavljaju svaki kompajlirani pack uključen.	Шукайте й обирайте locales. Усі мови лишають кожен скомпільований pack увімкненим.
mappingHint	Mapování jsou aliasy entit v grafu. Kalendáře se objeví po zahrnutí domény calendar. Assist následuje jazykový pack; toto rozhraní následuje jazyk operátora.	Mapovania sú aliasy entít v grafe. Kalendáre sa objavia po zahrnutí domény calendar. Assist nasleduje jazykový pack; toto rozhranie nasleduje jazyk operátora.	Mapowanie to aliasy encji grafu. Kalendarze pojawiają się po dołączeniu domeny calendar. Assist idzie za pakietem językowym; ten interfejs za językiem operatora.	A leképezés a gráf entitások aliasai. A naptárak a calendar tartomány felvétele után jelennek meg. Az Assist a nyelvi packot követi; ez a felület az operátor nyelvét követi.	Mapiranje su aliasi entiteta grafa. Kalendari se pojavljuju nakon uključivanja domene calendar. Assist prati jezični pack; ovo sučelje prati jezik operatora.	Preslikava so vzdevki entitet grafa. Koledarji se pokažejo po vključitvi domene calendar. Assist sledi jezikovnemu packu; ta vmesnik sledi jeziku operaterja.	Съответствията са псевдоними на обекти в графа. Календарите се появяват след включване на домейна calendar. Assist следва езиковия pack; този интерфейс следва езика на оператора.	Мапирање су алијаси ентитета графа. Календари се појављују после укључивања домена calendar. Assist прати језички pack; овај интерфејс прати језик оператера.	Mapiranje su alijasi entiteta grafa. Kalendari se pojavljuju posle uključivanja domena calendar. Assist prati jezički pack; ovaj interfejs prati jezik operatera.	Відповідності — аліаси сутностей графа. Календарі з’являються після включення домену calendar. Assist іде за мовним pack; цей інтерфейс іде за мовою оператора.
parseSample	Zapni světlo v obýváku	Zapni svetlo v obývačke	Włącz światło w salonie	Kapcsold fel a nappali villanyt	Upali svjetlo u dnevnom	Prižgi luč v dnevni	Включи светлината в хола	Укључи светло у дневној	Uključi svetlo u dnevnoj	Увімкни світло у вітальні
tryOn	Zapni světlo v {room}	Zapni svetlo v {room}	Włącz światło w {room}	Kapcsold fel a villanyt {room}	Upali svjetlo u {room}	Prižgi luč v {room}	Включи светлината в {room}	Укључи светло у {room}	Uključi svetlo u {room}	Увімкни світло в {room}
tryLock	Jsou dveře zamčené?	Sú dvere zamknuté?	Czy drzwi są zamknięte?	Be van zárva az ajtó?	Jesu li vrata zaključana?	So vrata zaklenjena?	Вратата заключена ли е?	Да ли су врата закључана?	Da li su vrata zaključana?	Двері замкнені?
tryTime	Kolik je hodin?	Koľko je hodín?	Która jest godzina?	Hány óra van?	Koliko je sati?	Koliko je ura?	Колко е часът?	Колико је сати?	Koliko je sati?	Котра година?
tryNight	Dobrou noc	Dobrú noc	Dobranoc	Jó éjszakát	Laku noć	Lahko noč	Лека нощ	Лаку ноћ	Laku noć	На добраніч
tryUndo	Vrať to	Vráť to	Cofnij to	Vond vissza	Poništi to	Razveljavi to	Отмени това	Опозови то	Opozovi to	Скасуй це
tryRoom	kuchyně	kuchyňa	kuchnia	a konyha	kuhinja	kuhinja	кухнята	кухиња	kuhinja	кухня
nluIgnore	Nevázat pro stav ani spínání	Neväzať pre stav ani spínanie	Nie wiąż dla stanu ani zasilania	Ne kösd állapothoz vagy kapcsoláshoz	Ne veži za status ili napajanje	Ne veži za stanje ali vklop	Не свързвай за състояние или захранване	Не везуј за статус или напајање	Ne vezuj za status ili napajanje	Не прив’язуй для стану чи живлення
nluIgnoreHint	Vyřadí zařízení z resolveru. Pro špatně pojmenované helpery.	Vyradí zariadenie z resolveru. Pre zle pomenované helpery.	Usuwa urządzenie z resolvera. Do źle nazwanych helperów.	Kiveszi az eszközt a resolverből. Rosszul elnevezett helperekhez.	Izbacuje uređaj iz resolvera. Za krivo imenovane helpere.	Odstrani napravo iz resolverja. Za napačno poimenovane helperje.	Маха устройството от resolver-а. За грешно именувани helper-и.	Избацује уређај из resolver-а. За погрешно именоване helper-е.	Izbacuje uređaj iz resolver-a. Za pogrešno imenovane helper-e.	Прибирає пристрій із resolver. Для погано названих helperів.
savePhrase	Uložit jako frázi	Uložiť ako frázu	Zapisz jako frazę	Mentés mondatként	Spremi kao frazu	Shrani kot frazo	Запази като фраза	Сачувај као фразу	Sačuvaj kao frazu	Зберегти як фразу
ignoreTarget	Ignorovat tento cíl	Ignorovať tento cieľ	Ignoruj ten cel	E cél mellőzése	Zanemari ovaj cilj	Prezri ta cilj	Игнорирай тази цел	Игнориши овај циљ	Ignoriši ovaj cilj	Ігнорувати цю ціль
teachSaved	Uloženo.	Uložené.	Zapisano.	Mentve.	Spremljeno.	Shranjeno.	Запазено.	Сачувано.	Sačuvano.	Збережено.
journal	Deník konverzací	Denník konverzácií	Dziennik rozmów	Beszélgetési napló	Dnevnik razgovora	Dnevnik pogovorov	Дневник на разговорите	Дневник разговора	Dnevnik razgovora	Журнал розмов
journalHint	Posledních 200 turns, 24 hodin, redigováno. Surový text jen s bundle.	Posledných 200 turns, 24 hodín, redigované. Surový text len s bundle.	Ostatnie 200 turns, 24 godziny, zredagowane. Surowy tekst tylko z bundle.	Utolsó 200 turns, 24 óra, szerkesztve. Nyers szöveg csak a bundle-lel.	Zadnjih 200 turns, 24 sata, redigirano. Sirovi tekst samo uz bundle.	Zadnjih 200 turns, 24 ur, redigirano. Surovo besedilo samo z bundle.	Последните 200 turns, 24 часа, редактирано. Суров текст само с bundle.	Последњих 200 turns, 24 сата, редиговано. Сирови текст само уз bundle.	Poslednjih 200 turns, 24 sata, redigovano. Sirovi tekst samo uz bundle.	Останні 200 turns, 24 години, відредаговано. Сирий текст лише з bundle.
decisionMix	Rozhodnutí	Rozhodnutia	Decyzje	Döntések	Odluke	Odločitve	Решения	Одлуке	Odluke	Рішення
mixCaption	Zdroj: deník konverzací, turns za den	Zdroj: denník konverzácií, turns za deň	Źródło: dziennik rozmów, turns dziennie	Forrás: beszélgetési napló, turns naponta	Izvor: dnevnik razgovora, turns na dan	Vir: dnevnik pogovorov, turns na dan	Източник: дневник на разговорите, turns на ден	Извор: дневник разговора, turns на дан	Izvor: dnevnik razgovora, turns na dan	Джерело: журнал розмов, turns на день
coverageCaption	Zdroj: graf domácnosti, podíl zařízení	Zdroj: graf domácnosti, podiel zariadení	Źródło: graf domu, udział urządzeń	Forrás: otthoni gráf, eszközök aránya	Izvor: graf kuće, udio uređaja	Vir: graf doma, delež naprav	Източник: граф на дома, дял на устройствата	Извор: граф дома, удео уређаја	Izvor: graf doma, udeo uređaja	Джерело: граф дому, частка пристроїв
latency	Čas fáze	Čas fázy	Czas etapu	Szakaszidő	Vrijeme faze	Čas stopnje	Време на етапа	Време фазе	Vreme faze	Час етапу
latencyCaption	Zdroj: parse trace, mikrosekundy	Zdroj: parse trace, mikrosekundy	Źródło: parse trace, mikrosekundy	Forrás: parse trace, mikroszekundum	Izvor: parse trace, mikrosekunde	Vir: parse trace, mikrosekunde	Източник: parse trace, микросекунди	Извор: parse trace, микросекунде	Izvor: parse trace, mikrosekunde	Джерело: parse trace, мікросекунди
unitsTurns	turns	turns	turns	turns	turns	turns	turns	turns	turns	turns
timeline	Časová osa	Časová os	Oś czasu	Idővonal	Vremenska crta	Časovnica	Времева линия	Временска линија	Vremenska linija	Часова шкала
noConversations	Zatím žádné záznamy v deníku.	Zatiaľ žiadne záznamy v denníku.	Brak wpisów w dzienniku.	Még nincs naplóbejegyzés.	Još nema unosa u dnevniku.	Še ni vnosov v dnevniku.	Все още няма записи в дневника.	Још нема уноса у дневнику.	Još nema unosa u dnevniku.	Ще немає записів у журналі.
when	Když	Keď	Kiedy	Ha	Kad	Ko	Когато	Кад	Kad	Коли
then	Pak	Potom	Wtedy	Akkor	Zatim	Nato	Тогава	Онда	Onda	Тоді
priority	Pořadí (první shodné uživatelské pravidlo vyhraje)	Poradie (prvé zhodné používateľské pravidlo vyhrá)	Kolejność (pierwsza pasująca reguła użytkownika wygrywa)	Sorrend (az első illeszkedő felhasználói szabály nyer)	Redoslijed (prvo podudarno korisničko pravilo pobjeđuje)	Vrstni red (prvo ujemajoče se uporabniško pravilo zmaga)	Ред (първото съвпаднало потребителско правило печели)	Редослед (прво подударно корисничко правило побеђује)	Redosled (prvo podudarno korisničko pravilo pobeđuje)	Порядок (перше відповідне правило користувача перемагає)
evaluator	Policy evaluátor	Policy evaluátor	Ewaluator Policy	Policy értékelő	Policy evaluator	Policy ocenjevalnik	Policy оценител	Policy евалуатор	Policy evaluator	Policy оцінювач
bakeSpeech	Generovat varianty	Generovať varianty	Generuj warianty	Változatok generálása	Generiraj varijante	Ustvari različice	Генерирай варианти	Генериши варијанте	Generiši varijante	Згенерувати варіанти
addRule	Pravidlo	Pravidlo	Reguła	Szabály	Pravilo	Pravilo	Правило	Правило	Pravilo	Правило
noPolicies	Zatím žádná policy pravidla.	Zatiaľ žiadne policy pravidlá.	Brak reguł Policy.	Még nincsenek policy szabályok.	Još nema policy pravila.	Še ni policy pravil.	Все още няма policy правила.	Још нема policy правила.	Još nema policy pravila.	Ще немає правил Policy.
compiledRisk	Zkompilované riziko	Skompilované riziko	Skompilowane ryzyko	Lefordított kockázat	Kompajlirani rizik	Prevedeno tveganje	Компилиран риск	Компајлирани ризик	Kompajlirani rizik	Скомпільований ризик
finalBand	Band	Band	Band	Band	Band	Band	Band	Band	Band	Band
triggerFirst	Nejdřív HA spouštěče vět, pak Klar, pak registrovaný intent.	Najprv HA spúšťače viet, potom Klar, potom registrovaný intent.	Najpierw wyzwalacze zdań HA, potem Klar, potem zarejestrowany intent.	Először HA mondatindítók, aztán Klar, aztán regisztrált intent.	Prvo HA okidači rečenica, zatim Klar, zatim registrirani intent.	Najprej HA sprožilci stavkov, nato Klar, nato registriran intent.	Първо HA тригери за изречения, после Klar, после регистриран intent.	Прво HA окидачи реченица, затим Klar, затим регистровани intent.	Prvo HA okidači rečenica, zatim Klar, zatim registrovani intent.	Спочатку тригери речень HA, потім Klar, потім зареєстрований intent.
discarded	Zahozeno	Zahodené	Odrzucone	Elvetve	Odbačeno	Zavrženo	Отхвърлено	Одбачено	Odbačeno	Відхилено
stageTokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens
stageBind	Bind	Bind	Bind	Bind	Bind	Bind	Bind	Bind	Bind	Bind
stageRank	Rank	Rank	Rank	Rank	Rank	Rank	Rank	Rank	Rank	Rank
stagePolicy	Policy	Policy	Policy	Policy	Policy	Policy	Policy	Policy	Policy	Policy
stageBand	Band	Band	Band	Band	Band	Band	Band	Band	Band	Band
effectConfirm	Potvrdit	Potvrdiť	Potwierdź	Megerősítés	Potvrdi	Potrdi	Потвърди	Потврди	Potvrdi	Підтвердити
effectBlock	Blokovat	Blokovať	Blokuj	Blokkolás	Blokiraj	Blokiraj	Блокирай	Блокирај	Blokiraj	Блокувати
effectAllow	Povolit	Povoliť	Zezwól	Engedélyezés	Dopusti	Dovoli	Разреши	Дозволи	Dozvoli	Дозволити
effectPreferEntity	Preferovat entitu	Preferovať entitu	Preferuj encję	Entitás előnyben	Preferiraj entitet	Prednost entiteti	Предпочитай обект	Преферирај ентитет	Preferiraj entitet	Надавати перевагу сутності
effectPreferArea	Preferovat místnost	Preferovať miestnosť	Preferuj pomieszczenie	Terület előnyben	Preferiraj prostoriju	Prednost prostoru	Предпочитай стая	Преферирај просторију	Preferiraj prostoriju	Надавати перевагу кімнаті
effectReply	Odpovědět bez intentu	Odpovedať bez intentu	Odpowiedz bez intent	Válasz intent nélkül	Odgovori bez intenta	Odgovori brez intenta	Отговори без intent	Одговори без intent-а	Odgovori bez intent-a	Відповісти без intent
effectScript	Skript	Skript	Skrypt	Script	Skripta	Skript	Скрипт	Скрипта	Skripta	Скрипт
effectTemplate	Template	Template	Template	Template	Template	Template	Template	Template	Template	Template
effectLlm	LLM prompt	LLM prompt	LLM prompt	LLM prompt	LLM prompt	LLM prompt	LLM prompt	LLM prompt	LLM prompt	LLM prompt
payloadReply	Text odpovědi	Text odpovede	Tekst odpowiedzi	Válaszszöveg	Tekst odgovora	Besedilo odgovora	Текст на отговора	Текст одговора	Tekst odgovora	Текст відповіді
payloadScript	Skript (script.good_night nebo good_night)	Skript (script.good_night alebo good_night)	Skrypt (script.good_night lub good_night)	Script (script.good_night vagy good_night)	Skripta (script.good_night ili good_night)	Skript (script.good_night ali good_night)	Скрипт (script.good_night или good_night)	Скрипта (script.good_night или good_night)	Skripta (script.good_night ili good_night)	Скрипт (script.good_night або good_night)
payloadTemplate	Home Assistant template; {{ text }} je výrok	Home Assistant template; {{ text }} je výrok	Szablon Home Assistant; {{ text }} to wypowiedź	Home Assistant template; a {{ text }} a kiejtett mondat	Home Assistant template; {{ text }} je izgovor	Home Assistant predloga; {{ text }} je izrek	Home Assistant template; {{ text }} е изказването	Home Assistant template; {{ text }} је изговор	Home Assistant template; {{ text }} je izgovor	Шаблон Home Assistant; {{ text }} — висловлювання
payloadLlm	Systémový prompt pro záložního agenta	Systémový prompt pre záložného agenta	Prompt systemowy agenta zapasowego	Rendszerprompt a tartalék ügynöknek	Sistemski prompt za rezervnog agenta	Sistemski poziv za rezervnega agenta	Системен prompt за резервния агент	Системски упит за резервног агента	Sistemski upit za rezervnog agenta	Системна підказка для резервного агента
whenPhrase	Fráze	Fráza	Fraza	Mondat	Fraza	Fraza	Фраза	Фраза	Fraza	Фраза
chatMode	Chat	Chat	Czat	Csevegés	Razgovor	Klepet	Чат	Ћаскање	Ćaskanje	Чат
variantPreview	Varianta řeči	Varianta reči	Wariant mowy	Beszédváltozat	Govorna varijanta	Govorna različica	Говорен вариант	Говорна варијанта	Govorna varijanta	Мовленнєвий варіант
policies	Policies	Policies	Policies	Policies	Policies	Policies	Policies	Policies	Policies	Policies
routines	Rutiny	Rutiny	Rutyny	Rutinok	Rutine	Rutine	Рутини	Рутине	Rutine	Рутини
routineHint	Vyslovený název spustí skript Home Assistant. Dobrou noc vyhraje nad pozdravem.	Vyslovený názov spustí skript Home Assistant. Dobrú noc vyhrá pred pozdravom.	Wypowiedziana nazwa uruchamia skrypt Home Assistant. Dobranoc wygrywa z powitaniem.	Egy kimondott név Home Assistant scriptet indít. A Jó éjszakát nyer a köszönéssel szemben.	Izgovoreni naziv pokreće skriptu Home Assistant. Laku noć pobjeđuje pozdrav.	Izgovorjeno ime zažene skript Home Assistant. Lahko noč zmaga pred pozdravom.	Изговореното име стартира скрипт на Home Assistant. Лека нощ печели пред поздрава.	Изговорено име покреће скрипту Home Assistant. Лаку ноћ побеђује поздрав.	Izgovoreno ime pokreće skriptu Home Assistant. Laku noć pobeđuje pozdrav.	Промовлена назва запускає скрипт Home Assistant. На добраніч перемагає привітання.
routinePhraseHint	Dobrou noc	Dobrú noc	Dobranoc	Jó éjszakát	Laku noć	Lahko noč	Лека нощ	Лаку ноћ	Laku noć	На добраніч
addRoutine	Přidat rutinu	Pridať rutinu	Dodaj rutynę	Rutin hozzáadása	Dodaj rutinu	Dodaj rutino	Добави рутина	Додај рутину	Dodaj rutinu	Додати рутину
noRoutines	Zatím žádné rutiny.	Zatiaľ žiadne rutiny.	Brak rutyn.	Még nincsenek rutinok.	Još nema rutina.	Še ni rutin.	Все още няма рутини.	Још нема рутина.	Još nema rutina.	Ще немає рутин.
routineInvalid	Fráze a script.xxx jsou nutné.	Fráza a script.xxx sú potrebné.	Fraza i script.xxx są wymagane.	Mondat és script.xxx kell.	Fraza i script.xxx su obavezni.	Fraza in script.xxx sta potrebna.	Нужни са фраза и script.xxx.	Потребни су фраза и script.xxx.	Potrebni su fraza i script.xxx.	Потрібні фраза і script.xxx.
lastTurn	Poslední tah	Posledný ťah	Ostatnia tura	Utolsó kör	Zadnji krug	Zadnja menjava	Последен ход	Последњи потез	Poslednji potez	Останній хід
heardIn	Slyšeno v	Počuté v	Usłyszane w	Hallva itt	Čuto u	Slišano v	Чуто в	Чуто у	Čuto u	Почуто в
tryThese	Pět vět ve vašich místnostech	Päť viet vo vašich miestnostiach	Pięć zdań w twoich pomieszczeniach	Öt mondat a szobáidban	Pet rečenica u vašim prostorijama	Pet stavkov v tvojih prostorih	Пет изречения във вашите стаи	Пет реченица у вашим просторијама	Pet rečenica u vašim prostorijama	П'ять речень у ваших кімнатах
tryTheseHint	Klepněte na větu a zkuste ji v laboru.	Ťuknite na vetu a vyskúšajte ju v labore.	Dotknij zdania, aby wypróbować je w labie.	Koppints egy mondatra, és próbáld ki a laborban.	Dodirnite rečenicu i isprobajte je u labu.	Tapnite stavek in ga preizkusite v labu.	Докоснете изречение, за да го пробвате в лаба.	Додирните реченицу да је пробате у лабу.	Dodirnite rečenicu da je probate u labu.	Торкніться речення, щоб спробувати його в лабі.
anyRoom	Žádný satelit	Žiadny satelit	Brak satelity	Nincs műhold	Nema satelita	Ni satelita	Няма сателит	Нема сателита	Nema satelita	Немає супутника
personalityHa	Osobnost nastavte v Home Assistant → Klar NLU → Osobnost.	Osobnosť nastavte v Home Assistant → Klar NLU → Osobnosť.	Ustaw osobowość w Home Assistant → Klar NLU → Osobowość.	A személyiséget a Home Assistant → Klar NLU → Személyiség alatt állítsd be.	Osobnost postavite u Home Assistant → Klar NLU → Osobnost.	Osebnost nastavite v Home Assistant → Klar NLU → Osebnost.	Задайте личността в Home Assistant → Klar NLU → Личност.	Подесите личност у Home Assistant → Klar NLU → Личност.	Podesite ličnost u Home Assistant → Klar NLU → Ličnost.	Задайте особистість у Home Assistant → Klar NLU → Особистість.
"""

PACKS = parse_table(CODES, TABLE)
