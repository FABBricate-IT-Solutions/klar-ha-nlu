# Klar NLU

Starts the Klar engine next to Home Assistant. The conversation integration is installed separately with [HACS](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu).

## After start

1. Install **Klar NLU** via HACS (category Integration).
2. Add the integration and choose **Use an engine that is already running**.
3. URL: `http://klar-nlu:10520`

UI: `http://<home-assistant-host>:10520`

## Docker without the add-on

```bash
docker run --rm --network host \
  -v /path/to/homeassistant/config:/config:ro \
  ghcr.io/fabbricate-it-solutions/klar-nlu:0.1.0
```

Then use `http://127.0.0.1:10520` in the integration.
