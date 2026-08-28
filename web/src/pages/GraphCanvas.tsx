import type { PointerEvent, ReactNode } from "react";
import type { Messages } from "../i18n";
import type { Assignment, UiState } from "../types";
import { confidenceColor, type HouseTree, type RoomBlock } from "./graphModel";

type GraphCanvasProps = {
  tree: HouseTree;
  ui: UiState;
  t: Messages;
  query: string;
  cursorId?: string;
  onInspect: (row: Assignment) => void;
  onMove: (entityId: string, x: number, y: number) => void;
};

function isInspectControl(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && Boolean(target.closest("button"));
}

function DeviceNode({
  row,
  t,
  room,
  query,
  cursorId,
  onInspect,
  onMove,
}: {
  row: Assignment;
  t: Messages;
  room: string;
  query: string;
  cursorId?: string;
  onInspect: (row: Assignment) => void;
  onMove: (entityId: string, x: number, y: number) => void;
}) {
  const match = !query.trim()
    || [row.name, row.entity_id, row.area || "", row.aliases.join(" "), t[row.confidence]].join(" ").toLowerCase().includes(query.trim().toLowerCase());
  const active = row.entity_id === cursorId;
  const drag = (ev: PointerEvent<HTMLDivElement>) => {
    if (isInspectControl(ev.target)) return;
    const node = ev.currentTarget;
    const canvas = node.closest("[data-graph-canvas]");
    if (!(canvas instanceof HTMLElement)) return;
    ev.preventDefault();
    node.setPointerCapture(ev.pointerId);
    const canvasRect = canvas.getBoundingClientRect();
    const nodeRect = node.getBoundingClientRect();
    const originX = nodeRect.left - canvasRect.left + canvas.scrollLeft;
    const originY = nodeRect.top - canvasRect.top + canvas.scrollTop;
    const startX = ev.clientX;
    const startY = ev.clientY;
    let moved = false;
    const move = (next: globalThis.PointerEvent) => {
      const dx = next.clientX - startX;
      const dy = next.clientY - startY;
      if (!moved && dx * dx + dy * dy < 16) return;
      moved = true;
      onMove(row.entity_id, Math.max(8, originX + dx), Math.max(8, originY + dy));
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <div
      className="node"
      data-entity={row.entity_id}
      onPointerDown={drag}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        minHeight: 44,
        minWidth: 44,
        padding: "6px 8px",
        background: "var(--surface-2)",
        border: `1px solid ${active ? "var(--accent)" : confidenceColor(row.confidence)}`,
        borderRadius: 0,
        opacity: match ? 1 : 0.38,
        color: "var(--text)",
      }}
    >
      <span
        aria-hidden="true"
        style={{
          width: 10,
          height: 10,
          flex: "0 0 10px",
          background: confidenceColor(row.confidence),
        }}
      />
      <div style={{ minWidth: 0, flex: 1 }}>
        <div>{row.name}</div>
        <div className="mono">{row.entity_id}</div>
        <p className="muted" style={{ margin: 0 }}>
          {room}
          {row.tags.includes("preferred") ? ` · ${t.preferred}` : ""}
          {` · ${t[row.confidence]}`}
        </p>
      </div>
      <button type="button" className="ghost" style={{ minHeight: 44, minWidth: 44 }} onClick={() => onInspect(row)}>
        {t.entities}
      </button>
    </div>
  );
}

function RoomCluster({
  block,
  t,
  query,
  cursorId,
  placed,
  onInspect,
  onMove,
}: {
  block: RoomBlock;
  t: Messages;
  query: string;
  cursorId?: string;
  placed: Set<string>;
  onInspect: (row: Assignment) => void;
  onMove: (entityId: string, x: number, y: number) => void;
}) {
  const clustered = block.rows.filter((row) => !placed.has(row.entity_id));
  return (
    <article
      className="card"
      style={{
        flex: "1 1 220px",
        minWidth: 0,
        maxWidth: "100%",
        display: "grid",
        gap: 8,
      }}
    >
      <div>
        <h3>{block.name}</h3>
        <p className="muted" style={{ margin: "4px 0 0" }}>{block.inbox} {t.open}</p>
      </div>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
        {clustered.map((row) => (
          <DeviceNode
            key={row.entity_id}
            row={row}
            t={t}
            room={block.name}
            query={query}
            cursorId={cursorId}
            onInspect={onInspect}
            onMove={onMove}
          />
        ))}
      </div>
    </article>
  );
}

