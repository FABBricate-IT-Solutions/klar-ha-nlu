import { useEffect, useState } from "react";
import { api } from "../api";
import { LabSpeechCompare, type LabLlmPreview } from "../components/LabSpeechCompare";
import { StageBars } from "../components/charts";
import { Pipeline } from "../components/pipeline";
import { PolicyPath } from "../components/PolicyPath";
import { SearchSelect, withCurrent } from "../components/SearchSelect";
import type { Messages } from "../i18n";
import { effectiveRefineBands, type ParseResult, type RefineBand, type Settings } from "../types";
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

function llmBanner(err: unknown, t: Messages): string {
  const message = asError(err);
  if (message === "llm-unconfigured" || message.startsWith("503")) return t.llmNotConfigured;
  return message;
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

export function armedPipeline(settings: Settings, t: Messages): string[] {
  const chips: string[] = [];
  if (settings.personality && settings.personality !== "default") chips.push(settings.personality);
  if (settings.mode === "context_only") chips.push(t.labChipContextOnly);
  if (settings.nlu_rag) chips.push(t.labChipNluRag);
  if (settings.semantic_adapters) chips.push(t.labChipSemantic);
  if (settings.confirm_risky_actions === false) chips.push(t.labChipNoConfirm);
  if (on(settings, "refine_speech")) chips.push(t.labChipLlmRefine);
  if (on(settings, "calendar_llm")) chips.push(t.labChipCalendarLlm);
  if (on(settings, "quiet_ack")) chips.push(t.labChipQuietAck);
  if (on(settings, "allow_llm_tools")) chips.push(t.labChipLlmTools);
  return chips;
}

export function labDecisionLabel(result: ParseResult | null, t: Messages): string {
  if (!result) return "…";
  const band = result.decision.type;
  const names = intentNames(result);
  const hit = result.policy_trace?.hit || "";
  if (band === "execute") return names.join(" · ") || t.labDecisionExecute;
  if (result.briefing) return t.labDecisionBriefing;
  if (hit === "llm") return t.effectLlm;
  if (hit === "template") return t.effectTemplate;
  if (hit === "script") return t.effectScript;
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

export function labChatLike(result: ParseResult | null): boolean {
  if (!result) return false;
  const hit = result.policy_trace?.hit || "";
  return Boolean(result.briefing) || hit === "llm" || result.decision.type === "chat";
}

const STATUS_INTENTS = new Set(["HassGetState", "HassClimateGetTemperature", "MassGetQueue", "KlarGetCalendarEvents"]);

export function refineBandLabel(band: RefineBand, t: Messages): string {
  switch (band) {
    case "status":
      return t.refineBandStatus;
    case "command":
      return t.refineBandCommand;
    case "prompt":
      return t.refineBandPrompt;
    case "reject":
      return t.refineBandReject;
    default: {
      const exhaustive: never = band;
      return exhaustive;
    }
  }
}

export function refineBandOf(result: ParseResult | null): RefineBand | null {
  if (!result) return null;
  if (result.refine_band) return result.refine_band;
  switch (result.decision.type) {
    case "execute": {
      const names = intentNames(result);
      return names.length > 0 && names.every((name) => STATUS_INTENTS.has(name)) ? "status" : "command";
    }
    case "clarify":
    case "confirm":
      return "prompt";
    case "reject":
      return "reject";
    case "chat":
    case "error":
      return null;
    default: {
      const exhaustive: never = result.decision;
      return exhaustive;
    }
  }
}

export function labRefineEligible(result: ParseResult | null, settings?: Settings): boolean {
  if (!result || labChatLike(result)) return false;
  const decision = result.decision.type;
  if (!result.speech?.trim() || SKIP_REFINE.has(decision)) return false;
  const band = refineBandOf(result);
  if (!band) return false;
  if (!settings) return true;
  return Boolean(settings.refine_speech) && effectiveRefineBands(settings).includes(band);
}

export function labPath(
  result: ParseResult | null,
  settings: Settings,
  t: Messages,
  parseLanguage?: string,
): string[] {
  const parse = parseLanguage ? `${t.labParse} · ${parseLanguage}` : t.labParse;
  if (!result) return [parse, "…"];
  const steps = [parse];
  const band = result.decision.type;
  const names = intentNames(result);
  const hit = result.policy_trace?.hit || "";
  const chatLike = labChatLike(result);
  if (settings.nlu_rag && (band === "chat" || band === "reject")) steps.push(t.labChipNluRag);
  if (settings.semantic_adapters && band === "reject") steps.push(t.labChipSemantic);
  if (settings.mode === "context_only") steps.push(t.labChipContextOnly);
  steps.push(labDecisionLabel(result, t));
  const calendar =
    on(settings, "calendar_llm")
    && band === "execute"
    && names.length > 0
    && names.every((name) => name === "KlarGetCalendarEvents");
  const quiet = on(settings, "quiet_ack") && quietAckLikely(result, names);
  if (chatLike) steps.push(t.labChipLlmChat);
  if (calendar) steps.push(t.labChipCalendarLlm);
  const refineBand = refineBandOf(result);
  if (
    !quiet
    && on(settings, "refine_speech")
    && !SKIP_REFINE.has(band)
    && !chatLike
    && refineBand
    && effectiveRefineBands(settings).includes(refineBand)
  ) {
    steps.push(t.labChipLlmRefine);
  }
  if (quiet) steps.push(t.labChipQuietAck);
  if (on(settings, "allow_llm_tools") && (hit === "llm" || (band === "chat" && !result.speech))) {
    steps.push(t.labChipLlmTools);
  }
  if (settings.confirm_risky_actions && band === "confirm") steps.push(t.labChipConfirmRisky);
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
  const [llm, setLlm] = useState<LabLlmPreview | null>(null);
  const intents = result?.plan?.steps.map((step) => step.intent) ?? [];
  const heardIn = result?.evidence.find((item) => item.kind === "preferred_area")?.value || area;
  const boundEntity = intents.flatMap((intent) => intent.slots).find((slot) => slot.name === "entity_id")?.value || "";
  const roomOptions = rooms.map((room) => ({ value: room.area_id, label: room.name }));
  const intentOptions = (knownIntents.length ? knownIntents : [teachIntent]).map((name) => ({ value: name, label: name }));
  const band = result?.decision.type;
  const refineBand = refineBandOf(result);
  const armed = armedPipeline(settings, t);
  const path = result ? labPath(result, settings, t, parseLanguage) : [];
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
    setLlm(null);
    try {
      const data = await api.parse(text, parseLanguage || "", conversationId, settings.nlu_rag || undefined, area || undefined);
      setConversationId(data.conversation_id);
      setResult(data);
      setTeachStatus("");
      const first = data.plan?.steps[0]?.intent.name;
      if (first) setTeachIntent(first);
      const preview: LabLlmPreview = { nlu: data.speech, busy: true };
      setLlm(preview);
      const language = parseLanguage || "de";
      const personality = settings.personality || "default";
      const extra = settings.extra_prompt || "";
      if (labChatLike(data)) {
        try {
          const chat = await api.llmAssist({
            text,
            language,
            personality,
            extra_prompt: extra,
            conversation_id: data.conversation_id,
          });
          setLlm({ ...preview, busy: false, chat: chat.text });
        } catch (err) {
          setLlm({ ...preview, busy: false, error: llmBanner(err, t) });
        }
      } else if (labRefineEligible(data, settings)) {
        try {
          const refined = await api.llmRefine({
            speech: data.speech,
            language,
            personality,
            extra_prompt: extra,
            conversation_id: data.conversation_id,
          });
          setLlm({ ...preview, busy: false, refined: refined.text, accepted: refined.accepted });
        } catch (err) {
          setLlm({ ...preview, busy: false, error: llmBanner(err, t) });
        }
      } else {
        setLlm({ ...preview, busy: false });
      }
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
          <p className="muted">{t.labGuide}</p>
          <p className="caption">{t.parseHint}</p>
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
          <div className="flow lab-pipeline-armed" aria-label={t.labPipeline}>
            {armed.map((chip) => <span className="chip" key={chip}>{chip}</span>)}
          </div>
        )}
        {result && (
          <div className="flow lab-pipeline-path" aria-label={t.labThisTurn}>
            {path.map((chip, index) => (
              <span className="chip" key={`${chip}-${index}`}>{chip}</span>
            ))}
            {refineBand ? <span className="chip">{refineBandLabel(refineBand, t)}</span> : null}
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
            <LabSpeechCompare
              t={t}
              band={result.decision.type}
              briefing={Boolean(result.briefing)}
              preview={llm || { nlu: result.speech, busy: false }}
            />
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
