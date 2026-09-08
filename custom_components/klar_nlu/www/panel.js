class KlarNluPanel extends HTMLElement {
  constructor() {
    super();
    this._mounted = false;
  }

  set hass(hass) {
    this._hass = hass;
    this._mount();
  }

  get hass() {
    return this._hass;
  }

  set panel(panel) {
    this._panel = panel;
  }

  async _mount() {
    if (this._mounted || !this._hass) {
      return;
    }
    this._mounted = true;
    this.style.display = "block";
    this.style.width = "100%";
    this.style.height = "100%";
    const iframe = document.createElement("iframe");
    iframe.title = "Klar NLU";
    iframe.style.cssText =
      "border:0;width:100%;height:100%;display:block;background:var(--primary-background-color,#100e0c)";
    this.appendChild(iframe);
    iframe.src = await this._uiSrc(this._hass);
  }

  async _uiSrc(hass) {
    const token = hass.auth && hass.auth.data && hass.auth.data.access_token;
    if (token) {
      try {
        const res = await fetch("/api/klar_nlu/session", {
          method: "POST",
          credentials: "same-origin",
          headers: { Authorization: "Bearer " + token },
        });
        if (res.ok) {
          return "/api/klar_nlu/ui/";
        }
      } catch (_err) {
        /* try a signed path next */
      }
    }
    if (hass.callWS) {
      try {
        const signed = await hass.callWS({
          type: "auth/sign_path",
          path: "/api/klar_nlu/ui/",
        });
        if (signed && signed.path) {
          return signed.path;
        }
      } catch (_err) {
        /* last resort below */
      }
    }
    return "/api/klar_nlu/ui/";
  }
}

customElements.define("klar-nlu-panel", KlarNluPanel);
