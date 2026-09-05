import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { LlmPublic } from "../types";

const OPENAI = "https://api.openai.com/v1";
const OLLAMA = "http://127.0.0.1:11434/v1";

export function LlmSettingsCard({ t }: { t: Messages }) {
  const [endpoint, setEndpoint] = useState<LlmPublic>({ configured: false });
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [status, setStatus] = useState("");

  const load = async () => {
    const next = await api.llmEndpoint();
    setEndpoint(next);
    setBaseUrl(next.base_url || "");
    setModel(next.model || "");
    setApiKey("");
  };

  useEffect(() => {
    load().catch(() => setStatus(t.trainerFail));
  }, [t.trainerFail]);

  const save = async () => {
    try {
      const next = await api.saveLlmEndpoint({
        base_url: baseUrl,
        model,
        ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
        configured: true,
      });
      setEndpoint(next);
      setApiKey("");
      setStatus(t.save);
    } catch {
      setStatus(t.trainerFail);
    }
  };

  const clear = async () => {
    try {
      const next = await api.saveLlmEndpoint({ configured: false });
      setEndpoint(next);
      setBaseUrl("");
      setModel("");
      setApiKey("");
      setStatus(t.llmClear);
    } catch {
      setStatus(t.trainerFail);
    }
  };

  return (
    <div className="card">
      <h2>{t.llm}</h2>
      <p className="muted">{t.llmHint}</p>
      <p className="caption">{endpoint.configured ? `${t.llmConfigured} · ${endpoint.model} · ${endpoint.base_url}` : t.llmNotConfigured}</p>
      <div className="row">
        <button type="button" className="secondary" onClick={() => setBaseUrl(OPENAI)}>{t.llmPresetOpenAi}</button>
        <button type="button" className="secondary" onClick={() => setBaseUrl(OLLAMA)}>{t.llmPresetOllama}</button>
      </div>
      <label>{t.llmBaseUrl}</label>
      <input value={baseUrl} onChange={(ev) => setBaseUrl(ev.target.value)} placeholder={OPENAI} />
      <label>{t.llmModel}</label>
      <input value={model} onChange={(ev) => setModel(ev.target.value)} placeholder="gpt-4o-mini" />
      <label>{t.llmApiKey}</label>
      <input type="password" value={apiKey} onChange={(ev) => setApiKey(ev.target.value)} placeholder={t.llmApiKeyHint} autoComplete="off" />
      <div className="row" style={{ marginTop: 12 }}>
        <button type="button" className="primary" onClick={() => void save()}>{t.save}</button>
        <button type="button" className="ghost" disabled={!endpoint.configured} onClick={() => void clear()}>{t.llmClear}</button>
      </div>
      {status ? <p className="muted">{status}</p> : null}
    </div>
  );
}
