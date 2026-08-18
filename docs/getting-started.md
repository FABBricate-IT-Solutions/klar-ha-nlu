# Einstieg

[Deutsch](getting-started.md) · [English](en/getting-started.md)

Haushaltsweg: HACS → Geräte freigeben → Assist-Pipeline → fünf Sätze → Zuordnung, wenn etwas fehlt.

Jede kompilierte Assist-Locale ist erstklassig. Deutsch und Englisch sind die üblichen Beispiele. Siehe [Sprachen](languages.md).

Nur V2: Engine und Home-Assistant-Integration dieselbe CalVer. HTTP-Parse ist `POST /api/v2/parse`.

## 1. Klar NLU installieren

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. HACS → Integrationen → ⋮ → Benutzerdefinierte Repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → **Integration**.
2. **Klar NLU** herunterladen und Home Assistant neu starten.
3. [Integration hinzufügen](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) und **Mitgelieferte Engine starten** behalten.

HACS startet den Rust-Prozess nicht. Die Integration lädt das passende GitHub-Release und startet es auf `127.0.0.1:10520`.

Läuft die Engine schon als [Add-on](../addon/DOCS.md) oder Docker? **Bereits laufende Engine verwenden** und die URL setzen (`http://klar-nlu:10520` unter HAOS).

## 2. Entitäten für Assist freigeben

Klar steuert standardmäßig nur, was Assist sehen darf. Einstellungen → Sprachassistenten → **Freigeben**.

Licht, Cover, Klima, Schlösser, Lüfter, Mediaplayer, Timer, Listen und Szenen einschalten, mit denen ihr sprechen wollt. Versteckte Sensoren und Infrastruktur aus lassen.

Sagt Assist, das Gerät fehle, ist es meist nicht freigegeben — kein Sprachproblem. Details: [Fehlerbehebung](troubleshooting.md).

## 3. Assist-Pipeline

Einstellungen → Sprachassistenten → Pipeline bearbeiten:

- **Conversation-Engine:** Klar NLU
- Sprache-zu-Text / Text-zu-Sprache: beliebig (lokal oder Cloud)

Nicht den LLM-Agenten als Engine wählen. Sonst umgeht Assist Klar und das Modell darf Geräte anfassen.

Die Integration registriert die Lovelace-Karte **Klar home** (`klar-home-card`) und legt beim ersten Start eine **Klar**-Seitenleiste an, damit der letzte Assist-Zug ohne Kartensuche sichtbar ist.

## 4. Fünf Sätze

Nach dem Speichern der Pipeline Assist (oder den Klar-Tab **Labor**) nutzen.

| Sagen | Erwartung |
|-------|-----------|
| Licht im Wohnzimmer an | Wohnzimmerlicht an |
| Garagentor auf 40 % | Cover-Position 40 % |
| mach das Licht aus und die Heizung auf 21 | Zwei Schritte: Licht aus, Klima 21 |
| Wohnzimmer Fernseher pausieren | Media-Pause auf dem Player |
| Spiel Queen | Music-Assistant-Suche auf einem Musik-Player |

Englisch läuft in derselben Pipeline: `Turn on the living room light`, `Play Queen`.

Player im Raum? Den Raum mitnennen (`Play the playlist Chill in the living room` / Playlist plus Raum). Klar erfindet keine Playlists oder Interpreten, die Music Assistant nicht auflösen kann.

## 5. Zuordnung

**Klar NLU** in der Seitenleiste öffnen (oder `http://127.0.0.1:10520`). **Haus → Zuordnung**.

Fragt Assist „welche Lampe?“ oder verpasst einen Spitznamen, dort einen Alias setzen oder einen Raumvorschlag übernehmen. Das Overlay liegt über den Home-Assistant-Namen — HA bleibt die Gerätedatenbank.

## Weiter

- [Fehlerbehebung und Datenschutz](troubleshooting.md) — Expose-Filter, Token, Support-Bundle
- [Home Assistant](home-assistant.md) — Persönlichkeiten, LLM-Verfeinerung, Add-on, Docker, Registry-Sync
- [API](api.md) — `POST /api/v2/parse` und die Operator-UI
