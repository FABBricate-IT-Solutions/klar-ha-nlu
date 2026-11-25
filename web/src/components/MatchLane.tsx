import { useState } from "react";
import { OriginChip } from "./OriginChip";
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
  /** Draft text while the operator edits; empty is allowed (U-24). */
  const [drafts, setDrafts] = useState<Record<string, string>>({});

  return (
    <>
      <div className="policy-lane-head">
        <h2>{t.laneMatchEngine}</h2>
        <OriginChip t={t} origin="engine" />
      </div>
      <div className="lane-body">
      <p className="caption">{t.matchReadOnly}</p>
      {catalog.map((row, index) => {
        const overlay = controls.find((item) => item.id === row.id);
        const enabled = overlay?.enabled ?? true;
        const precedence = overlay?.precedence ?? row.precedence;
        const draft = drafts[row.id];
        return (
          <div
            className={`lane-row match-row${index === selected ? " active" : ""}`}
            key={row.id}
            onClick={() => onSelect(index)}
          >
            <label className="row match-toggle" onClick={(ev) => ev.stopPropagation()}>
              <input
                type="checkbox"
                checked={enabled}
                onChange={(ev) => onChange(upsertMatchControl(controls, catalog, row.id, { enabled: ev.target.checked }))}
              />
              {enabled ? t.matchEnabled : t.matchDisabled}
            </label>
            <strong className="mono">{row.id}</strong>
            <input
              className="match-precedence"
              type="number"
              min={0}
              aria-label={t.matchPrecedence}
              value={draft ?? String(precedence)}
              onClick={(ev) => ev.stopPropagation()}
              onChange={(ev) => {
                const text = ev.target.value;
                setDrafts((prev) => ({ ...prev, [row.id]: text }));
                if (text === "") {
                  onChange(upsertMatchControl(controls, catalog, row.id, { precedence: undefined }));
                  return;
                }
                const value = Number(text);
                if (Number.isFinite(value) && value >= 0) {
                  onChange(upsertMatchControl(controls, catalog, row.id, { precedence: value }));
                }
              }}
              onBlur={() => {
                setDrafts((prev) => {
                  const next = { ...prev };
                  delete next[row.id];
                  return next;
                });
              }}
            />
          </div>
        );
      })}
      </div>
    </>
  );
}
