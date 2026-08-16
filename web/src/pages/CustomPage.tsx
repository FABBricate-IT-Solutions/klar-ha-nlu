import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";

export function CustomPage({ t }: { t: Messages }) {
  const [body, setBody] = useState("[]");
  const [status, setStatus] = useState("");
  useEffect(() => {
    api.custom().then((data) => setBody(JSON.stringify(data, null, 2))).catch((err) => setStatus(String(err)));
  }, []);
  const save = async () => {
    await api.saveCustom(JSON.parse(body || "[]"));
    setStatus(t.save);
  };
  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.custom}</h1>
          <p className="muted">{t.customJson}</p>
        </div>
        <button className="primary" onClick={save}>{t.save}</button>
      </section>
      <textarea value={body} onChange={(ev) => setBody(ev.target.value)} style={{ minHeight: 420 }} />
      {status && <p className="muted">{status}</p>}
    </div>
  );
}
