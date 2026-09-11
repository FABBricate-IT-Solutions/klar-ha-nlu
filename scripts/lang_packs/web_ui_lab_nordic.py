"""Lab / mapping / backup chrome for Nordic and Baltic Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_rows

CODES = ["da", "nb", "sv", "fi", "is", "et", "lt", "lv"]

TABLE = """
parseHint	Lab er Assist-stien for det valgte sprog. Beslutning og intents her kører Klar. Sætningsudløsere kører kun, hvis Klar er utilgængelig.	Lab er Assist-stien for valgt språk. Beslutning og intents her kjører Klar. Setningsutløsere kjører bare hvis Klar er utilgjengelig.	Labbet är Assist-vägen för det valda språket. Beslut och intents här kör Klar. Meningsutlösare körs bara om Klar är onåbar.	Labra on valitun kielen Assist-polku. Päätöksen ja intentit tässä suorittaa Klar. Lauselaukaisimet ajetaan vain, jos Klar ei ole tavoitettavissa.	Labbið er Assist-leiðin fyrir valið tungumál. Ákvörðun og intents hér keyrir Klar. Setningakveikjur keyra aðeins ef Klar er óaðgengilegt.	Labor on valitud keele Assist-rada. Otsuse ja intentid siin täidab Klar. Lausetrigerid käivituvad ainult siis, kui Klar pole kättesaadav.	Laboratorija yra pasirinktos kalbos Assist kelias. Sprendimą ir intentus čia vykdo Klar. Sakinių trigeriai paleidžiami tik jei Klar nepasiekiamas.	Laboratorija ir izvēlētās valodas Assist ceļš. Lēmumu un intentus šeit izpilda Klar. Teikumu trigeri darbojas tikai tad, ja Klar nav sasniedzams.
triggerFirst	Klar-parse, derefter den sti. Assist kører ikke en anden intent, en sætningsudløser eller et vejr-fallback.	Klar-parse, deretter den stien. Assist kjører ikke en annen intent, en setningsutløser eller et vær-fallback.	Klar-parse, sedan den vägen. Assist kör inte en annan intent, en meningsutlösare eller ett väder-fallback.	Klar-jäsennys, sitten tuo polku. Assist ei aja toista intenttiä, lauselaukaisinta eikä sää-vara reversiota.	Klar-greining, síðan sú leið. Assist keyrir ekki annan intent, setningakveikju eða veður-varaleið.	Klar-parsimine, seejärel see rada. Assist ei käivita teist intentit, lausetrigerit ega ilma-varuplaani.	Klar-analizė, tada tas kelias. Assist nepaleidžia kito intento, sakinio trigerio ar orų atsarginio kelio.	Klar-analīze, tad šis ceļš. Assist nepalaiz citu intentu, teikuma trigeri vai laikapstākļu rezervi.
labPipeline	Pipeline	Pipeline	Pipeline	Putki	Pípa	Torustik	Vamzdynas	Caurule
labChipContextOnly	kun kontekst	kun kontekst	endast kontext	vain konteksti	aðeins samhengi	ainult kontekst	tik kontekstas	tikai konteksts
labChipNluRag	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG	NLU-RAG
labChipSemantic	semantisk	semantisk	semantisk	semanttinen	merkingarleg	semantiline	semantinis	semantisks
labChipNoConfirm	uden bekræftelse	uten bekreftelse	utan bekräftelse	ilman vahvistusta	án staðfestingar	kinnituseta	be patvirtinimo	bez apstiprinājuma
labChipLlmRefine	LLM-forfinelse	LLM-forfining	LLM-förfining	LLM-viilaus	LLM-fínstilling	LLM-lihvimine	LLM patikslinimas	LLM precizēšana
labChipCalendarLlm	kalender-LLM	kalender-LLM	kalender-LLM	kalenteri-LLM	dagatal-LLM	kalendri-LLM	kalendoriaus LLM	kalendāra LLM
labChipQuietAck	kort bekræftelse	kort bekreftelse	kort bekräftelse	lyhyt kuittaus	stutt staðfesting	tyhi kinnitus	trumpas patvirtinimas	īss apstiprinājums
labChipLlmTools	LLM-værktøjer	LLM-verktøy	LLM-verktyg	LLM-työkalut	LLM-tæki	LLM-tööriistad	LLM įrankiai	LLM rīki
labChipLlmChat	LLM-chat	LLM-chat	LLM-chatt	LLM-keskustelu	LLM-spjall	LLM-vestlus	LLM pokalbis	LLM tērzēšana
labChipConfirmRisky	bekræft risiko	bekreft risiko	bekräfta risk	vahvista riski	staðfesta áhættu	kinnita risk	patvirtinti riziką	apstiprināt risku
labDecisionExecute	Klar udfører	Klar utfører	Klar kör	Klar suorittaa	Klar keyrir	Klar täidab	Klar vykdo	Klar izpilda
labDecisionBriefing	briefing	briefing	briefing	briefing	briefing	briefing	briefingas	brīfings
labParse	Klar-parse	Klar-parse	Klar-parse	Klar-jäsennys	Klar-greining	Klar-parsimine	Klar-analizė	Klar-analīze
reasonMissingArea	intet rum	uten rom	saknar rum	ei huonetta	án herbergis	ruumita	nėra kambario	nav telpas
reasonWeakName	svagt navn	svakt navn	svagt namn	heikko nimi	veikt nafn	nõrk nimi	silpnas vardas	vājš nosaukums
reasonReady	klar	klar	redo	valmis	tilbúið	valmis	paruošta	gatavs
reasonMatch	match	treff	träff	osuma	samsvörun	vaste	atitikmuo	sakritība
sentencesEmpty	Én sætning på en kendt intent. Ikke én sætning per lys.	Én setning på en kjent intent. Ikke én setning per lys.	En mening till en känd intent. Inte en fras per lampa.	Yksi lause tunnettuun intenttiin. Ei yhtä lausetta per valo.	Ein setning á þekktan intent. Ekki ein setning per ljós.	Üks lause tuntud intentile. Mitte üks lause iga tule kohta.	Vienas sakinys žinomam intentui. Ne po sakinį kiekvienai šviesai.	Viens teikums zināmam intentam. Neviens teikums katrai gaismai.
policiesEmpty	Første matchende regel vinder. Vælg enhed, rum eller etage — tast ikke id'er.	Første matchende regel vinner. Velg enhet, rom eller etasje — ikke skriv id-er.	Första matchande regel vinner. Välj enhet, rum eller våning — skriv inte id.	Ensimmäinen täsmäävä sääntö voittaa. Valitse laite, huone tai kerros — älä kirjoita tunnuksia.	Fyrsta samsvarandi regla vinnur. Veldu tæki, herbergi eða hæð — ekki slá inn auðkenni.	Esimene sobiv reegel võidab. Vali seade, ruum või korrus — ära trüki id-sid.	Pirmoji sutampanti taisyklė laimi. Rinkitės įrenginį, kambarį ar aukštą — nerašykite id.	Pirmais atbilstošais noteikums uzvar. Izvēlieties ierīci, telpu vai stāvu — nerakstiet id.
setupAgainWhere	Gentag setup ligger under Indstillinger.	Gjenta oppsett ligger under Innstillinger.	Spela om setup finns under Inställningar.	Setup uudelleen on Asetuksissa.	Endurtaka uppsetningu er undir Stillingar.	Setup uuesti on seadetes.	Kartoti sąranką rasite Nuostatose.	Atkārtot iestatīšanu ir sadaļā Iestatījumi.
whyThisBand	Hvorfor dette bånd?	Hvorfor dette båndet?	Varför det här bandet?	Miksi tämä kaista?	Af hverju þessi rönd?	Miks see riba?	Kodėl ši juosta?	Kāpēc šī josla?
rememberAsPhrase	Husk som sætning	Husk som setning	Kom ihåg som fras	Muista lauseena	Mundu sem setningu	Jäta fraasina meelde	Įsiminti kaip frazę	Atcerēties kā frāzi
evidence	Evidens	Evidens	Evidens	Näyttö	Sönnun	Tõendid	Įrodymai	Pierādījumi
names	Navne	Navn	Namn	Nimet	Nöfn	Nimed	Vardai	Nosaukumi
settingsBackup	Sikkerhedskopi af indstillinger	Sikkerhetskopi av innstillinger	Säkerhetskopia av inställningar	Asetusten varmuuskopio	Öryggisafrit stillinga	Seadete varukoopia	Nustatymų atsarginė kopija	Iestatījumu dublējums
settingsBackupHint	Overlay, tildelinger, regler og LLM-endepunktet. Samtaler og husgrafen bliver på denne engine.	Overlay, tildelinger, regler og LLM-endepunktet. Samtaler og husgrafen blir på denne motoren.	Overlay, tilldelningar, regler och LLM-ändpunkten. Konversationer och husgrafen stannar på den här motorn.	Peittokuva, sijoitukset, säännöt ja LLM-päätepiste. Keskustelut ja talograafi jäävät tälle moottorille.	Yfirlag, úthlutanir, reglur og LLM-endapunktur. Samtöl og húsnetið vera á þessari vél.	Kattekiht, määrangud, reeglid ja LLM-otspunkt. Vestlused ja majagraaf jäävad sellele mootorile.	Perdanga, priskyrimai, taisyklės ir LLM galinis taškas. Pokalbiai ir namo grafas lieka šiame variklyje.	Pārklājums, piešķīrumi, noteikumi un LLM galapunkts. Sarunas un mājas grafs paliek uz šī dzinēja.
settingsBackupDownload	Hent indstillinger	Last ned innstillinger	Ladda ned inställningar	Lataa asetukset	Sækja stillingar	Laadi seaded alla	Atsisiųsti nustatymus	Lejupielādēt iestatījumus
settingsBackupIncludeKey	Medtag API-nøgle	Ta med API-nøkkel	Inkludera API-nyckel	Sisällytä API-avain	Hafa API-lykil	Kaasa API-võti	Įtraukti API raktą	Iekļaut API atslēgu
settingsBackupIncludeKeyHint	Fra som standard. Til: arkivet indeholder den gemte LLM-nøgle.	Av som standard. På: arkivet inneholder den lagrede LLM-nøkkelen.	Av som standard. På: arkivet innehåller den lagrade LLM-nyckeln.	Oletuksena pois. Päällä: arkisto sisältää tallennetun LLM-avaimen.	Slökkt sjálfgefið. Kveikt: safninu fylgir vistaði LLM-lykillinn.	Vaikimisi väljas. Sees: arhiiv sisaldab salvestatud LLM-võtit.	Pagal numatymą išjungta. Įjungta: archyve yra saugomas LLM raktas.	Pēc noklusējuma izslēgts. Ieslēgts: arhīvā ir saglabātā LLM atslēga.
settingsBackupIncludeKeyConfirm	Arkivet kommer til at indeholde LLM-API-nøglen. Hent alligevel?	Arkivet vil inneholde LLM-API-nøkkelen. Laste ned likevel?	Arkivet kommer att innehålla LLM-API-nyckeln. Ladda ned ändå?	Arkisto sisältää LLM:n API-avaimen. Ladataanko silti?	Safnið mun innihalda LLM API-lykilinn. Sækja samt?	Arhiiv sisaldab LLM API-võtit. Laadi ikkagi alla?	Archyve bus LLM API raktas. vis tiek atsisiųsti?	Arhīvā būs LLM API atslēga. Lejupielādēt tik un tā?
settingsBackupRestore	Gendan indstillinger	Gjenopprett innstillinger	Återställ inställningar	Palauta asetukset	Endurheimta stillingar	Taasta seaded	Atkurti nustatymus	Atjaunot iestatījumus
settingsBackupRestoreConfirm	Dette overskriver alle Klar-indstillinger på denne engine.	Dette overskriver alle Klar-innstillinger på denne motoren.	Detta skriver över alla Klar-inställningar på den här motorn.	Tämä korvaa kaikki Klar-asetukset tällä moottorilla.	Þetta yfirskrifar allar Klar-stillingar á þessari vél.	See kirjutab üle kõik Klar-seaded sellel mootoril.	Tai perrašo visus Klar nustatymus šiame variklyje.	Tas pārraksta visus Klar iestatījumus uz šī dzinēja.
settingsBackupRestoreOk	Indstillinger gendannet.	Innstillinger gjenopprettet.	Inställningar återställda.	Asetukset palautettu.	Stillingar endurheimtar.	Seaded taastatud.	Nustatymai atkurti.	Iestatījumi atjaunoti.
settingsBackupRestoreFail	Kunne ikke gendanne indstillinger.	Kunne ikke gjenopprette innstillinger.	Kunde inte återställa inställningar.	Asetuksia ei voitu palauttaa.	Tókst ekki að endurheimta stillingar.	Seadeid ei õnnestunud taastada.	Nepavyko atkurti nustatymų.	Neizdevās atjaunot iestatījumus.
settingsBackupPickFile	Vælg arkiv	Velg arkiv	Välj arkiv	Valitse arkisto	Velja safn	Vali arhiiv	Pasirinkti archyvą	Izvēlēties arhīvu
speechRefined	LLM-forfinelse	LLM-forfining	LLM-förfining	LLM-viilaus	LLM-fínstilling	LLM-lihvimine	LLM patikslinimas	LLM precizēšana
speechChat	LLM-chat	LLM-chat	LLM-chatt	LLM-keskustelu	LLM-spjall	LLM-vestlus	LLM pokalbis	LLM tērzēšana
refineRejected	Forfinelsen beholdt NLU-svaret.	Forfiningen beholdt NLU-svaret.	Förfiningen behöll NLU-svaret.	Viilaus piti NLU-vastauksen.	Fínstillingin hélt NLU-svarinu.	Lihvimine jättis NLU vastuse.	Patikslinimas paliko NLU atsakymą.	Precizēšana saglabāja NLU atbildi.
"""

PACKS = parse_rows(CODES, TABLE)
