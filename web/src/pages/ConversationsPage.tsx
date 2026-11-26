import { useMemo, useState } from "react";
import { Empty } from "../components/common";
import { SetupHint } from "../components/SetupHint";
import { MarkWrong } from "../components/MarkWrong";
import { canTeachFromMiss, TeachFromMiss, teachIntentFromNames } from "../components/TeachFromMiss";
import { WhyDrawer, canJournalReplay, journalHeard, whyThisBand } from "../components/WhyDrawer";
import { fill, type Messages } from "../i18n";
import { turnMatchesQuery } from "../lib/journalLive";
import { refineBandLabel } from "./ParsePage";
import type { ConversationTurn, RefineBand } from "../types";

const DECISIONS = ["", "execute", "confirm", "clarify", "reject", "chat", "error"] as const;

type DecisionFilter = (typeof DECISIONS)[number];

function turnRefineBand(raw: string | null | undefined): RefineBand | null {
  switch (raw) {
    case "status":
    case "command":
    case "prompt":
    case "reject":
      return raw;
    default:
      return null;
  }
}

function turnTitle(turn: ConversationTurn, locale: string | undefined): string {
  const when = turn.ts_ms ? new Date(turn.ts_ms).toLocaleString(locale) : "";
  return [when, turn.decision].filter(Boolean).join(" · ");
}

function decisionLabel(t: Messages, decision: DecisionFilter): string {
  switch (decision) {
    case "":
      return t.journalFilterAll;
    case "execute":
      return t.labDecisionExecute;
    case "confirm":
      return "confirm";
    case "clarify":
      return "clarify";
    case "reject":
      return "reject";
    case "chat":
      return t.llmKindChat;
    case "error":
      return "error";
    default: {
      const _never: never = decision;
      return _never;
    }
  }
}

function asDecisionFilter(raw: string): DecisionFilter {
  for (const value of DECISIONS) {
    if (value === raw) return value;
  }
  return "";
}

export function ConversationsPage({
  t,
  locale,
  turns,
  live,
  onReplay,
  onTeach,
}: {
  t: Messages;
  locale: string;
  /** Live journal from App (SSE). Null while the first load is in flight. */
  turns: ConversationTurn[] | null;
  live?: boolean;
  onReplay: (text: string) => void;
  onTeach?: (heard: string) => void;
}) {
  const [why, setWhy] = useState<ConversationTurn | null>(null);
  const [query, setQuery] = useState("");
  const [decision, setDecision] = useState<DecisionFilter>("");

  const items = useMemo(() => {
    const sorted = [...(turns || [])].sort((a, b) => (b.ts_ms || 0) - (a.ts_ms || 0));
    return sorted.filter((turn) => turnMatchesQuery(turn, query, decision));
  }, [turns, query, decision]);

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.conversations}</h1>
          <p className="muted">{t.journalHint}</p>
          {live ? <p className="caption">{t.journalLive}</p> : null}
        </div>
      </section>
      <section className="card" style={{ marginBottom: 16 }}>
        <div className="row" style={{ flexWrap: "wrap", gap: 8 }}>
          <input
            value={query}
            onChange={(ev) => setQuery(ev.target.value)}
            placeholder={t.journalSearch}
            aria-label={t.journalSearch}
          />
          <select
            value={decision}
            onChange={(ev) => setDecision(asDecisionFilter(ev.target.value))}
            aria-label={t.journalFilterDecision}
          >
            {DECISIONS.map((value) => (
              <option key={value || "all"} value={value}>
                {decisionLabel(t, value)}
              </option>
            ))}
          </select>
        </div>
        {turns && (
          <p className="caption" style={{ marginTop: 8 }}>
            {fill(t.journalFilterCount, { shown: String(items.length), total: String(turns.length) })}
          </p>
        )}
      </section>
      {turns === null && <div className="card">{t.loading}</div>}
      {turns && items.length === 0 && (
        <Empty
          text={turns.length === 0 ? t.noConversations : t.journalFilterEmpty}
          action={
            turns.length === 0 ? (
              <>
                <p className="caption">{t.conversationsEmptyHint}</p>
                <SetupHint t={t} />
              </>
            ) : null
          }
        />
      )}
      {items.map((turn, index) => {
        const heard = journalHeard(turn);
        const refineBand = turnRefineBand(turn.refine_band);
        return (
          <article className="card" key={`${turn.conversation_id}-${turn.ts_ms}-${index}`} style={{ marginBottom: 16 }}>
            <div className="conv-head">
              <h2>{turnTitle(turn, locale)}</h2>
              <div className="row">
                <button className="ghost" type="button" onClick={() => setWhy(turn)}>{whyThisBand(t)}</button>
                <button
                  className="ghost"
                  type="button"
                  onClick={() => onReplay(heard)}
                  disabled={!canJournalReplay(turn)}
                >
                  {t.replay}
                </button>
              </div>
            </div>
            {heard ? <p>{heard}</p> : <p className="muted">—</p>}
            {turn.speech ? (
              <p className="muted">
                {turn.speech_source === "chat" ? <span className="chip">{t.speechChat}</span> : null}
                {turn.speech_source === "refine" ? <span className="chip">{t.speechRefined}</span> : null}
                {refineBand ? <span className="chip">{refineBandLabel(refineBand, t)}</span> : null}
                {" "}
                {turn.speech}
              </p>
            ) : null}
            {turn.preferred_area ? <p className="caption">{t.heardIn}: {turn.preferred_area}</p> : null}
            <div style={{ marginTop: 12 }} className="flow">
              <MarkWrong turn={turn} t={t} />
              {canTeachFromMiss(turn) && (
                <TeachFromMiss
                  heard={heard}
                  t={t}
                  onReplay={onReplay}
                  onTeach={onTeach}
                  intent={teachIntentFromNames(turn.last_names ?? [])}
                />
              )}
            </div>
          </article>
        );
      })}
      {why && <WhyDrawer turn={why} t={t} onClose={() => setWhy(null)} />}
    </div>
  );
}
