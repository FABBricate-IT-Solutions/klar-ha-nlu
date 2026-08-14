# Home Assistant

[Deutsch](home-assistant.md) · [English](en/home-assistant.md)

Klar hängt als Conversation-Entity an Assist. Die Engine läuft getrennt (Binary, Docker oder Add-on); die Integration spricht sie per HTTP an.

## Integration

1. `custom_components/klar_nlu` nach `<config>/custom_components/klar_nlu` kopieren.
2. Home Assistant neu starten.
3. Einstellungen → Geräte & Dienste → Integration hinzufügen → **Klar NLU**.
4. URL der Engine, Standard `http://127.0.0.1:10520`.

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
