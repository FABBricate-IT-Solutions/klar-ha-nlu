"""Operator UI chrome for Germanic and Celtic Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_table

CODES = ["de-CH", "de-AT", "en-GB", "af", "lb", "cy", "eu", "ga", "kw"]

TABLE = """
#key	de-CH	de-AT	en-GB	af	lb	cy	eu	ga	kw
home	Home	Home	Home	Tuis	Doheem	Cartref	Hasiera	Baile	Tre
conversations	Gspröch	Gespräche	Conversations	Gesprekke	Gespréicher	Sgyrsiau	Elkarrizketak	Comhráite	Keskowsow
rules	Regle	Regeln	Rules	Reëls	Reegelen	Rheolau	Arauak	Rialacha	Reyglow
house	Huus	Haus	House	Huis	Haus	Tŷ	Etxea	Teach	Chi
lab	Labor	Labor	Lab	Lab	Labo	Lab	Laborategia	Saotharlann	Labordy
graph	Graph	Graph	Graph	Grafiek	Graph	Graff	Grafoa	Graf	Graf
calibrate	Zuornig	Zuordnung	Mapping	Kartering	Zouuerdnung	Mapio	Esleipena	Mapáil	Mappa
entities	Grät	Geräte	Devices	Toestelle	Apparater	Dyfeisiau	Gailuak	Gléasanna	Devisys
custom	Sätz	Sätze	Phrases	Frases	Sätz	Ymadroddion	Esamoldeak	Frásaí	Lavarrow
settings	Istellige	Einstellungen	Settings	Instellings	Astellungen	Gosodiadau	Ezarpenak	Socruithe	Settyansow
open	offe	offen	open	oop	oppen	agored	irekita	oscailte	ygor
bundleOn	Bundle aa	Bundle an	Bundle on	Bundle aan	Bundle un	Bundle ymlaen	Bundle piztuta	Bundle ar siúl	Bundle war
bundleOff	Bundle uus	Bundle aus	Bundle off	Bundle af	Bundle aus	Bundle i ffwrdd	Bundle itzalita	Bundle as	Bundle dhe-ves
engineReady	Engine bereit	Engine bereit	Engine ready	Enjin gereed	Engine prett	Peiriant yn barod	Motorra prest	Inneall réidh	Jynn parys
understandsHome	Wieso Klar uusgführt, bestätigt oder gstoppt het	Warum Klar ausgeführt, bestätigt oder abgelehnt hat	Why Klar executed, confirmed, or stopped	Hoekom Klar uitgevoer, bevestig of gestop het	Firwat Klar ausgefouert, bestätegt oder gestoppt huet	Pam y gweithredodd, cadarnhaodd neu ataliodd Klar	Zergatik exekutatu, berretsi edo gelditu duen Klar-ek	Cén fáth ar rith, dheimhnigh nó a stop Klar	Prag y hwrug Klar oberi, afydhya, po hedhi
assistVisible	Assist sichtbar	Assist sichtbar	Assist visible	Assist sigbaar	Assist siichtbar	Assist yn weladwy	Assist ikusgai	Assist le feiceáil	Assist gweladow
certain	sicher	sicher	certain	seker	sécher	sicr	ziur	cinnte	sur
needsWork	bruucht Arbet	braucht Arbeit	needs work	benodig werk	brauch Aarbecht	angen gwaith	lan behar du	teastaíonn obair	res yw ober
recordings	Ufzeichnige	Aufzeichnungen	Recordings	Opnames	Ophuelungen	Recordiadau	Grabazioak	Taifeadtaí	Rekordyansow
processed	verarbeitet	verarbeitet	processed	verwerk	verschafft	wedi'i brosesu	prozesatuta	próiseáilte	argerdhys
coverage	Coverage	Coverage	Coverage	Dekking	Coverage	Cwmpas	Estaldura	Clúdach	Kudhans
confidence	Treffsicherheit	Treffsicherheit	Confidence	Vertroue	Treffsécherheet	Hyder	Konfiantza	Muinín	Fydh
domains	Domains	Domains	Domains	Domeine	Domains	Parthau	Domeinuak	Fearainn	Tiredhow
rooms	Rüüm	Räume	Rooms	Kamers	Zëmmer	Ystafelloedd	Gelak	Seomraí	Stevellow
recent	Letschti Sätz	Letzte Sätze	Recent sentences	Onlangse sinne	Lescht Sätz	Brawddegau diweddar	Azken esaldiak	Abairtí le déanaí	Lavarrow a-dhiwedhes
replay	Nonemal	Nochmal	Replay	Speel weer	Nach eng Kéier	Ailchwarae	Berriro	Athsheinm	Daswari
applyAll	Vorschläg übernää	Vorschläge übernehmen	Apply suggestions	Pas voorstelle toe	Virschléi iwwerhuelen	Cymhwyso awgrymiadau	Aplikatu iradokizunak	Cuir moltaí i bhfeidhm	Devnydhya profyansow
undo	Rückgängig	Rückgängig	Undo	Ontdoen	Réckgängeg	Dadwneud	Desegin	Cealaigh	Diswul
accept	Aanee	Annehmen	Accept	Aanvaar	Unhuelen	Derbyn	Onartu	Glac leis	Degemmeres
otherRoom	Andere Ruum	Anderer Raum	Other room	Ander kamer	Anert Zëmmer	Ystafell arall	Beste gela	Seomra eile	Stevell aral
dismiss	Wegtue	Verwerfen	Dismiss	Verwerp	Verwerfen	Gwrthod	Baztertu	Déan neamhshuim	Skonya
noGaps	Kei offeni Zuornige.	Keine offenen Zuordnungen.	No open mappings.	Geen oop karterings nie.	Keng oppen Zouuerdnungen.	Dim mapiau agored.	Ez dago esleipen irekirik.	Níl aon mhapálacha oscailte.	Nyns eus mappow ygor.
unmapped	Ohni Ruum	Ohne Raum	No room	Geen kamer	Ouni Zëmmer	Dim ystafell	Gelarik ez	Gan seomra	Heb stevell
parseHint	Satztrigger laufed in Home Assistant vor däm Parse. conversation.process: Trigger, denn Klar, denn intent_script.	Satztrigger laufen in Home Assistant vor diesem Parse. conversation.process: Trigger, dann Klar, dann intent_script.	Sentence triggers run in Home Assistant before this parse. conversation.process: trigger, then Klar, then intent_script.	Sinsnellers loop in Home Assistant voor hierdie ontleding. conversation.process: sneller, dan Klar, dan intent_script.	Saztrigger lafen an Home Assistant virun dësem Parse. conversation.process: Trigger, dann Klar, dann intent_script.	Mae sbardunau brawddeg yn rhedeg yn Home Assistant cyn y parse hwn. conversation.process: sbardun, yna Klar, yna intent_script.	Esaldi-abiarazleak Home Assistant-en exekutatzen dira parse honen aurretik. conversation.process: abiarazlea, gero Klar, gero intent_script.	Ritheann spreagthóirí abairte in Home Assistant roimh an parse seo. conversation.process: spreagthóir, ansin Klar, ansin intent_script.	Triggerow lavar a rewl yn Home Assistant kyns an parse ma. conversation.process: trigger, ena Klar, ena intent_script.
command	Befääl	Befehl	Command	Opdrag	Kommando	Gorchymyn	Komandoa	Ordú	Gorhemmynn
analyze	Zerlege	Zerlegen	Analyse	Ontleed	Zerleeën	Dadansoddi	Aztertu	Déan anailís	Analysya
raw	Roh	Roh	Raw	Rou	Réi	Amrwd	Gordina	Amh	Kriv
speech	Antwort	Antwort	Speech	Spraak	Äntwert	Lleferydd	Hizketa	Caint	Kows
intent	Intent	Intent	Intent	Intent	Intent	Intent	Intent	Intent	Intent
slots	Slots	Slots	Slots	Slots	Slots	Slots	Slots	Slots	Slots
searchDevice	Wie säit mer dezue?	Wie sagt man dazu?	What would you call it?	Wat sou jy dit noem?	Wéi géifs du et nennen?	Beth fyddech chi'n ei alw?	Nola deituko zenioke?	Cad a thabharfá air?	Pandra wruss'ta y henwel?
alias	Alias	Alias	Alias	Alias	Alias	Alias	Alias	Ailias	Alias
room	Ruum	Raum	Room	Kamer	Zëmmer	Ystafell	Gela	Seomra	Stevell
preferred	Standardliecht	Standardlicht	Default light	Standaardlig	Standardluucht	Golau diofyn	Argi lehenetsia	Solas réamhshocraithe	Golow defowt
save	Spichere	Speichern	Save	Stoor	Späicheren	Cadw	Gorde	Sábháil	Gwitha
personality	Persönlichkeit	Persönlichkeit	Personality	Persoonlikheid	Perséinlechkeet	Personoliaeth	Nortasuna	Pearsantacht	Personoleth
mode	Modus	Modus	Mode	Modus	Modus	Modd	Modua	Mód	Modh
supportBundle	Support-Bundle	Support-Bundle	Support bundle	Ondersteuningsbundel	Support-Bundle	Pecyn cymorth	Laguntza-sorta	Beart tacaíochta	Bundel skoodhyans
recordProtocol	Protokoll spichere	Protokoll speichern	Record protocol	Neem protokol op	Protokoll ophuelen	Cofnodi protocol	Grabatu protokoloa	Taifead prótacal	Rekordya protokol
includeRawText	Rohtext i Downloads ufnää	Rohtext in Downloads aufnehmen	Include raw text in downloads	Sluit rou teks in aflaaie in	Réitext an Downloads ophuelen	Cynnwys testun amrwd yn y lawrlwythiadau	Sartu testu gordina deskargetan	Cuir amhthéacs le híoslódálacha	Gorra tekst kriv y'n iskargansow
semanticAdapters	Lokali Semantik-Adapter	Lokale Semantik-Adapter	Local semantic adapters	Plaaslike semantiese adapters	Lokal Semantik-Adapter	Addasyddion semantig lleol	Semantika-egokitzaile lokalak	Oiriúnaitheoirí séimeantacha áitiúla	Adapterow semantek leel
downloadDataset	Dataset abelade	Dataset herunterladen	Download dataset	Laai datastel af	Dataset eroflueden	Lawrlwytho set ddata	Deskargatu datu-multzoa	Íoslódáil tacar sonraí	Iskarga sett data
downloadProtocol	Protokoll abelade	Protokoll herunterladen	Download protocol	Laai protokol af	Protokoll eroflueden	Lawrlwytho protocol	Deskargatu protokoloa	Íoslódáil prótacal	Iskarga protokol
deleteSelected	Uswahl lösche	Auswahl löschen	Delete selection	Vee seleksie uit	Auswiel läschen	Dileu'r dewis	Ezabatu hautaketa	Scrios an roghnú	Dilea an dewis
clearAll	Alls lösche	Alle löschen	Delete all	Vee alles uit	Alles läschen	Dileu'r cyfan	Ezabatu dena	Scrios gach rud	Dilea puptra
token	Write-Token (LAN)	Write-Token (LAN)	Write token (LAN)	Skryf-token (LAN)	Schreif-Token (LAN)	Tocyn ysgrifennu (LAN)	Idazteko tokena (LAN)	Comhartha scríofa (LAN)	Tokyn skrifa (LAN)
customJson	Eigeni Sätz als JSON	Eigene Sätze als JSON	Custom phrases as JSON	Eie frases as JSON	Eegen Sätz als JSON	Ymadroddion personol fel JSON	Esamolde pertsonalak JSON gisa	Frásaí saincheaptha mar JSON	Lavarrow personel avel JSON
customHint	Phrase uf bekannte Intent. Policies liged danäbe, nöd als HA-Automatone.	Phrase auf bekannten Intent. Policies liegen daneben, nicht in HA-Automationen.	Phrase onto a known intent. Policies sit beside this, not as HA automations.	Frase op 'n bekende intent. Policies sit hiernaas, nie as HA-outomatiserings nie.	Phrase op e bekannten Intent. Policies leien niewendrun, net als HA-Automatiounen.	Ymadrodd ar intent hysbys. Mae Policies wrth ymyl hyn, nid fel awtomeiddiadau HA.	Esamoldea intent ezagun bati. Policies ondoan daude, ez HA automatizazio gisa.	Frása ar intent aitheanta. Tá Policies in aice leis seo, ní mar uathoibrithe HA.	Lavar war intent aswonnys. Policies a esedh ryb henna, a-nans awtomatyansow HA.
addPhrase	Satz dezue	Satz hinzufügen	Add phrase	Voeg frase by	Saz derbäisetzen	Ychwanegu ymadrodd	Gehitu esamoldea	Cuir frása leis	Keworra lavar
previewRule	Vorschau	Vorschau	Preview	Voorskou	Virschau	Rhagolwg	Aurreikusi	Réamhamharc	Ragweles
explainRule	Erkläre	Erklären	Explain	Verklaar	Erklären	Egluro	Azaldu	Mínigh	Displegya
rollback	Zruggrolle	Zurückrollen	Roll back	Rol terug	Zeréckrullen	Rholio'n ôl	Desegin atzera	Roll siar	Dasrollen
noRules	No kei eigeni Sätz.	Noch keine eigenen Sätze.	No custom phrases yet.	Nog geen eie frases nie.	Nach keng eegen Sätz.	Dim ymadroddion personol eto.	Oraindik ez dago esamolde pertsonalik.	Níl aon fhrásaí saincheaptha fós.	Nyns eus lavarrow personel hwath.
engineOffline	Engine nöd erreichbar. D Liste blibt leer, bis es Live-Lade klappt.	Engine nicht erreichbar. Die Liste bleibt leer, bis ein Live-Laden klappt.	Engine unreachable. This list is empty until a live load succeeds.	Enjin onbereikbaar. Hierdie lys bly leeg totdat 'n lewendige laai slaag.	Engine net erreechbar. Dës Lëscht bleift eidel, bis e Live-Luede klappt.	Ni ellir cyrraedd yr injin. Mae'r rhestr hon yn wag nes bod llwyth byw yn llwyddo.	Motorra ez dago eskuragarri. Zerrenda hutsik geratzen da zuzeneko karga bat lortu arte.	Níl an t-inneall insroichte. Fanann an liosta seo folamh go dtí go n-éiríonn le luchtú beo.	Ny yllir drehedhes an jynn. Gwag yw an rol ma erna sew an karga byw.
emptyBundle	No kei Ufzeichnige. Schalt s Bundle aa und probiers mit eme Satz.	Noch keine Aufzeichnungen. Schalte das Bundle ein und teste einen Satz.	No recordings yet. Enable the bundle and try a sentence.	Nog geen opnames nie. Skakel die bundle aan en probeer 'n sin.	Nach keng Ophuelungen. Schalt de Bundle un a probéier e Saz.	Dim recordiadau eto. Galluogwch y bundle a cheisiwch frawddeg.	Oraindik ez dago grabaziorik. Gaitu bundlea eta proba ezazu esaldi bat.	Níl aon taifeadtaí fós. Cumasaigh an bundle agus bain triail as abairt.	Nyns eus rekordyansow hwath. Gweres an bundle ha previ lavar.
confirmApply	Die Vorschläg übernää?	Diese Vorschläge übernehmen?	Apply these suggestions?	Pas hierdie voorstelle toe?	Dës Virschléi iwwerhuelen?	Cymhwyso'r awgrymiadau hyn?	Aplikatu iradokizun hauek?	Na moltaí seo a chur i bhfeidhm?	Devnydhya an profyansow ma?
cancel	Abbräche	Abbrechen	Cancel	Kanselleer	Ofbriechen	Canslo	Utzi	Cuir ar ceal	Hedhi
apply	Übernää	Übernehmen	Apply	Pas toe	Iwwerhuelen	Cymhwyso	Aplikatu	Cuir i bhfeidhm	Devnydhya
close	Schliisse	Schließen	Close	Sluit	Zoumaachen	Cau	Itxi	Dún	Degea
low	tief	niedrig	low	laag	niddreg	isel	baxua	íseal	isel
medium	mittel	mittel	medium	medium	mëttel	canolig	ertaina	meánach	kres
high	hoch	hoch	high	hoog	héich	uchel	altua	ard	ughel
source	Quälle	Quelle	Source	Bron	Quell	Ffynhonnell	Iturria	Foinse	Fenten
language	Sprach	Sprache	Language	Taal	Sprooch	Iaith	Hizkuntza	Teanga	Yeth
time	Zyt	Zeit	Time	Tyd	Zäit	Amser	Ordua	Am	Termyn
text	Satz	Satz	Sentence	Sin	Saz	Brawddeg	Esaldi	Abairt	Lavar
answer	Antwort	Antwort	Answer	Antwoord	Äntwert	Ateb	Erantzuna	Freagra	Gorthyp
graphHint	Rüüm als Cluster, Grät nach Treffsicherheit.	Räume als Cluster, Geräte nach Treffsicherheit.	Rooms as clusters, devices coloured by confidence.	Kamers as groepe, toestelle gekleur volgens vertroue.	Zëmmer als Cluster, Apparater no Treffsécherheet.	Ystafelloedd fel clystyrau, dyfeisiau wedi'u lliwio yn ôl hyder.	Gelak kluster gisa, gailuak konfiantzaren arabera koloreztatuta.	Seomraí mar bhraislí, gléasanna daite de réir muiníne.	Stevellow avel klusteryow, devisys liwys dre fydh.
resetLayout	Layout zruggsetze	Layout zurücksetzen	Reset layout	Stel uitleg terug	Layout zerécksetzen	Ailosod cynllun	Berrezarri diseinua	Athshocraigh leagan amach	Daswul an layout
score	Score	Score	Score	Telling	Score	Sgôr	Puntuazioa	Scór	Skor
noIntent	Kei Intent	Kein Intent	No intent	Geen intent	Kee Intent	Dim intent	Intentik ez	Gan intent	Heb intent
loading	Klar ladet...	Klar lädt...	Loading Klar...	Klar laai...	Klar lued...	Yn llwytho Klar...	Klar kargatzen...	Klar á lódáil...	Ow karga Klar...
nluRagHint	Standard uus. Nume de scho gmatchti Uschnitt, nie Assist-Wärchzüüg.	Standard aus. Nur der bereits gematchte Ausschnitt, keine Assist-Werkzeuge.	Off by default. Matched slice only, never Assist tools.	Verstek af. Slegs die ooreenstemmende stuk, nooit Assist-nutsgoed nie.	Standard aus. Nëmmen de gematchten Ausschnëtt, keng Assist-Geschir.	I ffwrdd yn ddiofyn. Dim ond y darn sy'n cyfateb, byth offer Assist.	Itzalita lehenetsita. Bat datorren zatia soilik, inoiz ez Assist tresnak.	As de réir réamhshocraithe. An slisne meaitseáilte amháin, ná huirlisí Assist choíche.	Dhe-ves dre dhefowt. An tamm kevys yn unnik, nevra offerow Assist.
confirmRisky	Riskanti Aktione bestätige	Riskante Aktionen bestätigen	Confirm risky actions	Bevestig riskante aksies	Riskant Aktiounen bestätegen	Cadarnhau gweithredoedd peryglus	Berretsi ekintza arriskutsuak	Deimhnigh gníomhartha contúirteacha	Afydhya gwriansow peryllus
languages	Sprache	Sprachen	Languages	Tale	Sproochen	Ieithoedd	Hizkuntzak	Teangacha	Yethow
languageSearch	Sprach sueche	Sprache suchen	Search languages	Soek tale	Sprooche sichen	Chwilio ieithoedd	Bilatu hizkuntzak	Cuardaigh teangacha	Hwilas yethow
allLanguages	Alli Sprache	Alle Sprachen	All languages	Alle tale	All Sproochen	Pob iaith	Hizkuntza guztiak	Gach teanga	Pub yeth
noLanguageMatch	Kei Sprach gfunde	Keine Sprache gefunden	No language found	Geen taal gevind nie	Keng Sprooch fonnt	Ni chanfuwyd iaith	Ez da hizkuntzarik aurkitu	Níor aimsíodh teanga	Ny veu yeth kevys
languageHint	Suech und wähl Locales. Alli Sprache laat jedes kompilierti Pack aktiv.	Suche und wähle Locales. Alle Sprachen lässt jedes kompilierte Pack aktiv.	Search and pick locales. All languages keeps every compiled pack enabled.	Soek en kies locales. Alle tale hou elke gekompileerde pak aktief.	Sich a wiel Locales. All Sproochen léisst all kompiléiert Pack aktiv.	Chwiliwch a dewiswch locales. Pob iaith sy'n cadw pob pecyn wedi'i lunio wedi'i alluogi.	Bilatu eta aukeratu locales. Hizkuntza guztiek pakete konpilatu bakoitza gaituta uzten dute.	Cuardaigh agus roghnaigh locales. Coinníonn gach teanga gach pacáiste tiomsaithe ar siúl.	Hwil ha dewis locales. Pub yeth a with pub pakke kompilyes gwrys.
mappingHint	Zuornig sind Alias für Grät im Graph. Kalender erschined, wenn d Domain calendar übernoo wird. Assist folgt em Sprachpack, die Oberfläche de Operator-Sprach.	Zuordnung sind Aliase für Geräte im Graph. Kalender erscheinen, wenn die Domain calendar übernommen wird. Assist folgt dem Sprachpack, diese Oberfläche der Operator-Sprache.	Mapping is aliases for graph entities. Calendars appear after the calendar domain is included. Assist follows the language pack; this chrome follows the operator language.	Kartering is aliase vir grafiek-entiteite. Kalenders verskyn nadat die calendar-domein ingesluit is. Assist volg die taalpak; hierdie chrome volg die operateurtaal.	Zouuerdnung sinn Aliassen fir Apparater am Graph. Kalenner erschéngen, nodeems d Domain calendar iwwerholl gëtt. Assist follegt dem Sproochpack, dës Uewerfläch der Operator-Sprooch.	Mapio yw aliasau ar gyfer endidau'r graff. Mae calendrau'n ymddangos ar ôl cynnwys y parth calendar. Mae Assist yn dilyn y pecyn iaith; mae'r chrome hwn yn dilyn iaith y gweithredwr.	Esleipena grafo-entitateen aliasak dira. Egutegiak calendar domeinua sartu ondoren agertzen dira. Assist-ek hizkuntza-paketea jarraitzen du; chrome honek operadorearen hizkuntza.	Is ailiasanna iad mapáil d'eintitis an ghraif. Tá féilirí le feiceáil tar éis fearann calendar a chur isteach. Leanann Assist an pacáiste teanga; leanann an chrome seo teanga an oibreora.	Mappa yw aliasow rag entites an graf. Devisow a omdhiskwa wosa tiredh calendar dhe vos ygor. Assist a hol an pakke yeth; an chrome ma a hol yeth an oberyas.
parseSample	Mach s Liecht i de Stube aa	Mach das Licht in der Wohnstube an	Turn on the lounge light	Skakel die sitkamerlig aan	Maach d Luucht am Wunnzëmmer un	Troi golau'r ystafell fyw ymlaen	Piztu egongelako argia	Cas solas an tseomra suí	Enow golow an stafell
tryOn	Liecht i {room} aa	Licht in {room} an	Turn on the light in {room}	Skakel die lig in {room} aan	Maach d Luucht an {room} un	Troi'r golau ymlaen yn {room}	Piztu argia {room}	Cas an solas {room}	Enow an golow {room}
tryLock	Isch d Tüür zuegschlosse?	Ist die Tür versperrt?	Is the door locked?	Is die deur gesluit?	Ass d Dier zougeschloss?	Ydy'r drws wedi'i gloi?	Atea giltzatuta dago?	An bhfuil an doras faoi ghlas?	Yw an daras alhwedhys?
tryTime	Wie spaat ischs?	Wie spät ist es?	What time is it?	Hoe laat is dit?	Wéi spéit ass et?	Faint o'r gloch ydy hi?	Zer ordu da?	Cén t-am é?	Py eur yw?
tryNight	Gueti Nacht	Gute Nacht	Good night	Goeie nag	Gutt Nuecht	Nos da	Gabon	Oíche mhaith	Nos da
tryUndo	Rückgängig	Rückgängig	Undo that	Ontdoen dit	Réckgängeg	Dadwneud hynny	Desegin hori	Cealaigh é sin	Diswul henna
tryRoom	de Chuchi	der Küche	the kitchen	die kombuis	der Kichen	y gegin	sukaldean	sa chistin	y'n gegin
nluIgnore	Nöd für Status oder Schalte binde	Nicht für Status oder Schalten binden	Do not bind for status or power	Moenie vir status of krag bind nie	Net fir Status oder Schalten bannen	Peidio â rhwymo ar gyfer statws neu bŵer	Ez lotu egoerarako edo potentziarako	Ná ceangail le haghaidh stádais nó cumhachta	Na gevren rag studh po nerth
nluIgnoreHint	Nimmt s Grät usem Resolver. Hilft bi falsch benennte Helfer.	Nimmt das Gerät aus dem Resolver. Hilft bei falsch benannten Helfern.	Drops this device from the resolver. Use for misnamed helpers.	Verwyder hierdie toestel uit die resolver. Gebruik vir verkeerd genoemde helpers.	Hëlt den Apparat aus dem Resolver. Hëlleft bei falsch benannten Hëllefer.	Tynnu'r ddyfais hon o'r datrysydd. Defnyddiwch ar gyfer cynorthwywyr wedi'u camenwi.	Gailu hau resolvertik kentzen du. Erabili gaizki izendatutako laguntzaileentzat.	Baineann sé an gléas seo ón resolver. Úsáid le haghaidh cúntóirí mí-ainmnithe.	A dhrop an devis ma dhyworth an resolver. Us rag gweresow kammhenwys.
savePhrase	Als Phrase spichere	Als Phrase speichern	Save as phrase	Stoor as frase	Als Phrase späicheren	Cadw fel ymadrodd	Gorde esamolde gisa	Sábháil mar fhrása	Gwitha avel lavar
ignoreTarget	Das Ziel ignoriere	Dieses Ziel ignorieren	Ignore this target	Ignoreer hierdie teiken	Dëst Zil ignoréieren	Anwybyddu'r targed hwn	Ez ikusi helburu hau	Déan neamhaird den sprioc seo	Skonya an amkan ma
teachSaved	Gspicheret.	Gespeichert.	Saved.	Gestoor.	Gespäichert.	Wedi'i gadw.	Gordeta.	Sábháilte.	Gwithys.
journal	Gspröchsjournal	Gesprächsjournal	Conversation journal	Gespreksjoernaal	Gespréichsjournal	Dyddiadur sgwrs	Elkarrizketa-egunkaria	Dialann comhrá	Lyver keskows
journalHint	Letschti 200 Turns, 24 Stunde, redigiert. Rohtext nume mit em Bundle.	Letzte 200 Turns, 24 Stunden, redigiert. Rohtext nur mit Bundle.	Last 200 turns, 24 hours, redacted. Raw text only with the bundle.	Laaste 200 beurte, 24 uur, geredigeer. Rou teks slegs met die bundle.	Lescht 200 Turns, 24 Stonnen, redigéiert. Réitext nëmme mam Bundle.	200 tro diwethaf, 24 awr, wedi'u golygu. Testun amrwd dim ond gyda'r bundle.	Azken 200 txanda, 24 ordu, redactatuta. Testu gordina bundlearekin soilik.	200 casadh deireanach, 24 uair, deargtha. Amhthéacs leis an bundle amháin.	200 treylyans diwettha, 24 our, redaktys. Tekst kriv gans an bundle yn unnik.
decisionMix	Entscheidige	Entscheidungen	Decisions	Besluite	Decisiounen	Penderfyniadau	Erabakiak	Cinntí	Ernansow
mixCaption	Quälle: Gspröchsjournal, Turns pro Tag	Quelle: Gesprächsjournal, Turns pro Tag	Source: conversation journal, turns per day	Bron: gespreksjoernaal, beurte per dag	Quell: Gespréichsjournal, Turns pro Dag	Ffynhonnell: dyddiadur sgwrs, troeon y dydd	Iturria: elkarrizketa-egunkaria, txandak eguneko	Foinse: dialann comhrá, casanna in aghaidh an lae	Fenten: lyver keskows, treylyansow pub dydh
coverageCaption	Quälle: Home-Graph, Aateil vo de Grät	Quelle: Home-Graph, Anteil der Geräte	Source: home graph, share of devices	Bron: huisgrafiek, aandeel van toestelle	Quell: Home-Graph, Undeel vun den Apparater	Ffynhonnell: graff y cartref, cyfran y dyfeisiau	Iturria: etxeko grafoa, gailuen kuota	Foinse: graf an tí, sciar na ngléasanna	Fenten: graf an dre, rann an devisys
latency	Stufezyt	Stufenzeit	Stage time	Stadiatyd	Stufenzäit	Amser cam	Etapa-denbora	Am céime	Termyn gradh
latencyCaption	Quälle: Parse-Trace, Mikrosekunde	Quelle: Parse-Trace, Mikrosekunden	Source: parse trace, microseconds	Bron: ontleedspoor, mikrosekondes	Quell: Parse-Trace, Mikrosekonnen	Ffynhonnell: ôl parse, microsecondau	Iturria: parse-aztarna, mikrosegundoak	Foinse: rian parse, micrea-soicindí	Fenten: lorgh parse, mikrosekondys
unitsTurns	Turns	Turns	turns	beurte	Turns	troeon	txanda	casanna	treylyansow
timeline	Verlauf	Verlauf	Timeline	Tydlyn	Zäitlinn	Llinell amser	Denbora-lerroa	Amlíne	Linen-termyn
noConversations	No kei Journal-Yträg.	Noch keine Journal-Einträge.	No journal entries yet.	Nog geen joernaalinskrywings nie.	Nach keng Journal-Andréi.	Dim cofnodion dyddiadur eto.	Oraindik ez dago egunkari-sarrerarik.	Níl aon iontrálacha dialainne fós.	Nyns eus entraow lyver hwath.
when	Wenn	Wenn	When	Wanneer	Wann	Pan	Noiz	Nuair	Pan
then	Denn	Dann	Then	Dan	Dann	Yna	Orduan	Ansin	Ena
priority	Reiefolg (erschti passendi Regle gwünnt)	Reihenfolge (erste zutreffende Regel gewinnt)	Order (first matching user rule wins)	Volgorde (eerste passende gebruikersreël wen)	Reiefolleg (éischt passend Benotzerreegel gewënnt)	Trefn (y rheol defnyddiwr gyntaf sy'n cyfateb sy'n ennill)	Ordena (erabiltzaile-arau bat datorren lehenengoak irabazten du)	Ord (buaileann an chéad riail úsáideora a mheaitseálann)	Aray (an kynsa rewl usyer a gev a wayn)
evaluator	Policy-Prüefer	Policy-Prüfer	Policy evaluator	Policy-evalueerder	Policy-Prüfer	Gwerthuswr policy	Policy ebaluatzailea	Measúnóir policy	Prisyer policy
bakeSpeech	Variante erzüge	Varianten erzeugen	Generate variants	Genereer variante	Varianten generéieren	Cynhyrchu amrywiadau	Sortu aldaerak	Gin malairtí	Produya variantow
addRule	Regle	Regel	Rule	Reël	Reegel	Rheol	Araua	Riail	Rewl
noPolicies	No kei Policy-Regle.	Noch keine Policy-Regeln.	No policy rules yet.	Nog geen policy-reëls nie.	Nach keng Policy-Reegelen.	Dim rheolau policy eto.	Oraindik ez dago policy-araurik.	Níl aon rialacha policy fós.	Nyns eus reylow policy hwath.
compiledRisk	Kompilierts Risiko	Kompiliertes Risiko	Compiled risk	Gekompileerde risiko	Kompiléiert Risiko	Risg wedi'i lunio	Konpilatutako arriskua	Riosca tiomsaithe	Peryl kompilyes
finalBand	Band	Band	Band	Band	Band	Band	Band	Band	Band
triggerFirst	HA-Satztrigger zersch, denn Klar, denn registrierte Intent.	HA-Satztrigger zuerst, dann Klar, dann registrierter Intent.	HA sentence triggers first, then Klar, then a registered intent.	HA-sinsnellers eers, dan Klar, dan 'n geregistreerde intent.	HA-Saztrigger als éischt, dann Klar, dann e registréierten Intent.	Sbardunau brawddeg HA yn gyntaf, yna Klar, yna intent cofrestredig.	HA esaldi-abiarazleak lehenik, gero Klar, gero erregistratutako intent.	Spreagthóirí abairte HA ar dtús, ansin Klar, ansin intent cláraithe.	Triggerow lavar HA kynsa, ena Klar, ena intent registrys.
discarded	Weggleit	Verworfen	Discarded	Verwerp	Verworf	Wedi'i wrthod	Baztertuta	Caite i leataobh	Skonyes
stageTokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens	Tokens
stageBind	Bind	Bind	Bind	Bind	Bind	Bind	Bind	Bind	Bind
stageRank	Rank	Rank	Rank	Rank	Rank	Rank	Rank	Rank	Rank
stagePolicy	Policy	Policy	Policy	Policy	Policy	Policy	Policy	Policy	Policy
stageBand	Band	Band	Band	Band	Band	Band	Band	Band	Band
effectConfirm	Bestätige	Bestätigen	Confirm	Bevestig	Bestätegen	Cadarnhau	Berretsi	Deimhnigh	Afydhya
effectBlock	Blockiere	Blockieren	Block	Blokkeer	Blockéieren	Rhwystro	Blokeatu	Blocáil	Lett
effectAllow	Erlaube	Erlauben	Allow	Laat toe	Erlaben	Caniatáu	Baimendu	Ceadaigh	Asoen
effectPreferEntity	Grät bevorzuge	Gerät bevorzugen	Prefer entity	Verkies entiteit	Apparat léiwer	Ffafrio endid	Entitatea hobetsi	Tabhair tús áite don eintiteas	Gwella entite
effectPreferArea	Ruum bevorzuge	Raum bevorzugen	Prefer area	Verkies area	Zëmmer léiwer	Ffafrio ardal	Eremua hobetsi	Tabhair tús áite don limistéar	Gwella tiredh
effectReply	Antwort ohni Intent	Antwort ohne Intent	Reply without intent	Antwoord sonder intent	Äntwert ouni Intent	Ateb heb intent	Erantzun intentik gabe	Freagair gan intent	Gorthyp heb intent
effectScript	Skript	Skript	Script	Skrip	Skript	Sgript	Script	Script	Skript
effectTemplate	Template	Template	Template	Template	Template	Template	Template	Template	Template
effectLlm	LLM-Prompt	LLM-Prompt	LLM prompt	LLM-opdrag	LLM-Prompt	Anogwr LLM	LLM gonbita	Leid LLM	Prompt LLM
payloadReply	Antworttext	Antworttext	Reply text	Antwoordteks	Äntwerttext	Testun ateb	Erantzun-testua	Téacs freagra	Tekst gorthyp
payloadScript	Skript (script.good_night oder good_night)	Skript (script.good_night oder good_night)	Script (script.good_night or good_night)	Skrip (script.good_night of good_night)	Skript (script.good_night oder good_night)	Sgript (script.good_night neu good_night)	Script (script.good_night edo good_night)	Script (script.good_night nó good_night)	Skript (script.good_night po good_night)
payloadTemplate	Home Assistant Template; {{ text }} isch de Satz	Home Assistant-Template; {{ text }} ist der Satz	Home Assistant template; {{ text }} is the utterance	Home Assistant-sjabloon; {{ text }} is die uiting	Home Assistant Template; {{ text }} ass de Saz	Templed Home Assistant; {{ text }} yw'r ymadrodd	Home Assistant txantiloia; {{ text }} da esaldia	Teimpléad Home Assistant; is é {{ text }} an caint	Template Home Assistant; {{ text }} yw an lavar
payloadLlm	System-Prompt für de Fallback-Agent	System-Prompt für den Fallback-Agenten	System prompt for the fallback agent	Stelselopdrag vir die terugval-agent	System-Prompt fir den Ersatz-Agent	Anogwr system ar gyfer yr asiant wrth gefn	Sistema-gonbita ordezko agentearentzat	Leid chórais don ghníomhaire cúltaca	Prompt system rag an agent reserv
whenPhrase	Satz	Satz	Phrase	Frase	Saz	Ymadrodd	Esamoldea	Frása	Lavar
chatMode	Gspröch	Gespräch	Chat	Klets	Chat	Sgwrs	Berriketa	Comhrá	Keskows
variantPreview	Sprachvariante	Sprachvariante	Speech variant	Spraakvariant	Sproochvariant	Amrywiad lleferydd	Hizketa-aldaera	Malairt cainte	Variant kows
policies	Policies	Policies	Policies	Policies	Policies	Policies	Policies	Policies	Policies
routines	Routine	Routinen	Routines	Routines	Routinen	Trefnau	Errutinak	Gnáthaimh	Routinyow
routineHint	En gredte Name startet es Home Assistant Skript. Gueti Nacht gwünnt vor de Begrüessig.	Ein gesprochener Name startet ein Home Assistant-Skript. Gute Nacht gewinnt vor der Begrüßung.	A spoken name starts a Home Assistant script. Good night wins over the greeting.	'n Gesproke naam begin 'n Home Assistant-skrip. Goeie nag wen bo die groet.	En ausgeschwatenen Numm start e Home Assistant Skript. Gutt Nuecht gewënnt virun der Begréissung.	Mae enw llafar yn cychwyn sgript Home Assistant. Nos da sy'n ennill dros y cyfarchiad.	Izen mintzatu batek Home Assistant script bat abiatzen du. Gabon-ek agurra gainditzen du.	Tosaíonn ainm labhartha script Home Assistant. Buaileann Oíche mhaith an beannacht.	Hanow kewsys a dhalleth skript Home Assistant. Nos da a wayn dres an dynnargh.
routinePhraseHint	Gueti Nacht	Gute Nacht	Good night	Goeie nag	Gutt Nuecht	Nos da	Gabon	Oíche mhaith	Nos da
addRoutine	Routine aalege	Routine anlegen	Add routine	Voeg routine by	Routine derbäisetzen	Ychwanegu trefn	Gehitu errutina	Cuir gnáthamh leis	Keworra routin
noRoutines	No kei Routine.	Noch keine Routinen.	No routines yet.	Nog geen routines nie.	Nach keng Routinen.	Dim trefnau eto.	Oraindik ez dago errutinarik.	Níl aon ghnáthaimh fós.	Nyns eus routinyow hwath.
routineInvalid	Phrase und script.xxx sind nötig.	Phrase und script.xxx sind nötig.	A phrase and script.xxx are required.	'n Frase en script.xxx is nodig.	Phrase an script.xxx sinn néideg.	Mae angen ymadrodd a script.xxx.	Esamoldea eta script.xxx behar dira.	Tá frása agus script.xxx ag teastáil.	Lavar ha script.xxx yw res.
lastTurn	Letschte Satz	Letzter Satz	Last turn	Laaste beurt	Leschte Saz	Tro diwethaf	Azken txanda	Casadh deireanach	Treylyans diwettha
heardIn	Ghört in	Gehört in	Heard in	Gehoor in	Héieren zu	Clywyd yn	Hemen entzuna	Cloiste i	Klewys yn
tryThese	Foif Sätz i dine Rüüm	Fünf Sätze in deinen Räumen	Five sentences in your rooms	Vyf sinne in jou kamers	Fënnef Sätz an dengen Zëmmer	Pump brawddeg yn eich ystafelloedd	Bost esaldi zure geletan	Cúig abairt i do sheomraí	Pymp lavar y'th stevellow
tryTheseHint	Tipp uf en Satz, zum ihn im Labor z prüefe.	Tippe einen Satz, um ihn im Labor zu prüfen.	Tap a sentence to try it in the lab.	Tik 'n sin om dit in die lab te probeer.	Tippt op e Saz, fir e am Labo ze testen.	Tapiwch frawddeg i'w threialu yn y lab.	Sakatu esaldi bat laborategian probatzeko.	Tapáil abairt chun triail a bhaint aisti sa tsaotharlann.	Klik lavar rag y brevi y'n labordy.
anyRoom	Kei Satellit	Kein Satellit	No satellite	Geen satelliet	Kee Satellit	Dim lloeren	Sateliterik ez	Gan satailít	Heb loer
personalityHa	Persönlichkeit i Home Assistant → Klar NLU → Persönlichkeit setze.	Persönlichkeit nur in Home Assistant → Klar NLU → Persönlichkeit.	Set personality in Home Assistant → Klar NLU → Personality.	Stel persoonlikheid in Home Assistant → Klar NLU → Persoonlikheid.	Perséinlechkeet an Home Assistant → Klar NLU → Perséinlechkeet setzen.	Gosodwch bersonoliaeth yn Home Assistant → Klar NLU → Personoliaeth.	Ezarri nortasuna Home Assistant → Klar NLU → Nortasuna atalean.	Socraigh pearsantacht in Home Assistant → Klar NLU → Pearsantacht.	Settya personoleth yn Home Assistant → Klar NLU → Personoleth.
"""

PACKS = parse_table(CODES, TABLE)
