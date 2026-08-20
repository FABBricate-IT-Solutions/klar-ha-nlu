# Home Assistant

[Deutsch](home-assistant.md) · [English](en/home-assistant.md)

Haushaltsweg: [Einstieg](getting-started.md). Fehltreffer, Token, Bundle: [Fehlerbehebung](troubleshooting.md).

Klar NLU hängt als Conversation-Entity an Assist. Das ist die **HACS-Integration**. Die **Engine**, die Sprache parst, ist getrennt: Klar-NLU-App (Add-on), das mitgelieferte Kind, das die Integration startet, oder Docker. Derselbe Parser — die App macht Assist nicht schlauer.

Unter Home Assistant OS beides installieren, wenn Assist **und** Zuordnung/Labor gebraucht werden. Nur HACS reicht für Assist ohne App-UI. Nur die App koppelt Assist nicht.

V2 spricht nur `POST /api/v2/parse`. Integration und Engine im selben Release aktualisieren; ein gemischtes Paar schlägt fehl. Jede kompilierte Assist-Locale ist erstklassig; Assist pinnt pro Request ein Pack.

## Integration

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Badge klicken, oder HACS → Integrationen → ⋮ → Benutzerdefinierte Repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → Kategorie **Integration**.
2. **Klar NLU** herunterladen und Home Assistant neu starten.
3. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu)  
   oder Einstellungen → Geräte & Dienste → Integration hinzufügen → **Klar NLU**.
4. Mit laufender App **Klar-NLU-App oder Docker verwenden**. Ohne App **Mitgelieferte Engine starten (nur HACS)** (braucht ein GitHub Release mit Linux-Tarballs).
5. Mitgelieferte Engine: **Release-Kanal** = Stable (CalVer) oder Staging (neuestes GitHub-Prerelease). Später unter Konfigurieren änderbar.

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

Mit LLM-Refine steckt die Stimme im Satz — nicht in einem Stempel wie „Sehr wohl“. Ohne Refine rotiert eine kurze gesprochene Variante (manchmal gar keine Formel). Der Fallback ist kein fester Stempel. Refine folgt der Sprache der Input-Zeile, nicht stillschweigend Deutsch.

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

Nur **einen** Host. Ein Reload der Integration auf die App-URL beendet das mitgelieferte Kind.

**Home Assistant OS (empfohlen):** HACS-Integration **plus** App. Die App ist Engine-Host und Zuordnung/Labor (Seitenleiste **Klar NLU**).

