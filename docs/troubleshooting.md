# Fehlerbehebung und Datenschutz

[Deutsch](troubleshooting.md) · [English](en/troubleshooting.md)

Zuerst der Haushaltsweg: [Einstieg](getting-started.md). Hier: Fehltreffer, Write-Token, was im Haus bleibt.

## App zweimal, Seitenleiste, Einrichten kommt wieder

**Klar NLU** einmal installieren. **Klar NLU (Staging)** ist eine zweite App. Ein Neustart startet die App (`boot: auto`); er installiert keine Kopie.

| Was sichtbar ist | Was es ist |
|------------------|------------|
| Einstellungen → Apps → **Klar NLU**, plus Seitenleiste nach **In der Seitenleiste anzeigen** | Die App. Integration auf `http://klar-nlu:10520` stellen. |
| Seitenleiste **Klar engine**, nicht unter Apps | Die mitgelieferte Engine in Home Assistant. Keine zweite App. |
| Seitenleiste **Klar** | Lovelace-Karte für den letzten Assist-Zug. Nicht die Operator-UI. |
| Einrichten bei jedem Öffnen der App | Die App konnte nicht speichern. Ein Write-Token in der App setzen, App neu starten, denselben Wert unter Operator-**Einstellungen** und in der Integration unter **Write-Token** einfügen. |

Fragt die Integration immer wieder nach dem Einrichten, hängt sie an der mitgelieferten Engine und der Download ist nicht fertig, oder sie zeigt auf eine entfernte App. Konfigurieren → **Klar-NLU-App oder Docker verwenden**.

## Sprachsatellit (3rd Reality und andere)

Freigeben listet Klar nicht. Geräte für **Assist** freigeben. Diese Pipeline bearbeiten: Conversation-Agent **Klar NLU**. Sprache-zu-Text und Text-zu-Sprache zeigen Home Assistant Cloud, bis eine lokale Sprach-App installiert ist. Klar liefert sie nicht.

Wakeword am 3rd-Reality-Voice-&-Music-Assistant: **Okay Nabu** (oder Hey Jarvis / Hey Mycroft auf der ESPHome-Geräteseite). Klar hat kein Wakeword.

Die Glocke am Klar-Gerät (**Kurze Bestätigung**) bleibt aus, außer ein Ton statt Sprache bei einfachem An/Aus gewünscht ist. Die Entität mit dem Stern ist der Conversation-Agent; aktivieren, wenn dort Deaktiviert steht.

## Gerät nicht gefunden

1. **Freigabe.** Einstellungen → Sprachassistenten → Freigeben. Die Option **Nur für Assist freigegebene Entitäten steuern** ist standardmäßig an. Versteckte Sensoren und Schalter sind keine Ziele.
2. **Name und Raum.** Die Entität braucht einen sprechbaren Namen und einen Raum in Home Assistant. Ein generisches „Licht“ in einem Raum mit drei Lampen wird zur Rückfrage.
3. **Zuordnung.** App-Seitenleiste **Klar NLU** → **Haus → Zuordnung** (nicht Lovelace **Klar**). Alias setzen oder Raumvorschlag übernehmen. Keine zweite Geräteliste in Klar bauen.
4. **Sprache.** Das Sprachassistenten-Dropdown listet jede kompilierte Locale. Pipeline-Sprache wählen (`de`, `en`, `fr`, …). Klar bindet dieses Pack, außer Sprachen ist ein einzelnes gepinntes Pack.

Die Integrationsoption **Nur für Assist freigegebene Entitäten steuern** ist eine Entwickler-Ausnahme. Aus trifft auch versteckte Entitäten — leichter das falsche Gerät.

## Assist redet, nichts bewegt sich

- Conversation-Engine der Pipeline muss **Klar NLU** sein, nicht das Smalltalk-LLM.
- Engine und Integration dieselbe CalVer (V2: nur `POST /api/v2/parse`).
- Mitgelieferte Engine: warten, bis das GitHub-Release in `/config/klar_nlu/` liegt.
- App / Docker: Integrations-URL `http://klar-nlu:10520` (HAOS) oder `http://127.0.0.1:10520` (Host-Netz). Löst `klar-nlu` im Supervisor nicht auf, versucht Klar automatisch `http://klar-nlu.local.hass.io:10520` (bzw. `{slug}.local.hass.io`). App und mitgelieferte Engine nicht gleichzeitig.
- Confirm / Clarify rufen keine Services. `ja` / `yes` in derselben Conversation, oder das Gerät nennen.

