import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { Locale, ParseResult } from "../types";

export function ParsePage({ t, locale, replayText }: { t: Messages; locale: Locale; replayText: string }) {
  const [text, setText] = useState(locale === "en" ? "Turn on the living room light" : "Mach das Licht im Wohnzimmer an");
  const [result, setResult] = useState<ParseResult | null>(null);
  const [raw, setRaw] = useState(false);
  const [conversationId, setConversationId] = useState<string | undefined>();
  useEffect(() => {
    if (replayText) setText(replayText);
  }, [replayText]);

  const submit = async () => {
    const data = await api.parse(text, locale, conversationId);
    setConversationId(data.conversation_id);
    setResult(data);
  };

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.parse}</h1>
          <p className="muted">{t.parseHint}</p>
        </div>
        <button className="primary" onClick={submit}>{t.analyze}</button>
      </section>
      <label>{t.command}</label>
      <textarea
        value={text}
        onChange={(ev) => setText(ev.target.value)}
        onKeyDown={(ev) => {
          if (ev.key === "Enter" && !ev.shiftKey) {
            ev.preventDefault();
            submit();
          }
        }}
      />
      {result && (
        <section className="grid two" style={{ marginTop: 16 }}>
          <div className="card hot">
            <h2>{t.speech}</h2>
            <p>{result.speech || "..."}</p>
            <div className="row">
              {result.clarify && <span className="chip">clarify</span>}
              {result.chat && <span className="chip">chat</span>}
              {result.briefing && <span className="chip">briefing</span>}
            </div>
          </div>
          <div className="card">
            <h2>{t.intent}</h2>
            {result.intents.map((intent) => (
              <div key={intent.name} style={{ marginTop: 12 }}>
                <strong>{intent.name}</strong>
                <div className="row">
                  {intent.slots.map((slot) => <span className="chip" key={`${slot.name}-${slot.value}`}>{slot.name}: {slot.value}</span>)}
                </div>
              </div>
            ))}
            {result.intents.length === 0 && <p className="muted">{t.noIntent}</p>}
          </div>
          <div className="card" style={{ gridColumn: "1 / -1" }}>
            <button className="ghost" onClick={() => setRaw(!raw)}>{t.raw}</button>
            {raw && <pre>{JSON.stringify(result, null, 2)}</pre>}
          </div>
        </section>
      )}
    </div>
  );
}
