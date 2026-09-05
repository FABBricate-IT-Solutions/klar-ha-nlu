import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { AreaTrend, Bars, DecisionMix, Donut, type MixRow } from "../components/charts";
import { Empty, Kpi } from "../components/common";
import { Snackbar } from "../components/Snackbar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
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
        <div className="flex flex-col gap-2">
          <Badge variant="outline" className="w-fit">{t.engineReady}</Badge>
          <h1>{t.understandsHome}</h1>
          <p className="muted">{view.counts.assist} {t.assistVisible} · {view.counts.leftover} {t.open}</p>
        </div>
        {canApply && <Button type="button" onClick={onApply}>{t.applyAll}</Button>}
      </section>

      {last && (
        <Card className="mb-4">
          <CardHeader className="flex-row items-start justify-between gap-4">
            <div className="flex flex-col gap-2">
              <CardTitle>{t.lastTurn}</CardTitle>
              <Badge variant="secondary" className="w-fit">{last.decision}</Badge>
              {heard ? <p>{heard}</p> : <p className="muted">—</p>}
              {last.speech ? <p className="muted">{last.speech}</p> : null}
              {last.preferred_area && <p className="caption">{t.heardIn}: {last.preferred_area}</p>}
            </div>
            <div className="flex flex-wrap gap-2">
              <Button variant="ghost" type="button" onClick={() => setWhyOpen(true)}>{whyThisBand(t)}</Button>
              <Button variant="ghost" type="button" onClick={() => onReplay(heard)} disabled={!canReplay}>{t.inLab}</Button>
              <Button variant="outline" type="button" onClick={() => void undoLast()} disabled={busy}>{t.undoLastCommand}</Button>
            </div>
          </CardHeader>
        </Card>
      )}

      {leftoverMiss && (
        <Card className="mb-4 ring-1 ring-primary/50">
          <CardContent className="flex flex-col gap-3">
          {inbox.length > 0 && (
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div>
                <h2>{view.counts.leftover} {t.needsWork}</h2>
                <p className="muted">{inbox[0]?.name} · {inbox[0]?.reasons.join(", ")}</p>
              </div>
              <Button variant="outline" type="button" onClick={onOpenCalibrate}>{t.calibrate}</Button>
            </div>
          )}
          {canTeach && (
            <div className="flex flex-wrap gap-2">
              <Button
                variant="outline"
                type="button"
                disabled={!heard}
                onClick={() => {
                  if (heard) onTeach?.(heard);
                }}
              >
                {rememberAsPhrase(t)}
              </Button>
              <Button variant="ghost" type="button" onClick={() => onReplay(heard)} disabled={!canReplay}>{t.inLab}</Button>
            </div>
          )}
          </CardContent>
        </Card>
      )}

      <Card className="mb-4">
        <CardHeader>
          <CardTitle>{t.tryThese}</CardTitle>
          <CardDescription>{t.tryTheseHint}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-2">
          {samples.map((sentence) => (
            <Button key={sentence} variant="ghost" type="button" onClick={() => onReplay(sentence)}>{sentence}</Button>
          ))}
        </CardContent>
      </Card>

      <details className="card">
        <summary style={{ cursor: "pointer", minHeight: 44, display: "flex", alignItems: "center" }}>{moreLabel(t)}</summary>
        <section className="grid three" style={{ margin: "16px 0" }}>
          <Kpi value={view.counts.assist} label={t.assistVisible} />
          <Kpi value={view.counts.high} label={t.certain} />
          <Kpi value={turns.length || view.traffic.total} label={t.processed} />
        </section>
        <section className="grid gap-4 md:grid-cols-2">
          <Card className="overflow-visible">
            <CardHeader>
              <CardTitle>{t.decisionMix}</CardTitle>
              <CardDescription>{t.mixCaption}</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-3">
              <DecisionMix data={mix} unit={t.unitsTurns} />
              <div className="flex flex-wrap gap-2">
                {MIX_LEGEND.map((item) => (
                  <span className="chip" key={item.key}>
                    <span className="size-2.5 rounded-sm" style={{ background: item.color }} />
                    {item.key}
                  </span>
                ))}
              </div>
            </CardContent>
          </Card>
          <Card className="overflow-visible">
            <CardHeader>
              <CardTitle>{t.coverage}</CardTitle>
              <CardDescription>{t.coverageCaption}</CardDescription>
            </CardHeader>
            <CardContent>
              <Bars data={[
                { label: t.coverageGraph, value: view.coverage.all },
                { label: t.coverageAssist, value: view.coverage.assist },
                { label: t.coverageReady, value: view.coverage.high },
                { label: t.coverageOpen, value: view.coverage.leftover },
              ]} unit={t.unitsPercent} />
            </CardContent>
          </Card>
          <Card className="overflow-visible">
            <CardHeader>
              <CardTitle>{t.confidence}</CardTitle>
            </CardHeader>
            <CardContent>
              <Donut high={view.counts.high} medium={view.counts.medium} low={view.counts.low} />
            </CardContent>
          </Card>
          <Card className="overflow-visible">
            <CardHeader>
              <CardTitle>{t.rooms}</CardTitle>
            </CardHeader>
            <CardContent>
              <Bars data={view.rooms.slice(0, 8).map((room) => ({ label: room.name, value: room.inbox }))} />
            </CardContent>
          </Card>
          <Card className="overflow-visible">
            <CardHeader>
              <CardTitle>{t.recordings}</CardTitle>
            </CardHeader>
            <CardContent>
              <AreaTrend data={view.traffic.by_day} unit={t.unitsTurns} />
            </CardContent>
          </Card>
          <Card className="overflow-visible">
            <CardHeader>
              <CardTitle>{t.recent}</CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col gap-2">
              {turns.length === 0 ? <Empty text={t.emptyBundle} /> : null}
              {turns.slice(-6).reverse().map((row) => {
                const line = journalHeard(row);
                return (
                  <div className="flex items-center justify-between gap-3 border-b border-border py-2" key={`${row.conversation_id}-${row.ts_ms}`}>
                    <span>{line || row.decision}</span>
                    <Button variant="ghost" type="button" onClick={() => onReplay(line)} disabled={!canJournalReplay(row)}>{t.replay}</Button>
                  </div>
                );
              })}
            </CardContent>
          </Card>
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
            <Button variant="ghost" type="button" onClick={() => void undoApply()} disabled={applyBusy}>{t.undo}</Button>
          ) : null}
        />
      )}
    </div>
  );
}
