import { useEffect, useState } from "react";
import { api, type LangOverlay } from "../api";
import type { Messages } from "../i18n";
import type {
  LlmPublic,
  MatchControl,
  PolicyRule,
  TrainerChatEvent,
  TrainerProposal,
  TrainerTurn,
  TrainerValidateOut,
} from "../types";

function layerOf(proposal: TrainerProposal): "match" | "language" | "house" | "all" {
  const layer = proposal.layer || "all";
  switch (layer) {
    case "match":
    case "language":
    case "house":
    case "all":
      return layer;
    default:
      return "all";
  }
}

function applyEvent(
  event: TrainerChatEvent,
  setLines: (fn: (prev: TrainerTurn[]) => TrainerTurn[]) => void,
  setProposal: (next: TrainerProposal | null) => void,
  setRaw: (next: string) => void,
  setResult: (next: TrainerValidateOut | null) => void,
  onStatus: (status: string) => void,
  t: Messages,
) {
  switch (event.type) {
    case "delta":
      setLines((prev) => {
        const next = [...prev];
        const last = next[next.length - 1];
        if (last?.role === "assistant") {
          next[next.length - 1] = { role: "assistant", content: last.content + event.text };
        }
        return next;
      });
      return;
    case "proposal":
      setProposal(event.value);
      setRaw(JSON.stringify(event.value, null, 2));
      return;
    case "validate":
      setResult(event.value);
      onStatus(event.value.ok ? t.trainerOk : t.trainerFail);
      return;
    case "done":
      setLines((prev) => {
        const next = [...prev];
        const last = next[next.length - 1];
        if (last?.role === "assistant" && !last.content.trim() && event.text) {
          next[next.length - 1] = { role: "assistant", content: event.text };
        }
        return next;
      });
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

export function TrainerDrawer({
  t,
  language,
  overlay,
  onApplyHouse,
  onApplyMatch,
  onStatus,
}: {
  t: Messages;
  language?: string;
  overlay: LangOverlay | null;
  onApplyHouse: (next: PolicyRule[]) => Promise<void>;
  onApplyMatch: (next: MatchControl[]) => Promise<void>;
  onStatus: (status: string) => void;
}) {
  const [endpoint, setEndpoint] = useState<LlmPublic | null>(null);
  const [draft, setDraft] = useState("");
  const [lines, setLines] = useState<TrainerTurn[]>([]);
  const [busy, setBusy] = useState(false);
  const [raw, setRaw] = useState("");
  const [result, setResult] = useState<TrainerValidateOut | null>(null);
  const [proposal, setProposal] = useState<TrainerProposal | null>(null);

  useEffect(() => {
    api.llmEndpoint().then(setEndpoint).catch(() => setEndpoint({ configured: false }));
  }, []);

  const parsed = (): TrainerProposal | null => {
    if (proposal) return proposal;
    if (!raw.trim()) return null;
    try {
      return JSON.parse(raw) as TrainerProposal;
    } catch {
      return null;
    }
  };

  const send = async () => {
    const message = draft.trim();
    if (!message || busy) return;
    setDraft("");
    const history = lines.slice(-8);
    setLines((prev) => [...prev, { role: "user", content: message }, { role: "assistant", content: "" }]);
    setBusy(true);
    setResult(null);
    setProposal(null);
    try {
      await api.trainerChat({ message, layer: "all", language, history }, (event) => {
        applyEvent(event, setLines, setProposal, setRaw, setResult, onStatus, t);
      });
    } catch (err) {
      if (err instanceof Error && err.message === "llm-unconfigured") {
        setEndpoint({ configured: false });
        onStatus(t.trainerNeedLlm);
      } else {
        onStatus(t.trainerFail);
      }
    } finally {
      setBusy(false);
    }
  };

  const runValidate = async () => {
    const next = parsed();
    if (!next) {
      onStatus(t.trainerFail);
      return;
    }
    if (!next.language && language) next.language = language;
    const out = await api.validateProposal(next);
    setResult(out);
    onStatus(out.ok ? t.trainerOk : t.trainerFail);
  };

  const applyLane = async (lane: "house" | "match" | "language") => {
    if (!result?.ok) return;
    const next = parsed();
    if (!next) return;
    const layer = layerOf(next);
    if (layer !== "all" && layer !== lane) return;
    if (lane === "house" && next.policies) await onApplyHouse(next.policies);
    if (lane === "match" && next.match_controls) await onApplyMatch(next.match_controls);
    if (lane === "language" && next.language_overlay) {
      await api.saveLangOverlay({ custom: overlay?.custom || [], language: next.language_overlay, label: "trainer" });
    }
    onStatus(t.trainerApply);
  };

  const canApply = (lane: "house" | "match" | "language") => {
    if (!result?.ok) return false;
    const next = parsed();
    if (!next) return false;
    const layer = layerOf(next);
    if (layer !== "all" && layer !== lane) return false;
    if (lane === "house") return Boolean(next.policies);
    if (lane === "match") return Boolean(next.match_controls);
    return Boolean(next.language_overlay);
  };

  if (!endpoint || !endpoint.configured) {
    return (
      <div className="card trainer-setup" style={{ marginTop: 16 }}>
        <h2>{t.trainer}</h2>
        <p className="muted">{endpoint ? t.trainerNeedLlm : t.trainerStreaming}</p>
        {endpoint ? (
          <button className="primary" onClick={() => { window.location.hash = "#/settings"; }}>{t.trainerOpenSettings}</button>
        ) : null}
      </div>
    );
  }

  return (
    <div className="card" style={{ marginTop: 16 }}>
      <h2>{t.trainer}</h2>
      <p className="muted">{t.trainerHint}</p>
      <div className="trainer-layout">
        <div>
          <div className="trainer-thread">
            {lines.map((line, index) => (
              <p className={`trainer-bubble ${line.role}`} key={`${line.role}-${index}`}>{line.content || (busy ? t.trainerStreaming : "")}</p>
            ))}
          </div>
          <label>{t.trainerSend}</label>
          <textarea
            className="trainer-json"
            value={draft}
            rows={3}
            disabled={busy}
            onChange={(ev) => setDraft(ev.target.value)}
            onKeyDown={(ev) => {
              if (ev.key === "Enter" && !ev.shiftKey) {
                ev.preventDefault();
                void send();
              }
            }}
          />
          <div className="row">
            <button className="primary" disabled={busy || !draft.trim()} onClick={() => void send()}>{busy ? t.trainerStreaming : t.trainerSend}</button>
          </div>
        </div>
        <div>
          {result && (
            <div>
              <p className="muted">{result.ok ? t.trainerOk : t.trainerFail}</p>
              {result.errors.map((row) => (
                <p className="muted" key={`${row.path}-${row.message}`}>{row.path}: {row.message}</p>
              ))}
              {result.warnings.map((row) => (
                <p className="muted" key={`w-${row.path}-${row.message}`}>{row.path}: {row.message}</p>
              ))}
              {result.dry_run.map((row) => (
                <p className="lexicon-delta" key={row.text}>
                  {row.text} → {row.decision}
                  {row.seed ? ` · seed ${row.seed}` : ""}
                  {row.house ? ` · house ${row.house}` : ""}
                </p>
              ))}
            </div>
          )}
          <div className="row" style={{ marginTop: 12 }}>
            <button className="primary" disabled={!canApply("house")} onClick={() => void applyLane("house")}>{t.trainerApplyHouse}</button>
            <button className="primary" disabled={!canApply("match")} onClick={() => void applyLane("match")}>{t.trainerApplyMatch}</button>
            <button className="primary" disabled={!canApply("language")} onClick={() => void applyLane("language")}>{t.trainerApplyLanguage}</button>
          </div>
          <details style={{ marginTop: 16 }}>
            <summary>{t.trainerAdvanced}</summary>
            <div className="row" style={{ marginTop: 8 }}>
              <button className="secondary" onClick={() => void runValidate()}>{t.trainerValidate}</button>
            </div>
            <label>{t.trainerProposal}</label>
            <textarea className="trainer-json" value={raw} onChange={(ev) => { setRaw(ev.target.value); setProposal(null); }} rows={8} />
          </details>
        </div>
      </div>
    </div>
  );
}
