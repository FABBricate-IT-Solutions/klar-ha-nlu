import { useEffect, useState } from "react";
import { api, download, setToken } from "../api";
import type { Messages } from "../i18n";
import type { BundleList, Settings } from "../types";

const personalities = ["default", "butler", "locker", "fuersorglich", "party", "grantig", "sarkastisch", "pirat", "hippie", "gollum"];
const packs = ["de", "en"];

export function SettingsPage({ t, settings, onSettings }: { t: Messages; settings: Settings; onSettings: (s: Settings) => void }) {
  const [bundle, setBundle] = useState<BundleList | null>(null);
  const [selected, setSelected] = useState<string[]>([]);
  const [token, setTokenValue] = useState(localStorage.getItem("klar_token") || "");
  const refresh = () => api.bundle().then(setBundle).catch(() => undefined);
  useEffect(() => {
    refresh();
  }, []);
  const save = async (next = settings) => {
    setToken(token);
    onSettings(await api.saveSettings(next));
    refresh();
  };
  const remove = async (ids: string[]) => {
    setBundle(await api.deleteBundle(ids));
    setSelected([]);
  };
  const clear = async () => {
    await api.clearBundle();
    refresh();
  };
  const toggleLang = (code: string) => {
    const languages = settings.languages.includes(code)
      ? settings.languages.filter((item) => item !== code)
      : [...settings.languages, code];
    onSettings({ ...settings, languages: languages.length ? languages : ["de"] });
  };
  return (
    <div className="page">
      <section className="hero">
        <div><h1>{t.settings}</h1><p className="muted">{t.journalHint}</p></div>
        <button className="primary" onClick={() => save()}>{t.save}</button>
      </section>
      <section className="grid two">
        <div className="card">
          <label>{t.personality}</label>
          <select value={settings.personality} onChange={(ev) => onSettings({ ...settings, personality: ev.target.value })}>
            {personalities.map((p) => <option value={p} key={p}>{p}</option>)}
          </select>
          <label>{t.mode}</label>
          <select value={settings.mode} onChange={(ev) => onSettings({ ...settings, mode: ev.target.value as Settings["mode"] })}>
            <option value="full">full</option>
            <option value="context_only">context_only</option>
          </select>
          <label>{t.languages}</label>
          <div className="row">
            {packs.map((code) => (
              <label key={code} className="row">
                <input type="checkbox" checked={settings.languages.includes(code)} onChange={() => toggleLang(code)} style={{ width: "auto" }} />
                {code}
              </label>
            ))}
          </div>
          <label className="row">
            <input type="checkbox" checked={settings.confirm_risky_actions} onChange={(ev) => onSettings({ ...settings, confirm_risky_actions: ev.target.checked })} style={{ width: "auto" }} />
            {t.confirmRisky}
          </label>
          <label className="row">
            <input type="checkbox" checked={settings.nlu_rag} onChange={(ev) => onSettings({ ...settings, nlu_rag: ev.target.checked })} style={{ width: "auto" }} />
            {settings.nlu_rag ? t.ragMode : t.chatMode} · {t.nluRag}
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
            <button className="ghost danger" onClick={() => selected.length && remove(selected)}>{t.deleteSelected}</button>
            <button className="ghost danger" onClick={clear}>{t.clearAll}</button>
          </div>
        </div>
      </section>
    </div>
  );
}
