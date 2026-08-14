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

## Engine starten

Lokal, mit der HA-Config nur lesend:

```bash
cargo run --release -- --config-dir /config
```

Oder Docker (Root-Dockerfile):

```bash
docker build -t klar-nlu .
docker run --network host -v /config:/config:ro klar-nlu
```

Add-on-Metadaten liegen unter `addon/` (`host_network`, Ports 10520/10500, Config read-only).

Die Engine liest `.storage/core.entity_registry` und `core.area_registry`. Aliase und Areas in HA pflegen — Klar hat keine zweite Gerätedatenbank.

## Intents

Die Integration führt die von Klar gelieferten Intent-Namen aus. Dafür müssen die Standard-Assist-Intents in HA verfügbar sein (eingebaut). Custom Sentences in Klar (`/api/custom`) können auf dieselben oder eigene Intent-Namen zeigen; eigene Namen brauchen einen Handler in HA.
