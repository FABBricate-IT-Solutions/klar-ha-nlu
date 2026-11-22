# Klar NLU (Staging)

Release-candidate **App**. Assist still uses the integration; this App only hosts the engine. Not production — use **Klar NLU** for the last CalVer cut.

## Install

Settings → Apps → App store → ⋮ → Repositories, add:

```
https://github.com/FABBricate-IT-Solutions/klar-ha-nlu#staging-store
```

Every merge to `staging` publishes that branch with the new RC version (`{CalVer}-staging.{sha7}`), so Home Assistant offers it as a normal app update. With auto-update on, it installs by itself.

The copy of this App in the main repository keeps the fixed version `staging` and never updates. If you installed that one, uninstall it and install from the URL above. Copy the **token** first and paste it back after the install. The integration finds the new hostname by itself.

## Switch

Settings → Devices & services → Klar NLU → Configure → **Release channel**.

- **Staging** points the integration at `http://klar-nlu-staging:10520` (or downloads the latest GitHub prerelease if the engine is bundled).
- **Stable** points back at `http://klar-nlu:10520` (or the CalVer release).
- A custom URL is left alone.

Install and start this app before switching to Staging. Do not edit `.storage`.

If you set the app **token**, paste the same value into the integration write token.

## Options

Same as the stable app (`token`, `support_bundle`). See [addon/DOCS.md](../addon/DOCS.md).
