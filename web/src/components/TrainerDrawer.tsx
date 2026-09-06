import { useEffect, useRef, useState } from "react";
import { Loader2Icon } from "lucide-react";
import { api } from "../api";
import type { PolicyLane } from "./PolicyPath";
import type { Messages } from "../i18n";
import type { LlmPublic, TrainerChatEvent, TrainerConsent, TrainerTurn, TrainerValidateOut } from "../types";
import { LotseAnswer, lotseFallbackChips, lotseQuickChips, lotseReplyChoices, unansweredAssistant, visibleLotseText } from "./LotseAnswer";
import { TrainerToolCard } from "./TrainerToolCard";

type ThreadLine =
  | { role: "user" | "assistant"; content: string }
  | { role: "tool"; name: string; args: string; result?: string };

function trainerLayer(lane: PolicyLane): "match" | "language" | "house" {
  switch (lane) {
    case "match":
    case "language":
    case "house":
      return lane;
    default: {
      const _never: never = lane;
      return _never;
    }
  }
}

function shortModel(model?: string): string {
  if (!model) return "LLM";
  if (/gemma-4-26b/i.test(model) && /mtp/i.test(model)) return "Gemma 26B MTP";
  return model.replace(/-GGUF$/i, "");
}

function lanePrompts(t: Messages, lane: PolicyLane): [string, string] {
  switch (lane) {
    case "match":
      return [t.trainerPromptMatchers, t.trainerPromptPrecedence];
    case "language":
      return [t.trainerPromptLexicon, t.trainerPromptSlang];
    case "house":
      return [t.trainerPromptGaps, t.trainerPromptNight];
    default: {
      const _never: never = lane;
      return _never;
    }
  }
}

function chatHistory(lines: ThreadLine[]): TrainerTurn[] {
  return lines
    .filter((line): line is TrainerTurn => (line.role === "user" || line.role === "assistant") && Boolean(line.content.trim()))
    .map((line) => (line.role === "assistant" ? { role: line.role, content: visibleLotseText(line.content) } : line))
    .filter((line) => line.content.trim())
    .slice(-8);
}

function isAbort(err: unknown): boolean {
  return err instanceof DOMException && err.name === "AbortError";
}

function appendAssistantText(prev: ThreadLine[], text: string): ThreadLine[] {
  if (!text) {
    return prev;
  }
  const next = [...prev];
  const last = next[next.length - 1];
  if (last?.role === "assistant") {
    next[next.length - 1] = { role: "assistant", content: last.content + text };
    return next;
  }
  return [...next, { role: "assistant", content: text }];
}

function finishAssistantText(prev: ThreadLine[], text: string): ThreadLine[] {
  if (!text.trim()) {
    return prev;
  }
  const next = [...prev];
  const last = next[next.length - 1];
  if (last?.role === "assistant") {
    if (!last.content.trim()) {
      next[next.length - 1] = { role: "assistant", content: text };
    }
    return next;
  }
  return [...next, { role: "assistant", content: text }];
}

function lineRole(t: Messages, line: ThreadLine): string {
  switch (line.role) {
    case "user":
      return t.trainerYou;
    case "assistant":
      return t.trainer;
    case "tool":
      return t.trainerTool;
    default: {
      const _never: never = line;
      return _never;
    }
  }
}

function laneLabel(t: Messages, lane: PolicyLane): string {
  switch (lane) {
    case "match":
      return t.laneMatch;
    case "language":
      return t.laneLanguage;
    case "house":
      return t.laneHouse;
    default: {
      const _never: never = lane;
      return _never;
    }
  }
}

