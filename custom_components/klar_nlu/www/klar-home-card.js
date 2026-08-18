class KlarHomeCard extends HTMLElement {
  setConfig(config) {
    this.config = config || {};
  }

  set hass(hass) {
    this._hass = hass;
    if (!this.shadowRoot) {
      this.attachShadow({ mode: "open" });
    }
    const heard = this._state("last_heard");
    const decision = this._state("last_decision");
    const speech = this._state("last_speech");
    const area = this._state("last_area");
    this.shadowRoot.innerHTML = `
      <style>
        ha-card { --ha-card-border-radius: 0; }
        .body { padding: 16px; font-family: "IBM Plex Sans", var(--ha-font-family-body); }
        h2 { margin: 0 0 8px; font-size: 18px; }
        p { margin: 0 0 8px; }
        .muted { color: var(--secondary-text-color); }
        button { min-height: 44px; padding: 0 16px; border: 0; background: #c45c26; color: #fff; font: inherit; }
        button:focus { outline: 2px solid currentColor; outline-offset: 2px; }
      </style>
      <ha-card>
        <div class="body">
          <h2>Klar</h2>
          <p>${this._esc(heard?.state || "—")}</p>
          <p class="muted">${this._esc(speech?.state || "")}</p>
          <p class="muted">${this._esc(decision?.state || "")}${area?.state ? ` · ${this._esc(area.state)}` : ""}</p>
          <button type="button">${this._label()}</button>
        </div>
      </ha-card>
    `;
    this.shadowRoot.querySelector("button")?.addEventListener("click", () => {
      hass.callService("klar_nlu", "undo", {});
    });
  }

  getCardSize() {
    return 3;
  }

  _label() {
    const lang = (this._hass?.language || "").startsWith("de") ? "de" : "en";
    return lang === "de" ? "Rückgängig" : "Undo";
  }

  _state(kind) {
    return Object.values(this._hass?.states || {}).find((item) => item.attributes?.klar_kind === kind);
  }

  _esc(value) {
    return String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }
}

customElements.define("klar-home-card", KlarHomeCard);
window.customCards = window.customCards || [];
window.customCards.push({
  type: "klar-home-card",
  name: "Klar home",
  description: "Last Assist turn, room, and undo",
});
