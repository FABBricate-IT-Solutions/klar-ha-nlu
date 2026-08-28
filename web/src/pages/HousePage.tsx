import { useEffect, useRef, useState } from "react";
import { InspectDrawer } from "../components/InspectDrawer";
import type { Messages } from "../i18n";
import type { Assignment, Dashboard, HouseView, UiState } from "../types";
import { CalibratePage } from "./CalibratePage";
import { EntitiesPage } from "./EntitiesPage";
import { GraphPage } from "./GraphPage";

const views: HouseView[] = ["graph", "entities", "calibrate"];

function viewLabel(view: HouseView, t: Messages): string {
  switch (view) {
    case "graph":
      return t.houseGraph;
    case "entities":
      return t.houseDevices;
    case "calibrate":
      return t.houseMap;
    default: {
      const _never: never = view;
      return _never;
    }
  }
}

function deviceIdFromHash(hash = window.location.hash): string {
  const parts = hash.replace(/^#/, "").replace(/^\//, "").split("/").filter(Boolean);
  if (parts[0] !== "house" || parts[1] !== "devices" || !parts[2]) return "";
  return parts.slice(2).map((part) => decodeURIComponent(part)).join("/");
}

export function HousePage({
  data,
  ui,
  t,
  onUi,
  onInspect,
  onRefresh,
  onApply,
  houseView,
  onHouseView,
  inspectId,
}: {
  data: Dashboard;
  ui: UiState;
  t: Messages;
  onUi: (ui: UiState) => void;
  onInspect: (row: Assignment | null) => void;
  onRefresh: () => void;
  onApply: () => void;
  houseView?: HouseView;
  onHouseView?: (view: HouseView) => void;
  inspectId?: string;
}) {
  const view = houseView ?? ui.house_view ?? "calibrate";
  const [inspecting, setInspecting] = useState<Assignment | null>(null);
  const assignmentRef = useRef(data.assignment);
  assignmentRef.current = data.assignment;
  const openInspect = (row: Assignment) => {
    setInspecting(row);
    onInspect(row);
  };
  const closeInspect = () => {
    setInspecting(null);
    onInspect(null);
  };
  const live = inspecting
    ? data.assignment.find((row) => row.entity_id === inspecting.entity_id) || inspecting
    : null;
  const activeId = inspectId || live?.entity_id;

  useEffect(() => {
    const id = inspectId || deviceIdFromHash();
    if (!id) {
      setInspecting(null);
      return;
    }
    const row = assignmentRef.current.find((item) => item.entity_id === id);
    if (row) setInspecting(row);
  }, [inspectId]);

  useEffect(() => {
    const applyHash = () => {
      const id = deviceIdFromHash();
      if (!id) return;
      const row = assignmentRef.current.find((item) => item.entity_id === id);
      if (row) setInspecting(row);
    };
    applyHash();
    window.addEventListener("hashchange", applyHash);
    return () => window.removeEventListener("hashchange", applyHash);
  }, []);

  return (
    <div>
      <div className="page" style={{ paddingBottom: 0 }}>
        <nav className="subnav" aria-label={t.house}>
          {views.map((item) => (
            <button
              key={item}
              type="button"
              className={view === item ? "active" : ""}
              aria-current={view === item ? "page" : undefined}
              onClick={() => (onHouseView ? onHouseView(item) : onUi({ ...ui, house_view: item }))}
            >
              {viewLabel(item, t)}
            </button>
          ))}
        </nav>
      </div>
      {view === "graph" && <GraphPage data={data} ui={ui} t={t} onUi={onUi} onInspect={openInspect} />}
      {view === "entities" && (
        <EntitiesPage data={data} t={t} onInspect={openInspect} activeId={activeId} />
      )}
      {view === "calibrate" && (
        <CalibratePage data={data} ui={ui} t={t} onUi={onUi} onRefresh={onRefresh} onInspect={openInspect} onApply={onApply} />
      )}
      {live && (
        <InspectDrawer
          row={live}
          rooms={data.rooms}
          t={t}
          onClose={closeInspect}
          onSaved={() => {
            closeInspect();
            onRefresh();
          }}
          onDismiss={(row) => {
            onUi({ ...ui, dismissed: [...new Set([...ui.dismissed, row.entity_id])] });
            closeInspect();
          }}
        />
      )}
    </div>
  );
}
