import { useEffect, useState } from "react";
import { api } from "../api";
import { StageBars } from "../components/charts";
import { Pipeline } from "../components/pipeline";
import { SearchSelect, withCurrent } from "../components/SearchSelect";
import type { Messages } from "../i18n";
import type { ParseResult } from "../types";

function asError(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function bannerText(error: string, result: ParseResult | null): string {
  if (error) return error;
  if (result?.decision.type === "error") return result.decision.message || result.decision.code;
  return "";
}

export function ParsePage({
  t,
  parseLanguage,
  replayText,
  nluRag,
  rooms,
}: {
  t: Messages;
  parseLanguage?: string;
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
  const [error, setError] = useState("");
  const [knownIntents, setKnownIntents] = useState<string[]>([]);
  const [teachIntent, setTeachIntent] = useState("KlarGetCalendarEvents");
  const intents = result?.plan?.steps.map((step) => step.intent) ?? [];
  const heardIn = result?.evidence.find((item) => item.kind === "preferred_area")?.value || area;
  const boundEntity = intents.flatMap((intent) => intent.slots).find((slot) => slot.name === "entity_id")?.value || "";
  const roomOptions = rooms.map((room) => ({ value: room.area_id, label: room.name }));
  const intentOptions = (knownIntents.length ? knownIntents : [teachIntent]).map((name) => ({ value: name, label: name }));
  const band = result?.decision.type;
  const banner = bannerText(error, result);

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
    setError("");
    try {
      const data = await api.parse(text, parseLanguage || "", conversationId, nluRag || undefined, area || undefined);
      setConversationId(data.conversation_id);
      setResult(data);
      setTeachStatus("");
      const first = data.plan?.steps[0]?.intent.name;
      if (first) setTeachIntent(first);
    } catch (err) {
      setError(asError(err));
    }
  };

  const savePhrase = async () => {
    const phrase = text.trim();
    if (phrase.length < 4) return;
    try {
      const overlay = await api.langOverlay();
      await api.saveLangOverlay({
        custom: [...overlay.custom, { phrase, intent: teachIntent, slots: {} }],
        language: overlay.language,
        label: phrase,
      });
      setTeachStatus(t.teachSaved);
      setError("");
    } catch (err) {
      setError(asError(err));
    }
  };

  const ignoreTarget = async () => {
    if (!boundEntity) return;
    try {
      await api.tagEntity({ entity_id: boundEntity, nlu_ignore: true });
      setTeachStatus(t.teachSaved);
      setError("");
    } catch (err) {
      setError(asError(err));
    }
  };

  return (
    <div className="page lab" data-band={band || ""}>
      <section className="hero">
        <div>
          <h1>{t.lab}</h1>
          <p className="muted">{t.parseHint}</p>
        </div>
        <button className="primary" type="button" onClick={submit}>{t.analyze}</button>
      </section>
      {banner && <div className="lab-error" role="alert">{banner}</div>}
      <div className="lab-toolbar">
        <div className="lab-satellite">
          <label>{t.heardIn}</label>
          <SearchSelect
            value={area}
            options={withCurrent(roomOptions, area)}
            onChange={setArea}
            emptyLabel={t.anyRoom}
            placeholder={t.anyRoom}
          />
        </div>
        {heardIn ? <span className="chip">{t.heardIn}: {heardIn}</span> : null}
        <div className="flow" aria-label={t.processPath}>
          <span className="chip">HA trigger</span>
          <span className="muted">→</span>
          <span className={`chip${band && band !== "chat" ? " intent" : ""}`}>Klar parse</span>
          <span className="muted">→</span>
          <span className={`chip lab-band-chip${band === "execute" ? " intent" : ""}`}>
            {band === "execute" ? "dispatch / intent_script" : band || "…"}
          </span>
        </div>
        <p className="caption">{t.triggerFirst}</p>
      </div>
      <label htmlFor="lab-command">{t.command}</label>
      <textarea
        id="lab-command"
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
        <section className="lab-out">
          <Pipeline result={result} t={t} />
          {result.trace.stages.length > 0 && (
            <div className="card">
              <h2>{t.latency}</h2>
              <StageBars data={result.trace.stages.map((stage) => ({ label: stage.stage, value: stage.duration_us }))} unit={t.unitsUs} />
              <p className="caption">{t.latencyCaption}</p>
            </div>
          )}
          <section className="grid two">
            <div className="card">
              <h2>{t.speech}</h2>
              <p>{result.speech || "..."}</p>
              <div className="row">
                <span className={`chip lab-band-chip${result.decision.type !== "chat" ? " intent" : ""}`}>{result.decision.type}</span>
                {result.briefing && <span className="chip">briefing</span>}
              </div>
            </div>
            <div className="card">
              <h2>{t.intent}</h2>
              {intents.map((intent, index) => (
                <div key={`${intent.name}-${index}`} className="lab-intent">
                  <strong className="intent-name">{intent.name}</strong>
                  <div className="row">
                    {intent.slots.map((slot) => <span className="slot-chip chip" key={`${slot.name}-${slot.value}`}>{slot.name}: {slot.value}</span>)}
                  </div>
                </div>
              ))}
              {intents.length === 0 && <p className="muted">{t.noIntent}</p>}
              <div className="lab-teach">
                <div>
                  <label>{t.intent}</label>
                  <SearchSelect
                    value={teachIntent}
                    options={withCurrent(intentOptions, teachIntent)}
                    onChange={setTeachIntent}
                    allowEmpty={false}
                    placeholder={t.intent}
                  />
                </div>
                <button className="secondary" type="button" onClick={savePhrase}>{t.savePhrase}</button>
                <button className="ghost" type="button" onClick={ignoreTarget} disabled={!boundEntity}>{t.ignoreTarget}</button>
              </div>
              {teachStatus && <p className="muted">{teachStatus}</p>}
            </div>
          </section>
          <div className="card lab-raw">
            <button className="ghost" type="button" onClick={() => setRaw(!raw)}>{t.raw}</button>
            {raw && <pre>{JSON.stringify(result, null, 2)}</pre>}
          </div>
        </section>
      )}
    </div>
  );
}
