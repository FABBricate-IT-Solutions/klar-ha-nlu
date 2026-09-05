import type { Messages } from "../i18n";
import type { MatchCatalogRow, MatchControl } from "../types";

export function upsertMatchControl(
  rows: MatchControl[],
  catalog: MatchCatalogRow[],
  id: string,
  patch: { enabled?: boolean; precedence?: number | undefined },
): MatchControl[] {
  const defaults = catalog.find((row) => row.id === id);
  const current = rows.find((row) => row.id === id);
  const enabled = patch.enabled ?? current?.enabled ?? true;
  const precedence = Object.prototype.hasOwnProperty.call(patch, "precedence") ? patch.precedence : current?.precedence;
  const matchesDefault = enabled && (precedence === undefined || precedence === defaults?.precedence);
  const without = rows.filter((row) => row.id !== id);
  if (matchesDefault) {
    return without;
  }
  return [...without, { id, enabled, precedence: precedence === defaults?.precedence ? undefined : precedence }];
}

export function MatchLane({
  t,
  catalog,
  controls,
  selected,
  onSelect,
  onChange,
}: {
  t: Messages;
  catalog: MatchCatalogRow[];
  controls: MatchControl[];
  selected: number;
  onSelect: (index: number) => void;
  onChange: (next: MatchControl[]) => void;
}) {
  return (
    <>
      <h2>{t.matchCatalog}</h2>
      <p className="caption">{t.matchReadOnly}</p>
      {catalog.map((row, index) => {
        const overlay = controls.find((item) => item.id === row.id);
        const enabled = overlay?.enabled ?? true;
        const precedence = overlay?.precedence ?? row.precedence;
        return (
          <div
            className={`rule-row${index === selected ? " active" : ""}`}
            key={row.id}
            onClick={() => onSelect(index)}
          >
            <span className="chip origin">{t.originEngine}</span>
            <strong className="mono">{row.id}</strong>
            <label className="row match-toggle" onClick={(ev) => ev.stopPropagation()}>
              <input
                type="checkbox"
                checked={enabled}
                onChange={(ev) => onChange(upsertMatchControl(controls, catalog, row.id, { enabled: ev.target.checked }))}
                style={{ width: "auto" }}
              />
              {enabled ? t.matchEnabled : t.matchDisabled}
            </label>
            <input
              className="match-precedence"
              type="number"
              min={0}
              aria-label={t.matchPrecedence}
              value={precedence}
              onClick={(ev) => ev.stopPropagation()}
              onChange={(ev) => {
                const value = ev.target.value === "" ? undefined : Number(ev.target.value);
                onChange(upsertMatchControl(controls, catalog, row.id, { precedence: Number.isFinite(value) ? value : undefined }));
              }}
            />
          </div>
        );
      })}
    </>
  );
}
