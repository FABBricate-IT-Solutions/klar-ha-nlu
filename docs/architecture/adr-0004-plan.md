# Umsetzungsplan — ADR 0004

[Deutsch](adr-0004-plan.md) · [English](adr-0004-plan.en.md)

Rahmen: [ADR 0004](adr-0004-operator-console.md). Jede Stufe ist ein eigener PR gegen **`staging`**. Defaults bleiben heutiges Assist-Verhalten, bis jemand Settings ändert. Kein Kalender.

## Auslieferung: Staging, kein Hauptrelease

Dieselben Kanalregeln wie ADR 0001 / 0003: jeder PR gegen `staging`; nach Merge taggt der Staging-Workflow ein Prerelease und Docker `staging`; `staging` → `main` gehört nicht zu diesem Plan.

## Stufe 1 — Engine ist der Produkt-Settings-Store (dieser PR)

**Ziel.** Home Assistant Konfigurieren schrumpft auf Anbindung. Operator-Settings wird der geführte Editor. Assist liest Engine-Settings.

| Änderung | Detail |
|----------|--------|
| `Settings.extra_prompt` | Optionale Extra-Zeile für Refine/Assist; leer = nur Pack-Persönlichkeit |
| HA-Options-Schema | Mode, Kanal, URL, Token, Fallback-Agent, Assist-Filter. Persönlichkeit, Sprachen, Refine, Quiet-Ack, RAG, Kalender-LLM, Werkzeuge raus |
| Seed | Einmaliges POST nicht-default HA-Produktoptionen auf `/api/settings`, dann `product_in_engine` |
| Assist | Cache `GET /api/settings` pro Turn; Fallback auf übrig gebliebene Options nur wenn der Cache leer ist |
| Select / Switch | Patchen die Engine, nicht `entry.options` (Options bleiben letzter Fallback) |
| Operator-Settings | Geführte Karten: Stimme, Assist-Sprachen, LLM, Misses, diese Oberfläche, Diagnose |
| Texte | HA-Strings + Operator-i18n schicken niemanden mehr ins Integrationsformular für Stimme/Sprache |

**Gate:** `python3 -m unittest discover -s tests -p 'test_*.py'`. `cargo nextest run --locked` für `extra_prompt`. Web-Typecheck.

**Rollback:** Produkt-Keys bleiben in `entry.options`; Assist fällt zurück, wenn GET fehlschlägt.

## Stufe 2 — Figma 05 Staging Overhaul

Neue Seite in [Klar Visual Refresh](https://www.figma.com/design/IOMwQ0Fkkg3YhFTfkRhGed), **04 Visual Refresh** nicht zerschlagen. Screens: geführte Settings, Home, Rules-Pfad, Labor-Pfad, Haus-Graph. IBM Plex Sans. shadcn-Mapping, kein Code Connect.

Braucht: Figma-Schreibzugriff; eine laufende Staging-UI hilft bei Screenshots.

## Stufe 3 — Overhaul in `web/`

Figma 05 mit bestehenden `web/src/components/ui/*`. Bessere Pfad-Visualisierung. Browser-Verify mit Klicks, nicht einem Screenshot. Operator-Chrome: dieselben Keys für jede kompilierte Assist-Locale, übersetzt — keine englischen Reste.

## Stufe 4 — Wizard schreibt Stimme + LLM beim Erststart

Wizard-Schritte schreiben `/api/settings` und `/api/v2/llm/endpoint`, damit ein neues Haus HA Konfigurieren nie für Produktknöpfe öffnet.

## Nicht in diesem Plan

- `staging` → `main`
- CalVer-Bump
- Produktflags zurück nach Python
- `conversation.py` als Rust-Plugin
- Personality-Select / Quiet-Ack-Switch entfernen (Automationen)
