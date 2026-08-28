import { useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { Assignment, Dashboard, UiState } from "../types";
import { GraphCanvas } from "./GraphCanvas";
import { GraphList } from "./GraphList";
import { flattenHouse, groupHouse, matchesAssignment } from "./graphModel";

export function GraphPage({
  data,
  ui,
  t,
  onUi,
  onInspect,
  activeId,
}: {
  data: Dashboard;
  ui: UiState;
  t: Messages;
  onUi: (ui: UiState) => void;
  onInspect: (row: Assignment) => void;
  activeId?: string;
}) {
  const [query, setQuery] = useState("");
  const [areaFloor, setAreaFloor] = useState<Record<string, string>>({});
  const [cursorId, setCursorId] = useState(activeId || "");
  const searchRef = useRef<HTMLInputElement>(null);
  const uiRef = useRef(ui);
  uiRef.current = ui;

  useEffect(() => {
    if (activeId) setCursorId(activeId);
  }, [activeId]);

  useEffect(() => {
    if ((data.floors ?? []).length === 0) return;
    let live = true;
    api
      .gaps()
      .then((gaps) => {
        if (!live) return;
        const next: Record<string, string> = {};
        for (const room of gaps.rooms) {
          if (room.floor_id) next[room.area_id] = room.floor_id;
        }
        setAreaFloor(next);
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [data.floors]);

  const filtered = useMemo(
    () => data.assignment.filter((row) => matchesAssignment(row, query, [t[row.confidence]])),
    [data.assignment, query, t],
  );
  const mapTree = useMemo(() => groupHouse(data.assignment, data, areaFloor), [data, areaFloor]);
  const listTree = useMemo(() => groupHouse(filtered, data, areaFloor), [filtered, data, areaFloor]);
  const listRows = useMemo(() => flattenHouse(listTree), [listTree]);

  useEffect(() => {
    if (listRows.some((row) => row.entity_id === cursorId)) return;
    setCursorId(listRows[0]?.entity_id || "");
  }, [cursorId, listRows]);

  const moveNode = (entityId: string, x: number, y: number) => {
    const current = uiRef.current;
    onUi({ ...current, graph: { ...current.graph, [entityId]: { x, y } } });
  };

  return (
    <div className="page" style={{ minWidth: 0 }}>
      <style>{graphCss}</style>
      <section className="hero">
        <div>
          <h1>{t.graph}</h1>
          <p className="muted">{t.graphHint}</p>
        </div>
        <button type="button" className="secondary" onClick={() => onUi({ ...ui, graph: {} })}>
          {t.resetLayout}
        </button>
      </section>
      <div className="graph-split">
        <GraphCanvas
          tree={mapTree}
          ui={ui}
          t={t}
          query={query}
          cursorId={cursorId}
          onInspect={onInspect}
          onMove={moveNode}
        />
        <GraphList
          tree={listTree}
          rows={listRows}
          t={t}
          query={query}
          cursorId={cursorId}
          searchRef={searchRef}
          onQuery={setQuery}
          onCursor={setCursorId}
          onInspect={onInspect}
        />
      </div>
    </div>
  );
}

const graphCss = `
.graph-split {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(220px, 320px);
  gap: 16px;
  align-items: start;
  min-width: 0;
}
.graph-split .graph-canvas,
.graph-split .card {
  border-radius: 0;
}
@media (max-width: 860px) {
  .graph-split { grid-template-columns: 1fr; }
}
`;
