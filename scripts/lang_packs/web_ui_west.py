"""Operator UI chrome for West European Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_table import parse_table

CODES = ["fr", "nl", "es", "it", "pt", "ca", "ro", "pt-BR", "gl"]

TABLE = """
home	Accueil	Start	Inicio	Inizio	Início	Inici	Acasă	Início	Inicio
conversations	Conversations	Gesprekken	Conversaciones	Conversazioni	Conversas	Converses	Conversații	Conversas	Conversas
rules	Règles	Regels	Reglas	Regole	Regras	Regles	Reguli	Regras	Regras
house	Maison	Huis	Casa	Casa	Casa	Casa	Casă	Casa	Casa
lab	Labo	Lab	Laboratorio	Laboratorio	Laboratório	Laboratori	Laborator	Laboratório	Laboratorio
graph	Graphe	Grafiek	Grafo	Grafo	Grafo	Graf	Graf	Grafo	Grafo
calibrate	Affectation	Toewijzing	Asignación	Associazione	Mapeamento	Assignació	Mapare	Mapeamento	Asignación
entities	Appareils	Apparaten	Dispositivos	Dispositivi	Dispositivos	Dispositius	Dispozitive	Dispositivos	Dispositivos
custom	Phrases	Zinnen	Frases	Frasi	Frases	Frases	Fraze	Frases	Frases
settings	Paramètres	Instellingen	Ajustes	Impostazioni	Definições	Configuració	Setări	Configurações	Configuración
open	ouvert	open	abierto	aperto	aberto	obert	deschis	aberto	aberto
bundleOn	Lot activé	Bundel aan	Paquete activo	Pacchetto attivo	Pacote ligado	Lot actiu	Pachet pornit	Pacote ligado	Paquete activo
bundleOff	Lot désactivé	Bundel uit	Paquete inactivo	Pacchetto spento	Pacote desligado	Lot inactiu	Pachet oprit	Pacote desligado	Paquete inactivo
engineReady	Moteur prêt	Motor gereed	Motor listo	Motore pronto	Motor pronto	Motor llest	Motor gata	Mecanismo pronto	Motor listo
understandsHome	Pourquoi Klar a exécuté, confirmé ou arrêté	Waarom Klar uitvoerde, bevestigde of stopte	Por qué Klar ejecutó, confirmó o detuvo	Perché Klar ha eseguito, confermato o interrotto	Porque é que o Klar executou, confirmou ou parou	Per què Klar ha executat, confirmat o aturat	De ce Klar a executat, confirmat sau oprit	Por que o Klar executou, confirmou ou parou	Por que Klar executou, confirmou ou detivo
assistVisible	Assist visible	Assist zichtbaar	Assist visible	Assist visibile	Assist visível	Assist visible	Assist vizibil	Assist visível	Assist visíbel
certain	certain	zeker	seguro	certo	certo	segur	sigur	certo	certo
needsWork	à retravailler	moet beter	por mejorar	da rivedere	por melhorar	per millorar	de revizuit	precisa de ajuste	por mellorar
recordings	Enregistrements	Opnamen	Grabaciones	Registrazioni	Gravações	Enregistraments	Înregistrări	Gravações	Gravacións
processed	traités	verwerkt	procesados	elaborati	processados	processats	procesate	processados	procesados
coverage	Couverture	Dekking	Cobertura	Copertura	Cobertura	Cobertura	Acoperire	Cobertura	Cobertura
confidence	Confiance	Zekerheid	Confianza	Affidabilità	Confiança	Confiança	Încredere	Confiança	Confianza
domains	Domaines	Domeinen	Dominios	Domini	Domínios	Dominis	Domenii	Domínios	Dominios
rooms	Pièces	Ruimtes	Estancias	Stanze	Divisões	Estances	Camere	Cômodos	Estancias
recent	Phrases récentes	Recente zinnen	Frases recientes	Frasi recenti	Frases recentes	Frases recents	Fraze recente	Frases recentes	Frases recentes
replay	Relire	Opnieuw	Repetir	Ripeti	Repetir	Torna	Reluare	Repetir	Repetir
applyAll	Appliquer les suggestions	Suggesties toepassen	Aplicar sugerencias	Applica i suggerimenti	Aplicar sugestões	Aplica els suggeriments	Aplică sugestiile	Aplicar sugestões	Aplicar suxestións
undo	Annuler	Ongedaan maken	Deshacer	Annulla	Anular	Desfés	Anulează	Desfazer	Desfacer
accept	Accepter	Accepteren	Aceptar	Accetta	Aceitar	Accepta	Acceptă	Aceitar	Aceptar
otherRoom	Autre pièce	Andere ruimte	Otra estancia	Altra stanza	Outra divisão	Altra estança	Altă cameră	Outro cômodo	Outra estancia
dismiss	Ignorer	Negeren	Descartar	Ignora	Ignorar	Descarta	Respinge	Dispensar	Descartar
noGaps	Aucune affectation ouverte.	Geen open toewijzingen.	No hay asignaciones abiertas.	Nessuna associazione aperta.	Sem mapeamentos em aberto.	No hi ha assignacions obertes.	Nicio mapare deschisă.	Nenhum mapeamento em aberto.	Sen asignacións abertas.
unmapped	Sans pièce	Geen ruimte	Sin estancia	Senza stanza	Sem divisão	Sense estança	Fără cameră	Sem cômodo	Sen estancia
parseHint	Les déclencheurs de phrases s'exécutent dans Home Assistant avant cette analyse. conversation.process : déclencheur, puis Klar, puis intent_script.	Zintriggers lopen in Home Assistant vóór deze analyse. conversation.process: trigger, dan Klar, dan intent_script.	Los disparadores de frases se ejecutan en Home Assistant antes de este análisis. conversation.process: disparador, luego Klar, luego intent_script.	Gli attivatori di frase vengono eseguiti in Home Assistant prima di questa analisi. conversation.process: attivatore, poi Klar, poi intent_script.	Os acionadores de frases correm no Home Assistant antes desta análise. conversation.process: acionador, depois Klar, depois intent_script.	Els activadors de frases s'executen a Home Assistant abans d'aquesta anàlisi. conversation.process: activador, després Klar, després intent_script.	Declanșatoarele de fraze rulează în Home Assistant înainte de această analiză. conversation.process: declanșator, apoi Klar, apoi intent_script.	Os acionadores de frases rodam no Home Assistant antes desta análise. conversation.process: acionador, depois Klar, depois intent_script.	Os disparadores de frases execútanse en Home Assistant antes desta análise. conversation.process: disparador, logo Klar, logo intent_script.
command	Commande	Commando	Comando	Comando	Comando	Ordre	Comandă	Comando	Comando
analyze	Analyser	Analyseren	Analizar	Analizza	Analisar	Analitza	Analizează	Analisar	Analizar
raw	Brut	Rauw	Crudo	Grezzo	Em bruto	Cru	Brut	Bruto	Cru
speech	Parole	Spraak	Voz	Voce	Fala	Veu	Vorbire	Fala	Fala
intent	Intention	Intentie	Intención	Intento	Intenção	Intenció	Intenție	Intenção	Intención
slots	Emplacements	Velden	Ranuras	Slot	Ranhuras	Ranures	Sloturi	Campos	Ranhuras
searchDevice	Comment l'appelleriez-vous ?	Hoe noem je het?	¿Cómo lo llamarías?	Come lo chiameresti?	Como lhe chamaria?	Com ho anomenaries?	Cum l-ați numi?	Como você chamaria isso?	Como o chamarías?
alias	Alias	Alias	Alias	Alias	Alias	Àlies	Alias	Alias	Alias
room	Pièce	Ruimte	Estancia	Stanza	Divisão	Estança	Cameră	Cômodo	Estancia
preferred	Lumière par défaut	Standaardlamp	Luz predeterminada	Luce predefinita	Luz predefinida	Llum per defecte	Lumina implicită	Luz padrão	Luz predeterminada
save	Enregistrer	Opslaan	Guardar	Salva	Guardar	Desa	Salvează	Salvar	Gardar
personality	Personnalité	Persoonlijkheid	Personalidad	Personalità	Personalidade	Personalitat	Personalitate	Personalidade	Personalidade
mode	Mode	Modus	Modo	Modalità	Modo	Mode	Mod	Modo	Modo
supportBundle	Lot de support	Ondersteuningsbundel	Paquete de soporte	Pacchetto di supporto	Pacote de suporte	Lot de suport	Pachet de suport	Pacote de suporte	Paquete de soporte
recordProtocol	Enregistrer le protocole	Protocol vastleggen	Registrar protocolo	Registra protocollo	Gravar protocolo	Enregistra el protocol	Înregistrează protocolul	Gravar protocolo	Rexistrar o protocolo
includeRawText	Inclure le texte brut dans les téléchargements	Ruwe tekst in gedownloade bestanden opnemen	Incluir texto en bruto en las descargas	Includi il testo grezzo nei file scaricati	Incluir texto em bruto nas transferências	Incloure el text cru a les baixades	Include textul brut în descărcări	Incluir texto bruto nos arquivos baixados	Incluír o texto cru nas descargas
semanticAdapters	Adaptateurs sémantiques locaux	Lokale semantische adapters	Adaptadores semánticos locales	Adattatori semantici locali	Adaptadores semânticos locais	Adaptadors semàntics locals	Adaptoare semantice locale	Adaptadores semânticos locais	Adaptadores semánticos locais
downloadDataset	Télécharger le jeu de données	Gegevensset downloaden	Descargar conjunto de datos	Scarica il set di dati	Transferir o conjunto de dados	Baixa el conjunt de dades	Descarcă setul de date	Baixar o conjunto de dados	Descargar o conxunto de datos
downloadProtocol	Télécharger le protocole	Protocol downloaden	Descargar protocolo	Scarica il protocollo	Transferir o protocolo	Baixa el protocol	Descarcă protocolul	Baixar o protocolo	Descargar o protocolo
deleteSelected	Supprimer la sélection	Selectie verwijderen	Eliminar selección	Elimina selezione	Eliminar seleção	Suprimeix la selecció	Șterge selecția	Excluir seleção	Eliminar a selección
clearAll	Tout supprimer	Alles verwijderen	Eliminar todo	Elimina tutto	Eliminar tudo	Suprimeix-ho tot	Șterge tot	Excluir tudo	Eliminar todo
token	Jeton d'écriture (LAN)	Schrijftoken (LAN)	Token de escritura (LAN)	Token di scrittura (LAN)	Token de escrita (LAN)	Testimoni d'escriptura (LAN)	Token de scriere (LAN)	Token de escrita (LAN)	Token de escritura (LAN)
customJson	Phrases personnalisées en JSON	Aangepaste zinnen als JSON	Frases personalizadas como JSON	Frasi personalizzate come JSON	Frases personalizadas em JSON	Frases personalitzades com a JSON	Fraze personalizate ca JSON	Frases personalizadas como JSON	Frases personalizadas como JSON
customHint	Associez une phrase à une intention connue. Les politiques sont à côté, pas dans les automatisations HA.	Koppel een zin aan een bekende intentie. Beleid staat ernaast, niet als HA-automatiseringen.	Asocia una frase a una intención conocida. Las políticas están al lado, no como automatizaciones HA.	Associa una frase a un intento noto. Le politiche stanno accanto, non come automazioni HA.	Associe uma frase a uma intenção conhecida. As políticas ficam ao lado, não como automações HA.	Associa una frase a una intenció coneguda. Les polítiques són al costat, no com a automatitzacions HA.	Leagă o frază de o intenție cunoscută. Politicile stau alături, nu ca automatizări HA.	Associe uma frase a uma intenção conhecida. As políticas ficam ao lado, não como automações HA.	Asocia unha frase a unha intención coñecida. As políticas están ao lado, non como automatizacións HA.
addPhrase	Ajouter une phrase	Zin toevoegen	Añadir frase	Aggiungi frase	Adicionar frase	Afegeix una frase	Adaugă frază	Adicionar frase	Engadir frase
previewRule	Aperçu	Voorbeeld	Vista previa	Anteprima	Pré-visualizar	Previsualitza	Previzualizare	Visualizar	Vista previa
explainRule	Expliquer	Uitleggen	Explicar	Spiega	Explicar	Explica	Explică	Explicar	Explicar
rollback	Revenir en arrière	Terugdraaien	Revertir	Ripristina	Reverter	Reverteix	Revino	Reverter	Reverter
noRules	Aucune phrase personnalisée pour l'instant.	Nog geen aangepaste zinnen.	Aún no hay frases personalizadas.	Nessuna frase personalizzata.	Ainda sem frases personalizadas.	Encara no hi ha frases personalitzades.	Nicio frază personalizată încă.	Ainda não há frases personalizadas.	Aínda non hai frases personalizadas.
emptyBundle	Aucun enregistrement. Activez le lot et essayez une phrase.	Nog geen opnamen. Zet de bundel aan en probeer een zin.	Aún no hay grabaciones. Activa el paquete y prueba una frase.	Nessuna registrazione. Attiva il pacchetto e prova una frase.	Ainda sem gravações. Ative o pacote e experimente uma frase.	Encara no hi ha enregistraments. Activeu el lot i proveu una frase.	Nicio înregistrare încă. Activați pachetul și încercați o frază.	Ainda não há gravações. Ative o pacote e tente uma frase.	Aínda non hai gravacións. Activa o paquete e proba unha frase.
confirmApply	Appliquer ces suggestions ?	Deze suggesties toepassen?	¿Aplicar estas sugerencias?	Applicare questi suggerimenti?	Aplicar estas sugestões?	Aplicar aquests suggeriments?	Aplicați aceste sugestii?	Aplicar estas sugestões?	Aplicar estas suxestións?
cancel	Annuler	Annuleren	Cancelar	Annulla	Cancelar	Cancel·la	Anulează	Cancelar	Cancelar
apply	Appliquer	Toepassen	Aplicar	Applica	Aplicar	Aplica	Aplică	Aplicar	Aplicar
close	Fermer	Sluiten	Cerrar	Chiudi	Fechar	Tanca	Închide	Fechar	Pechar
low	faible	laag	bajo	basso	baixo	baix	scăzut	baixo	baixo
medium	moyen	middel	medio	medio	médio	mitjà	mediu	médio	medio
high	élevé	hoog	alto	alto	alto	alt	ridicat	alto	alto
source	Source	Bron	Fuente	Origine	Fonte	Font	Sursă	Fonte	Fonte
language	Langue	Taal	Idioma	Lingua	Idioma	Llengua	Limbă	Idioma	Lingua
time	Heure	Tijd	Hora	Ora	Hora	Hora	Oră	Hora	Hora
text	Phrase	Zin	Frase	Frase	Frase	Frase	Frază	Frase	Frase
answer	Réponse	Antwoord	Respuesta	Risposta	Resposta	Resposta	Răspuns	Resposta	Resposta
graphHint	Pièces en grappes, appareils colorés selon la confiance.	Ruimtes als groepen, apparaten gekleurd op zekerheid.	Estancias en clústeres, dispositivos coloreados por confianza.	Stanze a gruppi, dispositivi colorati per affidabilità.	Divisões em agrupamentos, dispositivos coloridos pela confiança.	Estances en clústers, dispositius acolorits per confiança.	Camere ca grupuri, dispozitive colorate după încredere.	Cômodos em agrupamentos, dispositivos coloridos pela confiança.	Estancias en grupos, dispositivos coloreados por confianza.
resetLayout	Réinitialiser la disposition	Indeling herstellen	Restablecer disposición	Reimposta disposizione	Repor disposição	Restableix la disposició	Resetează aranjamentul	Redefinir disposição	Restablecer a disposición
score	Score	Score	Puntuación	Punteggio	Pontuação	Puntuació	Scor	Pontuação	Puntuación
noIntent	Aucune intention	Geen intentie	Sin intención	Nessun intento	Sem intenção	Sense intenció	Fără intenție	Sem intenção	Sen intención
loading	Chargement de Klar...	Klar wordt geladen...	Cargando Klar...	Caricamento di Klar...	A carregar o Klar...	S'està carregant Klar...	Se încarcă Klar...	Carregando o Klar...	Cargando Klar...
nluRagHint	Désactivé par défaut. Uniquement l'extrait correspondant, jamais les outils Assist.	Standaard uit. Alleen het herkende stuk, nooit Assist-hulpmiddelen.	Desactivado por defecto. Solo el fragmento coincidente, nunca las herramientas de Assist.	Disattivato per impostazione predefinita. Solo il frammento corrispondente, mai gli strumenti Assist.	Desligado por predefinição. Só o excerto correspondente, nunca as ferramentas Assist.	Desactivat per defecte. Només el fragment coincident, mai les eines d'Assist.	Dezactivat implicit. Doar fragmentul potrivit, niciodată uneltele Assist.	Desligado por padrão. Só o trecho correspondente, nunca as ferramentas do Assist.	Desactivado por defecto. Só o anaco coincidente, nunca as ferramentas de Assist.
confirmRisky	Confirmer les actions risquées	Riskante acties bevestigen	Confirmar acciones de riesgo	Conferma le azioni rischiose	Confirmar ações arriscadas	Confirmar accions de risc	Confirmă acțiunile riscante	Confirmar ações arriscadas	Confirmar accións de risco
languages	Langues	Talen	Idiomas	Lingue	Idiomas	Llengües	Limbi	Idiomas	Linguas
languageSearch	Rechercher des langues	Talen zoeken	Buscar idiomas	Cerca lingue	Procurar idiomas	Cerca llengües	Caută limbi	Pesquisar idiomas	Buscar linguas
allLanguages	Toutes les langues	Alle talen	Todos los idiomas	Tutte le lingue	Todos os idiomas	Totes les llengües	Toate limbile	Todos os idiomas	Todas as linguas
noLanguageMatch	Aucune langue trouvée	Geen taal gevonden	Ningún idioma encontrado	Nessuna lingua trovata	Nenhum idioma encontrado	No s'ha trobat cap llengua	Nicio limbă găsită	Nenhum idioma encontrado	Non se atopou ningunha lingua
languageHint	Recherchez et choisissez des locales. Toutes les langues laisse chaque paquet compilé activé.	Zoek en kies locales. Alle talen houdt elk gecompileerd pakket ingeschakeld.	Busca y elige locales. Todos los idiomas mantiene activo cada paquete compilado.	Cerca e scegli i locale. Tutte le lingue lascia attivi tutti i pacchetti compilati.	Pesquise e escolha locales. Todos os idiomas mantém cada pacote compilado ativo.	Cerqueu i trieu locales. Totes les llengües deixa actiu cada paquet compilat.	Căutați și alegeți localele. Toate limbile păstrează fiecare pachet compilat activ.	Pesquise e escolha locales. Todos os idiomas mantém cada pacote compilado ativo.	Busca e escolle locales. Todas as linguas mantén activo cada paquete compilado.
mappingHint	L'affectation, ce sont des alias pour les entités du graphe. Les agendas apparaissent une fois le domaine calendar inclus. Assist suit le paquet de langue ; cette interface suit la langue de l'opérateur.	Toewijzing zijn aliassen voor grafiekentiteiten. Agenda's verschijnen nadat het calendar-domein is opgenomen. Assist volgt het taalpakket; deze interface volgt de operator-taal.	La asignación son alias de las entidades del grafo. Los calendarios aparecen al incluir el dominio calendar. Assist sigue el paquete de idioma; esta interfaz sigue el idioma del operador.	La mappatura sono alias per le entità del grafo. I calendari compaiono dopo aver incluso il dominio calendar. Assist segue il pacchetto di lingua; questa interfaccia segue la lingua dell'operatore.	O mapeamento são alias das entidades do grafo. Os calendários aparecem depois de incluir o domínio calendar. O Assist segue o pacote de idioma; esta interface segue o idioma do operador.	L'assignació són àlies de les entitats del graf. Els calendaris apareixen quan s'inclou el domini calendar. Assist segueix el paquet de llengua; aquesta interfície segueix la llengua de l'operador.	Maparea înseamnă aliasuri pentru entitățile grafului. Calendarele apar după includerea domeniului calendar. Assist urmează pachetul de limbă; această interfață urmează limba operatorului.	O mapeamento são aliases das entidades do grafo. Os calendários aparecem depois de incluir o domínio calendar. O Assist segue o pacote de idioma; esta interface segue o idioma do operador.	A asignación son alias das entidades do grafo. Os calendarios aparecen despois de incluír o dominio calendar. Assist segue o paquete de lingua; esta interface segue a lingua do operador.
parseSample	Allume la lumière du salon	Doe het licht in de woonkamer aan	Enciende la luz del salón	Accendi la luce del soggiorno	Liga a luz da sala	Encén el llum del saló	Aprinde lumina din sufragerie	Liga a luz da sala	Acende a luz do salón
tryOn	Allume la lumière dans {room}	Doe het licht in {room} aan	Enciende la luz en {room}	Accendi la luce in {room}	Liga a luz em {room}	Encén el llum a {room}	Aprinde lumina în {room}	Liga a luz em {room}	Acende a luz en {room}
tryLock	La porte est-elle verrouillée ?	Is de deur op slot?	¿Está la puerta cerrada con llave?	La porta è chiusa a chiave?	A porta está trancada?	La porta està tancada amb clau?	Ușa este încuiată?	A porta está trancada?	A porta está pechada?
tryTime	Quelle heure est-il ?	Hoe laat is het?	¿Qué hora es?	Che ore sono?	Que horas são?	Quina hora és?	Cât e ceasul?	Que horas são?	Que hora é?
tryNight	Bonne nuit	Welterusten	Buenas noches	Buonanotte	Boa noite	Bona nit	Noapte bună	Boa noite	Boas noites
tryUndo	Annule ça	Maak dat ongedaan	Deshaz eso	Annulla quello	Anula isso	Desfés això	Anulează asta	Desfaz isso	Desfai iso
tryRoom	la cuisine	de keuken	la cocina	la cucina	a cozinha	la cuina	bucătăria	a cozinha	a cociña
nluIgnore	Ne pas lier pour l'état ou la commande	Niet koppelen voor status of schakelen	No vincular para estado o encendido	Non associare per stato o alimentazione	Não associar para estado ou energia	No vincular per estat o alimentació	Nu lega pentru stare sau alimentare	Não vincular para estado ou energia	Non vincular para estado ou acendido
nluIgnoreHint	Retire cet appareil du résolveur. Utile pour les helpers mal nommés.	Haalt dit apparaat uit de resolver. Gebruik bij verkeerd genoemde helpers.	Quita este dispositivo del resolvedor. Úsalo para helpers mal nombrados.	Esclude questo dispositivo dal resolver. Per gli helper con nome sbagliato.	Remove este dispositivo do resolvedor. Use para helpers mal nomeados.	Treu aquest dispositiu del resolvedor. Per a helpers mal anomenats.	Scoate acest dispozitiv din rezolvator. Folosiți pentru helperi denumiți greșit.	Remove este dispositivo do resolvedor. Use para helpers com nome errado.	Quita este dispositivo do resolvedor. Úsao para helpers mal nomeados.
savePhrase	Enregistrer comme phrase	Opslaan als zin	Guardar como frase	Salva come frase	Guardar como frase	Desa com a frase	Salvează ca frază	Salvar como frase	Gardar como frase
ignoreTarget	Ignorer cette cible	Dit doel negeren	Ignorar este destino	Ignora questo obiettivo	Ignorar este alvo	Ignora aquest objectiu	Ignoră această țintă	Ignorar este alvo	Ignorar este destino
teachSaved	Enregistré.	Opgeslagen.	Guardado.	Salvato.	Guardado.	Desat.	Salvat.	Salvo.	Gardado.
journal	Journal des conversations	Gespreksjournaal	Diario de conversaciones	Diario delle conversazioni	Diário de conversas	Diari de converses	Jurnal de conversații	Diário de conversas	Diario de conversas
journalHint	200 derniers tours, 24 heures, expurgé. Texte brut uniquement avec le lot.	Laatste 200 beurten, 24 uur, geredigeerd. Ruwe tekst alleen met de bundel.	Últimos 200 turnos, 24 horas, redactado. Texto en bruto solo con el paquete.	Ultimi 200 turni, 24 ore, oscurati. Testo grezzo solo con il pacchetto.	Últimos 200 turnos, 24 horas, redigido. Texto em bruto só com o pacote.	Darrers 200 torns, 24 hores, redactat. Text cru només amb el lot.	Ultimele 200 de ture, 24 de ore, redactate. Text brut doar cu pachetul.	Últimos 200 turnos, 24 horas, redigido. Texto bruto só com o pacote.	Últimas 200 quendas, 24 horas, redactado. Texto cru só co paquete.
decisionMix	Décisions	Beslissingen	Decisiones	Decisioni	Decisões	Decisions	Decizii	Decisões	Decisións
mixCaption	Source : journal des conversations, tours par jour	Bron: gespreksjournaal, beurten per dag	Fuente: diario de conversaciones, turnos por día	Origine: diario delle conversazioni, turni al giorno	Fonte: diário de conversas, turnos por dia	Font: diari de converses, torns per dia	Sursă: jurnal de conversații, ture pe zi	Fonte: diário de conversas, turnos por dia	Fonte: diario de conversas, quendas por día
coverageCaption	Source : graphe du foyer, part des appareils	Bron: huisgrafiek, aandeel apparaten	Fuente: grafo del hogar, proporción de dispositivos	Origine: grafo della casa, quota di dispositivi	Fonte: grafo da casa, quota de dispositivos	Font: graf de la llar, quota de dispositius	Sursă: graful casei, ponderea dispozitivelor	Fonte: grafo da casa, parcela de dispositivos	Fonte: grafo do fogar, proporción de dispositivos
latency	Temps d'étape	Fasetijd	Tiempo de etapa	Tempo di fase	Tempo de etapa	Temps d'etapa	Timp de etapă	Tempo de etapa	Tempo de etapa
latencyCaption	Source : trace d'analyse, microsecondes	Bron: analysespoor, microseconden	Fuente: traza de análisis, microsegundos	Origine: traccia di analisi, microsecondi	Fonte: rasto de análise, microssegundos	Font: traça d'anàlisi, microsegons	Sursă: urmă de analiză, microsecunde	Fonte: traço de análise, microssegundos	Fonte: traza de análise, microsegundos
unitsTurns	tours	beurten	turnos	turni	turnos	torns	ture	turnos	quendas
timeline	Chronologie	Tijdlijn	Línea de tiempo	Cronologia	Cronologia	Cronologia	Cronologie	Linha do tempo	Liña de tempo
noConversations	Aucune entrée de journal.	Nog geen journaalregels.	Aún no hay entradas en el diario.	Nessuna voce nel diario.	Ainda sem entradas no diário.	Encara no hi ha entrades al diari.	Nicio înregistrare în jurnal.	Ainda não há entradas no diário.	Aínda non hai entradas no diario.
when	Quand	Wanneer	Cuando	Quando	Quando	Quan	Când	Quando	Cando
then	Alors	Dan	Entonces	Allora	Então	Aleshores	Atunci	Então	Entón
priority	Ordre (la première règle utilisateur correspondante l'emporte)	Volgorde (eerste passende gebruikersregel wint)	Orden (gana la primera regla de usuario coincidente)	Ordine (vince la prima regola utente corrispondente)	Ordem (vence a primeira regra de utilizador correspondente)	Ordre (guanya la primeira regla d'usuari coincident)	Ordine (prima regulă de utilizator potrivită câștigă)	Ordem (vence a primeira regra de usuário correspondente)	Orde (gaña a primeira regra de usuario coincidente)
evaluator	Évaluateur de politiques	Beleidsevaluator	Evaluador de políticas	Valutatore di politiche	Avaliador de políticas	Avaluador de polítiques	Evaluator de politici	Avaliador de políticas	Avaliador de políticas
bakeSpeech	Générer des variantes	Varianten genereren	Generar variantes	Genera varianti	Gerar variantes	Genera variants	Generează variante	Gerar variantes	Xerar variantes
addRule	Règle	Regel	Regla	Regola	Regra	Regla	Regulă	Regra	Regra
noPolicies	Aucune règle de politique.	Nog geen beleidsregels.	Aún no hay reglas de política.	Nessuna regola di politica.	Ainda sem regras de política.	Encara no hi ha regles de política.	Nicio regulă de politică încă.	Ainda não há regras de política.	Aínda non hai regras de política.
compiledRisk	Risque compilé	Gecompileerd risico	Riesgo compilado	Rischio compilato	Risco compilado	Risc compilat	Risc compilat	Risco compilado	Risco compilado
finalBand	Bande	Band	Banda	Banda	Banda	Banda	Bandă	Banda	Banda
triggerFirst	Les déclencheurs de phrases HA d'abord, puis Klar, puis une intention enregistrée.	Eerst HA-zintriggers, dan Klar, dan een geregistreerde intentie.	Primero los disparadores de frases HA, luego Klar, luego una intención registrada.	Prima gli attivatori di frase HA, poi Klar, poi un intento registrato.	Primeiro os acionadores de frases HA, depois o Klar, depois uma intenção registada.	Primer els activadors de frases HA, després Klar, després una intenció registrada.	Mai întâi declanșatoarele de fraze HA, apoi Klar, apoi o intenție înregistrată.	Primeiro os acionadores de frases HA, depois o Klar, depois uma intenção registrada.	Primeiro os disparadores de frases HA, logo Klar, logo unha intención rexistrada.
discarded	Écarté	Verworpen	Descartado	Scartato	Descartado	Descartat	Renunțat	Descartado	Descartado
stageTokens	Jetons	Tokens	Tokens	Token	Tokens	Tokens	Tokenuri	Tokens	Tokens
stageBind	Liaison	Koppelen	Vincular	Associa	Associar	Vincle	Legare	Vincular	Vincular
stageRank	Rang	Rang	Rango	Rango	Classificação	Rang	Clasare	Classificação	Rango
stagePolicy	Politique	Beleid	Política	Politica	Política	Política	Politică	Política	Política
stageBand	Bande	Band	Banda	Banda	Banda	Banda	Bandă	Banda	Banda
effectConfirm	Confirmer	Bevestigen	Confirmar	Conferma	Confirmar	Confirma	Confirmă	Confirmar	Confirmar
effectBlock	Bloquer	Blokkeren	Bloquear	Blocca	Bloquear	Bloca	Blochează	Bloquear	Bloquear
effectAllow	Autoriser	Toestaan	Permitir	Consenti	Permitir	Permet	Permite	Permitir	Permitir
effectPreferEntity	Préférer l'entité	Entiteit prefereren	Preferir entidad	Preferisci entità	Preferir entidade	Prefereix l'entitat	Preferă entitatea	Preferir entidade	Preferir entidade
effectPreferArea	Préférer la pièce	Ruimte prefereren	Preferir estancia	Preferisci area	Preferir divisão	Prefereix l'estança	Preferă camera	Preferir cômodo	Preferir estancia
effectReply	Répondre sans intention	Antwoorden zonder intentie	Responder sin intención	Rispondi senza intento	Responder sem intenção	Respon sense intenció	Răspunde fără intenție	Responder sem intenção	Responder sen intención
effectScript	Script	Script	Script	Script	Script	Script	Script	Script	Script
effectTemplate	Modèle	Sjabloon	Plantilla	Modello	Modelo	Plantilla	Șablon	Modelo	Modelo
effectLlm	Invite LLM	LLM-prompt	Indicación LLM	Prompt LLM	Prompt LLM	Indicació LLM	Prompt LLM	Prompt LLM	Indicación LLM
payloadReply	Texte de réponse	Antwoordtekst	Texto de respuesta	Testo di risposta	Texto de resposta	Text de resposta	Text de răspuns	Texto de resposta	Texto de resposta
payloadScript	Script (script.good_night ou good_night)	Script (script.good_night of good_night)	Script (script.good_night o good_night)	Script (script.good_night o good_night)	Script (script.good_night ou good_night)	Script (script.good_night o good_night)	Script (script.good_night sau good_night)	Script (script.good_night ou good_night)	Script (script.good_night ou good_night)
payloadTemplate	Modèle Home Assistant ; {{ text }} est l'énoncé	Home Assistant-sjabloon; {{ text }} is de uiting	Plantilla de Home Assistant; {{ text }} es el enunciado	Modello Home Assistant; {{ text }} è l'enunciato	Modelo Home Assistant; {{ text }} é o enunciado	Plantilla de Home Assistant; {{ text }} és l'enunciat	Șablon Home Assistant; {{ text }} este enunțul	Modelo Home Assistant; {{ text }} é o enunciado	Modelo de Home Assistant; {{ text }} é o enunciado
payloadLlm	Invite système pour l'agent de secours	Systeemprompt voor de reserve-agent	Indicación de sistema para el agente de reserva	Prompt di sistema per l'agente di riserva	Prompt de sistema para o agente de recurso	Indicació de sistema per a l'agent de reserva	Prompt de sistem pentru agentul de rezervă	Prompt de sistema para o agente reserva	Indicación de sistema para o axente de reserva
whenPhrase	Phrase	Zin	Frase	Frase	Frase	Frase	Frază	Frase	Frase
chatMode	Discussion	Gesprek	Conversación	Conversazione	Conversação	Xat	Conversație	Conversa	Conversa
variantPreview	Variante vocale	Spraakvariant	Variante de voz	Variante vocale	Variante de fala	Variant de veu	Variantă de vorbire	Variante de fala	Variante de fala
policies	Politiques	Beleid	Políticas	Politiche	Políticas	Polítiques	Politici	Políticas	Políticas
routines	Routines	Routines	Rutinas	Routine	Rotinas	Rutines	Rutine	Rotinas	Rutinas
routineHint	Un nom prononcé lance un script Home Assistant. Bonne nuit l'emporte sur la salutation.	Een gesproken naam start een Home Assistant-script. Welterusten wint van de begroeting.	Un nombre hablado inicia un script de Home Assistant. Buenas noches gana sobre el saludo.	Un nome pronunciato avvia uno script di Home Assistant. Buonanotte vince sul saluto.	Um nome falado inicia um script do Home Assistant. Boa noite ganha à saudação.	Un nom dit en veu inicia un script de Home Assistant. Bona nit s'imposa a la salutació.	Un nume rostit pornește un script Home Assistant. Noapte bună învinge salutul.	Um nome falado inicia um script do Home Assistant. Boa noite ganha da saudação.	Un nome falado inicia un script de Home Assistant. Boas noites gana ao saúdo.
routinePhraseHint	Bonne nuit	Welterusten	Buenas noches	Buonanotte	Boa noite	Bona nit	Noapte bună	Boa noite	Boas noites
addRoutine	Ajouter une routine	Routine toevoegen	Añadir rutina	Aggiungi routine	Adicionar rotina	Afegeix una rutina	Adaugă rutină	Adicionar rotina	Engadir rutina
noRoutines	Aucune routine pour l'instant.	Nog geen routines.	Aún no hay rutinas.	Nessuna routine.	Ainda sem rotinas.	Encara no hi ha rutines.	Nicio rutină încă.	Ainda não há rotinas.	Aínda non hai rutinas.
routineInvalid	Une phrase et script.xxx sont requis.	Een zin en script.xxx zijn verplicht.	Se requieren una frase y script.xxx.	Servono una frase e script.xxx.	São necessários uma frase e script.xxx.	Calen una frase i script.xxx.	Sunt necesare o frază și script.xxx.	São necessários uma frase e script.xxx.	Requírense unha frase e script.xxx.
lastTurn	Dernier tour	Laatste beurt	Último turno	Ultimo turno	Último turno	Darrer torn	Ultima tură	Último turno	Última quenda
heardIn	Entendu dans	Gehoord in	Oído en	Ascoltato in	Ouvido em	Escoltat a	Auzit în	Ouvido em	Oído en
tryThese	Cinq phrases dans vos pièces	Vijf zinnen in je ruimtes	Cinco frases en tus estancias	Cinque frasi nelle tue stanze	Cinco frases nas suas divisões	Cinc frases a les vostres estances	Cinci fraze în camerele tale	Cinco frases nos seus cômodos	Cinco frases nas túas estancias
tryTheseHint	Touchez une phrase pour l'essayer dans le laboratoire.	Tik op een zin om die in het lab te proberen.	Toca una frase para probarla en el laboratorio.	Tocca una frase per provarla in laboratorio.	Toque numa frase para a experimentar no laboratório.	Toqueu una frase per provar-la al laboratori.	Atingeți o frază ca s-o încercați în laborator.	Toque numa frase para experimentá-la no laboratório.	Toca unha frase para probeala no laboratorio.
anyRoom	Aucun satellite	Geen satelliet	Sin satélite	Nessun satellite	Sem satélite	Sense satèl·lit	Fără satelit	Sem satélite	Sen satélite
personalityHa	Réglez la personnalité dans Home Assistant → Klar NLU → Personnalité.	Stel de persoonlijkheid in via Home Assistant → Klar NLU → Persoonlijkheid.	Configura la personalidad en Home Assistant → Klar NLU → Personalidad.	Imposta la personalità in Home Assistant → Klar NLU → Personalità.	Defina a personalidade em Home Assistant → Klar NLU → Personalidade.	Definiu la personalitat a Home Assistant → Klar NLU → Personalitat.	Setați personalitatea în Home Assistant → Klar NLU → Personalitate.	Defina a personalidade em Home Assistant → Klar NLU → Personalidade.	Define a personalidade en Home Assistant → Klar NLU → Personalidade.
"""

PACKS = parse_table(CODES, TABLE)
