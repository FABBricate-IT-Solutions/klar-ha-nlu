# Klar NLU

Runs the Klar NLU **engine** next to Home Assistant (Mapping / Lab in sidebar **Klar NLU**). Assist still needs the **HACS integration** — this App does not replace it, and installing both does not make parsing more accurate.

Household path: [getting started](../docs/en/getting-started.md) · [Einstieg](../docs/getting-started.md). Token, expose, bundle: [troubleshooting](../docs/en/troubleshooting.md).

## After start

1. Install **Klar NLU** via HACS (category Integration) if it is not already there.
2. Add the integration and choose **Use the Klar NLU App or Docker**.
3. URL: `http://klar-nlu:10520`
4. If you set the App **token**, paste the same value into the integration **Write token**. Overlay writes from Home Assistant need it (Supervisor is not loopback).
5. In the integration options: personality, optional chit-chat agent, optional **Let the LLM refine NLU replies**. Assist’s conversation engine must stay **Klar NLU**.

Do not also pick **Start the bundled engine (HACS only)** while this App is running.

UI: open **Klar NLU** from the Home Assistant sidebar (the App). Lovelace **Klar** is only the last Assist turn. Direct access still works at `http://<home-assistant-host>:10520` if you expose the port.

## Add-on options

| Option | Meaning |
|--------|---------|
| `token` | Shared write secret → `KLAR_TOKEN`. Empty = no token. Same string as the integration write token. |
| `support_bundle` | Record Assist requests, replies, and actions under `/data`. |
| `ui_locale` | Seeds operator chrome → `KLAR_UI_LOCALE` until App → Settings → Operator language is saved. |

Settings stay in `/data/klar_nlu.json`, recordings in `/data/support_bundle.jsonl`. In the Klar UI you can download, delete selected rows, or clear the log. Downloads are redacted (hashed conversation IDs, pseudonymized names) unless you turn on raw text in Settings. Environment `KLAR_SUPPORT_BUNDLE=1` only seeds the first start; the UI setting wins afterwards. `KLAR_UI_LOCALE` / `ui_locale` seeds the operator language until Settings is saved.

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
