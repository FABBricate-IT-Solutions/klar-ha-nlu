import { useEffect, useState } from "react";
import { api, download, setToken, type LanguagePack } from "../api";
import type { Messages } from "../i18n";
import type { BundleList, Settings } from "../types";

export function SettingsPage({ t, settings, onSettings }: { t: Messages; settings: Settings; onSettings: (s: Settings) => void }) {
  const [bundle, setBundle] = useState<BundleList | null>(null);
  const [packs, setPacks] = useState<LanguagePack[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [token, setTokenValue] = useState(localStorage.getItem("klar_token") || "");
  const refresh = () => api.bundle().then(setBundle).catch(() => undefined);
  useEffect(() => {
    refresh();
    api.languages().then(setPacks).catch(() => setPacks([]));
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
  const allCodes = packs.map((pack) => pack.code);
  const enabled = settings.languages.length ? settings.languages : allCodes;
  const toggleLang = (code: string) => {
    const current = settings.languages.length ? settings.languages : allCodes;
    const next = current.includes(code) ? current.filter((item) => item !== code) : [...current, code];
    const languages = next.length === 0 || (allCodes.length > 0 && next.length === allCodes.length) ? [] : next;
    onSettings({ ...settings, languages });
  };
  return (
    <div className="page">
      <section className="hero">
        <div><h1>{t.settings}</h1><p className="muted">{t.journalHint}</p></div>
        <button className="primary" onClick={() => save()}>{t.save}</button>
      </section>
      <section className="grid two">
        <div className="card">
          <p className="caption">{t.personalityHa}</p>
          <label>{t.mode}</label>
          <select value={settings.mode} onChange={(ev) => onSettings({ ...settings, mode: ev.target.value as Settings["mode"] })}>
            <option value="full">full</option>
            <option value="context_only">context_only</option>
          </select>
          <label>{t.languages}</label>
          <div className="row" style={{ flexWrap: "wrap" }}>
            {packs.map((pack) => (
              <label key={pack.code} className="row">
                <input type="checkbox" checked={enabled.includes(pack.code)} onChange={() => toggleLang(pack.code)} style={{ width: "auto" }} />
                {pack.native_name} ({pack.code})
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
