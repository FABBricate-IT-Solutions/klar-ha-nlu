import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { AreaTrend, Bars, DecisionMix, Donut, type MixRow } from "../components/charts";
import { Empty, Kpi } from "../components/common";
import { fill, type Messages } from "../i18n";
import type { ConversationTurn, Dashboard as DashboardData, Locale } from "../types";

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

export function DashboardPage({
  data,
  t,
  locale,
  onReplay,
  onApply,
  onOpenCalibrate,
  canApply,
}: {
  data: DashboardData;
  t: Messages;
  locale: Locale;
  onReplay: (text: string) => void;
  onApply: () => void;
  onOpenCalibrate: () => void;
  canApply: boolean;
}) {
  const [turns, setTurns] = useState<ConversationTurn[]>([]);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    api.conversations().then(setTurns).catch(() => undefined);
  }, [data.traffic.total]);
  const mix = useMemo(() => mixFrom(turns), [turns]);
  const inbox = data.assignment.filter((row) => row.confidence !== "high");
  const last = turns.at(-1);
  const samples = trySentences(data.rooms, t);
  const undoLast = async () => {
    setBusy(true);
    try {
      await api.parse(t.tryUndo, locale);
      const next = await api.conversations();
      setTurns(next);
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="page">
      <section className="hero">
        <div>
          <p className="pill" style={{ display: "inline-block" }}>{t.engineReady}</p>
          <h1>{t.understandsHome}</h1>
          <p className="muted">{data.counts.assist} {t.assistVisible} · {data.counts.leftover} {t.open}</p>
        </div>
        {canApply && <button className="primary" onClick={onApply}>{t.applyAll}</button>}
      </section>

      {last && (
        <section className="card" style={{ marginBottom: 16 }}>
          <div className="row" style={{ justifyContent: "space-between" }}>
            <div>
              <h2>{t.lastTurn}</h2>
              <p>{last.text || last.speech || last.decision}</p>
              <p className="muted">{last.speech}</p>
              {last.preferred_area && <p className="caption">{t.heardIn}: {last.preferred_area}</p>}
            </div>
            <button className="secondary" onClick={undoLast} disabled={busy}>{t.undo}</button>
          </div>
        </section>
      )}

      <section className="card" style={{ marginBottom: 16 }}>
        <h2>{t.tryThese}</h2>
        <p className="muted">{t.tryTheseHint}</p>
        <div className="row" style={{ flexWrap: "wrap", marginTop: 8 }}>
          {samples.map((sentence) => (
            <button key={sentence} className="ghost" onClick={() => onReplay(sentence)}>{sentence}</button>
          ))}
        </div>
      </section>

      {inbox.length > 0 && (
        <section className="card hot" style={{ marginBottom: 16 }}>
          <div className="row" style={{ justifyContent: "space-between" }}>
            <div>
              <h2>{data.counts.leftover} {t.needsWork}</h2>
              <p className="muted">{inbox[0]?.name} · {inbox[0]?.reasons.join(", ")}</p>
            </div>
            <button className="secondary" onClick={onOpenCalibrate}>{t.calibrate}</button>
          </div>
        </section>
      )}

      <section className="grid three" style={{ marginBottom: 16 }}>
        <Kpi value={data.counts.assist} label={t.assistVisible} />
        <Kpi value={data.counts.high} label={t.certain} />
        <Kpi value={turns.length || data.traffic.total} label={t.processed} />
      </section>

      <section className="grid two">
        <div className="card">
          <h2>{t.decisionMix}</h2>
          <DecisionMix data={mix} unit={t.unitsTurns} />
          <p className="caption">{t.mixCaption}</p>
        </div>
        <div className="card">
          <h2>{t.coverage}</h2>
          <Bars data={[
            { label: "graph", value: data.coverage.all },
            { label: "assist", value: data.coverage.assist },
            { label: "ready", value: data.coverage.high },
            { label: "open", value: data.coverage.leftover },
          ]} unit={t.unitsPercent} />
          <p className="caption">{t.coverageCaption}</p>
        </div>
        <div className="card">
          <h2>{t.confidence}</h2>
          <Donut high={data.counts.high} medium={data.counts.medium} low={data.counts.low} />
        </div>
        <div className="card">
          <h2>{t.rooms}</h2>
          <Bars data={data.rooms.slice(0, 8).map((room) => ({ label: room.name, value: room.inbox }))} />
        </div>
        <div className="card">
          <h2>{t.recordings}</h2>
          <AreaTrend data={data.traffic.by_day} unit={t.unitsTurns} />
        </div>
        <div className="card">
          <h2>{t.recent}</h2>
          {turns.length === 0 && data.traffic.recent.length === 0 ? <Empty text={t.emptyBundle} /> : null}
          {(turns.length ? turns.slice(-6).reverse() : []).map((row) => (
            <div className="row" key={`${row.conversation_id}-${row.ts_ms}`} style={{ justifyContent: "space-between", borderBottom: "1px solid var(--line)", padding: "8px 0" }}>
              <span>{row.text || row.speech || row.decision}</span>
              <button className="ghost" onClick={() => onReplay(row.text || "")}>{t.replay}</button>
            </div>
          ))}
          {turns.length === 0 && data.traffic.recent.slice(-6).reverse().map((row) => (
            <div className="row" key={row.id} style={{ justifyContent: "space-between", borderBottom: "1px solid var(--line)", padding: "8px 0" }}>
              <span>{row.text}</span>
              <button className="ghost" onClick={() => onReplay(row.text)}>{t.replay}</button>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
