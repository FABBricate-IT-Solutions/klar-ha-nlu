import type { Assignment, Dashboard } from "../types";

export type RoomBlock = {
  area_id: string;
  name: string;
  inbox: number;
  rows: Assignment[];
};

export type FloorBlock = {
  floor_id: string;
  name: string;
  rooms: RoomBlock[];
};

export type HouseTree = {
  floors: FloorBlock[];
  loose: RoomBlock[];
  unmapped: Assignment[];
};

export function confidenceColor(confidence: string): string {
  if (confidence === "high") return "var(--high)";
  if (confidence === "medium") return "var(--medium)";
  return "var(--low)";
}

function floorOf(areaId: string, mapped: Record<string, string>, rooms: Dashboard["rooms"]): string {
  if (mapped[areaId]) return mapped[areaId];
  const extra = rooms.find((room) => room.area_id === areaId) as { floor_id?: string | null } | undefined;
  return extra?.floor_id || "";
}

function roomBlock(areaId: string, name: string, rows: Assignment[], data: Dashboard): RoomBlock {
  const known = data.rooms.find((room) => room.area_id === areaId);
  return {
    area_id: areaId,
    name: known?.name || name,
    inbox: known?.inbox ?? rows.filter((row) => row.confidence !== "high").length,
    rows,
  };
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
    .map((room) => roomBlock(room.area_id, room.name, buckets.get(room.area_id) || [], data));
  const extras = [...buckets.entries()]
    .filter(([id]) => !data.rooms.some((room) => room.area_id === id))
    .map(([areaId, block]) => roomBlock(areaId, areaId, block, data));
  return [...known, ...extras];
}

export function groupHouse(rows: Assignment[], data: Dashboard, mapped: Record<string, string>): HouseTree {
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
      .map((floor) => ({
        floor_id: floor.floor_id,
        name: floor.name || floor.floor_id,
        rooms: byFloor.get(floor.floor_id) || [],
      }))
      .filter((floor) => floor.rooms.length > 0),
    loose,
    unmapped,
  };
}

export function flattenHouse(tree: HouseTree): Assignment[] {
  const out: Assignment[] = [];
  for (const floor of tree.floors) {
    for (const room of floor.rooms) out.push(...room.rows);
  }
  for (const room of tree.loose) out.push(...room.rows);
  out.push(...tree.unmapped);
  return out;
}

export function matchesAssignment(row: Assignment, query: string, extra: string[] = []): boolean {
  const needle = query.trim().toLowerCase();
  if (!needle) return true;
  return [row.name, row.entity_id, row.area || "", row.aliases.join(" "), row.confidence, ...extra]
    .join(" ")
    .toLowerCase()
    .includes(needle);
}
