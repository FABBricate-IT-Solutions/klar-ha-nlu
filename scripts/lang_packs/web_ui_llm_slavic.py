"""Dashboard LLM chrome for Slavic and Hungarian Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_rows

CODES = ["cs", "sk", "pl", "hu", "hr", "sl", "bg", "sr", "sr-Latn", "uk"]

TABLE = """
llmCalls	Volání LLM	Volania LLM	Wywołania LLM	LLM-hívások	LLM pozivi	Klici LLM	LLM повиквания	LLM позиви	LLM pozivi	Виклики LLM
llmCallsCaption	Zdroj: LLM enginu, posledních 24 hodin	Zdroj: LLM enginu, posledných 24 hodín	Źródło: LLM silnika, ostatnie 24 godziny	Forrás: a motor LLM-je, elmúlt 24 óra	Izvor: LLM motora, posljednja 24 sata	Vir: LLM pogona, zadnjih 24 ur	Източник: LLM на двигателя, последните 24 часа	Извор: LLM мотора, последња 24 сата	Izvor: LLM motora, poslednja 24 sata	Джерело: LLM рушія, останні 24 години
llmKindRefine	zpřesnění	spresnenie	wygładzenie	finomítás	dorada	izpilitev	уточнение	дорада	dorada	уточнення
llmKindAssist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist	Assist
llmKindChat	chat	chat	czat	csevegés	razgovor	klepet	чат	ћет	ćet	чат
llmErrors	chyby	chyby	błędy	hibák	greške	napake	грешки	грешке	greške	помилки
llmAcceptRate	míra přijetí	miera prijatia	wskaźnik akceptacji	elfogadási arány	stopa prihvaćanja	stopnja sprejema	дял приети	стопа прихватања	stopa prihvatanja	частка прийняття
llmLatency	Latence LLM	Latencia LLM	Opóźnienie LLM	LLM-késleltetés	Latencija LLM	Zakasnitev LLM	Латентност LLM	Латенција LLM	Latencija LLM	Затримка LLM
llmLatencyCaption	Zdroj: LLM enginu, posuvné okno	Zdroj: LLM enginu, posuvné okno	Źródło: LLM silnika, okno przesuwne	Forrás: a motor LLM-je, gördülő ablak	Izvor: LLM motora, klizni prozor	Vir: LLM pogona, drseče okno	Източник: LLM на двигателя, плъзгащ прозорец	Извор: LLM мотора, клизни прозор	Izvor: LLM motora, klizni prozor	Джерело: LLM рушія, ковзне вікно
llmTokens	tokeny	tokeny	tokeny	tokenek	tokeni	žetoni	токени	токени	tokeni	токени
llmTokensCaption	Zdroj: upstream použití, když ho model pošle	Zdroj: upstream použitie, keď ho model pošle	Źródło: użycie upstream, gdy model je wyśle	Forrás: upstream használat, ha a modell elküldi	Izvor: upstream potrošnja, kad je model pošalje	Vir: upstream poraba, ko jo model pošlje	Източник: upstream потребление, когато моделът го изпрати	Извор: upstream потрошња, кад је модел пошаље	Izvor: upstream potrošnja, kad je model pošalje	Джерело: upstream використання, коли модель його надсилає
llmNoCalls	Zatím žádná volání LLM.	Zatiaľ žiadne volania LLM.	Nie ma jeszcze wywołań LLM.	Még nincs LLM-hívás.	Još nema LLM poziva.	Še ni klicev LLM.	Все още няма LLM повиквания.	Још нема LLM позива.	Još nema LLM poziva.	Ще немає викликів LLM.
labThisTurn	Tento tah	Tento ťah	Ta tura	Ez a kör	Ovaj krug	Ta krog	Този ход	Овај круг	Ovaj krug	Цей хід
"""

PACKS = parse_rows(CODES, TABLE)
