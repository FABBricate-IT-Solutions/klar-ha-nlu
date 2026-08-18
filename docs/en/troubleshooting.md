# Troubleshooting and privacy

[Deutsch](../troubleshooting.md) · [English](troubleshooting.md)

Household setup first: [getting started](getting-started.md). This page covers misses, the write token, and what stays in the house.

## Device not found

1. **Expose.** Settings → Voice assistants → Expose. Klar’s option **Only control entities exposed to Assist** is on by default. Hidden sensors and switches are not targets.
2. **Name and area.** The entity needs a spoken name and a room in Home Assistant. Generic “light” in a room with three lamps becomes a clarify question.
3. **Mapping.** Klar UI → **House → Mapping**. Add an alias or accept a room suggestion. Do not build a second device list in Klar.
4. **Language.** Pin Assist to the locale you speak (`de`, `en`, `fr`, …). Klar binds that pack for the request.

The integration option **Only control entities exposed to Assist** is a developer escape hatch. Off matches hidden entities too — easier to hit the wrong device.

## Assist talks but nothing moves

- Pipeline conversation engine must be **Klar NLU**, not the chit-chat LLM.
- Engine and integration must be the same CalVer (V2: `POST /api/v2/parse` only).
- Bundled engine: wait until the GitHub Release has finished downloading into `.storage/klar_nlu/`.
- Add-on / Docker: integration URL `http://klar-nlu:10520` (HAOS) or `http://127.0.0.1:10520` (host network).
- Confirm / clarify never call services. Answer `yes` / `ja` on the same conversation, or name the device.

## Media and Music Assistant

- Pause / next / mute use the `media_player` you named or the one in that room.
- `Play Queen` / `Spiel Queen` needs a Music Assistant player (or a media player Klar can search on). Klar does not invent a library.
- Unavailable players are skipped. Expose the player you want.

## Write token

Loopback may read and write. The Supervisor network may read. Writes from Supervisor or the LAN need a token (`x-klar-token` or `Authorization: Bearer`).

| How Klar runs | Where the token lives |
|---------------|------------------------|
| Bundled engine | Created under `.storage/klar_nlu/token` and sent by the integration |
| Add-on | Add-on option **token** → `KLAR_TOKEN`. Paste the same value into the integration **Write token** |
| Docker / cargo | `--token`, `KLAR_TOKEN`, or `--token-file` |

Empty add-on token means no shared secret: overlay writes from Home Assistant fail unless they come from loopback.

## Support bundle

Settings in the Klar UI (or add-on option **support_bundle**): record parse traffic under `/data/support_bundle.jsonl` (max 2000 rows). `KLAR_SUPPORT_BUNDLE=1` only seeds the first start.

Downloads are redacted:

- Conversation IDs are hashed
- Entity and area names are pseudonymized
- Raw utterance and speech stay out unless **support_bundle_raw_text** is on (off by default)

The conversation journal (UI **Conversations**) keeps the last 200 turns for 24 hours. Raw text follows the same flag.

## What never leaves the house

The engine is local. No cloud, no model weights, no phone-home.

An optional LLM in Home Assistant may rewrite a finished confirmation or handle chit-chat. That is your agent, not Klar. Keep Assist tools **off** on that agent. NLU-RAG (off by default) may send an already-matched slice to that fallback — never Assist or Home Assistant control tools.

Do not commit `KLAR_TOKEN`, `klar.token`, or unredacted bundles.
