# Klar NLU (Staging)

Release-candidate **App**. Supervisor pulls `ghcr.io/fabbricate-it-solutions/klar-nlu-{arch}:staging`. Assist still uses the HACS integration; this App only hosts the engine. Not production — use **Klar NLU** for the last CalVer cut.

## Switch

Settings → Devices & services → Klar NLU → Configure → **Release channel**.

- **Staging** points the integration at `http://klar-nlu-staging:10520` (or downloads the latest GitHub prerelease if the engine is bundled).
- **Stable** points back at `http://klar-nlu:10520` (or the CalVer release).
- A custom URL is left alone.

Install and start this app before switching to Staging. After a merge to `staging`, rebuild (the app version stays `staging`). Do not edit `.storage`.

If you set the app **token**, paste the same value into the integration write token.

## Options

Same as the stable app (`token`, `support_bundle`). See [addon/DOCS.md](../addon/DOCS.md).