## Medien und Music Assistant

- Pause / weiter / stumm nutzen den genannten `media_player` oder den im Raum.
- `Spiel Queen` / `Play Queen` braucht einen Music-Assistant-Player (oder einen Player, auf dem Klar suchen kann). Klar erfindet keine Bibliothek.
- Nicht erreichbare Player werden übersprungen. Den gewünschten Player freigeben.
- Sagt Assist Fertig, aber nichts spielt: Klar hat einen schwachen oder leeren Music-Assistant-Treffer verworfen. Playlist/Titel klar nennen und den MASS-Player im Raum prüfen.

## Wetter, Klima, Kalender

| Sagen | Klar nutzt |
|-------|------------|
| Wie ist das Wetter? / Wird es regnen? | Freigegebenes `weather.*` |
| Wie warm ist es draußen? | Dieselbe Outdoor-Wetter-Entität |
| Wie warm ist es im Wohnzimmer? | `climate.*` / Raumsensoren |
| Was steht morgen im Kalender? | `KlarGetCalendarEvents` — nicht Wetter |

Lab ist Parse-only (`speech` bleibt leer bis Assist ausführt). Zeigt Lab `KlarGetCalendarEvents`, spricht Assist aber etwas anderes: Pipeline-Conversation-Engine noch **Klar NLU**? Kalender-LLM-Rewrite zum Testen aus.

°F vs °C: App → Settings → **Einheitensystem** (`imperial` / `metric`). Das wandelt gesprochene Temps um, nicht die HA-Speichereinheit.

## Eigene Sätze

Unter Rules gespeicherte Phrasen liegen im Engine-Data-Dir (`klar_nlu.json` → `custom`) und laden nach Restart. Ist `/api/custom` nach Reboot leer auf einem alten Build (&lt; 2026.9.5): Engine + Integration gemeinsam updaten. Die Operator-UI leert die Phrasenliste, wenn die Engine offline ist — ein alter Tab kann voll aussehen, obwohl der Store leer ist.

## Sprachen

Kompilierte Assist-Locales (Codes + Namen): [Sprachen](languages.md). Deutsch und Englisch sind die handgepflegten Packs; andere Locales sind generiert und smoke-getestet.

## Write-Token

Loopback darf lesen und schreiben. Das Supervisor-Netz darf lesen. Schreibzugriffe vom Supervisor oder aus dem LAN brauchen einen Token (`x-klar-token` oder `Authorization: Bearer`).

| Betrieb | Wo der Token liegt |
|---------|---------------------|
| Mitgelieferte Engine | Unter `/config/klar_nlu/token`, die Integration schickt ihn mit |
| App | App-Option **token** → `KLAR_TOKEN`. Denselben Wert in der Integration unter **Write-Token** |
| Docker / Cargo | `--token`, `KLAR_TOKEN` oder `--token-file` |

Leerer App-Token heißt kein gemeinsames Geheimnis: Overlay-Writes aus Home Assistant scheitern, außer sie kommen von Loopback.

## Support-Bundle

In der Klar-UI (oder Add-on-Option **support_bundle**): Parse-Verkehr unter `/data/support_bundle.jsonl` (max. 2000 Zeilen). `KLAR_SUPPORT_BUNDLE=1` setzt nur den ersten Start.

Downloads sind redigiert:

- Conversation-IDs werden gehasht
- Entity- und Area-Namen werden pseudonymisiert
- Rohtext und Sprachausgabe bleiben draußen, solange **support_bundle_raw_text** aus ist (Standard)

Das Conversation-Journal (UI **Gespräche**) hält die letzten 200 Turns 24 Stunden. Rohtext folgt derselben Flagge.

## Was das Haus nicht verlässt

Die Engine ist lokal. Keine Cloud, keine Modellgewichte, kein Phone-Home.

Das Klar-Engine-LLM darf eine fertige Bestätigung umformulieren oder Chat führen. Konfiguration in der Operator-UI (Einstellungen → LLM). **Assist-Werkzeuge beim Chat** ist standardmäßig aus; an: Core-Assist-Toolnamen (2026.9-Präfixe) nach dem Klar-Parse. NLU-RAG (standardmäßig aus) schickt nur den gematchten Ausschnitt und schließt HA-Tools in derselben Runde aus.

`KLAR_TOKEN`, `klar.token` und unredigierte Bundles nicht committen.
