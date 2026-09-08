"""Operator UI chrome for Nordic and Baltic Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_table

CODES = ["da", "nb", "sv", "fi", "is", "et", "lt", "lv"]

TABLE = """
home	Hjem	Hjem	Hem	Koti	Heim	Avaleht	Pradžia	Sākums
conversations	Samtaler	Samtaler	Konversationer	Keskustelut	Samtöl	Vestlused	Pokalbiai	Sarunas
rules	Regler	Regler	Regler	Säännöt	Reglur	Reeglid	Taisyklės	Noteikumi
house	Hus	Hus	Hus	Talo	Hús	Maja	Namas	Māja
lab	Lab	Lab	Lab	Labra	Lab	Labor	Laboratorija	Laboratorija
graph	Graf	Graf	Graf	Graafi	Graf	Graaf	Grafas	Grafs
calibrate	Tilknytning	Tilordning	Mappning	Kartoitus	Kortlagning	Vastendus	Susiejimas	Kartēšana
entities	Enheder	Enheter	Enheter	Laitteet	Tæki	Seadmed	Įrenginiai	Ierīces
custom	Sætninger	Setninger	Fraser	Lauseet	Setningar	Laused	Frazės	Frāzes
settings	Indstillinger	Innstillinger	Inställningar	Asetukset	Stillingar	Seaded	Nuostatos	Iestatījumi
open	åben	åpen	öppen	auki	opið	avatud	atvira	atvērts
bundleOn	Bundt til	Bunt på	Bunt på	Paketti päällä	Pakki kveikt	Pakett sees	Paketas įjungtas	Paka ieslēgta
bundleOff	Bundt fra	Bunt av	Bunt av	Paketti pois	Pakki slökkt	Pakett väljas	Paketas išjungtas	Paka izslēgta
engineReady	Motor klar	Motor klar	Motor redo	Moottori valmis	Vél tilbúin	Mootor valmis	Variklis paruoštas	Dzinējs gatavs
understandsHome	Hvorfor Klar udførte, bekræftede eller stoppede	Hvorfor Klar utførte, bekreftet eller stoppet	Varför Klar utförde, bekräftade eller stoppade	Miksi Klar suoritti, vahvisti tai pysähtyi	Af hverju Klar framkvæmdi, staðfesti eða stöðvaði	Miks Klar täitis, kinnitas või peatus	Kodėl Klar įvykdė, patvirtino arba sustojo	Kāpēc Klar izpildīja, apstiprināja vai apturēja
assistVisible	Assist synlig	Assist synlig	Assist synlig	Assist näkyvissä	Assist sýnilegt	Assist nähtav	Assist matomas	Assist redzams
certain	sikker	sikker	säker	varma	öruggt	kindel	tikras	drošs
needsWork	skal justeres	trenger arbeid	behöver arbete	vaatii työtä	þarf vinnu	vajab tööd	reikia darbo	vajadzīgs darbs
recordings	Optagelser	Opptak	Inspelningar	Tallenteet	Upptökur	Salvestised	Įrašai	Ieraksti
processed	behandlet	behandlet	bearbetade	käsitelty	unnið	töödeldud	apdorota	apstrādāti
coverage	Dækning	Dekning	Täckning	Kattavuus	Þekja	Katvus	Aprėptis	Pārklājums
confidence	Sikkerhed	Sikkerhet	Tillförlitlighet	Luottamus	Öryggi	Kindlus	Patikimumas	Ticamība
domains	Domæner	Domener	Domäner	Toimialueet	Lén	Domeenid	Sritys	Domēni
rooms	Rum	Rom	Rum	Huoneet	Herbergi	Ruumid	Kambariai	Telpas
recent	Seneste sætninger	Nylige setninger	Senaste meningar	Viimeisimmät lauseet	Nýlegar setningar	Viimased laused	Naujausi sakiniai	Jaunākie teikumi
replay	Afspil igen	Spill av igjen	Spela igen	Toista	Spila aftur	Esita uuesti	Pakartoti	Atskaņot
applyAll	Anvend forslag	Bruk forslag	Tillämpa förslag	Käytä ehdotuksia	Nota tillögur	Rakenda ettepanekud	Taikyti pasiūlymus	Lietot ieteikumus
undo	Fortryd	Angre	Ångra	Kumoa	Afturkalla	Võta tagasi	Anuliuoti	Atsaukt
accept	Acceptér	Godta	Godkänn	Hyväksy	Samþykkja	Nõustu	Priimti	Pieņemt
otherRoom	Andet rum	Annet rom	Annat rum	Muu huone	Annað herbergi	Muu ruum	Kitas kambarys	Cita telpa
dismiss	Afvis	Avvis	Avfärda	Hylkää	Hunsa	Loobu	Atmesti	Noraidīt
noGaps	Ingen åbne tilknytninger.	Ingen åpne tilordninger.	Inga öppna mappningar.	Ei avoimia kartoituksia.	Engar opnar kortlagningar.	Avatud vastendusi pole.	Nėra atvirų susiejimų.	Nav atvērtu kartējumu.
unmapped	Intet rum	Ingen rom	Inget rum	Ei huonetta	Ekkert herbergi	Ilma ruumita	Be kambario	Bez telpas
parseHint	Sætningsudløsere kører i Home Assistant før denne analyse. conversation.process: udløser, derefter Klar, derefter intent_script.	Setningsutløsere kjører i Home Assistant før denne analysen. conversation.process: utløser, deretter Klar, deretter intent_script.	Meningsutlösare körs i Home Assistant före denna analys. conversation.process: utlösare, sedan Klar, sedan intent_script.	Lauseiden laukaisimet ajetaan Home Assistantissa ennen tätä jäsentämistä. conversation.process: laukaisin, sitten Klar, sitten intent_script.	Setningakveikjur keyra í Home Assistant á undan þessari greiningu. conversation.process: kveikja, síðan Klar, síðan intent_script.	Lausetrigerid käivituvad Home Assistantis enne seda parsimist. conversation.process: triger, seejärel Klar, seejärel intent_script.	Sakinių trigeriai paleidžiami Home Assistant prieš šią analizę. conversation.process: trigeris, tada Klar, tada intent_script.	Teikumu trigeri darbojas Home Assistant pirms šīs analīzes. conversation.process: trigeris, tad Klar, tad intent_script.
command	Kommando	Kommando	Kommando	Komento	Skipun	Käsk	Komanda	Komanda
analyze	Analysér	Analyser	Analysera	Analysoi	Greina	Analüüsi	Analizuoti	Analizēt
raw	Rå	Rå	Rå	Raaka	Hrátt	Toores	Žalia	Neapstrādāts
speech	Tale	Tale	Tal	Puhe	Tal	Kõne	Kalba	Runa
intent	Hensigt	Intensjon	Avsikt	Tarkoitus	Ásetningur	Kavatsus	Ketinimas	Nolūks
slots	Slots	Felt	Fält	Paikat	Reitir	Pesad	Lizdai	Lauki
searchDevice	Hvad ville du kalde det?	Hva ville du kalle det?	Vad skulle du kalla det?	Mitä kutsaisit sitä?	Hvað myndirðu kalla það?	Kuidas sa seda nimetaksid?	Kaip jį pavadintumėte?	Kā jūs to sauktu?
alias	Alias	Alias	Alias	Alias	Alias	Alias	Pseudonimas	Aizstājvārds
room	Rum	Rom	Rum	Huone	Herbergi	Ruum	Kambarys	Telpa
preferred	Standardlys	Standardlys	Standardljus	Oletusvalo	Sjálfgefið ljós	Vaiketuli	Numatytoji šviesa	Noklusējuma gaisma
save	Gem	Lagre	Spara	Tallenna	Vista	Salvesta	Įrašyti	Saglabāt
personality	Personlighed	Personlighet	Personlighet	Persoonallisuus	Persónuleiki	Isiksus	Asmenybė	Personība
mode	Tilstand	Modus	Läge	Tila	Hamur	Režiim	Veiksena	Režīms
supportBundle	Supportbundt	Støttebunt	Supportbunt	Tukipaketti	Stuðningspakki	Tugipakett	Palaikymo paketas	Atbalsta paka
recordProtocol	Optag protokol	Ta opp protokoll	Spela in protokoll	Tallenna protokolla	Skrá samskiptareglur	Salvesta protokoll	Įrašyti protokolą	Ierakstīt protokolu
includeRawText	Medtag råtekst i overførsler	Ta med råtekst i nedlastinger	Inkludera råtext i nedladdningar	Sisällytä raakateksti latauksiin	Hafa hráan texta með í niðurhali	Kaasa toortekst allalaadimistesse	Įtraukti žalią tekstą į atsisiuntimus	Iekļaut neapstrādāto tekstą lejupielādēs
semanticAdapters	Lokale semantiske adaptere	Lokale semantiske adaptere	Lokala semantiska adaptrar	Paikalliset semanttiset sovittimet	Staðbundnir merkingaradapterar	Kohalikud semantilised adapterid	Vietiniai semantiniai adapteriai	Vietējie semantiskie adapteri
downloadDataset	Hent datasæt	Last ned datasett	Ladda ner dataset	Lataa tietoaineisto	Sækja gagnasafn	Laadi andmestik alla	Atsisiųsti duomenų rinkinį	Lejupielādēt datu kopu
downloadProtocol	Hent protokol	Last ned protokoll	Ladda ner protokoll	Lataa protokolla	Sækja samskiptareglur	Laadi protokoll alla	Atsisiųsti protokolą	Lejupielādēt protokolu
deleteSelected	Slet markering	Slett utvalg	Ta bort markering	Poista valinta	Eyða vali	Kustuta valik	Ištrinti pažymėjimą	Dzēst atlasi
clearAll	Slet alt	Slett alt	Ta bort allt	Poista kaikki	Eyða öllu	Kustuta kõik	Ištrinti viską	Dzēst visu
token	Skrivetoken (LAN)	Skrivetoken (LAN)	Skrivtoken (LAN)	Kirjoituspoletti (LAN)	Skriftóki (LAN)	Kirjutamistõend (LAN)	Rašymo raktas (LAN)	Rakstīšanas marķieris (LAN)
customJson	Egne sætninger som JSON	Egne setninger som JSON	Egna fraser som JSON	Omat lauseet JSONina	Sérsniðnar setningar sem JSON	Kohandatud laused JSON-ina	Pasirinktinės frazės kaip JSON	Pielāgotas frāzes kā JSON
customHint	Knyt en sætning til en kendt hensigt. Politikker ligger ved siden af, ikke som HA-automatiseringer.	Knytt en setning til en kjent intensjon. Policyer ligger ved siden av, ikke som HA-automatiseringer.	Koppla en fras till en känd avsikt. Policyer ligger bredvid, inte som HA-automatiseringar.	Liitä lause tunnettuun tarkoitukseen. Käytännöt ovat rinnalla, eivät HA-automaatioina.	Tengdu setningu við þekktan ásetning. Stefnur standa við hliðina, ekki sem HA-sjálfvirkni.	Seo lause teadaoleva kavatsusega. Poliitikad on kõrval, mitte HA automatiseerimistena.	Susiek frazę su žinomu ketinimu. Politikos yra šalia, ne kaip HA automatizacijos.	Piesaisti frāzi zināmam nolūkam. Politikas ir blakus, ne kā HA automatizācijas.
addPhrase	Tilføj sætning	Legg til setning	Lägg till fras	Lisää lause	Bæta við setningu	Lisa lause	Pridėti frazę	Pievienot frāzi
previewRule	Forhåndsvis	Forhåndsvis	Förhandsgranska	Esikatsele	Forskoða	Eelvaade	Peržiūra	Priekšskatīt
explainRule	Forklar	Forklar	Förklara	Selitä	Útskýra	Selgita	Paaiškinti	Paskaidrot
rollback	Rul tilbage	Rull tilbake	Återställ	Palauta	Afturheimta	Taasta	Grąžinti	Atgriezt
noRules	Ingen egne sætninger endnu.	Ingen egne setninger ennå.	Inga egna fraser ännu.	Ei omia lauseita vielä.	Engar sérsniðnar setningar enn.	Kohandatud lauseid pole veel.	Dar nėra pasirinktinių frazių.	Vēl nav pielāgotu frāžu.
engineOffline	Engine ikke tilgængelig. Listen er tom, indtil en live indlæsning lykkes.	Engine ikke tilgjengelig. Listen er tom til en live innlasting lykkes.	Engine otillgänglig. Listan är tom tills en live-inläsning lyckas.	Moottori ei ole tavoitettavissa. Lista on tyhjä, kunnes live-lataus onnistuu.	Vél náist ekki. Listinn er tómur uns rauntímahleðsla tekst.	Mootor pole kättesaadav. Loend on tühi, kuni reaalajas laadimine õnnestub.	Variklis nepasiekiamas. Sąrašas tuščias, kol pavyksta gyvas įkėlimas.	Dzinējs nav sasniedzams. Saraksts ir tukšs, līdz izdodas tiešsaistes ielāde.
emptyBundle	Ingen optagelser endnu. Aktivér bundtet og prøv en sætning.	Ingen opptak ennå. Slå på bunten og prøv en setning.	Inga inspelningar ännu. Aktivera bunten och prova en mening.	Ei tallenteita vielä. Ota paketti käyttöön ja kokeile lausetta.	Engar upptökur enn. Kveiktu á pakkanum og prófaðu setningu.	Salvestisi pole veel. Lülita pakett sisse ja proovi lauset.	Dar nėra įrašų. Įjunkite paketą ir išbandykite sakinį.	Vēl nav ierakstu. Ieslēdziet paku un izmēģiniet teikumu.
confirmApply	Anvend disse forslag?	Bruke disse forslagene?	Tillämpa dessa förslag?	Käytetäänkö näitä ehdotuksia?	Nota þessar tillögur?	Rakenda need ettepanekud?	Taikyti šiuos pasiūlymus?	Lietot šos ieteikumus?
cancel	Annuller	Avbryt	Avbryt	Peruuta	Hætta við	Tühista	Atšaukti	Atcelt
apply	Anvend	Bruk	Tillämpa	Käytä	Nota	Rakenda	Taikyti	Lietot
close	Luk	Lukk	Stäng	Sulje	Loka	Sulge	Uždaryti	Aizvērt
low	lav	lav	låg	matala	lágt	madal	žemas	zems
medium	middel	middels	medel	keski	miðlungs	keskmine	vidutinis	vidējs
high	høj	høy	hög	korkea	hátt	kõrge	aukštas	augsts
source	Kilde	Kilde	Källa	Lähde	Uppruni	Allikas	Šaltinis	Avots
language	Sprog	Språk	Språk	Kieli	Tungumál	Keel	Kalba	Valoda
time	Tid	Tid	Tid	Aika	Tími	Aeg	Laikas	Laiks
text	Sætning	Setning	Mening	Lause	Setning	Lause	Sakinys	Teikums
answer	Svar	Svar	Svar	Vastaus	Svar	Vastus	Atsakymas	Atbilde
graphHint	Rum som klynger, enheder farvet efter sikkerhed.	Rom som klynger, enheter farget etter sikkerhet.	Rum som kluster, enheter färgade efter tillförlitlighet.	Huoneet klustereina, laitteet väritetty luottamuksen mukaan.	Herbergi sem klasar, tæki lituð eftir öryggi.	Ruumid klastritena, seadmed värvitud kindluse järgi.	Kambariai kaip klasteriai, įrenginiai spalvinti pagal patikimumą.	Telpas kā kopas, ierīces krāsotas pēc ticamības.
resetLayout	Nulstil layout	Tilbakestill oppsett	Återställ layout	Palauta asettelu	Endurstilla útlit	Lähtesta paigutus	Atstatyti išdėstymą	Atiestatīt izkārtojumu
score	Score	Poeng	Poäng	Pisteet	Einkunn	Tulemus	Balas	Punkti
noIntent	Ingen hensigt	Ingen intensjon	Ingen avsikt	Ei tarkoitusta	Enginn ásetningur	Kavatsust pole	Nėra ketinimo	Nav nolūka
loading	Indlæser Klar...	Laster Klar...	Laddar Klar...	Ladataan Klar...	Hleð Klar...	Laadin Klar...	Įkeliamas Klar...	Ielādē Klar...
nluRagHint	Fra som standard. Kun det matchede udsnit, aldrig Assist-værktøjer.	Av som standard. Bare det gjenkjente utsnittet, aldri Assist-verktøy.	Av som standard. Endast det matchade utsnittet, aldrig Assist-verktyg.	Oletuksena pois. Vain tunnistettu ote, ei koskaan Assist-työkaluja.	Slökkt sjálfgefið. Aðeins samsvarandi sneið, aldrei Assist-verkfæri.	Vaikimisi väljas. Ainult vastav lõik, mitte kunagi Assist tööriistad.	Pagal numatymą išjungta. Tik atpažinta dalis, niekada Assist įrankiai.	Pēc noklusējuma izslēgts. Tikai atpazītais fragments, nekad Assist rīki.
confirmRisky	Bekræft risikable handlinger	Bekreft risikable handlinger	Bekräfta riskfyllda åtgärder	Vahvista riskialttiit toiminnot	Staðfesta áhættusamar aðgerðir	Kinnita riskantsed toimingud	Patvirtinti rizikingus veiksmus	Apstiprināt riskantas darbības
languages	Sprog	Språk	Språk	Kielet	Tungumál	Keeled	Kalbos	Valodas
languageSearch	Søg sprog	Søk språk	Sök språk	Hae kieliä	Leita að tungumálum	Otsi keeli	Ieškoti kalbų	Meklēt valodas
allLanguages	Alle sprog	Alle språk	Alla språk	Kaikki kielet	Öll tungumál	Kõik keeled	Visos kalbos	Visas valodas
noLanguageMatch	Intet sprog fundet	Ingen språk funnet	Inget språk hittades	Kieltä ei löytynyt	Ekkert tungumál fannst	Keelt ei leitud	Kalba nerasta	Valoda nav atrasta
languageHint	Søg og vælg locales. Alle sprog holder hvert kompileret pakke aktiveret.	Søk og velg locales. Alle språk holder hver kompilert pakke aktivert.	Sök och välj locales. Alla språk håller varje kompilerat paket aktiverat.	Hae ja valitse locales. Kaikki kielet pitää jokaisen käännetyn paketin käytössä.	Leitaðu og veldu locales. Öll tungumál heldur öllum þýddum pökkum virkum.	Otsi ja vali locales. Kõik keeled hoiab iga kompileeritud paki sisselülitatuna.	Ieškokite ir rinkitės locales. Visos kalbos palieka kiekvieną sukompiliuotą paketą įjungtą.	Meklējiet un izvēlieties locales. Visas valodas atstāj katru kompilēto pakotni ieslēgtu.
mappingHint	Tilknytning er aliasser for grafenheder. Kalendere vises, når calendar-domænet er medtaget. Assist følger sprogpakken; denne grænseflade følger operatørsproget.	Tilordning er aliaser for grafenheter. Kalendere vises når calendar-domenet er inkludert. Assist følger språkpakken; dette grensesnittet følger operatørspråket.	Mappning är alias för grafentiteter. Kalendrar visas när calendar-domänen är inkluderad. Assist följer språkpaketet; detta gränssnitt följer operatörsspråket.	Kartoitus on aliaksia graafin entiteeteille. Kalenterit näkyvät, kun calendar-toimialue on mukana. Assist seuraa kielipakettia; tämä käyttöliittymä seuraa operaattorin kieltä.	Kortlagning eru samheiti fyrir graf-einingar. Dagatöl birtast þegar calendar-lénið er innifalið. Assist fylgir tungumálapakkanum; þetta viðmót fylgir tungumáli stjórnandans.	Vastendus on aliased graafi olemitele. Kalendrid ilmuvad pärast calendar domeeni lisamist. Assist järgib keelepakki; see liides järgib operaatori keelt.	Susiejimas yra grafų objektų slapyvardžiai. Kalendoriai pasirodo įtraukus calendar sritį. Assist seka kalbos paketą; ši sąsaja seka operatoriaus kalbą.	Kartēšana ir grafu entītiju aizstājvārdi. Kalendāri parādās pēc calendar domēna iekļaušanas. Assist seko valodas pakotnei; šī saskarne seko operatora valodai.
parseSample	Tænd lyset i stuen	Skru på lyset i stuen	Tänd ljuset i vardagsrummet	Sytytä olohuoneen valo	Kveiktu á ljósinu í stofunni	Pane elutoa tuli põlema	Įjunk svetainės šviesą	Ieslēdz viesistabas gaismu
tryOn	Tænd lyset i {room}	Skru på lyset i {room}	Tänd ljuset i {room}	Sytytä valo huoneessa {room}	Kveiktu á ljósinu í {room}	Pane tuli põlema ruumis {room}	Įjunk šviesą kambaryje {room}	Ieslēdz gaismu telpā {room}
tryLock	Er døren låst?	Er døren låst?	Är dörren låst?	Onko ovi lukossa?	Er hurðin læst?	Kas uks on lukus?	Ar durys užrakintos?	Vai durvis ir aizslēgtas?
tryTime	Hvad er klokken?	Hva er klokken?	Vad är klockan?	Mitä kello on?	Hvað er klukkan?	Mis kell on?	Kiek valandų?	Cik ir pulkstenis?
tryNight	Godnat	God natt	God natt	Hyvää yötä	Góða nótt	Head ööd	Labanakt	Labu nakti
tryUndo	Fortryd det	Angre det	Ångra det	Kumoa se	Afturkallaðu það	Võta see tagasi	Atšauk tai	Atsauc to
tryRoom	køkkenet	kjøkkenet	köket	keittiö	eldhúsið	köök	virtuvė	virtuve
nluIgnore	Bind ikke for status eller tænd/sluk	Ikke bind for status eller strøm	Bind inte för status eller ström	Älä sido tilaa tai virtaa varten	Ekki binda fyrir stöðu eða afl	Ära seo oleku ega toite jaoks	Nerišti būsenai ar maitinimui	Nesaistīt stāvoklim vai jaudai
nluIgnoreHint	Fjerner denne enhed fra resolveren. Brug ved fejlnavngivne helpers.	Fjerner denne enheten fra resolveren. Bruk for feilnavngitte helpers.	Tar bort den här enheten från resolvern. Använd för felnamngivna helpers.	Poistaa tämän laitteen resolverista. Käytä väärin nimetyille helpers.	Fjarlægir þetta tæki úr leysinum. Notaðu fyrir rangnefnda helpers.	Eemaldab selle seadme resolvrist. Kasuta valesti nimetatud helpers.	Pašalina šį įrenginį iš sprendiklio. Naudokite blogai pavadintiems helpers.	Izņem šo ierīci no risinātāja. Izmantojiet nepareizi nosauktiem helpers.
savePhrase	Gem som sætning	Lagre som setning	Spara som fras	Tallenna lauseena	Vista sem setningu	Salvesta lausena	Įrašyti kaip frazę	Saglabāt kā frāzi
ignoreTarget	Ignorér dette mål	Ignorer dette målet	Ignorera det här målet	Ohita tämä kohde	Hunsa þetta mark	Eira seda sihtmärki	Ignoruoti šį taikinį	Ignorēt šo mērķi
teachSaved	Gemt.	Lagret.	Sparat.	Tallennettu.	Vistað.	Salvestatud.	Įrašyta.	Saglabāts.
journal	Samtalejournal	Samtalejournal	Konversationsjournal	Keskusteluloki	Samtalsdagbók	Vestluspäevik	Pokalbių žurnalas	Sarunu žurnāls
journalHint	Sidste 200 ture, 24 timer, redigeret. Råtekst kun med bundtet.	Siste 200 turer, 24 timer, redigert. Råtekst bare med bunten.	Senaste 200 turerna, 24 timmar, redigerat. Råtext endast med bunten.	Viimeiset 200 vuoroa, 24 tuntia, redaktoitu. Raakateksti vain paketin kanssa.	Síðustu 200 umferðir, 24 klukkustundir, ritstýrt. Hrár texti aðeins með pakkanum.	Viimased 200 vooru, 24 tundi, redigeeritud. Toortekst ainult paketiga.	Paskutiniai 200 ėjimų, 24 valandos, redaguota. Žalias tekstas tik su paketu.	Pēdējie 200 gājieni, 24 stundas, rediģēts. Neapstrādāts teksts tikai ar paku.
decisionMix	Beslutninger	Beslutninger	Beslut	Päätökset	Ákvarðanir	Otsused	Sprendimai	Lēmumi
mixCaption	Kilde: samtalejournal, ture pr. dag	Kilde: samtalejournal, turer per dag	Källa: konversationsjournal, turer per dag	Lähde: keskusteluloki, vuoroja päivässä	Uppruni: samtalsdagbók, umferðir á dag	Allikas: vestluspäevik, voorud päevas	Šaltinis: pokalbių žurnalas, ėjimai per dieną	Avots: sarunu žurnāls, gājieni dienā
coverageCaption	Kilde: hjemmegraf, andel af enheder	Kilde: husgraf, andel enheter	Källa: husgraf, andel enheter	Lähde: kotigraafi, laitteiden osuus	Uppruni: húsagraf, hlutdeild tækja	Allikas: kodugraaf, seadmete osakaal	Šaltinis: namų grafas, įrenginių dalis	Avots: mājas grafs, ierīču daļa
latency	Trintid	Trinntid	Stegtid	Vaiheaika	Stigatími	Etapiaeg	Etapo laikas	Posma laiks
latencyCaption	Kilde: analysespor, mikrosekunder	Kilde: analysespor, mikrosekunder	Källa: analyspår, mikrosekunder	Lähde: jäsennysjälki, mikrosekunnit	Uppruni: greiningarslóð, míkrósekúndur	Allikas: parsimisjälg, mikrosekundid	Šaltinis: analizės pėdsakas, mikrosekundės	Avots: analīzes pēda, mikrosekundes
unitsTurns	ture	turer	turer	vuoroa	umferðir	vooru	ėjimai	gājieni
timeline	Tidslinje	Tidslinje	Tidslinje	Aikajana	Tímalína	Ajajoon	Laiko juosta	Laika skala
noConversations	Ingen journalposter endnu.	Ingen journaloppføringer ennå.	Inga journalposter ännu.	Ei lokimerkintöjä vielä.	Engar dagbókarfærslur enn.	Päeviku kirjeid pole veel.	Dar nėra žurnalo įrašų.	Vēl nav žurnāla ierakstu.
when	Når	Når	När	Kun	Þegar	Kui	Kai	Kad
then	Så	Så	Då	Sitten	Þá	Siis	Tada	Tad
priority	Rækkefølge (første matchende brugerregel vinder)	Rekkefølge (første treffende brukerregel vinner)	Ordning (första matchande användarregeln vinner)	Järjestys (ensimmäinen täsmäävä käyttäjäsääntö voittaa)	Röð (fyrsta samsvarandi notandaregla vinnur)	Järjekord (esimene sobiv kasutajareegel võidab)	Eilė (pirmoji atitinkanti naudotojo taisyklė laimi)	Secība (pirmais atbilstošais lietotāja noteikums uzvar)
evaluator	Politikvurdering	Policyevaluator	Policyutvärderare	Käytäntöarvioija	Stefnumat	Poliitikahindaja	Politikos vertintojas	Politiku vērtētājs
bakeSpeech	Generér varianter	Generer varianter	Generera varianter	Luo muunnelmia	Búa til afbrigði	Loo variandid	Generuoti variantus	Ģenerēt variantus
addRule	Regel	Regel	Regel	Sääntö	Regla	Reegel	Taisyklė	Noteikums
noPolicies	Ingen politikregler endnu.	Ingen policyregler ennå.	Inga policyregler ännu.	Ei käytäntösääntöjä vielä.	Engar stefnureglur enn.	Poliitikareegleid pole veel.	Dar nėra politikos taisyklių.	Vēl nav politikas noteikumu.
compiledRisk	Kompileret risiko	Kompilert risiko	Kompilerad risk	Käännetty riski	Þýdd áhætta	Kompileeritud risk	Sukompiliuota rizika	Kompilēts risks
finalBand	Bånd	Bånd	Band	Kaista	Band	Riba	Juosta	Josla
triggerFirst	HA-sætningsudløsere først, derefter Klar, derefter en registreret hensigt.	HA-setningsutløsere først, deretter Klar, deretter en registrert intensjon.	HA-meningsutlösare först, sedan Klar, sedan en registrerad avsikt.	HA-lauselaukaisimet ensin, sitten Klar, sitten rekisteröity tarkoitus.	HA-setningakveikjur fyrst, síðan Klar, síðan skráður ásetningur.	Kõigepealt HA lausetrigerid, seejärel Klar, seejärel registreeritud kavatsus.	Pirmiausia HA sakinių trigeriai, tada Klar, tada registruotas ketinimas.	Vispirms HA teikumu trigeri, tad Klar, tad reģistrēts nolūks.
discarded	Kasseret	Forkastet	Förkastad	Hylätty	Hunsað	Hüljatud	Atmesta	Atmests
stageTokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens
stageBind	Bind	Bind	Bind	Sidonta	Binda	Sidumine	Susieti	Saistīt
stageRank	Rang	Rang	Rang	Sijoitus	Röð	Järk	Rangas	Rangs
stagePolicy	Politik	Policy	Policy	Käytäntö	Stefna	Poliitika	Politika	Politika
stageBand	Bånd	Bånd	Band	Kaista	Band	Riba	Juosta	Josla
effectConfirm	Bekræft	Bekreft	Bekräfta	Vahvista	Staðfesta	Kinnita	Patvirtinti	Apstiprināt
effectBlock	Blokér	Blokker	Blockera	Estä	Loka	Blokeeri	Blokuoti	Bloķēt
effectAllow	Tillad	Tillat	Tillåt	Salli	Leyfa	Luba	Leisti	Atļaut
effectPreferEntity	Foretræk enhed	Foretrekk enhet	Föredra enhet	Suosi entiteettiä	Kjósa einingu	Eelista olemit	Teikti pirmenybę objektui	Dot priekšroku entītijai
effectPreferArea	Foretræk rum	Foretrekk rom	Föredra rum	Suosi tilaa	Kjósa svæði	Eelista ruumi	Teikti pirmenybę kambariui	Dot priekšroku telpai
effectReply	Svar uden hensigt	Svar uten intensjon	Svara utan avsikt	Vastaa ilman tarkoitusta	Svara án ásetnings	Vasta ilma kavatsuseta	Atsakyti be ketinimo	Atbildēt bez nolūka
effectScript	Script	Script	Script	Script	Script	Script	Script	Script
effectTemplate	Skabelon	Mal	Mall	Malli	Sniðmát	Mall	Šablonas	Veidne
effectLlm	LLM-prompt	LLM-prompt	LLM-prompt	LLM-kehote	LLM-vísbending	LLM-viip	LLM užklausa	LLM uzvedne
payloadReply	Svartekst	Svartekst	Svarstext	Vastauksen teksti	Svarstexti	Vastustekst	Atsakymo tekstas	Atbildes teksts
payloadScript	Script (script.good_night eller good_night)	Script (script.good_night eller good_night)	Script (script.good_night eller good_night)	Script (script.good_night tai good_night)	Script (script.good_night eða good_night)	Script (script.good_night või good_night)	Script (script.good_night arba good_night)	Script (script.good_night vai good_night)
payloadTemplate	Home Assistant-skabelon; {{ text }} er ytringen	Home Assistant-mal; {{ text }} er ytringen	Home Assistant-mall; {{ text }} är yttrandet	Home Assistant -malli; {{ text }} on lausuma	Home Assistant-sniðmát; {{ text }} er yfirlýsingin	Home Assistant mall; {{ text }} on ütlus	Home Assistant šablonas; {{ text }} yra pasakymas	Home Assistant veidne; {{ text }} ir izteikums
payloadLlm	Systemprompt til reserveagenten	Systemprompt for reserveagenten	Systemprompt för reservagenten	Järjestelmäkehote varagentille	Kerfisvísbending fyrir varaumboðsmanninn	Süsteemiviip varuagendile	Sistemos užklausa atsarginiam agentui	Sistēmas uzvedne rezerves aģentam
whenPhrase	Sætning	Setning	Fras	Lause	Setning	Lause	Frazė	Frāze
chatMode	Chat	Chat	Chatt	Keskustelu	Spjall	Vestlus	Pokalbis	Tērzēšana
variantPreview	Talevariant	Talevariant	Talvariant	Puhemuunnelma	Talsafbrigði	Kõnevariant	Kalbos variantas	Runas variants
policies	Politikker	Policyer	Policyer	Käytännöt	Stefnur	Poliitikad	Politikos	Politikas
routines	Rutiner	Rutiner	Rutiner	Rutiinit	Rútínur	Rutiinid	Rutinos	Rutīnas
routineHint	Et talt navn starter et Home Assistant-script. Godnat vinder over hilsenen.	Et talt navn starter et Home Assistant-script. God natt vinner over hilsenen.	Ett talat namn startar ett Home Assistant-script. God natt vinner över hälsningen.	Puhuttu nimi käynnistää Home Assistant -scriptin. Hyvää yötä voittaa tervehdyksen.	Talað nafn ræsir Home Assistant-script. Góða nótt vinnur af kveðjunni.	Öeldud nimi käivitab Home Assistant scripti. Head ööd võidab tervituse.	Ištartas vardas paleidžia Home Assistant script. Labanakt laimi prieš pasisveikinimą.	Izrunāts nosaukums sāk Home Assistant script. Labu nakti uzvar sveicienu.
routinePhraseHint	Godnat	God natt	God natt	Hyvää yötä	Góða nótt	Head ööd	Labanakt	Labu nakti
addRoutine	Tilføj rutine	Legg til rutine	Lägg till rutin	Lisää rutiini	Bæta við rútínu	Lisa rutiin	Pridėti rutiną	Pievienot rutīnu
noRoutines	Ingen rutiner endnu.	Ingen rutiner ennå.	Inga rutiner ännu.	Ei rutiineja vielä.	Engar rútínur enn.	Rutiine pole veel.	Dar nėra rutinų.	Vēl nav rutīnu.
routineInvalid	En sætning og script.xxx er påkrævet.	En setning og script.xxx er påkrevd.	En fras och script.xxx krävs.	Lause ja script.xxx vaaditaan.	Setning og script.xxx eru nauðsynleg.	Lause ja script.xxx on nõutavad.	Reikalinga frazė ir script.xxx.	Nepieciešama frāze un script.xxx.
lastTurn	Sidste tur	Siste tur	Senaste turen	Viimeinen vuoro	Síðasta umferð	Viimane voor	Paskutinis ėjimas	Pēdējais gājiens
heardIn	Hørt i	Hørt i	Hördes i	Kuultu alueella	Heyrt í	Kuuldud kohas	Girdėta	Dzirdēts
tryThese	Fem sætninger i dine rum	Fem setninger i rommene dine	Fem meningar i dina rum	Viisi lauset huoneissasi	Fimm setningar í herbergjunum þínum	Viis lauset sinu ruumides	Penki sakiniai jūsų kambariuose	Pieci teikumi jūsu telpās
tryTheseHint	Tryk på en sætning for at prøve den i labbet.	Trykk på en setning for å prøve den i laben.	Tryck på en mening för att prova den i labbet.	Napauta lausetta kokeillaksesi sitä labrassa.	Pikkaðu á setningu til að prófa hana í labinu.	Puuduta lauset, et proovida seda laboris.	Bakstelėkite sakinį, kad išbandytumėte jį laboratorijoje.	Pieskarieties teikumam, lai izmēģinātu to laboratorijā.
anyRoom	Ingen satellit	Ingen satellitt	Ingen satellit	Ei satelliittia	Enginn gervihnöttur	Satelliiti pole	Nėra palydovo	Nav satelīta
personalityHa	Indstil personlighed i Home Assistant → Klar NLU → Personlighed.	Angi personlighet i Home Assistant → Klar NLU → Personlighet.	Ställ in personligheten i Home Assistant → Klar NLU → Personlighet.	Aseta persoonallisuus kohdassa Home Assistant → Klar NLU → Persoonallisuus.	Stilltu persónuleika í Home Assistant → Klar NLU → Persónuleiki.	Sea isiksus jaotises Home Assistant → Klar NLU → Isiksus.	Nustatykite asmenybę: Home Assistant → Klar NLU → Asmenybė.	Iestatiet personību sadaļā Home Assistant → Klar NLU → Personība.
"""

PACKS = parse_table(CODES, TABLE)
