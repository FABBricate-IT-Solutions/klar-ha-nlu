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

Die Auswahl liegt in dieser Integration, nicht in der Klar-App, und überlebt eine Neuinstallation der Engine. Assist und der LLM-Verfeinerungs-Prompt wechseln mit. Nur die Persönlichkeit zu ändern startet die Engine nicht neu.

| Id | Stimme |
|----|--------|
| `default` | schlicht, freundlich |
| `butler` | höflicher Butler, gewählt und diskret |
| `locker` | kumpelhaft, locker |
| `fuersorglich` | warm, beruhigend |
| `party` | euphorisch, feiernd |
| `grantig` | knurrig, widerwillig |
| `sarkastisch` | trocken sarkastisch |
| `pirat` | piratenhaft, verständlich |
| `hippie` | entspannt, weich |
| `gollum` | knisternd, verständlich |

Mit LLM-Refine steckt die Stimme im Satz — nicht in einem Stempel wie „Sehr wohl“. Ohne Refine bleibt eine kurze Formel als Fallback (`Sehr wohl.`, `Aye.`, …).

## LLM-Verfeinerung

Standardmäßig aus. Einstellungen → Geräte & Dienste → Klar NLU → Konfigurieren:

1. **Conversation-Agent für Smalltalk** setzen (OpenAI-kompatibel, lokales Gemma reicht).
2. **NLU-Antworten vom LLM verfeinern** einschalten.
3. Assist-Pipeline: Conversation-Engine = **Klar NLU**.
4. Assist-Werkzeuge bei diesem LLM-Agenten **aus**. Kann der Agent das Haus steuern, fällt Refine aus.

Ablauf nach einem Hausbefehl:

1. Klar parst, HA führt die Intents aus.
2. Das Fallback-LLM formuliert die fertige NLU-Antwort in der gewählten Persönlichkeit um — ein oder zwei gesprochene Sätze, nach Steuerung und nach Statusabfrage (kein Smalltalk, kein News-Briefing, keine Rückfrage).
3. Die Umformulierung bleibt stehen. Klar klebt keine Formel mehr davor oder dahinter.
4. Schlägt Refine fehl, bleibt die kurze Fallback-Formel.

Der Prompt ist pro Persönlichkeit (Stimme plus Few-Shots). Das Feld **Verfeinerungs-Prompt** ist nur eine Extra-Zeile darüber — es ersetzt die Stimme nicht.

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

Die Engine liest `.storage/core.entity_registry`, `core.device_registry`, `core.area_registry` und die Assist-Expose-Liste. Aliase und Areas in HA pflegen — Klar hat keine zweite Gerätedatenbank.

## Registry, Overlay und Reload

Klar baut beim Start einen effektiven Home-Graph:

1. HA-Registries aus `--config-dir` lesen: Entities, Devices, Areas, Labels und Expose-Liste.
2. Fehlen die Registries, die eingebaute Musterwohnung nutzen.
3. Overlay aus `--config-dir` anwenden.
4. Wenn `--data-dir` existiert und von `--config-dir` abweicht, Overlay aus `--data-dir` darüber anwenden.

Das Overlay enthält Kalibrierung aus der Klar-UI: Aliase, manuelle Areas, bevorzugte Geräte, Infrastruktur-Filter, Timer-Hinweise, Settings und Custom Sentences. Im Add-on ist `/config` read-only gedacht und `/data` beschreibbar; bei Cargo/Docker kann beides auf dasselbe Verzeichnis zeigen.

Klar beobachtet die HA-Registry-Dateien und lädt den Home-Graph bei Änderungen neu. Settings und Custom Sentences werden dabei aus den Overlays erneut übernommen. Änderungen über die Klar-UI werden sofort gespeichert und auf den laufenden `HomeStore` angewendet.

## Zugriff und Token

Loopback darf lesen und schreiben. Das Supervisor-Netz darf lesen; Schreibzugriffe von dort oder aus dem LAN brauchen einen Token. Setze ihn mit `--token`, `KLAR_TOKEN` oder `--token-file`.

```bash
cargo run --release -- --config-dir /config --data-dir /data --token-file /data/klar.token
```

HTTP akzeptiert den Token als `x-klar-token` oder `Authorization: Bearer ...`. Wyoming ist auf Loopback und Supervisor-Netz begrenzt.

## Intents

Die Integration führt die von Klar gelieferten Intent-Namen aus. Dafür müssen die Standard-Assist-Intents in HA verfügbar sein (eingebaut). Custom Sentences in Klar (`/api/custom`) können auf dieselben oder eigene Intent-Namen zeigen; eigene Namen brauchen einen Handler in HA.
