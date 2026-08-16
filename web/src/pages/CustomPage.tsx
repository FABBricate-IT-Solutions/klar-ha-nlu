import { useEffect, useState } from "react";
import { api, type CustomRule, type LangExplain, type LangOverlay } from "../api";
import type { Messages } from "../i18n";
import type { Locale } from "../types";

const intents = ["HassTurnOn", "HassTurnOff", "HassToggle", "HassLightSet", "HassGetState", "HassClimateSetTemperature"];

export function CustomPage({ t, locale }: { t: Messages; locale: Locale }) {
  const [rules, setRules] = useState<CustomRule[]>([]);
  const [language, setLanguage] = useState<unknown>({});
  const [history, setHistory] = useState<LangOverlay["history"]>([]);
  const [phrase, setPhrase] = useState("");
  const [intent, setIntent] = useState(intents[0]);
  const [entityId, setEntityId] = useState("");
  const [previewText, setPreviewText] = useState("");
  const [explain, setExplain] = useState<LangExplain | null>(null);
  const [jsonMode, setJsonMode] = useState(false);
  const [jsonBody, setJsonBody] = useState("[]");
  const [status, setStatus] = useState("");

  const load = (overlay: LangOverlay) => {
    setRules(overlay.custom);
    setLanguage(overlay.language);
    setHistory(overlay.history);
    setJsonBody(JSON.stringify(overlay.custom, null, 2));
  };

  useEffect(() => {
    api.langOverlay().then(load).catch((err) => setStatus(String(err)));
  }, []);

  const persist = async (next: CustomRule[], label: string) => {
    load(await api.saveLangOverlay({ custom: next, language, label }));
    setStatus(t.save);
  };

  const add = async () => {
    const next = [...rules, { phrase: phrase.trim(), intent, slots: entityId.trim() ? { entity_id: entityId.trim() } : {} }];
    setPhrase("");
    setEntityId("");
    await persist(next, phrase.trim() || "add");
  };

  const remove = async (index: number) => {
    await persist(rules.filter((_, row) => row !== index), "remove");
  };

  const saveJson = async () => {
    await persist(JSON.parse(jsonBody || "[]"), "json");
  };

  const runPreview = async () => {
    const text = previewText.trim() || phrase.trim();
    const out = await api.explainLang({ text, language: locale, custom: rules });
    setExplain(out);
  };

  const rollback = async (hash?: string) => {
    load(await api.rollbackLang(hash));
    setStatus(t.rollback);
  };

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.custom}</h1>
          <p className="muted">{t.customHint}</p>
        </div>
        <div className="row">
          <button className="ghost" onClick={() => setJsonMode(!jsonMode)}>{t.advancedJson}</button>
          <button className="primary" onClick={() => jsonMode ? saveJson() : add()}>{jsonMode ? t.save : t.addPhrase}</button>
        </div>
      </section>
      {jsonMode ? (
        <textarea value={jsonBody} onChange={(ev) => setJsonBody(ev.target.value)} style={{ minHeight: 320 }} />
      ) : (
        <section className="grid two">
          <div className="card">
            <label>{t.command}</label>
            <input value={phrase} onChange={(ev) => setPhrase(ev.target.value)} />
            <label>{t.intent}</label>
            <select value={intent} onChange={(ev) => setIntent(ev.target.value)}>
              {intents.map((name) => <option key={name} value={name}>{name}</option>)}
            </select>
            <label>{t.entityId}</label>
            <input value={entityId} onChange={(ev) => setEntityId(ev.target.value)} placeholder="light.wohnzimmer" />
          </div>
          <div className="card">
            <label>{t.previewRule}</label>
            <input value={previewText} onChange={(ev) => setPreviewText(ev.target.value)} placeholder={phrase || t.command} />
            <div className="row" style={{ marginTop: 12 }}>
              <button className="secondary" onClick={runPreview}>{t.explainRule}</button>
              <button className="ghost" onClick={() => rollback()}>{t.rollback}</button>
            </div>
            {explain && (
              <p className="muted" style={{ marginTop: 12 }}>
                {explain.decision} · {explain.confidence.toFixed(2)} · {explain.speech}
              </p>
            )}
          </div>
        </section>
      )}
      <section className="card" style={{ marginTop: 16 }}>
        {rules.length === 0 && <p className="muted">{t.noRules}</p>}
        {rules.map((rule, index) => (
          <div className="row" key={`${rule.phrase}-${index}`} style={{ marginTop: 8 }}>
            <strong>{rule.phrase}</strong>
            <span className="chip">{rule.intent}</span>
            {rule.slots.entity_id && <span className="chip">{rule.slots.entity_id}</span>}
            <button className="ghost danger" onClick={() => remove(index)}>{t.deleteSelected}</button>
          </div>
        ))}
      </section>
      {history.length > 0 && (
        <section className="card" style={{ marginTop: 16 }}>
          <h2>{t.rollback}</h2>
          {history.slice(0, 6).map((row) => (
            <div className="row" key={row.hash} style={{ marginTop: 8 }}>
              <span className="muted">{row.label}</span>
              <button className="ghost" onClick={() => rollback(row.hash)}>{row.hash.slice(0, 8)}</button>
            </div>
          ))}
        </section>
      )}
      {status && <p className="muted">{status}</p>}
    </div>
  );
}