function FloorSection({
  title,
  rooms,
  t,
  query,
  cursorId,
  placed,
  onInspect,
  onMove,
}: {
  title: string;
  rooms: RoomBlock[];
  t: Messages;
  query: string;
  cursorId?: string;
  placed: Set<string>;
  onInspect: (row: Assignment) => void;
  onMove: (entityId: string, x: number, y: number) => void;
}) {
  return (
    <section style={{ display: "grid", gap: 12, minWidth: 0 }}>
      <h2>{title}</h2>
      <div style={{ display: "flex", flexWrap: "wrap", gap: 12, minWidth: 0 }}>
        {rooms.map((block) => (
          <RoomCluster
            key={block.area_id}
            block={block}
            t={t}
            query={query}
            cursorId={cursorId}
            placed={placed}
            onInspect={onInspect}
            onMove={onMove}
          />
        ))}
      </div>
    </section>
  );
}

export function GraphCanvas({ tree, ui, t, query, cursorId, onInspect, onMove }: GraphCanvasProps) {
  const placed = new Set(Object.keys(ui.graph));
  const overlay: { row: Assignment; x: number; y: number; room: string }[] = [];
  const walk = (rows: Assignment[], room: string) => {
    for (const row of rows) {
      const point = ui.graph[row.entity_id];
      if (point) overlay.push({ row, x: point.x, y: point.y, room });
    }
  };
  for (const floor of tree.floors) {
    for (const room of floor.rooms) walk(room.rows, room.name);
  }
  for (const room of tree.loose) walk(room.rows, room.name);
  walk(tree.unmapped, t.unmapped);
  const freeW = overlay.reduce((width, node) => Math.max(width, node.x + 280), 0);
  const freeH = overlay.reduce((height, node) => Math.max(height, node.y + 88), 0);

  let clusters: ReactNode = null;
  if (tree.floors.length > 0) {
    clusters = (
      <>
        {tree.floors.map((floor) => (
          <FloorSection
            key={floor.floor_id}
            title={floor.name}
            rooms={floor.rooms}
            t={t}
            query={query}
            cursorId={cursorId}
            placed={placed}
            onInspect={onInspect}
            onMove={onMove}
          />
        ))}
        {tree.loose.length > 0 && (
          <FloorSection
            title={t.rooms}
            rooms={tree.loose}
            t={t}
            query={query}
            cursorId={cursorId}
            placed={placed}
            onInspect={onInspect}
            onMove={onMove}
          />
        )}
      </>
    );
  } else {
    clusters = tree.loose.map((block) => (
      <RoomCluster
        key={block.area_id}
        block={block}
        t={t}
        query={query}
        cursorId={cursorId}
        placed={placed}
        onInspect={onInspect}
        onMove={onMove}
      />
    ));
  }

  return (
    <div
      data-graph-canvas
      className="card graph-canvas"
      style={{
        position: "relative",
        width: "100%",
        minWidth: 0,
        maxWidth: "100%",
        minHeight: 280,
        overflow: "auto",
      }}
    >
      <div style={{ display: "grid", gap: 20, minWidth: 0, position: "relative", zIndex: 0 }}>
        {clusters}
        {tree.unmapped.length > 0 && (
          <section style={{ display: "grid", gap: 12, minWidth: 0 }}>
            <h3>{t.unmapped}</h3>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
              {tree.unmapped.filter((row) => !placed.has(row.entity_id)).map((row) => (
                <DeviceNode
                  key={row.entity_id}
                  row={row}
                  t={t}
                  room={t.unmapped}
                  query={query}
                  cursorId={cursorId}
                  onInspect={onInspect}
                  onMove={onMove}
                />
              ))}
            </div>
          </section>
        )}
      </div>
      {overlay.length > 0 && (
        <div
          style={{
            position: "absolute",
            left: 0,
            top: 0,
            width: freeW,
            height: freeH,
            pointerEvents: "none",
            zIndex: 1,
          }}
        >
          {overlay.map(({ row, x, y, room }) => (
            <div key={row.entity_id} style={{ position: "absolute", left: x, top: y, pointerEvents: "auto" }}>
              <DeviceNode
                row={row}
                t={t}
                room={room}
                query={query}
                cursorId={cursorId}
                onInspect={onInspect}
                onMove={onMove}
              />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
