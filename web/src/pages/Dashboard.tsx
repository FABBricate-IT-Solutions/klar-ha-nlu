import { AreaTrend, Bars, Donut } from "../components/charts";
import { Empty, Kpi } from "../components/common";
import type { Messages } from "../i18n";
import type { Dashboard as DashboardData } from "../types";

export function DashboardPage({
  data,
  t,
  onReplay,
  onApply,
  onOpenCalibrate,
  canApply,
}: {
  data: DashboardData;
  t: Messages;
  onReplay: (text: string) => void;
  onApply: () => void;
  onOpenCalibrate: () => void;
  canApply: boolean;
}) {
  const inbox = data.assignment.filter((row) => row.confidence !== "high");
  return (
    <div className="page">
      <section className="hero">
        <div>
          <p className="pill hot" style={{ display: "inline-block" }}>{t.engineReady}</p>
          <h1>{t.understandsHome}</h1>
          <p className="muted">{data.counts.assist} {t.assistVisible} · {data.counts.leftover} {t.open}</p>
        </div>
        {canApply && <button className="primary" onClick={onApply}>{t.applyAll}</button>}
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
        <Kpi value={data.counts.high} label={t.certain} hot />
        <Kpi value={data.traffic.total} label={t.processed} />
      </section>

      <section className="grid two">
        <div className="card">
          <h2>{t.coverage}</h2>
          <Bars data={[
            { label: "graph", value: data.coverage.all },
            { label: "assist", value: data.coverage.assist },
            { label: "ready", value: data.coverage.high },
            { label: "open", value: data.coverage.leftover },
          ]} />
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
          <h2>{t.domains}</h2>
          <Bars data={data.domains.map((d) => ({ label: d.domain, value: d.count }))} />
        </div>
        <div className="card">
          <h2>{t.recordings}</h2>
          <AreaTrend data={data.traffic.by_day} />
        </div>
        <div className="card">
          <h2>{t.recent}</h2>
          {data.traffic.recent.length === 0 ? <Empty text={t.emptyBundle} /> : data.traffic.recent.slice(-6).reverse().map((row) => (
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
