import { useEffect, useState } from "react";
import { api } from "../api";
import { StageBars } from "../components/charts";
import { Pipeline } from "../components/pipeline";
import type { Messages } from "../i18n";
import type { Locale, ParseResult } from "../types";

export function ParsePage({
  t,
  locale,
  replayText,
  nluRag,
  rooms,
}: {
  t: Messages;
  locale: Locale;
  replayText: string;
  nluRag: boolean;
  rooms: { area_id: string; name: string }[];
}) {
  const [text, setText] = useState(t.parseSample);
  const [result, setResult] = useState<ParseResult | null>(null);
  const [raw, setRaw] = useState(false);
  const [conversationId, setConversationId] = useState<string | undefined>();
  const [area, setArea] = useState("");
  const [teachStatus, setTeachStatus] = useState("");
  const [knownIntents, setKnownIntents] = useState<string[]>([]);
  const [teachIntent, setTeachIntent] = useState("KlarGetCalendarEvents");
  const intents = result?.plan?.steps.map((step) => step.intent) ?? [];
  const heardIn = result?.evidence.find((item) => item.kind === "preferred_area")?.value || area;
  const boundEntity = intents.flatMap((intent) => intent.slots).find((slot) => slot.name === "entity_id")?.value || "";
  useEffect(() => {
    if (replayText) setText(replayText);
  }, [replayText]);
  useEffect(() => {
    if (!replayText) setText(t.parseSample);
  }, [t.parseSample, replayText]);
  useEffect(() => {
    api.intents().then((names) => { if (names.length) setKnownIntents(names); }).catch(() => undefined);
  }, []);

  const submit = async () => {
    const data = await api.parse(text, locale, conversationId, nluRag || undefined, area || undefined);
    setConversationId(data.conversation_id);
    setResult(data);
    setTeachStatus("");
    const first = data.plan?.steps[0]?.intent.name;
    if (first) setTeachIntent(first);
  };

  const savePhrase = async () => {
    const phrase = text.trim();
    if (phrase.length < 4) return;
    const overlay = await api.langOverlay();
    await api.saveLangOverlay({
      custom: [...overlay.custom, { phrase, intent: teachIntent, slots: {} }],
      language: overlay.language,
      label: phrase,
    });
    setTeachStatus(t.teachSaved);
  };

  const ignoreTarget = async () => {
    if (!boundEntity) return;
    await api.tagEntity({ entity_id: boundEntity, nlu_ignore: true });
    setTeachStatus(t.teachSaved);
  };

  const band = result?.decision.type;
  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.lab}</h1>
          <p className="muted">{t.parseHint}</p>
        </div>
        <button className="primary" onClick={submit}>{t.analyze}</button>
      </section>
      <div className="card" style={{ marginBottom: 16 }}>
        <h3>{t.processPath}</h3>
        <div className="flow" style={{ marginTop: 8 }}>
          <span className="chip">HA trigger</span>
          <span className="muted">→</span>
          <span className={`chip${band && band !== "chat" ? " intent" : ""}`}>Klar parse</span>
          <span className="muted">→</span>
          <span className="chip intent">{band === "execute" ? "dispatch / intent_script" : band || "…"}</span>
        </div>
        <p className="caption">{t.triggerFirst}</p>
      </div>
      <label>{t.heardIn}</label>
      <select value={area} onChange={(ev) => setArea(ev.target.value)}>
        <option value="">{t.anyRoom}</option>
        {rooms.map((room) => <option key={room.area_id} value={room.area_id}>{room.name}</option>)}
      </select>
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
        <section style={{ marginTop: 16 }}>
          <Pipeline result={result} t={t} />
          {result.trace.stages.length > 0 && (
            <div className="card" style={{ marginTop: 16 }}>
              <h2>{t.latency}</h2>
              <StageBars data={result.trace.stages.map((stage) => ({ label: stage.stage, value: stage.duration_us }))} unit={t.unitsUs} />
              <p className="caption">{t.latencyCaption}</p>
            </div>
          )}
          <section className="grid two" style={{ marginTop: 16 }}>
            <div className="card">
              <h2>{t.speech}</h2>
              <p>{result.speech || "..."}</p>
              <div className="row">
                <span className="chip intent">{result.decision.type}</span>
                {result.briefing && <span className="chip">briefing</span>}
                {heardIn && <span className="chip">{t.heardIn}: {heardIn}</span>}
              </div>
            </div>
            <div className="card">
              <h2>{t.intent}</h2>
              {intents.map((intent, index) => (
                <div key={`${intent.name}-${index}`} style={{ marginTop: 12 }}>
                  <strong className="intent-name">{intent.name}</strong>
                  <div className="row">
                    {intent.slots.map((slot) => <span className="slot-chip chip" key={`${slot.name}-${slot.value}`}>{slot.name}: {slot.value}</span>)}
                  </div>
                </div>
              ))}
              {intents.length === 0 && <p className="muted">{t.noIntent}</p>}
              <label style={{ marginTop: 16 }}>{t.intent}</label>
              <select value={teachIntent} onChange={(ev) => setTeachIntent(ev.target.value)}>
                {(knownIntents.length ? knownIntents : [teachIntent]).map((name) => <option key={name} value={name}>{name}</option>)}
              </select>
              <div className="row" style={{ marginTop: 12 }}>
                <button className="secondary" onClick={savePhrase}>{t.savePhrase}</button>
                <button className="ghost" onClick={ignoreTarget} disabled={!boundEntity}>{t.ignoreTarget}</button>
              </div>
              {teachStatus && <p className="muted">{teachStatus}</p>}
            </div>
            <div className="card" style={{ gridColumn: "1 / -1" }}>
              <button className="ghost" onClick={() => setRaw(!raw)}>{t.raw}</button>
              {raw && <pre>{JSON.stringify(result, null, 2)}</pre>}
            </div>
          </section>
        </section>
      )}
    </div>
  );
}
