import { Group } from "@visx/group";
import { scaleBand, scaleLinear } from "@visx/scale";
import { Bar, Pie } from "@visx/shape";

type Slice = { label: string; value: number; color: string };

export function Donut({ high, medium, low }: { high: number; medium: number; low: number }) {
  const data: Slice[] = [
    { label: "high", value: high, color: "var(--high)" },
    { label: "medium", value: medium, color: "var(--medium)" },
    { label: "low", value: low, color: "var(--low)" },
  ].filter((item) => item.value > 0);
  const total = high + medium + low || 1;
  return (
    <svg viewBox="0 0 240 220" width="100%" height="220" role="img" aria-label="confidence">
      <Group top={110} left={120}>
        <Pie data={data} pieValue={(d) => d.value} outerRadius={88} innerRadius={58} padAngle={0.02}>
          {(pie) => pie.arcs.map((arc) => (
            <path key={arc.data.label} d={pie.path(arc) || ""} fill={arc.data.color} />
          ))}
        </Pie>
        <text textAnchor="middle" y="-2" fill="var(--text)" fontSize="28" fontWeight="600">{Math.round((high / total) * 100)}%</text>
        <text textAnchor="middle" y="20" fill="var(--muted)" fontSize="11">ready</text>
      </Group>
    </svg>
  );
}

export function Bars({ data, unit = "" }: { data: { label: string; value: number }[]; unit?: string }) {
  const width = 420;
  const height = Math.max(140, data.length * 28 + 28);
  const max = Math.max(1, ...data.map((d) => d.value));
  const x = scaleLinear({ domain: [0, max], range: [0, width - 140] });
  return (
    <svg viewBox={`0 0 ${width} ${height}`} width="100%" height={height} role="img">
      <line x1={118} y1={4} x2={118} y2={height - 20} stroke="var(--line)" />
      <line x1={118} y1={height - 20} x2={width - 8} y2={height - 20} stroke="var(--line)" />
      <text x={width - 8} y={height - 6} textAnchor="end" fill="var(--muted)" fontSize="10">{unit}</text>
      {data.map((d, i) => (
        <Group key={d.label} top={i * 28 + 8}>
          <text x={0} y={14} fill="var(--muted)" fontSize="12">{d.label}</text>
          <Bar x={118} y={2} width={x(d.value)} height={14} fill="var(--accent)" />
          <text x={122 + x(d.value)} y={13} fill="var(--text)" fontSize="12">{d.value}</text>
        </Group>
      ))}
    </svg>
  );
}

export function AreaTrend({ data, unit = "turns" }: { data: { day: string; count: number }[]; unit?: string }) {
  const width = 520;
  const height = 200;
  const points = data.length ? data : [{ day: "—", count: 0 }];
  const max = Math.max(1, ...points.map((d) => d.count));
  const x = scaleLinear({ domain: [0, Math.max(1, points.length - 1)], range: [40, width - 12] });
  const y = scaleLinear({ domain: [0, max], range: [height - 28, 12] });
  const path = points.map((d, i) => `${i === 0 ? "M" : "L"} ${x(i)} ${y(d.count)}`).join(" ");
  return (
    <svg viewBox={`0 0 ${width} ${height}`} width="100%" height={height} role="img">
      <line x1={40} y1={12} x2={40} y2={height - 28} stroke="var(--line)" />
      <line x1={40} y1={height - 28} x2={width - 12} y2={height - 28} stroke="var(--line)" />
      <text x={4} y={16} fill="var(--muted)" fontSize="10">{max}</text>
      <text x={4} y={height - 28} fill="var(--muted)" fontSize="10">0</text>
      <text x={width - 12} y={height - 8} textAnchor="end" fill="var(--muted)" fontSize="10">{unit}</text>
      <path d={path} fill="none" stroke="var(--accent)" strokeWidth={1.5} />
      {points.map((d, i) => (
        <text key={d.day} x={x(i)} y={height - 10} textAnchor="middle" fill="var(--muted)" fontSize="9">{d.day.slice(-5)}</text>
      ))}
    </svg>
  );
}

export type MixRow = { day: string; execute: number; confirm: number; clarify: number; reject: number; chat: number };

const MIX_KEYS = ["execute", "confirm", "clarify", "reject", "chat"] as const;
const MIX_COLOR: Record<(typeof MIX_KEYS)[number], string> = {
  execute: "var(--high)",
  confirm: "var(--accent)",
  clarify: "var(--medium)",
  reject: "var(--danger)",
  chat: "var(--cyan)",
};

export function DecisionMix({ data, unit }: { data: MixRow[]; unit: string }) {
  const width = 520;
  const height = 220;
  const rows = data.length ? data : [{ day: "—", execute: 0, confirm: 0, clarify: 0, reject: 0, chat: 0 }];
  const x = scaleBand({ domain: rows.map((d) => d.day), range: [40, width - 12], padding: 0.25 });
  const max = Math.max(1, ...rows.map((d) => MIX_KEYS.reduce((sum, key) => sum + d[key], 0)));
  const y = scaleLinear({ domain: [0, max], range: [height - 36, 16] });
  return (
    <svg viewBox={`0 0 ${width} ${height}`} width="100%" height={height} role="img" aria-label="decision mix">
      <line x1={40} y1={16} x2={40} y2={height - 36} stroke="var(--line)" />
      <line x1={40} y1={height - 36} x2={width - 12} y2={height - 36} stroke="var(--line)" />
      <text x={4} y={20} fill="var(--muted)" fontSize="10">{max}</text>
      <text x={width - 12} y={12} textAnchor="end" fill="var(--muted)" fontSize="10">{unit}</text>
      {rows.map((row) => {
        let top = 0;
        return MIX_KEYS.map((key) => {
          const value = row[key];
          const y0 = y(top + value);
          const y1 = y(top);
          top += value;
          return (
            <Bar
              key={`${row.day}-${key}`}
              x={x(row.day) || 0}
              y={y0}
              width={x.bandwidth()}
              height={Math.max(0, y1 - y0)}
              fill={MIX_COLOR[key]}
            />
          );
        });
      })}
      {rows.map((d) => (
        <text key={d.day} x={(x(d.day) || 0) + x.bandwidth() / 2} y={height - 16} textAnchor="middle" fill="var(--muted)" fontSize="9">{d.day.slice(-5)}</text>
      ))}
    </svg>
  );
}

export function StageBars({ data, unit }: { data: { label: string; value: number }[]; unit: string }) {
  return <Bars data={data} unit={unit} />;
}
