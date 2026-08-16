# Klar NLU

Starts the Klar engine next to Home Assistant. The conversation integration is installed separately with [HACS](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu).

## After start

1. Install **Klar NLU** via HACS (category Integration).
2. Add the integration and choose **Use an engine that is already running**.
3. URL: `http://klar-nlu:10520`
4. In the integration options: personality, optional chit-chat agent, optional **Let the LLM refine NLU replies**. Assist’s conversation engine must stay **Klar NLU**.

UI: open **Klar NLU** from the Home Assistant sidebar. Direct access still works at `http://<home-assistant-host>:10520` if you expose the port.

Optional add-on option **support_bundle**: record Assist requests, replies, and actions under `/data`. Settings stay in `/data/klar_nlu.json`, recordings in `/data/support_bundle.jsonl`. In the Klar UI you can download, delete selected rows, or clear the log. Environment `KLAR_SUPPORT_BUNDLE=1` only seeds the first start; the UI setting wins afterwards.

## Docker without the add-on

```bash
docker run --rm --network host \
  -v /path/to/homeassistant/config:/config:ro \
  -v /path/to/klar-data:/data \
  ghcr.io/fabbricate-it-solutions/klar-nlu:0.1.0
```

Then use `http://127.0.0.1:10520` in the integration.