function applyEvent(
  event: TrainerChatEvent,
  setLines: (fn: (prev: ThreadLine[]) => ThreadLine[]) => void,
  setConsent: (next: TrainerConsent | null) => void,
  setYolo: (next: boolean) => void,
  setResult: (next: TrainerValidateOut | null) => void,
  onStatus: (status: string) => void,
  t: Messages,
  live: () => boolean,
) {
  if (!live()) {
    return;
  }
  switch (event.type) {
    case "delta":
      setLines((prev) => appendAssistantText(prev, event.text));
      return;
    case "consent":
      setConsent({
        call_id: event.call_id,
        tool: event.tool,
        summary: event.summary,
        validate: event.validate,
      });
      setResult(event.validate);
      return;
    case "session":
      setYolo(event.yolo);
      return;
    case "validate":
      setResult(event.value);
      onStatus(event.value.ok ? t.trainerOk : t.trainerFail);
      return;
    case "proposal":
      return;
    case "tool_call":
      setLines((prev) => [...prev, { role: "tool", name: event.name, args: event.arguments }]);
      return;
    case "tool":
      if (event.tool === "apply_ui" || event.tool === "apply_engine") {
        window.dispatchEvent(new CustomEvent("klar-lotse-applied", { detail: { tool: event.tool } }));
      }
      setLines((prev) => {
        const next = [...prev];
        for (let index = next.length - 1; index >= 0; index -= 1) {
          const row = next[index];
          if (row?.role === "tool" && row.name === event.tool && row.result === undefined) {
            next[index] = { ...row, result: event.text };
            return next;
          }
        }
        return [...next, { role: "tool", name: event.tool, args: "", result: event.text }];
      });
      return;
    case "done":
      setLines((prev) => finishAssistantText(prev, event.text));
      return;
    case "error":
      onStatus(event.message || t.trainerFail);
      return;
    default: {
      const _never: never = event;
      return _never;
    }
  }
}

const LANES: PolicyLane[] = ["match", "language", "house"];

