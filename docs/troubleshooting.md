# Fehlerbehebung und Datenschutz

[Deutsch](troubleshooting.md) · [English](en/troubleshooting.md)

Zuerst der Haushaltsweg: [Einstieg](getting-started.md). Hier: Fehltreffer, Write-Token, was im Haus bleibt.

## Gerät nicht gefunden

1. **Freigabe.** Einstellungen → Sprachassistenten → Freigeben. Die Option **Nur für Assist freigegebene Entitäten steuern** ist standardmäßig an. Versteckte Sensoren und Schalter sind keine Ziele.
2. **Name und Raum.** Die Entität braucht einen sprechbaren Namen und einen Raum in Home Assistant. Ein generisches „Licht“ in einem Raum mit drei Lampen wird zur Rückfrage.
3. **Zuordnung.** Klar-UI → **Haus → Zuordnung**. Alias setzen oder Raumvorschlag übernehmen. Keine zweite Geräteliste in Klar bauen.
4. **Sprache.** Assist auf die gesprochene Locale pinnen (`de`, `en`, `fr`, …). Klar bindet dieses Pack für den Request.

Die Integrationsoption **Nur für Assist freigegebene Entitäten steuern** ist eine Entwickler-Ausnahme. Aus trifft auch versteckte Entitäten — leichter das falsche Gerät.

## Assist redet, nichts bewegt sich

- Conversation-Engine der Pipeline muss **Klar NLU** sein, nicht das Smalltalk-LLM.
- Engine und Integration dieselbe CalVer (V2: nur `POST /api/v2/parse`).
- Mitgelieferte Engine: warten, bis das GitHub-Release in `.storage/klar_nlu/` liegt.
- Add-on / Docker: Integrations-URL `http://klar-nlu:10520` (HAOS) oder `http://127.0.0.1:10520` (Host-Netz).
- Confirm / Clarify rufen keine Services. `ja` / `yes` in derselben Conversation, oder das Gerät nennen.

## Medien und Music Assistant

- Pause / weiter / stumm nutzen den genannten `media_player` oder den im Raum.
- `Spiel Queen` / `Play Queen` braucht einen Music-Assistant-Player (oder einen Player, auf dem Klar suchen kann). Klar erfindet keine Bibliothek.
- Nicht erreichbare Player werden übersprungen. Den gewünschten Player freigeben.

## Write-Token

Loopback darf lesen und schreiben. Das Supervisor-Netz darf lesen. Schreibzugriffe vom Supervisor oder aus dem LAN brauchen einen Token (`x-klar-token` oder `Authorization: Bearer`).

| Betrieb | Wo der Token liegt |
|---------|---------------------|
| Mitgelieferte Engine | Unter `.storage/klar_nlu/token`, die Integration schickt ihn mit |
| Add-on | Add-on-Option **token** → `KLAR_TOKEN`. Denselben Wert in der Integration unter **Write-Token** |
| Docker / Cargo | `--token`, `KLAR_TOKEN` oder `--token-file` |

Leerer Add-on-Token heißt kein gemeinsames Geheimnis: Overlay-Writes aus Home Assistant scheitern, außer sie kommen von Loopback.

## Support-Bundle

In der Klar-UI (oder Add-on-Option **support_bundle**): Parse-Verkehr unter `/data/support_bundle.jsonl` (max. 2000 Zeilen). `KLAR_SUPPORT_BUNDLE=1` setzt nur den ersten Start.

Downloads sind redigiert:

- Conversation-IDs werden gehasht
- Entity- und Area-Namen werden pseudonymisiert
- Rohtext und Sprachausgabe bleiben draußen, solange **support_bundle_raw_text** aus ist (Standard)

Das Conversation-Journal (UI **Gespräche**) hält die letzten 200 Turns 24 Stunden. Rohtext folgt derselben Flagge.

## Was das Haus nicht verlässt

Die Engine ist lokal. Keine Cloud, keine Modellgewichte, kein Phone-Home.

Ein optionales LLM in Home Assistant darf eine fertige Bestätigung umformulieren oder Smalltalk führen. Das ist euer Agent, nicht Klar. Assist-Werkzeuge bei diesem Agenten **aus**. NLU-RAG (standardmäßig aus) darf dem Fallback nur den bereits gematchten Ausschnitt schicken — nie Assist- oder Home-Assistant-Steuerwerkzeuge.

`KLAR_TOKEN`, `klar.token` und unredigierte Bundles nicht committen.
