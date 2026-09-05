import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { api } from "../api";
import type { Dashboard, Entity } from "../types";

export type SearchOption = { value: string; label: string };

type SearchSelectProps = {
  value: string;
  options: SearchOption[];
  onChange: (value: string) => void;
  placeholder?: string;
  emptyLabel?: string;
  allowEmpty?: boolean;
  allowCustom?: boolean;
  id?: string;
};

function floorsFromDashboard(data: Dashboard): SearchOption[] {
  const floors = (data as { floors?: { floor_id?: string; id?: string; name?: string }[] }).floors ?? [];
  return floors
    .map((floor) => {
      const id = floor.floor_id || floor.id || "";
      return { value: id, label: floor.name || id };
    })
    .filter((row) => row.value);
}

export function withCurrent(options: SearchOption[], value: string): SearchOption[] {
  if (!value || options.some((row) => row.value === value)) return options;
  return [{ value, label: value }, ...options];
}

export function useHouseCatalog() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [rooms, setRooms] = useState<SearchOption[]>([]);
  const [domains, setDomains] = useState<SearchOption[]>([]);
  const [floors, setFloors] = useState<SearchOption[]>([]);

  useEffect(() => {
    api.entities().then(setEntities).catch(() => undefined);
    api
      .dashboard()
      .then((dash) => {
        setRooms(dash.rooms.map((room) => ({ value: room.area_id, label: room.name || room.area_id })));
        setDomains(dash.domains.map((row) => ({ value: row.domain, label: row.domain })));
        setFloors(floorsFromDashboard(dash));
      })
      .catch(() => undefined);
  }, []);

  const entityOptions = useMemo(
    () => entities.map((entity) => ({ value: entity.entity_id, label: entity.name || entity.entity_id })),
    [entities],
  );
  const scriptOptions = useMemo(
    () => entityOptions.filter((row) => row.value.startsWith("script.")),
    [entityOptions],
  );

  return { entities, entityOptions, scriptOptions, rooms, domains, floors };
}

export function SearchSelect({
  value,
  options,
  onChange,
  placeholder,
  emptyLabel = "any",
  allowEmpty = true,
  allowCustom = false,
  id,
}: SearchSelectProps) {
  const listId = useId();
  const root = useRef<HTMLDivElement>(null);
  const activeRef = useRef<HTMLLIElement>(null);
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const selected = options.find((row) => row.value === value);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const rows = needle
      ? options.filter((row) => row.label.toLowerCase().includes(needle) || row.value.toLowerCase().includes(needle))
      : options;
    const custom = allowCustom && query.trim() && !options.some((row) => row.value === query.trim())
      ? [{ value: query.trim(), label: query.trim() }]
      : [];
    const listed = custom.length ? [...custom, ...rows] : rows;
    return allowEmpty ? [{ value: "", label: emptyLabel }, ...listed] : listed;
  }, [allowCustom, allowEmpty, emptyLabel, options, query]);

  useEffect(() => {
    if (!open) return;
    const close = (event: MouseEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [open]);

  useEffect(() => {
    activeRef.current?.scrollIntoView({ block: "nearest" });
  }, [active, open]);

  const highlightSelected = () => {
    const rows = allowEmpty ? [{ value: "", label: emptyLabel }, ...options] : options;
    const index = rows.findIndex((row) => row.value === value);
    setActive(index >= 0 ? index : 0);
  };

  const openList = () => {
    setQuery("");
    highlightSelected();
    setOpen(true);
  };

  const pick = (next: string) => {
    onChange(next);
    setOpen(false);
    setQuery("");
  };

  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (!open) {
        openList();
        return;
      }
      setActive((index) => Math.min(index + 1, Math.max(filtered.length - 1, 0)));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setActive((index) => Math.max(index - 1, 0));
      return;
    }
    if (event.key === "Home") {
      event.preventDefault();
      setActive(0);
      return;
    }
    if (event.key === "End") {
      event.preventDefault();
      setActive(Math.max(filtered.length - 1, 0));
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      const row = filtered[active];
      if (row) pick(row.value);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setOpen(false);
      setQuery("");
    }
  };

  return (
    <div className="search-select" ref={root}>
      <style>{searchSelectCss}</style>
      <input
        id={id}
        role="combobox"
        aria-expanded={open}
        aria-controls={listId}
        aria-haspopup="listbox"
        aria-autocomplete="list"
        autoComplete="off"
        aria-activedescendant={open && filtered[active] ? `${listId}-${active}` : undefined}
        value={open ? query : selected?.label || value}
        placeholder={placeholder || emptyLabel}
        onChange={(event) => {
          setQuery(event.target.value);
          setOpen(true);
          setActive(0);
        }}
        onFocus={openList}
        onKeyDown={onKeyDown}
      />
      {open && (
        <ul id={listId} role="listbox" className="search-select-list">
          {filtered.length === 0 && (
            <li className="muted" role="presentation">
              —
            </li>
          )}
          {filtered.map((row, index) => (
            <li
              key={`${row.value || "any"}-${index}`}
              id={`${listId}-${index}`}
              ref={index === active ? activeRef : undefined}
              role="option"
              aria-selected={row.value === value}
              className={index === active ? "active" : ""}
              onMouseEnter={() => setActive(index)}
              onMouseDown={(event) => {
                event.preventDefault();
                pick(row.value);
              }}
            >
              <span>{row.label}</span>
              {row.value && row.label !== row.value && <span className="mono">{row.value}</span>}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

const searchSelectCss = `
.search-select { position: relative; }
.search-select-list {
  position: absolute;
  z-index: 8;
  inset-inline: 0;
  top: calc(100% - 1px);
  max-height: 240px;
  margin: 0;
  padding: 0;
  overflow: auto;
  list-style: none;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 0;
}
.search-select-list li {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: baseline;
  padding: 10px 12px;
  cursor: pointer;
  border-radius: 0;
}
.search-select-list li.active,
.search-select-list li[aria-selected="true"] {
  background: var(--surface-2);
}
.search-select-list li.active { outline: 1px solid var(--accent); outline-offset: -1px; }
.search-select input:focus-visible { outline: 1px solid var(--accent); outline-offset: 0; }
`;
