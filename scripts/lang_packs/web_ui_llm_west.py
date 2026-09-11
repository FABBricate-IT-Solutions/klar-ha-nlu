"""Dashboard LLM chrome for West European Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_rows

CODES = ["fr", "nl", "es", "it", "pt", "ca", "ro", "pt-BR", "gl"]

TABLE = """
llmCalls	Appels LLM	LLM-aanroepen	Llamadas LLM	Chiamate LLM	Chamadas LLM	Crides LLM	Apeluri LLM	Chamadas LLM	Chamadas LLM
llmCallsCaption	Source : LLM du moteur, dernières 24 heures	Bron: engine-LLM, laatste 24 uur	Fuente: LLM del motor, últimas 24 horas	Fonte: LLM del motore, ultime 24 ore	Fonte: LLM do motor, últimas 24 horas	Font: LLM del motor, darreres 24 hores	Sursă: LLM-ul motorului, ultimele 24 de ore	Fonte: LLM do mecanismo, últimas 24 horas	Fonte: LLM do motor, últimas 24 horas
llmKindRefine	raffinage	verfijning	refino	raffinamento	refino	refinament	rafinare	refino	refino
llmKindAssist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist
llmKindChat	discussion	gesprek	charla	conversazione	conversa	xat	discuție	conversa	conversa
llmErrors	erreurs	fouten	errores	errori	erros	errors	erori	erros	erros
llmAcceptRate	taux d'acceptation	acceptatiegraad	tasa de aceptación	tasso di accettazione	taxa de aceitação	taxa d'acceptació	rată de acceptare	taxa de aceitação	taxa de aceptación
llmLatency	Latence LLM	LLM-latentie	Latencia LLM	Latenza LLM	Latência LLM	Latència LLM	Latență LLM	Latência LLM	Latencia LLM
llmLatencyCaption	Source : LLM du moteur, fenêtre glissante	Bron: engine-LLM, rollend venster	Fuente: LLM del motor, ventana deslizante	Fonte: LLM del motore, finestra mobile	Fonte: LLM do motor, janela deslizante	Font: LLM del motor, finestra mòbil	Sursă: LLM-ul motorului, fereastră glisantă	Fonte: LLM do mecanismo, janela deslizante	Fonte: LLM do motor, xanela desprazante
llmTokens	jetons	tokens	tokens	token	tokens	fitxes	tokeni	tokens	tokens
llmTokensCaption	Source : usage amont, lorsque le modèle le fournit	Bron: upstream-gebruik, als het model het stuurt	Fuente: uso del proveedor, cuando el modelo lo envía	Fonte: usage a monte, quando il modello lo invia	Fonte: uso a montante, quando o modelo o envia	Font: ús amunt, quan el model l'envia	Sursă: utilizare upstream, când modelul o trimite	Fonte: uso upstream, quando o modelo envia	Fonte: uso upstream, cando o modelo o envía
llmNoCalls	Aucun appel LLM pour l'instant.	Nog geen LLM-aanroepen.	Aún no hay llamadas LLM.	Nessuna chiamata LLM per ora.	Ainda sem chamadas LLM.	Encara no hi ha crides LLM.	Niciun apel LLM deocamdată.	Ainda não há chamadas LLM.	Aínda non hai chamadas LLM.
labThisTurn	Ce tour	Deze beurt	Este turno	Questo turno	Este turno	Aquest torn	Acest tur	Este turno	Este turno
"""

PACKS = parse_rows(CODES, TABLE)