[![Open your Home Assistant instance and show the add add-on repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu)

`https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` als App-Repository hinzufügen, **Klar NLU** oder **Klar NLU (Staging)** installieren und starten, in der Integration **Klar-NLU-App oder Docker verwenden** (`http://klar-nlu:10520`).

**Mitgeliefert (keine App):** HACS-Integration → **Mitgelieferte Engine starten (nur HACS)**. Lädt das GitHub-Release nach `/config/klar_nlu/`. Assist funktioniert. Zuordnung/Labor binden `127.0.0.1` in Core, ein Handy kommt nicht ran. Lovelace **Klar** ist nur der letzte Assist-Zug.

**Stable vs Staging:** ein Schalter. Einstellungen → Geräte & Dienste → Klar NLU → Konfigurieren → **Release-Kanal**. Stable zeigt auf die Klar-NLU-App (`http://klar-nlu:10520`) bzw. das CalVer-GitHub-Release. Staging zeigt auf Klar NLU (Staging) (`http://klar-nlu-staging:10520`) bzw. das neueste Prerelease. Eine eigene URL bleibt unverändert. Nach dem Wechsel lädt die Integration neu. Nicht in `.storage` umbiegen.

Supervisor liest das Add-on-Repo von `main`. **Klar NLU (Staging)** erscheint im Store erst, wenn `addon-staging/` dort liegt. Nach einem Merge auf `staging` die Staging-App neu bauen (Version bleibt `staging`).

**Docker:**

```bash
docker run --rm --network host \
  -v /pfad/zur/homeassistant/config:/config:ro \
  ghcr.io/fabbricate-it-solutions/klar-nlu:2026.8.30
```

CalVer der Engine verwenden (`Cargo.toml` / GitHub-Release), kein altes `0.1.x`-Tag. Für einen Release Candidate: `ghcr.io/fabbricate-it-solutions/klar-nlu:staging`.

Integrations-URL: `http://127.0.0.1:10520`. Aus dem Quellcode: `docker build -t klar-nlu .` (Root-Dockerfile).

**Cargo:**

```bash
cargo run --release -- --config-dir /config
```

Die Engine kann `.storage/core.entity_registry`, `core.device_registry`, `core.area_registry`, `core.floor_registry`, `core.label_registry` und die Assist-Expose-Liste lesen. Das ist der Fallback für Wyoming-only oder wenn die Integration die Engine nicht anstößt. Aliase, Areas, Etagen und Assist-Freigabe in HA pflegen — Klar hat keine zweite Gerätedatenbank.

## Registry-Sync (HA ist Quelle)

Die Integration ist der offizielle Sync-Pfad. Nach dem Setup und bei Registry-/Expose-Änderungen (`entity`, `device`, `area`, `floor`, `label`, `exposed_entities`) schickt sie einen versionierten Snapshot an `POST /api/v2/home`.

```json
{
  "schema_version": "1",
  "entities": [
    {
      "entity_id": "light.living",
      "name": "Wohnzimmer Decke",
      "original_name": "Ceiling",
      "has_entity_name": true,
      "area_id": "living",
      "device_id": "dev1",
      "platform": "hue",
      "aliases": ["decke"],
      "labels": ["Licht"],
      "disabled": false
    }
  ],
  "devices": [{"id": "dev1", "name": "Hue", "name_by_user": null, "area_id": "living"}],
  "areas": [{"id": "living", "name": "Wohnzimmer", "aliases": ["wohnzimmer"], "floor_id": "upper"}],
  "floors": [{"floor_id": "upper", "name": "Obergeschoss", "aliases": ["upstairs"], "level": 1}],
  "labels": [{"label_id": "lbl_1", "name": "Licht"}],
  "assist": ["light.living"]
}
```

`schema_version` muss `"1"` sein. Die Engine prüft den Snapshot an der API-Grenze: unbekannte Felder und ungültiges JSON mit `422`, leere IDs, Steuerzeichen und Schemafehler mit `400`, zu große Bodies oder Listen mit `413`. Keiner dieser Fälle stürzt den Prozess ab. Caps: 4096 Entities, 2048 Devices, 256 Areas, 64 Etagen, 256 Labels, 4096 Assist-IDs, 32 Aliase je Eintrag. `assist: null` bedeutet keine Expose-Filterung; ein Array begrenzt die sichtbaren IDs.

Nach einem gültigen Push ist HA die laufende Quelle. Die `.storage`-Dateiüberwachung überschreibt diesen Live-Graph nicht mehr. Overlay-Kalibrierung (Aliase, manuelle Areas, Preferred, Infra) wird weiterhin darübergelegt.

## Registry, Overlay und Reload

Klar baut einen effektiven Home-Graph:

1. Live-Snapshot von der Integration (`POST /api/v2/home`), sobald vorhanden.
2. Sonst HA-Registries aus `--config-dir` lesen: Entities, Devices, Areas, Floors, Labels und Expose-Liste.
3. Fehlen die Registries, die eingebaute Musterwohnung nutzen.
4. Overlay aus `--config-dir` anwenden.
5. Wenn `--data-dir` existiert und von `--config-dir` abweicht, Overlay aus `--data-dir` darüber anwenden.

Das Overlay enthält Kalibrierung aus der Klar-UI: Aliase, manuelle Areas, bevorzugte Geräte, Infrastruktur-Filter, Timer-Hinweise, Settings und Custom Sentences. Im Add-on ist `/config` read-only gedacht und `/data` beschreibbar; bei Cargo/Docker kann beides auf dasselbe Verzeichnis zeigen.

Ohne Live-Sync beobachtet Klar die HA-Registry-Dateien und lädt den Home-Graph bei Änderungen neu. Settings und Custom Sentences werden dabei aus den Overlays erneut übernommen. Änderungen über die Klar-UI werden sofort gespeichert und auf den laufenden `HomeStore` angewendet.

## Zugriff und Token

Loopback darf lesen und schreiben. Das Supervisor-Netz darf lesen; Schreibzugriffe von dort oder aus dem LAN brauchen einen Token. Setze ihn mit `--token`, `KLAR_TOKEN` oder `--token-file`.

```bash
cargo run --release -- --config-dir /config --data-dir /data --token-file /data/klar.token
```

HTTP akzeptiert den Token als `x-klar-token` oder `Authorization: Bearer ...`. Wyoming ist auf Loopback und Supervisor-Netz begrenzt.

## Intents

Die Integration führt nur `decision.type == execute` aus. Confirm, Clarify und Reject lösen keine Services aus. Ein Execute-Plan läuft in Planreihenfolge über `intent.async_handle`. Jeder Schritt liefert success oder error; Teilfehler sind ein eigenes Ergebnis (Sprache plus strukturierte Fehler), kein stilles Gesamterfolg. Direkte Service-Calls gibt es nur, wo HA keinen nativen Intent hat (Music Assistant, Relativlautstärke, Mute). Custom Sentences in Klar NLU (`/api/custom`) können auf dieselben oder eigene Intent-Namen zeigen; eigene Namen brauchen einen Handler in HA.

## Medien und Music Assistant

Pause, weiter, zurück, stumm und Lautstärke nutzen den genannten `media_player` oder den im Raum. Den Player freigeben.

| Satz | Hinweis |
|------|---------|
| Wohnzimmer Fernseher pausieren | Native Media-Pause |
| Spiel Queen | Suche auf einem Music-Assistant-Player |
| Play the playlist Chill in the living room | Playlist-Klasse und Raum bleiben |
| Musik an | Setzt den MA-Player fort, nicht ein Script-Alias „musik“ |

Klar NLU erfindet keine Bibliothek. Nicht erreichbare Player werden übersprungen. Siehe [Einstieg](getting-started.md) und [Fehlerbehebung](troubleshooting.md).
