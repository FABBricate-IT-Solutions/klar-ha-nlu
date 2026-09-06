import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { languageFlag } from "../i18n/languageFlag";
import type { Dashboard, Entity } from "../types";
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox";

export type SearchOption = { value: string; label: string; hint?: string; flag?: string };

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

export function languageOptions(
  packs: { code: string; native_name: string }[],
  displayLocale = "en",
): SearchOption[] {
  const names = new Intl.DisplayNames([displayLocale, "en"], { type: "language" });
  return packs.map((pack) => {
    const pretty = pack.native_name && pack.native_name !== pack.code
      ? pack.native_name
      : names.of(pack.code) || pack.code;
    return {
      value: pack.code,
      label: pretty,
      hint: pack.code,
      flag: languageFlag(pack.code),
    };
  });
}

export function useHouseCatalog() {
  const [entities, setEntities] = useState<Entity[]>([]);
  const [rooms, setRooms] = useState<SearchOption[]>([]);
  const [domains, setDomains] = useState<SearchOption[]>([]);
  const [floors, setFloors] = useState<SearchOption[]>([]);

  useEffect(() => {
    api.entities().then((rows) => setEntities(Array.isArray(rows) ? rows : [])).catch(() => undefined);
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

function matchesQuery(row: SearchOption, query: string): boolean {
  const needle = query.trim().toLowerCase();
  if (!needle) return true;
  return (
    row.label.toLowerCase().includes(needle)
    || row.value.toLowerCase().includes(needle)
    || Boolean(row.hint?.toLowerCase().includes(needle))
  );
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
  const [query, setQuery] = useState("");
  const items = useMemo(() => {
    const listed = allowEmpty ? [{ value: "", label: emptyLabel }, ...options] : options;
    const typed = query.trim();
    if (allowCustom && typed && !listed.some((row) => row.value === typed)) {
      return [{ value: typed, label: typed }, ...listed];
    }
    return listed;
  }, [allowCustom, allowEmpty, emptyLabel, options, query]);
  const selected = items.find((row) => row.value === value) ?? null;

  return (
    <Combobox
      items={items}
      value={selected}
      onValueChange={(next) => onChange(next?.value ?? "")}
      onInputValueChange={(next) => setQuery(next)}
      isItemEqualToValue={(left, right) => left.value === right.value}
      itemToStringLabel={(row) => (row.flag ? `${row.flag} ${row.label}` : row.label)}
      itemToStringValue={(row) => row.value}
      filter={matchesQuery}
      autoHighlight
    >
      <ComboboxInput id={id} placeholder={placeholder || emptyLabel} />
      <ComboboxContent align="start">
        <ComboboxEmpty>—</ComboboxEmpty>
        <ComboboxList>
          {(row) => (
            <ComboboxItem key={row.value || "any"} value={row}>
              {row.flag ? (
                <span className="text-base leading-none" aria-hidden>
                  {row.flag}
                </span>
              ) : null}
              <span className="min-w-0 flex-1 truncate">{row.label}</span>
              {row.hint && row.hint !== row.label ? (
                <span className="font-mono text-xs text-muted-foreground">{row.hint}</span>
              ) : null}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
}
