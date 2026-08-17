import type { Messages } from "../i18n";
import type { Assignment, Dashboard, UiState } from "../types";

const color = (c: string) => c === "high" ? "var(--high)" : c === "medium" ? "var(--medium)" : "var(--low)";
const roomColumn = 24;
const nodeColumn = 280;
const nodeGapX = 270;
const nodeGapY = 88;
const nodeLabelWidth = 220;
const rowPad = 44;

const clamp = (value: number, min: number, max: number) => Math.max(min, Math.min(max, value));

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
  const areas = [
    ...data.rooms.map((room) => ({ id: room.area_id, name: room.name, inbox: room.inbox })),
    ...(data.assignment.some((row) => !row.area) ? [{ id: "_unmapped", name: t.unmapped, inbox: 0 }] : []),
  ];
  const byArea = Object.fromEntries(areas.map((area) => [area.id, data.assignment.filter((row) => (row.area || "_unmapped") === area.id)]));
  let cursor = 64;
  const roomY: Record<string, number> = {};
  const roomTop: Record<string, number> = {};
  for (const area of areas) {
    const count = Math.max(1, byArea[area.id].length);
    const rows = Math.ceil(count / 3);
    const block = Math.max(108, rows * nodeGapY + rowPad);
    roomTop[area.id] = cursor;
    roomY[area.id] = cursor + block / 2 - 8;
    cursor += block + 24;
  }
  const width = Math.max(1180, nodeColumn + 3 * nodeGapX + nodeLabelWidth);
  const height = Math.max(620, cursor + 20);
  const nodes = data.assignment.map((row) => {
    const area = row.area || "_unmapped";
    const siblings = byArea[area] || [];
    const index = Math.max(0, siblings.findIndex((item) => item.entity_id === row.entity_id));
    const auto = {
      x: nodeColumn + (index % 3) * nodeGapX,
      y: roomTop[area] + 34 + Math.floor(index / 3) * nodeGapY,
    };
    const saved = ui.graph[row.entity_id];
    return {
      row,
      x: clamp(saved?.x ?? auto.x, nodeColumn - 40, width - nodeLabelWidth),
      y: clamp(saved?.y ?? auto.y, 48, height - 42),
    };
  });

  const drag = (row: Assignment, ev: React.PointerEvent<SVGGElement>) => {
    const target = ev.currentTarget;
    target.setPointerCapture(ev.pointerId);
    const rect = target.ownerSVGElement!.getBoundingClientRect();
    const move = (next: PointerEvent) => {
      const x = clamp(((next.clientX - rect.left) / rect.width) * width, nodeColumn - 40, width - nodeLabelWidth);
      const y = clamp(((next.clientY - rect.top) / rect.height) * height, 48, height - 42);
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
        <svg viewBox={`0 0 ${width} ${height}`} width={width} height={height}>
          {areas.map((room) => (
            <g key={room.id}>
              <rect x={roomColumn} y={roomY[room.id] - 34} width={188} height={68} fill="var(--surface)" stroke="var(--line)" />
              <text x={roomColumn + 20} y={roomY[room.id] - 6} fill="var(--text)" fontSize="16">{room.name}</text>
              <text x={roomColumn + 20} y={roomY[room.id] + 18} fill="var(--muted)" fontSize="12">{room.inbox} {t.open}</text>
            </g>
          ))}
          {nodes.map(({ row, x, y }) => row.area && (
            <line key={`${row.entity_id}-area`} x1={roomColumn + 188} y1={roomY[row.area] || 30} x2={x} y2={y} stroke="var(--line)" strokeDasharray="5 6" />
          ))}
          {nodes.map(({ row, x, y }) => row.suggested_area && (
            <line key={`${row.entity_id}-suggest`} x1={roomColumn + 188} y1={roomY[row.suggested_area.area_id] || 30} x2={x} y2={y} stroke="var(--accent)" strokeDasharray="7 7" opacity=".62" />
          ))}
          {nodes.map(({ row, x, y }) => (
            <g className="node" key={row.entity_id} transform={`translate(${x} ${y})`} onPointerDown={(ev) => drag(row, ev)} onDoubleClick={() => onInspect(row)}>
              <circle r={22} fill="var(--surface-2)" stroke={color(row.confidence)} strokeWidth={2} />
              <text x={42} y="-4" fill="var(--text)" fontSize="14">{row.name}</text>
              <text x={42} y="16" fill="var(--muted)" fontSize="11">{row.entity_id}</text>
            </g>
          ))}
        </svg>
      </div>
    </div>
  );
}
