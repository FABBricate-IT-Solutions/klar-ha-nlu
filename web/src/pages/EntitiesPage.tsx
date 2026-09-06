import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { Assignment, Dashboard } from "../types";

type RoomBlock = { area_id: string; name: string; rows: Assignment[] };
type FloorBlock = { floor_id: string; name: string; rooms: RoomBlock[] };

function floorOf(areaId: string, mapped: Record<string, string>, rooms: Dashboard["rooms"]): string {
  if (mapped[areaId]) return mapped[areaId];
  const extra = rooms.find((room) => room.area_id === areaId) as { floor_id?: string | null } | undefined;
  return extra?.floor_id || "";
}

function roomsFor(rows: Assignment[], data: Dashboard): RoomBlock[] {
  const buckets = new Map<string, Assignment[]>();
  for (const row of rows) {
    if (!row.area) continue;
    const list = buckets.get(row.area) || [];
    list.push(row);
    buckets.set(row.area, list);
  }
  const known = data.rooms
    .filter((room) => buckets.has(room.area_id))
    .map((room) => ({ area_id: room.area_id, name: room.name, rows: buckets.get(room.area_id) || [] }));
  const extras = [...buckets.entries()]
    .filter(([id]) => !data.rooms.some((room) => room.area_id === id))
    .map(([area_id, block]) => ({ area_id, name: area_id, rows: block }));
  return [...known, ...extras];
}

function groupByFloor(rows: Assignment[], data: Dashboard, mapped: Record<string, string>): {
  floors: FloorBlock[];
  loose: RoomBlock[];
  unmapped: Assignment[];
} {
  const unmapped = rows.filter((row) => !row.area);
  const roomBlocks = roomsFor(rows, data);
  const floors = data.floors ?? [];
  if (floors.length === 0) {
    return { floors: [], loose: roomBlocks, unmapped };
  }
  const byFloor = new Map<string, RoomBlock[]>();
  const loose: RoomBlock[] = [];
  for (const block of roomBlocks) {
    const id = floorOf(block.area_id, mapped, data.rooms);
    if (id && floors.some((floor) => floor.floor_id === id)) {
      const list = byFloor.get(id) || [];
      list.push(block);
      byFloor.set(id, list);
    } else {
      loose.push(block);
    }
  }
  return {
    floors: floors
      .map((floor) => ({ floor_id: floor.floor_id, name: floor.name || floor.floor_id, rooms: byFloor.get(floor.floor_id) || [] }))
      .filter((floor) => floor.rooms.length > 0),
    loose,
    unmapped,
  };
}

function DeviceButton({
  row,
  room,
  t,
  active,
  onInspect,
}: {
  row: Assignment;
  room: string;
  t: Messages;
  active: boolean;
  onInspect: (row: Assignment) => void;
}) {
  return (
    <button
      type="button"
      className="inbox-card"
      aria-current={active ? "true" : undefined}
      onClick={() => onInspect(row)}
      style={{
        width: "100%",
        minHeight: 44,
        textAlign: "start",
        color: "inherit",
        background: "var(--surface-2)",
        borderColor: active ? "var(--accent)" : "var(--line)",
      }}
    >
      <div className="house-inbox-copy">
        <strong>{row.name}</strong>
        <div className="mono">{row.entity_id}</div>
        <p className="muted">
          {room}
          {row.tags.includes("preferred") ? ` · ${t.preferred}` : ""}
          {` · ${t[row.confidence]}`}
        </p>
        {row.aliases.length > 0 && <p className="muted">{row.aliases.join(", ")}</p>}
      </div>
    </button>
  );
}

function RoomSection({
  block,
  t,
  activeId,
  onInspect,
}: {
  block: RoomBlock;
  t: Messages;
  activeId?: string;
  onInspect: (row: Assignment) => void;
}) {
  return (
    <section style={{ display: "grid", gap: 8, marginTop: 16 }}>
      <h3>{block.name}</h3>
      {block.rows.map((row) => (
        <DeviceButton
          key={row.entity_id}
          row={row}
          room={block.name}
          t={t}
          active={row.entity_id === activeId}
          onInspect={onInspect}
        />
      ))}
    </section>
  );
}

export function EntitiesPage({
  data,
  t,
  onInspect,
  activeId,
}: {
  data: Dashboard;
  t: Messages;
  onInspect: (row: Assignment) => void;
  activeId?: string;
}) {
  const [query, setQuery] = useState("");
  const [areaFloor, setAreaFloor] = useState<Record<string, string>>({});
  const rows = useMemo(() => {
    const q = query.toLowerCase();
    return data.assignment.filter((row) => [row.name, row.entity_id, row.area || "", row.aliases.join(" ")].join(" ").toLowerCase().includes(q));
  }, [data, query]);

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

  const grouped = useMemo(() => groupByFloor(rows, data, areaFloor), [rows, data, areaFloor]);

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.entities}</h1>
          <p className="muted">{data.counts.assist} {t.assistVisible}</p>
        </div>
      </section>
      <label htmlFor="klar-device-search">{t.searchDevice}</label>
      <input
        id="klar-device-search"
        value={query}
        onChange={(ev) => setQuery(ev.target.value)}
        autoComplete="off"
      />
      <div style={{ display: "grid", gap: 8, marginTop: 16 }}>
        {grouped.floors.map((floor) => (
          <section key={floor.floor_id}>
            <h2>{floor.name}</h2>
            {floor.rooms.map((block) => (
              <RoomSection key={block.area_id} block={block} t={t} activeId={activeId} onInspect={onInspect} />
            ))}
          </section>
        ))}
        {grouped.loose.map((block) => (
          <RoomSection key={block.area_id} block={block} t={t} activeId={activeId} onInspect={onInspect} />
        ))}
        {grouped.unmapped.length > 0 && (
          <section style={{ display: "grid", gap: 8, marginTop: 16 }}>
            <h3>{t.unmapped}</h3>
            {grouped.unmapped.map((row) => (
              <DeviceButton
                key={row.entity_id}
                row={row}
                room={t.unmapped}
                t={t}
                active={row.entity_id === activeId}
                onInspect={onInspect}
              />
            ))}
          </section>
        )}
      </div>
    </div>
  );
}
