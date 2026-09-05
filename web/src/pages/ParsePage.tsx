import { useEffect, useState } from "react";
import { api } from "../api";
import { StageBars } from "../components/charts";
import { Pipeline } from "../components/pipeline";
import { PolicyPath } from "../components/PolicyPath";
import { SearchSelect, withCurrent } from "../components/SearchSelect";
import type { Messages } from "../i18n";
import type { ParseResult, Settings } from "../types";
import { Button } from "@/components/ui/button";

const SKIP_REFINE = new Set(["chat", "llm", "chime", "error", ""]);
const SIMPLE_ON_OFF = new Set(["HassTurnOn", "HassTurnOff"]);
const QUIET_DOMAINS = new Set(["light", "switch"]);
const QUIET_BLOCKED = new Set([
  "scene",
  "script",
  "cover",
  "lock",
  "climate",
  "fan",
  "media_player",
  "vacuum",
]);

function asError(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function bannerText(error: string, result: ParseResult | null): string {
  if (error) return error;
  if (result?.decision.type === "error") return result.decision.message || result.decision.code;
  return "";
}

function on(settings: Settings, key: keyof Settings): boolean {
  return Boolean(settings[key]);
}

function intentNames(result: ParseResult | null): string[] {
  return result?.plan?.steps.map((step) => step.intent.name).filter(Boolean) ?? [];
}

export function armedPipeline(settings: Settings): string[] {
  const chips: string[] = [];
  if (settings.personality && settings.personality !== "default") chips.push(settings.personality);
  if (settings.mode === "context_only") chips.push("context only");
  if (settings.nlu_rag) chips.push("NLU-RAG");
  if (settings.semantic_adapters) chips.push("semantic");
  if (settings.confirm_risky_actions === false) chips.push("no confirm");
  if (on(settings, "refine_speech")) chips.push("LLM refine");
  if (on(settings, "calendar_llm")) chips.push("calendar LLM");
  if (on(settings, "quiet_ack")) chips.push("quiet ack");
  if (on(settings, "allow_llm_tools")) chips.push("LLM tools");
  if (on(settings, "fallback_llm")) chips.push("fallback LLM");
  return chips;
}

export function labDecisionLabel(result: ParseResult | null): string {
  if (!result) return "…";
  const band = result.decision.type;
  const names = intentNames(result);
  const hit = result.policy_trace?.hit || "";
  if (band === "execute") return names.join(" · ") || "Klar execute";
  if (result.briefing) return "briefing";
  if (hit === "llm" || hit === "template" || hit === "script") return hit;
  return band || "…";
}

function firstSlots(result: ParseResult | null): Record<string, string> {
  const first = result?.plan?.steps[0]?.intent;
  if (!first) return {};
  return Object.fromEntries(first.slots.map((slot) => [slot.name, slot.value]));
}

function quietAckLikely(result: ParseResult | null, names: string[]): boolean {
  if (result?.decision.type !== "execute" || names.length !== 1) return false;
  if (!SIMPLE_ON_OFF.has(names[0] || "")) return false;
  const slots = firstSlots(result);
  const domain = slots.domain || "";
  const entity = slots.entity_id || "";
  const prefix = entity.includes(".") ? entity.split(".", 1)[0] : "";
  if (QUIET_BLOCKED.has(domain) || QUIET_BLOCKED.has(prefix)) return false;
  if (QUIET_DOMAINS.has(domain) || QUIET_DOMAINS.has(prefix)) return true;
  if (slots.area || slots.floor) return domain === "" || QUIET_DOMAINS.has(domain);
  return false;
}

export function labPath(
  result: ParseResult | null,
  settings: Settings,
  parseLanguage?: string,
): string[] {
  const parse = parseLanguage ? `Klar parse · ${parseLanguage}` : "Klar parse";
  if (!result) return [parse, "…"];
  const steps = [parse];
  const band = result.decision.type;
  const names = intentNames(result);
  const hit = result.policy_trace?.hit || "";
  const chatLike = Boolean(result.briefing) || hit === "llm" || band === "chat";
  if (settings.nlu_rag && (band === "chat" || band === "reject")) steps.push("NLU-RAG");
  if (settings.semantic_adapters && band === "reject") steps.push("semantic");
  if (settings.mode === "context_only") steps.push("context only");
  steps.push(labDecisionLabel(result));
  const calendar =
    on(settings, "calendar_llm")
    && on(settings, "fallback_llm")
    && band === "execute"
    && names.length > 0
    && names.every((name) => name === "KlarGetCalendarEvents");
  const quiet = on(settings, "quiet_ack") && quietAckLikely(result, names);
  if (calendar) steps.push("calendar LLM");
  if (
    !quiet
    && on(settings, "refine_speech")
    && on(settings, "fallback_llm")
    && !SKIP_REFINE.has(band)
    && !chatLike
  ) {
    steps.push("LLM refine");
  }
  if (quiet) steps.push("quiet ack");
  if (on(settings, "allow_llm_tools") && (hit === "llm" || (band === "chat" && !result.speech))) {
    steps.push("LLM tools");
  }
  if (settings.confirm_risky_actions && band === "confirm") steps.push("confirm risky");
  return steps;
}

export function ParsePage({
  t,
  parseLanguage,
  replayText,
  settings,
  rooms,
}: {
  t: Messages;
  parseLanguage?: string;
  replayText: string;
  settings: Settings;
  rooms: { area_id: string; name: string }[];
}) {
  const [text, setText] = useState(t.parseSample);
  const [result, setResult] = useState<ParseResult | null>(null);
  const [raw, setRaw] = useState(false);
  const [conversationId, setConversationId] = useState<string | undefined>();
  const [area, setArea] = useState("");
  const [teachStatus, setTeachStatus] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [knownIntents, setKnownIntents] = useState<string[]>([]);
  const [teachIntent, setTeachIntent] = useState("KlarGetCalendarEvents");
  const intents = result?.plan?.steps.map((step) => step.intent) ?? [];
  const heardIn = result?.evidence.find((item) => item.kind === "preferred_area")?.value || area;
  const boundEntity = intents.flatMap((intent) => intent.slots).find((slot) => slot.name === "entity_id")?.value || "";
  const roomOptions = rooms.map((room) => ({ value: room.area_id, label: room.name }));
  const intentOptions = (knownIntents.length ? knownIntents : [teachIntent]).map((name) => ({ value: name, label: name }));
  const band = result?.decision.type;
  const armed = armedPipeline(settings);
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
    setBusy(true);
    try {
      const data = await api.parse(text, parseLanguage || "", conversationId, settings.nlu_rag || undefined, area || undefined);
      setConversationId(data.conversation_id);
      setResult(data);
      setTeachStatus("");
      const first = data.plan?.steps[0]?.intent.name;
      if (first) setTeachIntent(first);
    } catch (err) {
      setError(asError(err));
    } finally {
      setBusy(false);
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
        <Button className="primary" type="button" onClick={() => void submit()} disabled={busy}>
          {busy ? t.loading : t.analyze}
        </Button>
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
        <div className="lab-policy-path">
          <PolicyPath t={t} trace={result?.policy_trace} />
        </div>
        {armed.length > 0 && (
          <div className="flow lab-pipeline-armed" aria-label="pipeline">
            {armed.map((chip) => <span className="chip" key={chip}>{chip}</span>)}
          </div>
        )}
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
