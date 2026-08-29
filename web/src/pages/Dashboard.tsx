import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { AreaTrend, Bars, DecisionMix, Donut, type MixRow } from "../components/charts";
import { Empty, Kpi } from "../components/common";
import { Snackbar } from "../components/Snackbar";
import { WhyDrawer, canJournalReplay, journalHeard, whyThisBand } from "../components/WhyDrawer";
import { fill, type Messages } from "../i18n";
import type { ApplyRow, ConversationTurn, Dashboard as DashboardData, Locale } from "../types";

const MISS = new Set(["chat", "reject", "clarify"]);
const MIX_LEGEND = [
  { key: "execute", color: "var(--high)" },
  { key: "confirm", color: "var(--accent)" },
  { key: "clarify", color: "var(--medium)" },
  { key: "reject", color: "var(--danger)" },
  { key: "chat", color: "var(--cyan)" },
] as const;

function isDe(t: Messages): boolean {
  return t.replay === "Nochmal";
}

function moreLabel(t: Messages): string {
  return isDe(t) ? "Mehr" : "More";
}

function rememberAsPhrase(t: Messages): string {
  return isDe(t) ? "Als Phrase merken" : "Remember as phrase";
}

function trySentences(rooms: DashboardData["rooms"], t: Messages): string[] {
  const room = rooms[0]?.name || t.tryRoom;
  return [fill(t.tryOn, { room }), t.tryLock, t.tryTime, t.tryNight, t.tryUndo];
}

function mixFrom(turns: ConversationTurn[]): MixRow[] {
  const byDay = new Map<string, MixRow>();
  for (const turn of turns) {
    const day = new Date(turn.ts_ms).toISOString().slice(0, 10);
    const row = byDay.get(day) || { day, execute: 0, confirm: 0, clarify: 0, reject: 0, chat: 0 };
    if (turn.decision === "execute") row.execute += 1;
    else if (turn.decision === "confirm") row.confirm += 1;
    else if (turn.decision === "clarify") row.clarify += 1;
    else if (turn.decision === "reject") row.reject += 1;
    else row.chat += 1;
    byDay.set(day, row);
  }
  return [...byDay.values()].sort((a, b) => a.day.localeCompare(b.day)).slice(-7);
}

function applyKey(rows: ApplyRow[]): string {
  return rows.map((row) => `${row.entity_id}:${row.after}`).join("|");
}

function resolveLast(passed: ConversationTurn | null | undefined, turns: ConversationTurn[]): ConversationTurn | null {
  if (passed !== undefined) return passed;
  return turns.at(-1) ?? null;
}

