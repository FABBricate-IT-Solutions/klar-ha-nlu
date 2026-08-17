import { useState } from "react";
import type { Messages } from "../i18n";
import type { Assignment, Dashboard, HouseView, UiState } from "../types";
import { CalibratePage } from "./CalibratePage";
import { EntitiesPage } from "./EntitiesPage";
import { GraphPage } from "./GraphPage";

export function HousePage({
  data,
  ui,
  t,
  onUi,
  onInspect,
  onRefresh,
  onApply,
}: {
  data: Dashboard;
  ui: UiState;
  t: Messages;
  onUi: (ui: UiState) => void;
  onInspect: (row: Assignment) => void;
  onRefresh: () => void;
  onApply: () => void;
}) {
  const [view, setView] = useState<HouseView>("graph");
  return (
    <div>
      <div className="page" style={{ paddingBottom: 0 }}>
        <nav className="subnav">
          {(["graph", "entities", "calibrate"] as HouseView[]).map((item) => (
            <button key={item} className={view === item ? "active" : ""} onClick={() => setView(item)}>
              {item === "graph" ? t.houseGraph : item === "entities" ? t.houseDevices : t.houseMap}
            </button>
          ))}
        </nav>
      </div>
      {view === "graph" && <GraphPage data={data} ui={ui} t={t} onUi={onUi} onInspect={onInspect} />}
      {view === "entities" && <EntitiesPage data={data} t={t} onInspect={onInspect} />}
      {view === "calibrate" && (
        <CalibratePage data={data} ui={ui} t={t} onUi={onUi} onRefresh={onRefresh} onInspect={onInspect} onApply={onApply} />
      )}
    </div>
  );
}
