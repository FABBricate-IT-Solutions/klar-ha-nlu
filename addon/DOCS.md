# Klar NLU

Starts the Klar NLU engine next to Home Assistant. The conversation integration is installed separately with [HACS](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu).

Household path: [getting started](../docs/en/getting-started.md) · [Einstieg](../docs/getting-started.md). Token, expose, bundle: [troubleshooting](../docs/en/troubleshooting.md).

## After start

1. Install **Klar NLU** via HACS (category Integration).
2. Add the integration and choose **Use an engine that is already running**.
3. URL: `http://klar-nlu:10520`
4. If you set the add-on **token**, paste the same value into the integration **Write token**. Overlay writes from Home Assistant need it (Supervisor is not loopback).
5. In the integration options: personality, optional chit-chat agent, optional **Let the LLM refine NLU replies**. Assist’s conversation engine must stay **Klar NLU**.

UI: open **Klar NLU** from the Home Assistant sidebar. Direct access still works at `http://<home-assistant-host>:10520` if you expose the port.

## Add-on options

| Option | Meaning |
|--------|---------|
| `token` | Shared write secret → `KLAR_TOKEN`. Empty = no token. Same string as the integration write token. |
| `support_bundle` | Record Assist requests, replies, and actions under `/data`. |

Settings stay in `/data/klar_nlu.json`, recordings in `/data/support_bundle.jsonl`. In the Klar UI you can download, delete selected rows, or clear the log. Downloads are redacted (hashed conversation IDs, pseudonymized names) unless you turn on raw text in Settings. Environment `KLAR_SUPPORT_BUNDLE=1` only seeds the first start; the UI setting wins afterwards.

## Docker without the add-on

Pin the image tag to the engine CalVer (same as `Cargo.toml` / the GitHub Release). Current tree: **2026.8.30**.

```bash
docker run --rm --network host \
  -v /path/to/homeassistant/config:/config:ro \
  -v /path/to/klar-data:/data \
  ghcr.io/fabbricate-it-solutions/klar-nlu:2026.8.30
```

Then use `http://127.0.0.1:10520` in the integration.

## Staging / release candidate

Same add-on repository, second slug **Klar NLU (Staging)** (`klar_nlu_staging`, image tag `staging`). URL: `http://klar-nlu-staging:10520`. Rebuild after each merge to `staging`. Bundled engine: Configure → **Release channel**. Details: [addon-staging/DOCS.md](../addon-staging/DOCS.md) and [releases](../docs/en/releases.md).
