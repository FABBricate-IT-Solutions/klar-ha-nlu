import { useEffect, useId, type KeyboardEvent, type RefObject } from "react";
import type { Messages } from "../i18n";
import type { Assignment } from "../types";
import type { FloorBlock, HouseTree, RoomBlock } from "./graphModel";

type GraphListProps = {
  tree: HouseTree;
  rows: Assignment[];
  t: Messages;
  query: string;
  cursorId: string;
  searchRef: RefObject<HTMLInputElement | null>;
  onQuery: (query: string) => void;
  onCursor: (entityId: string) => void;
  onInspect: (row: Assignment) => void;
};

function isTypingField(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false;
  if (el.closest(".drawer")) return true;
  const tag = el.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || el.isContentEditable;
}

function DeviceRow({
  row,
  room,
  t,
  active,
  optionId,
  onInspect,
  onCursor,
}: {
  row: Assignment;
  room: string;
  t: Messages;
  active: boolean;
  optionId: string;
  onInspect: (row: Assignment) => void;
  onCursor: (entityId: string) => void;
}) {
  return (
    <button
      type="button"
      id={optionId}
      role="option"
      aria-selected={active}
      tabIndex={active ? 0 : -1}
      className="inbox-card"
      onClick={() => onInspect(row)}
      onFocus={() => onCursor(row.entity_id)}
      style={{
        width: "100%",
        minHeight: 44,
        textAlign: "start",
        color: "inherit",
        background: "var(--surface-2)",
        borderColor: active ? "var(--accent)" : "var(--line)",
        borderRadius: 0,
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
      </div>
    </button>
  );
}

function RoomList({
  block,
  t,
  cursorId,
  optionPrefix,
  onInspect,
  onCursor,
}: {
  block: RoomBlock;
  t: Messages;
  cursorId: string;
  optionPrefix: string;
  onInspect: (row: Assignment) => void;
  onCursor: (entityId: string) => void;
}) {
  return (
    <section role="group" aria-label={block.name} style={{ display: "grid", gap: 8, marginTop: 12 }}>
      <h3>{block.name}</h3>
      {block.rows.map((row) => (
        <DeviceRow
          key={row.entity_id}
          row={row}
          room={block.name}
          t={t}
          active={row.entity_id === cursorId}
          optionId={`${optionPrefix}-${row.entity_id}`}
          onInspect={onInspect}
          onCursor={onCursor}
        />
      ))}
    </section>
  );
}

function FloorList({
  floor,
  t,
  cursorId,
  optionPrefix,
  onInspect,
  onCursor,
}: {
  floor: FloorBlock;
  t: Messages;
  cursorId: string;
  optionPrefix: string;
  onInspect: (row: Assignment) => void;
  onCursor: (entityId: string) => void;
}) {
  return (
    <section role="group" aria-label={floor.name}>
      <h2>{floor.name}</h2>
      {floor.rooms.map((block) => (
        <RoomList
          key={block.area_id}
          block={block}
          t={t}
          cursorId={cursorId}
          optionPrefix={optionPrefix}
          onInspect={onInspect}
          onCursor={onCursor}
        />
      ))}
    </section>
  );
}

export function GraphList({
  tree,
  rows,
  t,
  query,
  cursorId,
  searchRef,
  onQuery,
  onCursor,
  onInspect,
}: GraphListProps) {
  const listId = useId();
  const active = rows.find((row) => row.entity_id === cursorId) || rows[0];

  useEffect(() => {
    document.getElementById(`${listId}-${cursorId}`)?.scrollIntoView({ block: "nearest" });
  }, [cursorId, listId, rows]);

  useEffect(() => {
    const onSlash = (event: globalThis.KeyboardEvent) => {
      if (event.key !== "/" || event.altKey || event.ctrlKey || event.metaKey) return;
      if (event.target === searchRef.current) return;
      if (isTypingField(event.target)) return;
      event.preventDefault();
      searchRef.current?.focus();
    };
    window.addEventListener("keydown", onSlash);
    return () => window.removeEventListener("keydown", onSlash);
  }, [searchRef]);

  const move = (delta: number) => {
    if (rows.length === 0) return;
    const index = Math.max(0, rows.findIndex((row) => row.entity_id === cursorId));
    const next = rows[Math.min(rows.length - 1, Math.max(0, index + delta))];
    if (next) onCursor(next.entity_id);
  };

  const onSearchKey = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (!cursorId && rows[0]) onCursor(rows[0].entity_id);
      else move(1);
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      move(-1);
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      if (rows[0]) onCursor(rows[0].entity_id);
      return;
    }
    if (event.key === "End") {
      event.preventDefault();
      if (rows[rows.length - 1]) onCursor(rows[rows.length - 1].entity_id);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      if (active) onInspect(active);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      onQuery("");
    }
  };

  const onListKey = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      move(1);
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      move(-1);
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      if (rows[0]) onCursor(rows[0].entity_id);
      return;
    }
    if (event.key === "End") {
      event.preventDefault();
      if (rows[rows.length - 1]) onCursor(rows[rows.length - 1].entity_id);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      if (active) onInspect(active);
    }
  };

  return (
    <div className="card house-list" style={{ borderRadius: 0 }}>
      <label htmlFor={`${listId}-search`}>{t.entities}</label>
      <input
        id={`${listId}-search`}
        ref={searchRef}
        value={query}
        autoComplete="off"
        aria-keyshortcuts="/"
        aria-controls={listId}
        aria-activedescendant={active ? `${listId}-${active.entity_id}` : undefined}
        role="combobox"
        aria-expanded
        aria-autocomplete="list"
        placeholder={t.searchDevice}
        onChange={(event) => onQuery(event.target.value)}
        onKeyDown={onSearchKey}
      />
      <div
        id={listId}
        role="listbox"
        aria-label={t.entities}
        tabIndex={rows.length ? 0 : -1}
        onKeyDown={onListKey}
        style={{ display: "grid", gap: 8, marginTop: 12 }}
      >
        {tree.floors.map((floor) => (
          <FloorList
            key={floor.floor_id}
            floor={floor}
            t={t}
            cursorId={cursorId}
            optionPrefix={listId}
            onInspect={onInspect}
            onCursor={onCursor}
          />
        ))}
        {tree.loose.map((block) => (
          <RoomList
            key={block.area_id}
            block={block}
            t={t}
            cursorId={cursorId}
            optionPrefix={listId}
            onInspect={onInspect}
            onCursor={onCursor}
          />
        ))}
        {tree.unmapped.length > 0 && (
          <section role="group" aria-label={t.unmapped} style={{ display: "grid", gap: 8, marginTop: 12 }}>
            <h3>{t.unmapped}</h3>
            {tree.unmapped.map((row) => (
              <DeviceRow
                key={row.entity_id}
                row={row}
                room={t.unmapped}
                t={t}
                active={row.entity_id === cursorId}
                optionId={`${listId}-${row.entity_id}`}
                onInspect={onInspect}
                onCursor={onCursor}
              />
            ))}
          </section>
        )}
      </div>
    </div>
  );
}
