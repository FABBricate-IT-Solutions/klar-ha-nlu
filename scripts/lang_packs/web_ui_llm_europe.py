"""Dashboard LLM chrome for Germanic and Celtic Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_rows

CODES = ["de-CH", "de-AT", "en-GB", "af", "lb", "cy", "eu", "ga", "kw"]

TABLE = """
llmCalls	LLM-Ufrüef	LLM-Aufrufe	LLM calls	LLM-roepe	LLM-Opriff	Galwadau LLM	LLM deiak	Glaonna LLM	Galowow LLM
llmCallsCaption	Quälle: Engine-LLM, letschti 24 Stunde	Quelle: Engine-LLM, letzte 24 Stunden	Source: engine LLM, last 24 hours	Bron: enjin-LLM, laaste 24 uur	Quell: Engine-LLM, lescht 24 Stonnen	Ffynhonnell: LLM yr injan, 24 awr ddiwethaf	Iturria: motorraren LLM, azken 24 orduak	Foinse: LLM an innill, an 24 uair anuas	Fenten: LLM an jynn, an 24 our dewetha
llmKindRefine	Verfiinerig	Verfeinerung	refine	verfyning	Verfeinerung	mireinio	fintzea	scagadh	finwheth
llmKindAssist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist
llmKindChat	Gspröch	Chat	chat	gesprek	Chat	sgwrs	berbal	comhrá	keskows
llmErrors	Fähler	Fehler	errors	foute	Feeler	gwallau	erroreak	earráidí	gwallow
llmAcceptRate	Aanahmquot	Annahmequote	accept rate	aanvaardingskoers	Unhuelungsquot	cyfradd derbyn	onartze-tasa	ráta glactha	rát a wra
llmLatency	LLM-Latänz	LLM-Latenz	LLM latency	LLM-latency	LLM-Latenz	hwyrni LLM	LLM latentzia	moill LLM	lett LLM
llmLatencyCaption	Quälle: Engine-LLM, rollierend Fänschter	Quelle: Engine-LLM, rollierendes Fenster	Source: engine LLM, rolling window	Bron: enjin-LLM, rollende venster	Quell: Engine-LLM, rolléierend Fënster	Ffynhonnell: LLM yr injan, ffenestr dreiglol	Iturria: motorraren LLM, leiho irristakorra	Foinse: LLM an innill, fuinneog reatha	Fenten: LLM an jynn, fenester resek
llmTokens	Tokens	Tokens	tokens	tekens	Token	tocynnau	tokenak	comharthaí	tokens
llmTokensCaption	Quälle: Upstream-Usage, wenn s Modell si liefert	Quelle: Upstream-Usage, wenn das Modell sie liefert	Source: upstream usage, when the model reports it	Bron: stroomop-gebruik, as die model dit stuur	Quell: Upstream-Usage, wann d'Modell se liwwert	Ffynhonnell: defnydd i fyny'r afon, pan fydd y model yn ei anfon	Iturria: goranzko erabilera, ereduak bidaltzen duenean	Foinse: úsáid in aghaidh srutha, nuair a sheolann an tsamhail é	Fenten: usyans yn-unn, pan wra an model y dhannvon
llmNoCalls	No no LLM-Ufrüef.	Noch keine LLM-Aufrufe.	No LLM calls yet.	Nog geen LLM-roepe nie.	Nach keng LLM-Opriff.	Dim galwadau LLM eto.	Oraindik ez dago LLM deirik.	Níl aon ghlaonna LLM fós.	Nyns eus galowow LLM hwath.
labThisTurn	Deä Turn	Dieser Turn	This turn	Hierdie beurt	Dësen Tour	Y tro hwn	Txanda hau	An cas seo	An tro ma
"""

PACKS = parse_rows(CODES, TABLE)
