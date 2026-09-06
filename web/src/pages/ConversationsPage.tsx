import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { Empty } from "../components/common";
import { SetupHint } from "../components/SetupHint";
import { canTeachFromMiss, TeachFromMiss, teachIntentFromNames } from "../components/TeachFromMiss";
import { WhyDrawer, canJournalReplay, journalHeard, whyThisBand } from "../components/WhyDrawer";
import type { Messages } from "../i18n";
import type { ConversationTurn } from "../types";

function asTurns(rows: unknown): ConversationTurn[] {
  if (!Array.isArray(rows)) {
    return [];
  }
  return rows.filter((row): row is ConversationTurn => !!row && typeof row === "object");
}

function turnTitle(turn: ConversationTurn, locale: string | undefined): string {
  const when = turn.ts_ms ? new Date(turn.ts_ms).toLocaleString(locale) : "";
  return [when, turn.decision].filter(Boolean).join(" · ");
}

export function ConversationsPage({
  t,
  locale,
  onReplay,
  onTeach,
}: {
  t: Messages;
  locale: string;
  onReplay: (text: string) => void;
  onTeach?: (heard: string) => void;
}) {
  const [turns, setTurns] = useState<ConversationTurn[] | null>(null);
  const [error, setError] = useState("");
  const [why, setWhy] = useState<ConversationTurn | null>(null);
  useEffect(() => {
    api.conversations()
      .then((rows) => setTurns(asTurns(rows)))
      .catch((err) => {
        setTurns([]);
        setError(String(err));
      });
  }, []);
  const items = useMemo(
    () => [...(turns || [])].sort((a, b) => (b.ts_ms || 0) - (a.ts_ms || 0)),
    [turns],
  );
  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.conversations}</h1>
          <p className="muted">{t.journalHint}</p>
        </div>
      </section>
      {error && <div className="card danger">{error}</div>}
      {turns === null && !error && <div className="card">{t.loading}</div>}
      {turns && items.length === 0 && !error && (
        <Empty
          text={t.noConversations}
          action={(
            <>
              <p className="caption">{t.conversationsEmptyHint}</p>
              <SetupHint t={t} />
            </>
          )}
        />
      )}
      {items.map((turn, index) => {
        const heard = journalHeard(turn);
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
                {" "}
                {turn.speech}
              </p>
            ) : null}
            {turn.preferred_area ? <p className="caption">{t.heardIn}: {turn.preferred_area}</p> : null}
            {canTeachFromMiss(turn) && (
              <div style={{ marginTop: 12 }}>
                <TeachFromMiss
                  heard={heard}
                  t={t}
                  onReplay={onReplay}
                  onTeach={onTeach}
                  intent={teachIntentFromNames(turn.last_names ?? [])}
                />
              </div>
            )}
          </article>
        );
      })}
      {why && <WhyDrawer turn={why} t={t} onClose={() => setWhy(null)} />}
    </div>
  );
}
