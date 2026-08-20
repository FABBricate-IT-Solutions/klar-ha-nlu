# Einstieg

[Deutsch](getting-started.md) · [English](en/getting-started.md)

Haushaltsweg: Teile unten installieren → Geräte freigeben → Assist-Pipeline → fünf Sätze → Zuordnung, wenn etwas fehlt.

Jede kompilierte Assist-Locale ist erstklassig. Deutsch und Englisch sind die üblichen Beispiele. Siehe [Sprachen](languages.md).

Nur V2: Engine und Home-Assistant-Integration dieselbe CalVer. HTTP-Parse ist `POST /api/v2/parse`.

## Integration vs App

Klar kommt in zwei Teilen. Sie machen unterschiedliche Jobs. Beides zu installieren macht das Parsen **nicht** genauer.

| Teil | Rolle | Braucht ihr das? |
|------|-------|------------------|
| **HACS-Integration** | Conversation-Agent für Assist. Räume und Geräte synchronisieren, Intents ausführen. | Ja, wenn Assist Klar nutzen soll. |
| **App (Add-on)** | NLU-Engine im eigenen Container. Zuordnung / Labor in der Seitenleiste **Klar NLU**. | Home Assistant OS, wenn ihr diese UI wollt. |
| **Mitgelieferte Engine** | Die Integration lädt das GitHub-Release und startet dieselbe Engine in Core auf `127.0.0.1:10520`. | Wenn keine App da ist. |

Nur **einen** Engine-Host. App und mitgelieferte Engine nicht gleichzeitig.

Lovelace **Klar** ist der letzte Assist-Zug (`klar-home-card`). Zuordnung und Labor sind die App-UI (**Klar NLU**), nicht diese Karte.

## 1. Klar NLU installieren

### Home Assistant OS — beides (empfohlen)

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. HACS → Integrationen → ⋮ → Benutzerdefinierte Repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → **Integration**. **Klar NLU** herunterladen und Home Assistant neu starten.
2. [App-Repository hinzufügen](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu), **Klar NLU** installieren und starten. Details: [App-Doku](../addon/DOCS.md).
3. [Integration hinzufügen](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) und **Klar-NLU-App oder Docker verwenden** wählen. URL: `http://klar-nlu:10520`.

### Ohne Supervisor — nur HACS

Dieselben HACS-Schritte, dann [Integration hinzufügen](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) und **Mitgelieferte Engine starten (nur HACS)** behalten. Assist funktioniert. Zuordnung / Labor sind nicht in der Seitenleiste (die Engine bindet Loopback in Core).

Docker statt mitgeliefert: Image starten, dann **Klar-NLU-App oder Docker verwenden** mit `http://127.0.0.1:10520`. Siehe [Home Assistant](home-assistant.md).

## 2. Entitäten für Assist freigeben

Klar steuert standardmäßig nur, was Assist sehen darf. Einstellungen → Sprachassistenten → **Freigeben**.

Licht, Cover, Klima, Schlösser, Lüfter, Mediaplayer, Timer, Listen und Szenen einschalten, mit denen ihr sprechen wollt. Versteckte Sensoren und Infrastruktur aus lassen.

Sagt Assist, das Gerät fehle, ist es meist nicht freigegeben — kein Sprachproblem. Details: [Fehlerbehebung](troubleshooting.md).

## 3. Assist-Pipeline

Einstellungen → Sprachassistenten → Pipeline bearbeiten:

- **Conversation-Engine:** Klar NLU
- Sprache-zu-Text / Text-zu-Sprache: beliebig (lokal oder Cloud)

Nicht den LLM-Agenten als Engine wählen. Sonst umgeht Assist Klar und das Modell darf Geräte anfassen.

Die Integration registriert die Lovelace-Karte **Klar home** (`klar-home-card`) und legt beim ersten Start eine **Klar**-Seitenleiste an, damit der letzte Assist-Zug ohne Kartensuche sichtbar ist. Das ist nicht Zuordnung / Labor.

## 4. Fünf Sätze

Nach dem Speichern der Pipeline Assist nutzen. Unter Home Assistant OS hat die App-Seitenleiste **Klar NLU** auch **Labor**.

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

Unter Home Assistant OS die Seitenleiste **Klar NLU** öffnen (die App, nicht Lovelace **Klar**). **Haus → Zuordnung**.

Fragt Assist „welche Lampe?“ oder verpasst einen Spitznamen, dort einen Alias setzen oder einen Raumvorschlag übernehmen. Das Overlay liegt über den Home-Assistant-Namen — HA bleibt die Gerätedatenbank.

Ohne App ist Zuordnung die Engine-UI auf `http://127.0.0.1:10520` in Core, die ein Handy nicht erreicht. Aliase können weiterhin als Entitäts-Aliase in Home Assistant gesetzt werden.

## Weiter

- [Fehlerbehebung und Datenschutz](troubleshooting.md) — Expose-Filter, Token, Support-Bundle
- [Home Assistant](home-assistant.md) — Persönlichkeiten, LLM-Verfeinerung, App, Docker, Registry-Sync
- [API](api.md) — `POST /api/v2/parse` und die Operator-UI
