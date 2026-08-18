# Klar NLU (Staging)

Release-candidate add-on. Supervisor pulls `ghcr.io/fabbricate-it-solutions/klar-nlu-{arch}:staging`.

This is not production. Install **Klar NLU** (stable) for the last CalVer cut. Do not run both add-ons at once unless you point the integration at one URL only.

## Switch to staging

1. Settings → Add-ons → add the same repository `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` if it is missing.
2. Install **Klar NLU (Staging)** (`stage: experimental`).
3. Start it, then in the Klar NLU integration choose **Use an engine that is already running**.
4. URL: `http://klar-nlu-staging:10520`
5. If you set the add-on **token**, paste the same value into the integration write token.

The image tag `staging` is a moving tag. After a merge to `staging`, rebuild this add-on (no Supervisor version bump — the add-on version stays `staging`).

## Switch back to stable

Stop and uninstall **Klar NLU (Staging)** (or leave it stopped). Install/start **Klar NLU**. Point the integration at `http://klar-nlu:10520`.

## Bundled engine (no add-on)

Settings → Devices & services → Klar NLU → Configure → **Release channel** → Staging. That downloads the latest GitHub prerelease. Switch back to **Stable** for the current CalVer.

## Options

Same as the stable add-on (`token`, `support_bundle`). See [addon/DOCS.md](../addon/DOCS.md).
