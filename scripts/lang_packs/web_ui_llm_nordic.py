"""Dashboard LLM chrome for Nordic and Baltic Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_rows

CODES = ["da", "nb", "sv", "fi", "is", "et", "lt", "lv"]

TABLE = """
llmCalls	LLM-kald	LLM-kall	LLM-anrop	LLM-kutsut	LLM-köll	LLM-kutsed	LLM kvietimai	LLM izsaukumi
llmCallsCaption	Kilde: motor-LLM, seneste 24 timer	Kilde: motor-LLM, siste 24 timer	Källa: motorns LLM, senaste 24 timmarna	Lähde: moottorin LLM, viimeiset 24 tuntia	Uppruni: vélar-LLM, síðustu 24 klukkustundir	Allikas: mootori LLM, viimased 24 tundi	Šaltinis: variklio LLM, paskutinės 24 valandos	Avots: dzinēja LLM, pēdējās 24 stundas
llmKindRefine	forfinelse	forfining	förfining	viimeistely	fínslípun	viimistlus	tobulinimas	pilnveidošana
llmKindAssist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist
llmKindChat	samtale	samtale	chatt	keskustelu	spjall	vestlus	pokalbis	tērzēšana
llmErrors	fejl	feil	fel	virheet	villur	vead	klaidos	kļūdas
llmAcceptRate	acceptprocent	akseptgrad	acceptansgrad	hyväksyntäaste	samþykktarhlutfall	nõustumismäär	priėmimo dalis	pieņemšanas rādītājs
llmLatency	LLM-latens	LLM-latens	LLM-latens	LLM-viive	LLM-töf	LLM-latentsus	LLM delsna	LLM latentums
llmLatencyCaption	Kilde: motor-LLM, rullende vindue	Kilde: motor-LLM, rullende vindu	Källa: motorns LLM, rullande fönster	Lähde: moottorin LLM, vierivä ikkuna	Uppruni: vélar-LLM, rúllandi gluggi	Allikas: mootori LLM, veerev aken	Šaltinis: variklio LLM, slenkantis langas	Avots: dzinēja LLM, slīdošs logs
llmTokens	tokens	tokens	token	tokeneita	tákn	tokenid	žetonai	žetoni
llmTokensCaption	Kilde: upstream-forbrug, når modellen sender det	Kilde: oppstrøms-bruk, når modellen sender det	Källa: uppströmsanvändning, när modellen skickar den	Lähde: ylävirran käyttö, kun malli lähettää sen	Uppruni: upstream-notkun, þegar módelið sendir hana	Allikas: ülesvoolu kasutus, kui mudel selle saadab	Šaltinis: aukštupio naudojimas, kai modelis jį siunčia	Avots: augšupstraumes dati, kad modelis tos sūta
llmNoCalls	Ingen LLM-kald endnu.	Ingen LLM-kall ennå.	Inga LLM-anrop ännu.	Ei vielä LLM-kutsuja.	Engin LLM-köll enn.	LLM-kutseid veel pole.	LLM kvietimų dar nėra.	Vēl nav LLM izsaukumu.
labThisTurn	Denne tur	Denne turen	Denna tur	Tämä vuoro	Þessi umferð	See käik	Šis ėjimas	Šis gājiens
"""

PACKS = parse_rows(CODES, TABLE)
