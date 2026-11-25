class KlarHomeCard extends HTMLElement {
  setConfig(config) {
    this.config = config || {};
  }

  set hass(hass) {
    this._hass = hass;
    if (!this.shadowRoot) {
      this.attachShadow({ mode: "open" });
      this._build();
    }
    this._sync();
  }

  getCardSize() {
    return 3;
  }

  _build() {
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
          <p data-field="heard">—</p>
          <p class="muted" data-field="speech"></p>
          <p class="muted" data-field="meta"></p>
          <button type="button" data-field="undo"></button>
        </div>
      </ha-card>
    `;
    this.shadowRoot.querySelector("[data-field=undo]")?.addEventListener("click", () => {
      this._hass?.callService("klar_nlu", "undo", {});
    });
    this._nodes = {
      heard: this.shadowRoot.querySelector("[data-field=heard]"),
      speech: this.shadowRoot.querySelector("[data-field=speech]"),
      meta: this.shadowRoot.querySelector("[data-field=meta]"),
      undo: this.shadowRoot.querySelector("[data-field=undo]"),
    };
    this._last = {};
  }

  _sync() {
    const heard = this._state("last_heard");
    const decision = this._state("last_decision");
    const speech = this._state("last_speech");
    const area = this._state("last_area");
    const next = {
      heard: heard?.state || "—",
      speech: speech?.state || "",
      meta: `${decision?.state || ""}${area?.state ? ` · ${area.state}` : ""}`,
      undo: this._label(),
    };
    for (const key of Object.keys(next)) {
      if (this._last[key] === next[key]) continue;
      this._last[key] = next[key];
      const node = this._nodes[key];
      if (node) node.textContent = next[key];
    }
  }

  _label() {
    const lang = (this._hass?.language || "").startsWith("de") ? "de" : "en";
    return lang === "de" ? "Rückgängig" : "Undo";
  }

  _state(kind) {
    return Object.values(this._hass?.states || {}).find((item) => item.attributes?.klar_kind === kind);
  }
}

customElements.define("klar-home-card", KlarHomeCard);
window.customCards = window.customCards || [];
window.customCards.push({
  type: "klar-home-card",
  name: "Klar home",
  description: "Last Assist turn, room, and undo",
});