export function DashboardPage({
  data,
  t,
  dismissed,
  onReplay,
  onApply,
  onOpenCalibrate,
  canApply,
  lastTurn,
  onTeach,
  parseLanguage,
}: {
  data: DashboardData;
  t: Messages;
  locale: Locale;
  parseLanguage?: string;
  dismissed: string[];
  onReplay: (text: string) => void;
  onApply: () => void;
  onOpenCalibrate: () => void;
  canApply: boolean;
  lastTurn?: ConversationTurn | null;
  onTeach?: (text: string) => void;
}) {
  const [turns, setTurns] = useState<ConversationTurn[]>([]);
  const [busy, setBusy] = useState(false);
  const [overlay, setOverlay] = useState<DashboardData | null>(null);
  const [applyRows, setApplyRows] = useState<ApplyRow[]>([]);
  const [applyNotice, setApplyNotice] = useState<"ready" | "undone" | "failed" | null>(null);
  const [applyBusy, setApplyBusy] = useState(false);
  const [hiddenApply, setHiddenApply] = useState("");
  const [whyOpen, setWhyOpen] = useState(false);
  const view = overlay ?? data;
  useEffect(() => {
    setOverlay(null);
  }, [data]);
  useEffect(() => {
    api.conversations().then(setTurns).catch(() => undefined);
  }, [view.traffic.total]);
  useEffect(() => {
    let cancelled = false;
    api.ui().then((next) => {
      if (cancelled || next.last_apply.length === 0) return;
      setApplyRows(next.last_apply);
      setApplyNotice("ready");
    }).catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [data]);
  const mix = useMemo(() => mixFrom(turns), [turns]);
  const inbox = view.assignment.filter((row) => row.confidence !== "high" && !dismissed.includes(row.entity_id));
  const last = resolveLast(lastTurn, turns);
  const heard = last ? journalHeard(last) : "";
  const canReplay = last ? canJournalReplay(last) : false;
  const canTeach = Boolean(last && MISS.has(last.decision) && heard);
  const leftoverMiss = inbox.length > 0 || canTeach;
  const samples = trySentences(view.rooms, t);
  const undoLast = async () => {
    setBusy(true);
    try {
      await api.parse(t.tryUndo, parseLanguage || "");
      const next = await api.conversations();
      setTurns(next);
    } finally {
      setBusy(false);
    }
  };
  const undoApply = async () => {
    setApplyBusy(true);
    try {
      await api.undoApply();
      setApplyRows([]);
      setApplyNotice("undone");
      const next = await api.dashboard();
      setOverlay(next);
    } catch {
      setApplyNotice("failed");
    } finally {
      setApplyBusy(false);
    }
  };
  const snackbar = applyNotice === "ready" && applyRows.length > 0 && applyKey(applyRows) !== hiddenApply
    ? { message: fill(t.applyDone, { count: String(applyRows.length) }), tone: "default" as const, action: true }
    : applyNotice === "undone"
      ? { message: t.applyUndone, tone: "default" as const, action: false }
      : applyNotice === "failed"
        ? { message: t.applyUndoFailed, tone: "danger" as const, action: false }
        : null;
  return (
    <div className="page">
      <section className="hero">
        <div>
          <p className="pill" style={{ display: "inline-block" }}>{t.engineReady}</p>
          <h1>{t.understandsHome}</h1>
          <p className="muted">{view.counts.assist} {t.assistVisible} · {view.counts.leftover} {t.open}</p>
        </div>
        {canApply && <button className="primary" type="button" onClick={onApply}>{t.applyAll}</button>}
      </section>

      {last && (
        <section className="card" style={{ marginBottom: 16 }}>
          <div className="row" style={{ justifyContent: "space-between", alignItems: "flex-start" }}>
            <div>
              <h2>{t.lastTurn}</h2>
              <p className="pill" style={{ display: "inline-block" }}>{last.decision}</p>
              {heard ? <p>{heard}</p> : <p className="muted">—</p>}
              {last.speech ? <p className="muted">{last.speech}</p> : null}
              {last.preferred_area && <p className="caption">{t.heardIn}: {last.preferred_area}</p>}
            </div>
            <div className="row">
              <button className="ghost" type="button" onClick={() => setWhyOpen(true)}>{whyThisBand(t)}</button>
              <button className="ghost" type="button" onClick={() => onReplay(heard)} disabled={!canReplay}>{t.inLab}</button>
              <button className="secondary" type="button" onClick={undoLast} disabled={busy}>{t.undoLastCommand}</button>
            </div>
          </div>
        </section>
      )}

      {leftoverMiss && (
        <section className="card hot" style={{ marginBottom: 16 }}>
          {inbox.length > 0 && (
            <div className="row" style={{ justifyContent: "space-between", alignItems: "flex-start" }}>
              <div>
                <h2>{view.counts.leftover} {t.needsWork}</h2>
                <p className="muted">{inbox[0]?.name} · {inbox[0]?.reasons.join(", ")}</p>
              </div>
              <button className="secondary" type="button" onClick={onOpenCalibrate}>{t.calibrate}</button>
            </div>
          )}
          {canTeach && (
            <div className="row" style={{ marginTop: inbox.length > 0 ? 12 : 0 }}>
              <button
                className="secondary"
                type="button"
                disabled={!heard}
                onClick={() => {
                  if (heard) onTeach?.(heard);
                }}
              >
                {rememberAsPhrase(t)}
              </button>
              <button className="ghost" type="button" onClick={() => onReplay(heard)} disabled={!canReplay}>{t.inLab}</button>
            </div>
          )}
        </section>
      )}

      <section className="card" style={{ marginBottom: 16 }}>
        <h2>{t.tryThese}</h2>
        <p className="muted">{t.tryTheseHint}</p>
        <div className="row" style={{ flexWrap: "wrap", marginTop: 8 }}>
          {samples.map((sentence) => (
            <button key={sentence} className="ghost" type="button" onClick={() => onReplay(sentence)}>{sentence}</button>
          ))}
        </div>
      </section>

      <details className="card">
        <summary style={{ cursor: "pointer", minHeight: 44, display: "flex", alignItems: "center" }}>{moreLabel(t)}</summary>
        <section className="grid three" style={{ margin: "16px 0" }}>
          <Kpi value={view.counts.assist} label={t.assistVisible} />
          <Kpi value={view.counts.high} label={t.certain} />
          <Kpi value={turns.length || view.traffic.total} label={t.processed} />
        </section>
        <section className="grid two">
          <div className="card">
            <h2>{t.decisionMix}</h2>
            <DecisionMix data={mix} unit={t.unitsTurns} />
            <div className="row" style={{ marginTop: 8 }}>
              {MIX_LEGEND.map((item) => (
                <span className="chip" key={item.key}>
                  <span style={{ width: 10, height: 10, background: item.color }} />
                  {item.key}
                </span>
              ))}
            </div>
            <p className="caption">{t.mixCaption}</p>
          </div>
          <div className="card">
            <h2>{t.coverage}</h2>
            <Bars data={[
              { label: t.coverageGraph, value: view.coverage.all },
              { label: t.coverageAssist, value: view.coverage.assist },
              { label: t.coverageReady, value: view.coverage.high },
              { label: t.coverageOpen, value: view.coverage.leftover },
            ]} unit={t.unitsPercent} />
            <p className="caption">{t.coverageCaption}</p>
          </div>
          <div className="card">
            <h2>{t.confidence}</h2>
            <Donut high={view.counts.high} medium={view.counts.medium} low={view.counts.low} />
          </div>
          <div className="card">
            <h2>{t.rooms}</h2>
            <Bars data={view.rooms.slice(0, 8).map((room) => ({ label: room.name, value: room.inbox }))} />
          </div>
          <div className="card">
            <h2>{t.recordings}</h2>
            <AreaTrend data={view.traffic.by_day} unit={t.unitsTurns} />
          </div>
          <div className="card">
            <h2>{t.recent}</h2>
            {turns.length === 0 ? <Empty text={t.emptyBundle} /> : null}
            {turns.slice(-6).reverse().map((row) => {
              const line = journalHeard(row);
              return (
                <div className="row" key={`${row.conversation_id}-${row.ts_ms}`} style={{ justifyContent: "space-between", borderBottom: "1px solid var(--line)", padding: "8px 0" }}>
                  <span>{line || row.decision}</span>
                  <button className="ghost" type="button" onClick={() => onReplay(line)} disabled={!canJournalReplay(row)}>{t.replay}</button>
                </div>
              );
            })}
          </div>
        </section>
      </details>
      {whyOpen && last && <WhyDrawer turn={last} t={t} onClose={() => setWhyOpen(false)} />}
      {snackbar && (
        <Snackbar
          message={snackbar.message}
          tone={snackbar.tone}
          dismissLabel={t.close}
          onDismiss={() => {
            if (applyNotice === "ready") setHiddenApply(applyKey(applyRows));
            setApplyNotice(null);
          }}
          action={snackbar.action ? (
            <button className="ghost" type="button" onClick={undoApply} disabled={applyBusy}>{t.undo}</button>
          ) : null}
        />
      )}
    </div>
  );
}
