# Getting started

[Deutsch](../getting-started.md) · [English](getting-started.md)

Household path: HACS → expose devices → Assist pipeline → try five phrases → Mapping if something misses.

Every compiled Assist locale is first-class. German and English are the usual examples. See [languages](languages.md).

V2 only: the engine and the Home Assistant integration must be the same CalVer. HTTP parse is `POST /api/v2/parse`.

## 1. Install Klar NLU

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. HACS → Integrations → ⋮ → Custom repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → **Integration**.
2. Download **Klar NLU** and restart Home Assistant.
3. [Add the integration](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) and keep **Start the bundled engine**.

HACS cannot start the Rust process. The integration downloads the matching GitHub Release and runs it on `127.0.0.1:10520`.

Already running the [add-on](../../addon/DOCS.md) or Docker? Choose **Use an engine that is already running** and set the URL (`http://klar-nlu:10520` on HAOS).

## 2. Expose entities to Assist

Klar only steers what Assist may see (default). Settings → Voice assistants → **Expose**.

Turn on the lights, covers, climate, locks, fans, media players, timers, lists, and scenes you want to talk to. Leave hidden sensors and infrastructure off.

If Assist says the device is missing, it is usually not exposed — not a language problem. Details: [troubleshooting](troubleshooting.md).

## 3. Assist pipeline

Settings → Voice assistants → edit the pipeline:

- **Conversation engine:** Klar NLU
- Speech-to-text / text-to-speech: your choice (local or cloud)

Do not set an LLM as the conversation engine. Assist would skip Klar and the model could control devices.

The integration registers the **Klar home** Lovelace card (`klar-home-card`) and adds a **Klar** sidebar view on first setup so the last Assist turn is visible without hunting the card picker.

## 4. Five phrases

Use Assist (or the Klar **Lab** tab) after the pipeline is saved.

| Say | Expect |
|-----|--------|
| Turn on the living room light | Living-room lights on |
| Set the garage door to 40% | Cover position 40% |
| Turn the lights off and set heat to 21 | Two steps: lights off, climate 21 |
| Pause the living room TV | Media pause on that player |
| Play Queen | Music Assistant search-and-play on a music player |

German works in the same pipeline: `Licht im Wohnzimmer an`, `Spiel Queen`.

Need a player in a room? Name the area (`Play the playlist Chill in the living room`). Klar does not invent playlists or artists that Music Assistant cannot resolve.

## 5. Mapping tab

Open **Klar NLU** in the sidebar (or `http://127.0.0.1:10520`). **House → Mapping**.

If Assist asks “which lamp?” or misses a nickname, add an alias or accept a room suggestion there. That overlay sits on top of Home Assistant names — HA stays the device database.

## Next

- [Troubleshooting and privacy](troubleshooting.md) — expose filter, token, support bundle
- [Home Assistant](home-assistant.md) — personalities, LLM refine, add-on, Docker, registry sync
- [API](api.md) — `POST /api/v2/parse` and the operator UI
