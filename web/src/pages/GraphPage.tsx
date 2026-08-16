import type { Messages } from "../i18n";
import type { Assignment, Dashboard, UiState } from "../types";

const color = (c: string) => c === "high" ? "var(--high)" : c === "medium" ? "var(--medium)" : "var(--low)";

export function GraphPage({
  data,
  ui,
  t,
  onUi,
  onInspect,
}: {
  data: Dashboard;
  ui: UiState;
  t: Messages;
  onUi: (ui: UiState) => void;
  onInspect: (row: Assignment) => void;
}) {
  const width = 1100;
  const height = 620;
  const nodes = data.assignment.map((row, i) => ({
    row,
    x: ui.graph[row.entity_id]?.x ?? 90 + (i % 5) * 210,
    y: ui.graph[row.entity_id]?.y ?? 90 + Math.floor(i / 5) * 115,
  }));
  const roomY = Object.fromEntries(data.rooms.map((room, i) => [room.area_id, 70 + i * 88]));

  const drag = (row: Assignment, ev: React.PointerEvent<SVGGElement>) => {
    const target = ev.currentTarget;
    target.setPointerCapture(ev.pointerId);
    const rect = target.ownerSVGElement!.getBoundingClientRect();
    const move = (next: PointerEvent) => {
      const x = ((next.clientX - rect.left) / rect.width) * width;
      const y = ((next.clientY - rect.top) / rect.height) * height;
      onUi({ ...ui, graph: { ...ui.graph, [row.entity_id]: { x, y } } });
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.graph}</h1>
          <p className="muted">{t.graphHint}</p>
        </div>
        <button className="secondary" onClick={() => onUi({ ...ui, graph: {} })}>{t.resetLayout}</button>
      </section>
      <div className="card graph-canvas">
        <svg viewBox={`0 0 ${width} ${height}`} width="100%" height="100%">
          {data.rooms.map((room, i) => (
            <g key={room.area_id}>
              <rect x={22} y={roomY[room.area_id] - 32} width={164} height={62} rx={10} fill="#15110e" stroke="var(--line)" />
              <text x={42} y={roomY[room.area_id] - 4} fill="var(--text)" fontSize="16">{room.name}</text>
              <text x={42} y={roomY[room.area_id] + 18} fill="var(--muted)" fontSize="12">{room.inbox} {t.open}</text>
            </g>
          ))}
          {nodes.map(({ row, x, y }) => row.area && (
            <line key={`${row.entity_id}-area`} x1={186} y1={roomY[row.area] || 30} x2={x} y2={y} stroke="var(--line)" strokeDasharray="5 6" />
          ))}
          {nodes.map(({ row, x, y }) => row.suggested_area && (
            <line key={`${row.entity_id}-suggest`} x1={186} y1={roomY[row.suggested_area.area_id] || 30} x2={x} y2={y} stroke="var(--accent)" strokeDasharray="7 7" opacity=".62" />
          ))}
          {nodes.map(({ row, x, y }) => (
            <g className="node" key={row.entity_id} transform={`translate(${x} ${y})`} onPointerDown={(ev) => drag(row, ev)} onDoubleClick={() => onInspect(row)}>
              <circle r={28} fill="#17130f" stroke={color(row.confidence)} strokeWidth={3} />
              <text x={42} y="-4" fill="var(--text)" fontSize="14">{row.name}</text>
              <text x={42} y="16" fill="var(--muted)" fontSize="11">{row.entity_id}</text>
            </g>
          ))}
        </svg>
      </div>
    </div>
  );
}
