# ADR 0004 — Operator-UI ist die Produktkonsole; Home Assistant bleibt Kleber

[Deutsch](adr-0004-operator-console.md) · [English](adr-0004-operator-console.en.md)

Status: **akzeptierte Richtung; auf staging umgesetzt, kein Hauptrelease**. Stufe 1 (Engine-Settings-Store) und Stufe 4 (Wizard schreibt Stimme + LLM) sind auf staging. Stufen 2–3 (Figma 05) bleiben Zukunft. Umsetzung: [Plan](adr-0004-plan.md).

Klar bleibt eine deterministische, lokale NLU. `nlu::parse` hat kein Netz und kein Modell. Dieser ADR ersetzt nicht [ADR 0001](adr-0001-rules-and-trainer.md) (sichtbare Match-/Sprach-/Haus-Spuren), [ADR 0002](adr-0002-openai-llm-client.md) (Engine-LLM-Client) oder [ADR 0003](adr-0003-python-rust-boundary.md) (Engine besitzt Assist-Produktlogik). Er zieht die **Operator-Linie** durch: wer Klar betreibt, konfiguriert Klar **in Klar**, nicht in einem langen Home-Assistant-Optionsformular.

## Kontext

Nach ADR 0001–0003 speichert die Engine Persönlichkeit, Sprachen, Refine, Quiet-Ack, RAG, Kalender-LLM und Werkzeug-Flags schon in `GET`/`POST /api/settings`, den LLM-Endpoint in `/api/v2/llm/endpoint`. Die Operator-UI hat Settings und einen Setup-Wizard.

Home Assistant **besitzt** diese Produktknöpfe trotzdem noch in den `config_flow`-Optionen, **drückt** sie bei jedem Reload auf die Engine (und überschreibt die Operator-UI) und **schickt** sie bei jedem Parse noch einmal mit. Die Settings-Seite schickt Betreiber nach **Home Assistant → Klar NLU**. Das ist die falsche Konsole.

Gewünscht:

- **Weniger** Felder in der Home-Assistant-Integration.
- **Mehr**, und **geführte**, Konfiguration in der Operator-UI.
- Python/HA bleibt Plattformkleber (URL, Token, Expose, Legacy-Conversation-Agent, Chime, Registry-Sync).

## Entscheidung

### Quelle der Wahrheit

| Thema | Besitzer | Home Assistant |
|-------|----------|----------------|
| Persönlichkeit, Assist-Packs, Refine an/aus, Extra-Zeile, Quiet-Ack, NLU-RAG, Kalender-LLM, LLM-Werkzeuge, Confirm-risky, Modus | Engine `/api/settings` über die Operator-UI | Cache lesen; nach einmaligem Seed nicht überschreiben |
| LLM-Endpoint (URL, Modell, Key) | Engine `/api/v2/llm/endpoint` über die Operator-UI | Kein HA-Optionsfeld |
| Match / Lexikon / Hausregeln, Trainer | Operator-UI + Overlay | Nein |
| Engine-URL, Token, lokal vs App, Release-Kanal | HA Config Entry | In HA (Anbindung) |
| Fallback-Conversation-Agent | HA-Optionen | HA-Entity-ID; Engine speichert nur `fallback_llm: bool` |
| Assist-Expose-Filter | HA-Optionen | Registry-Kleber |
| Personality-Select + Quiet-Ack-Switch | Dünne Proxys, die **die Engine schreiben** | Automationen dürfen schalten; das ist nicht das Setup |

### Home-Assistant-Optionsformular (behalten)

Setup (`user`) und Konfigurieren (`options`) behalten nur:

1. Engine (mitgeliefert vs App/Docker)
2. Release-Kanal
3. URL + Write-Token
4. Optionaler Legacy-Conversation-Agent
5. Assist-Expose-Filter

Der Beschreibungstext zeigt auf die Operator-UI für Stimme, Sprachen und LLM.

### Migration

Bestehende Häuser mit Persönlichkeit / Flags in HA-Optionen: **einmaliger Seed** auf die Engine, wenn die Optionen nicht Default sind, danach Flag `product_in_engine` am Config Entry. Danach darf HA keine Produktfelder über Operator-Änderungen POSTen.

Ist die Engine nicht erreichbar, nutzt Assist die übrig gebliebenen `entry.options`, bis der nächste Fetch klappt. Nach einem erfolgreichen Fetch gilt der Engine-Cache (pro Turn frisch).

### Operator-UI

Settings ist eine **geführte Konsole**, kein Abklatsch des HA-Formulars:

1. **Stimme** — Persönlichkeit, Extra-Prompt, Refine, Quiet-Ack
2. **Assist-Sprachen** — alle Packs oder eines pinnen (leeres `languages` = jede kompilierte Locale)
3. **LLM** — bestehender Endpoint-Card (Assist-Chat + Trainer)
4. **Wenn Klar danebenliegt** — NLU-RAG, Kalender-LLM, Werkzeuge, Confirm-risky, Geräte vs nur Räume
5. **Diese Oberfläche** — Theme, Operator-Sprache, Write-Token
6. **Diagnose** — Support-Bundle, Adapter

Der Wizard bleibt der Erstpfad und zeigt hierher für Stimme und LLM. Lovelace „Klar“ bleibt der letzte Assist-Zug, nicht die Produktkonsole.

Operator-Chrome hat dieselben Keys für jede kompilierte Assist-Locale (`en.ts` / `de.ts` von Hand; `messages/*.json` generiert). Neue Settings-Texte werden übersetzt, nicht als englische Reste stehen gelassen.

### Visual Overhaul

Ein späterer Staging-PR zieht den Rest des Operator-Chroms (Home, Rules-Pfad, Labor, Haus) auf Figma **05 Staging Overhaul** im Klar-Visual-Refresh-File. Dieser ADR verlangt nicht, dass der Restyle im selben PR wie der Settings-Umzug landet. Code Connect bleibt raus aus dem Repo.

## Konsequenzen

- Betreiber konfigurieren Klar dort, wo sie das Haus kalibrieren und Spuren trainieren.
- HA Konfigurieren ist kurz genug fürs Telefon.
- Ein Reload der Integration setzt die Operator-Persönlichkeit nicht mehr auf den HA-Default zurück.
- Personality-Select / Quiet-Ack-Switch bleiben für Automationen; sie patchen `/api/settings`.
- Diese Knöpfe nicht wieder in Python-Produktmodule legen. `config_flow`-Optionen nicht wieder aufblähen. Kein LLM auf `nlu::parse`.
