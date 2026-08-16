import { LinearGradient } from "@visx/gradient";
import { Group } from "@visx/group";
import { scaleLinear } from "@visx/scale";
import { AreaClosed, Bar, Pie } from "@visx/shape";

type Slice = { label: string; value: number; color: string };

export function Donut({ high, medium, low }: { high: number; medium: number; low: number }) {
  const data: Slice[] = [
    { label: "high", value: high, color: "var(--high)" },
    { label: "medium", value: medium, color: "var(--medium)" },
    { label: "low", value: low, color: "var(--low)" },
  ].filter((item) => item.value > 0);
  const total = high + medium + low || 1;
  return (
    <svg viewBox="0 0 240 220" width="100%" height="220" role="img">
      <filter id="glow"><feGaussianBlur stdDeviation="4" result="b" /><feMerge><feMergeNode in="b" /><feMergeNode in="SourceGraphic" /></feMerge></filter>
      <Group top={110} left={120}>
        <Pie data={data} pieValue={(d) => d.value} outerRadius={92} innerRadius={62} padAngle={0.025}>
          {(pie) => pie.arcs.map((arc) => (
            <path key={arc.data.label} d={pie.path(arc) || ""} fill={arc.data.color} filter={arc.data.label === "high" ? "url(#glow)" : undefined} />
          ))}
        </Pie>
        <text textAnchor="middle" y="-4" fill="var(--text)" fontFamily="Fraunces" fontSize="38">{Math.round((high / total) * 100)}%</text>
        <text textAnchor="middle" y="22" fill="var(--muted)" fontSize="12">ready</text>
      </Group>
    </svg>
  );
}

export function Bars({ data }: { data: { label: string; value: number }[] }) {
  const width = 420;
  const height = Math.max(130, data.length * 28);
  const max = Math.max(1, ...data.map((d) => d.value));
  const x = scaleLinear({ domain: [0, max], range: [0, width - 120] });
  return (
    <svg viewBox={`0 0 ${width} ${height}`} width="100%" height={height}>
      {data.map((d, i) => (
        <Group key={d.label} top={i * 28 + 8}>
          <text x={0} y={14} fill="var(--muted)" fontSize="12">{d.label}</text>
          <Bar x={118} y={0} width={x(d.value)} height={16} rx={8} fill="url(#barGlow)" />
          <text x={122 + x(d.value)} y={13} fill="var(--text)" fontSize="12">{d.value}</text>
        </Group>
      ))}
      <LinearGradient id="barGlow" from="var(--accent)" to="var(--glow)" />
    </svg>
  );
}

export function AreaTrend({ data }: { data: { day: string; count: number }[] }) {
  const width = 520;
  const height = 180;
  const points = data.length ? data : [{ day: "0", count: 0 }];
  const max = Math.max(1, ...points.map((d) => d.count));
  const x = scaleLinear({ domain: [0, Math.max(1, points.length - 1)], range: [10, width - 10] });
  const y = scaleLinear({ domain: [0, max], range: [height - 18, 18] });
  return (
    <svg viewBox={`0 0 ${width} ${height}`} width="100%" height={height}>
      <LinearGradient id="areaGlow" from="var(--accent)" to="transparent" vertical />
      <AreaClosed
        data={points}
        x={(_, i) => x(i)}
        y={(d) => y(d.count)}
        yScale={y}
        fill="url(#areaGlow)"
        stroke="var(--glow)"
        strokeWidth={2}
      />
    </svg>
  );
}
