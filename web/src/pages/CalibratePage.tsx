import type { Messages } from "../i18n";
import { api } from "../api";
import type { Assignment, Dashboard, UiState } from "../types";

export function CalibratePage({
  data,
  ui,
  t,
  onUi,
  onRefresh,
  onInspect,
  onApply,
}: {
  data: Dashboard;
  ui: UiState;
  t: Messages;
  onUi: (ui: UiState) => void;
  onRefresh: () => void;
  onInspect: (row: Assignment) => void;
  onApply: () => void;
}) {
  const inbox = data.assignment.filter((row) => row.confidence !== "high" && !ui.dismissed.includes(row.entity_id));
  const accept = async (row: Assignment, area = row.suggested_area?.area_id || "") => {
    await api.tagEntity({ entity_id: row.entity_id, aliases: row.aliases, preferred: row.tags.includes("preferred"), area });
    onRefresh();
  };
  const dismiss = (row: Assignment) => {
    onUi({ ...ui, dismissed: [...new Set([...ui.dismissed, row.entity_id])] });
    onRefresh();
  };

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.calibrate}</h1>
          <p className="muted">{inbox.length ? `${inbox.length} ${t.open}` : t.noGaps}</p>
          <p className="muted">{t.mappingHint}</p>
        </div>
        {inbox.some((row) => (row.suggested_area?.score || 0) >= 3) && <button className="primary" onClick={onApply}>{t.applyAll}</button>}
      </section>
      <section className="grid">
        {inbox.length === 0 && <div className="card hot"><h2>{t.noGaps}</h2></div>}
        {inbox.map((row) => (
          <article className="inbox-card" key={row.entity_id}>
            <div>
              <h2>{row.name}</h2>
              <p className="mono">{row.entity_id}</p>
              <p className={`conf-${row.confidence}`}>{t[row.confidence]} · {row.reasons.join(", ")}</p>
              {row.suggested_area && <p>{row.suggested_area.name} · {t.score} {row.suggested_area.score}</p>}
            </div>
            <div className="grid" style={{ minWidth: 180 }}>
              {row.suggested_area && <button className="primary" onClick={() => accept(row)}>{t.accept}</button>}
              <select value={row.area || ""} onChange={(ev) => ev.target.value && accept(row, ev.target.value)}>
                <option value="">{t.otherRoom}</option>
                {data.rooms.map((room) => <option value={room.area_id} key={room.area_id}>{room.name}</option>)}
              </select>
              <button className="ghost" onClick={() => dismiss(row)}>{t.dismiss}</button>
              <button className="ghost" onClick={() => onInspect(row)}>{t.entities}</button>
            </div>
          </article>
        ))}
      </section>
    </div>
  );
}
