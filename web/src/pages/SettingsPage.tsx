import { useEffect, useState } from "react";
import { api, download, setToken } from "../api";
import { Drawer } from "../components/common";
import type { Messages } from "../i18n";
import type { BundleList, Settings, Theme } from "../types";

function readTheme(theme?: Theme): Theme {
  if (theme === "light" || theme === "dark") return theme;
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

export function SettingsPage({
  t,
  settings,
  onSettings,
  onReplayWizard,
  theme,
  onTheme,
}: {
  t: Messages;
  settings: Settings;
  onSettings: (s: Settings) => void;
  onReplayWizard?: () => void;
  theme?: Theme;
  onTheme?: (theme: Theme) => void;
}) {
  const de = document.documentElement.lang.startsWith("de");
  const [bundle, setBundle] = useState<BundleList | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);
  const [token, setTokenValue] = useState(localStorage.getItem("klar_token") || "");
  const [picked, setPicked] = useState<Theme>(() => readTheme(theme));
  const refresh = () => api.bundle().then(setBundle).catch(() => undefined);
  useEffect(() => {
    refresh();
  }, []);
  useEffect(() => {
    if (theme === "light" || theme === "dark") setPicked(theme);
  }, [theme]);
  const save = async (next = settings) => {
    setToken(token);
    onSettings(await api.saveSettings(next));
    refresh();
  };
  const clear = async () => {
    await api.clearBundle();
    refresh();
    setConfirmClear(false);
  };
  const setTheme = (next: Theme) => {
    setPicked(next);
    document.documentElement.dataset.theme = next;
    onTheme?.(next);
  };
  return (
    <div className="page">
      <section className="hero">
        <div><h1>{t.settings}</h1><p className="muted">{t.engineHint}</p></div>
        <div className="row">
          <button className="ghost" onClick={() => onReplayWizard?.()}>
            {de ? "Setup erneut" : "Replay setup"}
          </button>
          <button className="primary" onClick={() => save()}>{t.save}</button>
        </div>
      </section>
      <section className="grid two">
        <div className="card">
          <p className="caption">{t.personalityHa}</p>
          <label>{de ? "Darstellung" : "Appearance"}</label>
          <div className="row" role="group" aria-label={de ? "Darstellung" : "Appearance"}>
            <button type="button" className={picked === "dark" ? "primary" : "secondary"} aria-pressed={picked === "dark"} onClick={() => setTheme("dark")}>
              {de ? "Dunkel" : "Dark"}
            </button>
            <button type="button" className={picked === "light" ? "primary" : "secondary"} aria-pressed={picked === "light"} onClick={() => setTheme("light")}>
              {de ? "Hell" : "Light"}
            </button>
          </div>
          <label>{t.mode}</label>
          <select value={settings.mode} onChange={(ev) => onSettings({ ...settings, mode: ev.target.value as Settings["mode"] })}>
            <option value="full">{t.modeFull}</option>
            <option value="context_only">{t.modeContext}</option>
          </select>
          <label>{t.languages}</label>
          <p>{settings.languages.length ? settings.languages.join(" · ") : t.allLanguages}</p>
          <p className="caption">{t.languageHint}</p>
          <label className="row">
            <input type="checkbox" checked={settings.confirm_risky_actions} onChange={(ev) => onSettings({ ...settings, confirm_risky_actions: ev.target.checked })} style={{ width: "auto" }} />
            {t.confirmRisky}
          </label>
          <label className="row">
            <input type="checkbox" checked={settings.nlu_rag} onChange={(ev) => onSettings({ ...settings, nlu_rag: ev.target.checked })} style={{ width: "auto" }} />
            {t.nluRag}
          </label>
          <p className="caption">{t.nluRagHint}</p>
          <label>{t.token}</label>
          <input type="password" value={token} onChange={(ev) => setTokenValue(ev.target.value)} />
        </div>
        <div className="card">
          <h2>{t.supportBundle}</h2>
          <label className="row">
            <input type="checkbox" checked={settings.support_bundle} onChange={(ev) => save({ ...settings, support_bundle: ev.target.checked })} style={{ width: "auto" }} />
            {t.recordProtocol}
          </label>
          <label className="row">
            <input type="checkbox" checked={settings.support_bundle_raw_text} onChange={(ev) => save({ ...settings, support_bundle_raw_text: ev.target.checked })} style={{ width: "auto" }} />
            {t.includeRawText}
          </label>
          <label className="row">
            <input type="checkbox" checked={settings.semantic_adapters} onChange={(ev) => save({ ...settings, semantic_adapters: ev.target.checked })} style={{ width: "auto" }} />
            {t.semanticAdapters}
          </label>
          <h2 style={{ marginTop: 20 }}>{t.journal}</h2>
          <p className="muted">{t.journalHint}</p>
          <p className="muted">{bundle ? `${bundle.count} ${t.recordings}` : "..."}</p>
          <div className="row">
            <button className="secondary" onClick={() => download("/api/bundle/dataset", "klar-assist-dataset.yaml")}>{t.downloadDataset}</button>
            <button className="secondary" onClick={() => download("/api/bundle/protocol", "klar-support-bundle.jsonl")}>{t.downloadProtocol}</button>
            <button className="ghost danger" onClick={() => setConfirmClear(true)}>{t.clearAll}</button>
          </div>
        </div>
      </section>
      {confirmClear && (
        <Drawer title={t.clearAll} onClose={() => setConfirmClear(false)} closeLabel={t.close}>
          <div className="row">
            <button className="primary" onClick={clear}>{t.clearAll}</button>
            <button className="secondary" onClick={() => setConfirmClear(false)}>{t.cancel}</button>
          </div>
        </Drawer>
      )}
    </div>
  );
}
