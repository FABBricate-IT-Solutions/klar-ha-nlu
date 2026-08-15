# Home Assistant

[Deutsch](home-assistant.md) · [English](en/home-assistant.md)

Klar hängt als Conversation-Entity an Assist. HACS kann die Rust-Engine nicht starten. Die Integration schon: sie lädt das passende GitHub-Release nach `.storage/klar_nlu/` und startet es auf `127.0.0.1:10520`.

## Integration

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Badge klicken, oder HACS → Integrationen → ⋮ → Benutzerdefinierte Repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → Kategorie **Integration**.
2. **Klar NLU** herunterladen und Home Assistant neu starten.
3. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu)  
   oder Einstellungen → Geräte & Dienste → Integration hinzufügen → **Klar NLU**.
4. **Mitgelieferte Engine starten** behalten (braucht ein GitHub Release mit Linux-Tarballs). Oder **Bereits laufende Engine verwenden** und die URL setzen.

Ohne HACS `custom_components/klar_nlu` nach `<config>/custom_components/klar_nlu` kopieren und neu starten.

Nur eine Instanz. Die URL bleibt im ersten Schritt; der Smalltalk-Agent liegt in den Optionen.

## Assist-Pipeline

Einstellungen → Sprachassistenten → Pipeline bearbeiten:

- **Conversation-Engine:** Klar NLU
- STT/TTS beliebig (lokal oder Cloud)

Nicht den LLM-Agenten direkt als Engine wählen. Sonst umgeht Assist Klar und das LLM darf Geräte anfassen.

## LLM-Fallback

Einstellungen → Geräte & Dienste → Klar NLU → Konfigurieren → **Conversation-Agent für Smalltalk**.

Ablauf:

1. Klar parst.
2. Haus-Intents werden über `intent.async_handle` ausgeführt.
3. Rückfragen (`clarify`) bleiben bei Klar.
4. Keine Intents → Weiterleitung an den gewählten Agenten.
5. Klar selbst und ein unerreichbarer Motor lösen keinen Fallback aus.

Der Agent bekommt den Hinweis, keine Geräte zu steuern. Wenn der Agent in seiner eigenen Integration HA-Tools hat, können die trotzdem greifen — Tools dort aus lassen, wenn Smalltalk nur reden soll.

## Persönlichkeit

Einstellungen → Geräte & Dienste → Klar NLU → Konfigurieren → **Persönlichkeit**, oder die Select-Entity **Persönlichkeit** am Klar-Gerät.

Die Auswahl liegt in dieser Integration, nicht in der Klar-App, und überlebt eine Neuinstallation der Engine. Assist, Sprechformel und LLM-Verfeinerungs-Prompt wechseln mit. Nur die Persönlichkeit zu ändern startet die Engine nicht neu.

| Id | Formel (DE) | Stilwort |
|----|-------------|----------|
| `default` | — | schlichte Bestätigung |
| `butler` | Sehr wohl. | …, wie gewünscht. |
| `locker` | Geht klar. | …, passt. |
| `fuersorglich` | Mache ich sofort. | …, alles gut. |
| `party` | Läuft! | …, super! |
| `grantig` | Schon gut. | …, na gut. |
| `sarkastisch` | Wie überraschend, wieder ein Befehl. | …, natürlich. |
| `pirat` | Aye. | …, Käpt'n. |
| `hippie` | Alles easy. | …, ganz ruhig. |
| `gollum` | Ja, mein Schatz. | …, ja. |

Englisch nutzt die passenden Formeln (`Very well.`, `Got it.`, `Aye.`, `Yes, my precious.`, …).

## LLM-Verfeinerung

Standardmäßig aus. Einstellungen → Geräte & Dienste → Klar NLU → Konfigurieren:

1. **Conversation-Agent für Smalltalk** setzen (OpenAI-kompatibel, lokales Gemma reicht).
2. **NLU-Antworten vom LLM verfeinern** einschalten.
3. Assist-Pipeline: Conversation-Engine = **Klar NLU**.
4. Assist-Werkzeuge bei diesem LLM-Agenten **aus**. Kann der Agent das Haus steuern, fällt Refine aus.

Ablauf nach einem Hausbefehl:

1. Klar parst, HA führt die Intents aus.
2. Klar setzt die Formel der gewählten Persönlichkeit davor.
3. Das Fallback-LLM formuliert den fertigen Satz um — nach Steuerung und nach Statusabfrage (kein Smalltalk, kein News-Briefing, keine Rückfrage).
4. Fehlt die Formel danach, setzt Klar sie wieder.

Der Prompt ist pro Persönlichkeit (Few-Shots plus festes Stilwort). Das Feld **Verfeinerungs-Prompt** ist nur eine Extra-Zeile darüber — es ersetzt die Stimme nicht.

Die Sicherheit bleibt bei Klar, nicht beim Modell:

- keine Gerätesteuerung, keine Home-Assistant-Werkzeuge
- Räume, Namen, an/aus/offen/zu bleiben
- Ziffern bleiben Ziffern (`21` bleibt `21`, nicht einundzwanzig)
- keine erfundenen Zahlen (Temperatur ohne Wert bleibt ohne Wert)
- Intent-Namen wie `HassSetPosition` werden verworfen

Bei OpenAI-kompatiblen Agenten schickt Klar `chat_template_kwargs.enable_thinking=false`, damit Gemma 4 nicht die ganze Runde im Thought-Kanal verbringt. Prompt-Text schaltet Thinking nicht aus. Fehlt ein direkter Chat-Client, fällt Klar auf `conversation.async_converse` zurück und behält die NLU-Antwort nur, wenn der Rewrite fehlschlägt.

## Engine starten

**Mitgeliefert (am einfachsten):** HACS-Integration → **Mitgelieferte Engine starten**. Lädt das GitHub-Release nach `.storage/klar_nlu/`.

**Add-on (HAOS):**

[![Open your Home Assistant instance and show the add add-on repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu)

`https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` als Add-on-Repository hinzufügen, **Klar NLU** installieren, Integration auf `http://klar-nlu:10520` zeigen.

**Docker:**

```bash
docker run --rm --network host \
  -v /pfad/zur/homeassistant/config:/config:ro \
  ghcr.io/fabbricate-it-solutions/klar-nlu:0.1.0
```

Integrations-URL: `http://127.0.0.1:10520`. Aus dem Quellcode: `docker build -t klar-nlu .` (Root-Dockerfile).

**Cargo:**

```bash
cargo run --release -- --config-dir /config
```

Die Engine liest `.storage/core.entity_registry` und `core.area_registry`. Aliase und Areas in HA pflegen — Klar hat keine zweite Gerätedatenbank.

## Intents

Die Integration führt die von Klar gelieferten Intent-Namen aus. Dafür müssen die Standard-Assist-Intents in HA verfügbar sein (eingebaut). Custom Sentences in Klar (`/api/custom`) können auf dieselben oder eigene Intent-Namen zeigen; eigene Namen brauchen einen Handler in HA.