export function TrainerDrawer({
  t,
  lane,
  language,
  active = true,
  onLane,
  onClose,
  onStatus,
}: {
  t: Messages;
  lane: PolicyLane;
  language?: string;
  active?: boolean;
  onLane?: (lane: PolicyLane) => void;
  onClose?: () => void;
  onStatus: (status: string) => void;
}) {
  const [endpoint, setEndpoint] = useState<LlmPublic | null>(null);
  const [endpointError, setEndpointError] = useState(false);
  const [draft, setDraft] = useState("");
  const [lines, setLines] = useState<ThreadLine[]>([]);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<TrainerValidateOut | null>(null);
  const [consent, setConsent] = useState<TrainerConsent | null>(null);
  const [yolo, setYolo] = useState(false);
  const [consumedFor, setConsumedFor] = useState("");
  const endRef = useRef<HTMLDivElement>(null);
  const abortRef = useRef<AbortController | null>(null);
  const genRef = useRef(0);

  const resetThread = () => {
    abortRef.current?.abort();
    abortRef.current = null;
    genRef.current += 1;
    setLines([]);
    setConsent(null);
    setResult(null);
    setDraft("");
    setBusy(false);
    setConsumedFor("");
    onStatus("");
  };

  const loadEndpoint = () => {
    api.llmEndpoint()
      .then((next) => {
        setEndpoint(next);
        setEndpointError(false);
      })
      .catch(() => {
        setEndpointError(true);
      });
  };

  useEffect(() => {
    loadEndpoint();
  }, []);

  useEffect(() => {
    if (active) loadEndpoint();
  }, [active]);

  useEffect(() => {
    resetThread();
    return () => {
      abortRef.current?.abort();
    };
  }, [lane]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ block: "end" });
  }, [lines, consent, busy]);

  const prompts = lanePrompts(t, lane);
  const chips = lotseQuickChips({ lines, busy, consent: Boolean(consent), consumedFor, t });

  const send = async (text = draft) => {
    const message = text.trim();
    if (!message || busy) return;
    const openText = unansweredAssistant(lines);
    const open = lotseReplyChoices(openText);
    const offered = open.length > 0 ? open : lotseFallbackChips(openText, t);
    setConsumedFor(offered.includes(message) ? openText : "");
    setDraft("");
    const history = chatHistory(lines);
    const gen = genRef.current;
    abortRef.current?.abort();
    const abort = new AbortController();
    abortRef.current = abort;
    setLines((prev) => [...prev, { role: "user", content: message }, { role: "assistant", content: "" }]);
    setBusy(true);
    setResult(null);
    setConsent(null);
    try {
      await api.trainerChat({ message, layer: trainerLayer(lane), language, history }, (event) => {
        applyEvent(event, setLines, setConsent, setYolo, setResult, onStatus, t, () => genRef.current === gen);
      }, abort.signal);
    } catch (err) {
      if (isAbort(err) || genRef.current !== gen) {
        return;
      }
      if (err instanceof Error && err.message === "llm-unconfigured") {
        try {
          const next = await api.llmEndpoint();
          setEndpoint(next);
          setEndpointError(false);
          onStatus(next.configured ? t.trainerFail : t.trainerNeedLlm);
        } catch {
          setEndpointError(true);
          onStatus(t.trainerFail);
        }
      } else {
        onStatus(t.trainerFail);
      }
    } finally {
      if (genRef.current === gen) {
        setBusy(false);
      }
    }
  };

  const decide = async (decision: "allow_once" | "allow" | "yolo" | "deny" | "ask_again") => {
    try {
      const out = await api.trainerConsent({ call_id: consent?.call_id, decision });
      setYolo(out.yolo);
      if (decision !== "ask_again") setConsent(null);
    } catch {
      onStatus(t.trainerFail);
    }
  };

  if (!endpoint?.configured) {
    const waiting = !endpoint && !endpointError;
    const needLlm = Boolean(endpoint && !endpoint.configured && !endpointError);
    return (
      <section className="trainer">
        <header className="trainer-head">
          <div>
            <p className="trainer-kicker">{t.trainer}</p>
            <p className="muted">{waiting ? t.trainerStreaming : needLlm ? t.trainerNeedLlm : t.trainerFail}</p>
          </div>
          {onClose ? (
            <button className="ghost" type="button" onClick={onClose}>{t.close}</button>
          ) : null}
        </header>
        {waiting ? null : (
          <div className="trainer-composer">
            {needLlm ? (
              <button className="primary" type="button" onClick={() => { window.location.hash = "#/settings/llm"; }}>
                {t.trainerOpenSettings}
              </button>
            ) : (
              <button className="primary" type="button" onClick={() => loadEndpoint()}>{t.trainerValidate}</button>
            )}
          </div>
        )}
      </section>
    );
  }

  return (
    <section className="trainer" aria-label={t.trainer}>
      <header className="trainer-head">
        <div>
          <p className="trainer-kicker">{t.trainer}</p>
          {onLane ? (
            <nav className="trainer-lanes" aria-label={t.laneTabs}>
              {LANES.map((id) => (
                <button
                  key={id}
                  type="button"
                  aria-pressed={lane === id}
                  onClick={() => onLane(id)}
                >
                  {laneLabel(t, id)}
                </button>
              ))}
            </nav>
          ) : (
            <h2>{t.trainerForLane}</h2>
          )}
          <p className="muted">{t.trainerHint}</p>
        </div>
        <div className="trainer-meta">
          <span className="chip trainer-model" title={endpoint.model || "LLM"}>{shortModel(endpoint.model)}</span>
          {lines.length > 0 ? (
            <button className="ghost" type="button" onClick={resetThread}>
              {t.trainerClear}
            </button>
          ) : null}
          {yolo ? (
            <button className="chip on" type="button" onClick={() => void decide("ask_again")}>
              {t.trainerYolo} · {t.trainerAskAgain}
            </button>
          ) : null}
          {onClose ? (
            <button className="ghost" type="button" onClick={onClose}>{t.close}</button>
          ) : null}
        </div>
      </header>
      <div className="trainer-thread">
        {lines.length === 0 && !busy ? (
          <div className="trainer-empty">
            <p>{t.trainerEmpty}</p>
            <p className="muted">{t.trainerEmptyHint}</p>
            <div className="trainer-prompts">
              <button className="guide-step" type="button" onClick={() => void send(prompts[0])}>
                <span className="guide-num" aria-hidden="true">1</span>
                <span className="guide-copy"><strong>{prompts[0]}</strong></span>
              </button>
              <button className="guide-step" type="button" onClick={() => void send(prompts[1])}>
                <span className="guide-num" aria-hidden="true">2</span>
                <span className="guide-copy"><strong>{prompts[1]}</strong></span>
              </button>
            </div>
          </div>
        ) : null}
        {lines.map((line, index) => (
          <article className={`trainer-line ${line.role}`} key={`${line.role}-${index}`}>
            <span className="trainer-role">{lineRole(t, line)}</span>
            {line.role === "tool" ? (
              <TrainerToolCard name={line.name} args={line.args} result={line.result} t={t} />
            ) : line.role === "assistant" ? (
              <div className="trainer-bubble">
                {line.content ? <LotseAnswer text={line.content} t={t} /> : busy ? t.trainerStreaming : ""}
              </div>
            ) : (
              <p className="trainer-bubble">{line.content}</p>
            )}
          </article>
        ))}
        {consent ? (
          <div className="trainer-consent">
            <p className="trainer-kicker">{t.trainerPermit}</p>
            <p className="mono">{consent.tool}</p>
            <p>{consent.summary}</p>
            <div className="row">
              <button className="primary" type="button" onClick={() => void decide("allow")}>{t.trainerAllow}</button>
              <button className="secondary" type="button" onClick={() => void decide("allow_once")}>{t.trainerAllowOnce}</button>
              <button className="ghost" type="button" onClick={() => void decide("deny")}>{t.trainerDeny}</button>
              <button className="ghost danger" type="button" onClick={() => void decide("yolo")}>{t.trainerYolo}</button>
            </div>
          </div>
        ) : null}
        <div ref={endRef} />
      </div>
      {chips.length > 0 ? (
        <div className="trainer-quick">
          {chips.map((prompt) => (
            <button
              className="trainer-chip"
              disabled={busy}
              key={prompt}
              type="button"
              onClick={() => void send(prompt)}
            >
              {prompt}
            </button>
          ))}
        </div>
      ) : null}
      <form
        className="trainer-composer"
        onSubmit={(event) => {
          event.preventDefault();
          void send();
        }}
      >
        <label className="visually-hidden" htmlFor="trainer-draft">{t.trainerComposer}</label>
        <textarea
          id="trainer-draft"
          value={draft}
          disabled={busy && !consent}
          placeholder={t.trainerComposer}
          rows={2}
          onChange={(ev) => setDraft(ev.target.value)}
          onKeyDown={(ev) => {
            if (ev.key === "Enter" && !ev.shiftKey) {
              ev.preventDefault();
              void send();
            }
          }}
        />
        <button className="primary" type="submit" disabled={(busy && !consent) || !draft.trim()}>
          {busy && !consent ? <Loader2Icon data-icon="inline-start" className="animate-spin" /> : null}
          {busy && !consent ? t.trainerStreaming : t.trainerSend}
        </button>
      </form>
      {result ? (
        <div className={`trainer-result${result.ok ? "" : " danger"}`}>
          <strong>{result.ok ? t.trainerOk : t.trainerFail}</strong>
          {result.errors.map((row) => (
            <p key={`${row.path}-${row.message}`}>{row.path}: {row.message}</p>
          ))}
          {result.warnings.map((row) => (
            <p className="muted" key={`w-${row.path}-${row.message}`}>{row.path}: {row.message}</p>
          ))}
          {result.dry_run.map((row) => (
            <span className="mono" key={row.text}>
              {row.text} → {row.decision}
              {row.seed ? ` · seed ${row.seed}` : ""}
              {row.house ? ` · house ${row.house}` : ""}
            </span>
          ))}
        </div>
      ) : null}
    </section>
  );
}
