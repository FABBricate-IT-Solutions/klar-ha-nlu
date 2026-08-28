# Getting started

[Deutsch](../getting-started.md) · [English](getting-started.md)

Household path: install the pieces below → expose devices → Assist pipeline → try five phrases → Mapping if something misses.

Every compiled Assist locale is first-class. German and English are the usual examples. See [languages](languages.md).

V2 only: the engine and the Home Assistant integration must be the same CalVer. HTTP parse is `POST /api/v2/parse`.

## Integration vs App

Klar ships two pieces. They do different jobs. Installing both does **not** make parsing more accurate.

| Piece | Role | Need it? |
|-------|------|----------|
| **HACS integration** | Conversation agent for Assist. Syncs rooms and devices, runs intents. | Yes, if Assist should use Klar. |
| **App (add-on)** | Runs the NLU engine in its own container. Mapping / Lab in sidebar **Klar NLU**. | Home Assistant OS, if you want that UI. |
| **Bundled engine** | The integration downloads the GitHub Release and starts the same engine inside Core on `127.0.0.1:10520`. | When there is no App. |

Pick **one** host for the engine. Do not run the App and the bundled engine at the same time.

Lovelace **Klar** is the last Assist turn (`klar-home-card`). Mapping and Lab are the App UI (**Klar NLU**), not that card.

## 1. Install Klar NLU

### Home Assistant OS — both (recommended)

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. HACS → Integrations → ⋮ → Custom repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → **Integration**. Download **Klar NLU** and restart Home Assistant.
2. [Add the App repository](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu), install **Klar NLU**, and start it. Details: [App docs](../../addon/DOCS.md).
3. [Add the integration](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) and pick **Use the Klar NLU App or Docker**. URL: `http://klar-nlu:10520`.

### Without Supervisor — HACS only

Same HACS steps, then [add the integration](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) and keep **Start the bundled engine (HACS only)**. Assist works. Mapping / Lab are not in the sidebar (the engine binds loopback inside Core).

Docker instead of bundled: run the image, then pick **Use the Klar NLU App or Docker** with `http://127.0.0.1:10520`. See [Home Assistant](home-assistant.md).

## 2. Expose entities to Assist

Klar only steers what Assist may see (default). Settings → Voice assistants → **Expose**.

Turn on the lights, covers, climate, locks, fans, media players, timers, lists, and scenes you want to talk to. Leave hidden sensors and infrastructure off.

If Assist says the device is missing, it is usually not exposed — not a language problem. Details: [troubleshooting](troubleshooting.md).

## 3. Assist pipeline

Settings → Voice assistants → edit the pipeline:

- **Conversation engine:** Klar NLU
- Speech-to-text / text-to-speech: your choice (local or cloud)

Do not set an LLM as the conversation engine. Assist would skip Klar and the model could control devices.

The integration registers the **Klar home** Lovelace card (`klar-home-card`) and adds a **Klar** sidebar view on first setup so the last Assist turn is visible without hunting the card picker. That is not Mapping / Lab.

## 4. Five phrases

Use Assist after the pipeline is saved. On Home Assistant OS, the App sidebar **Klar NLU** also has **Lab**.

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

On Home Assistant OS open sidebar **Klar NLU** (the App, not Lovelace **Klar**). **House → Mapping**.

If Assist asks “which lamp?” or misses a nickname, add an alias or accept a room suggestion there. That overlay sits on top of Home Assistant names — HA stays the device database.

Without the App, Mapping is the engine UI on `http://127.0.0.1:10520` inside Core, which a phone cannot reach. Aliases can still be set as entity aliases in Home Assistant.

## Next

- [Troubleshooting and privacy](troubleshooting.md) — expose filter, token, support bundle
- [Home Assistant](home-assistant.md) — personalities, LLM refine, App, Docker, registry sync
- [API](api.md) — `POST /api/v2/parse` and the operator UI
