import { useEffect, useState } from "react";
import { api, download, setToken } from "../api";
import type { Messages } from "../i18n";
import type { BundleList, Settings } from "../types";

const personalities = ["default", "butler", "locker", "fuersorglich", "party", "grantig", "sarkastisch", "pirat", "hippie", "gollum"];

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
  return (
    <div className="page">
      <section className="hero">
        <div><h1>{t.settings}</h1><p className="muted">/data/klar_nlu.json · /data/support_bundle.jsonl</p></div>
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
          <label>{t.token}</label>
          <input type="password" value={token} onChange={(ev) => setTokenValue(ev.target.value)} />
        </div>
        <div className="card">
          <h2>{t.supportBundle}</h2>
          <label className="row">
            <input type="checkbox" checked={settings.support_bundle} onChange={(ev) => save({ ...settings, support_bundle: ev.target.checked })} style={{ width: "auto" }} />
            {t.recordProtocol}
          </label>
          <p className="muted">{bundle ? `${bundle.count} ${t.recordings}` : "..."}</p>
          <div className="row">
            <button className="secondary" onClick={() => download("/api/bundle/dataset", "klar-assist-dataset.yaml")}>{t.downloadDataset}</button>
            <button className="secondary" onClick={() => download("/api/bundle/protocol", "klar-support-bundle.jsonl")}>{t.downloadProtocol}</button>
            <button className="ghost danger" onClick={() => selected.length && remove(selected)}>{t.deleteSelected}</button>
            <button className="ghost danger" onClick={clear}>{t.clearAll}</button>
          </div>
        </div>
      </section>
      <section className="card" style={{ marginTop: 16 }}>
        <h2>{t.recordings}</h2>
        {!bundle?.entries.length && <p className="muted">{t.emptyBundle}</p>}
        <table>
          <tbody>
            {bundle?.entries.map((row) => (
              <tr key={row.id}>
                <td><input type="checkbox" checked={selected.includes(row.id)} onChange={(ev) => setSelected(ev.target.checked ? [...selected, row.id] : selected.filter((id) => id !== row.id))} /></td>
                <td>{new Date(row.ts_ms).toLocaleString()}<div className="mono">{row.source}</div></td>
                <td>{row.text}</td>
                <td>{row.speech}<div className="mono">{row.intents.join(", ")}</div></td>
                <td><button className="ghost danger" onClick={() => remove([row.id])}>{t.dismiss}</button></td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </div>
  );
}
