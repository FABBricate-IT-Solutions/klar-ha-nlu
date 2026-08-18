import { useMemo, useRef, useState, type KeyboardEvent } from "react";
import type { LanguagePack } from "../api";

type Props = {
  packs: LanguagePack[];
  value: string[];
  allLabel: string;
  searchLabel: string;
  emptyLabel: string;
  onChange: (codes: string[]) => void;
};

export function LanguagePicker({ packs, value, allLabel, searchLabel, emptyLabel, onChange }: Props) {
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const allCodes = packs.map((pack) => pack.code);
  const allOn = value.length === 0 || (allCodes.length > 0 && value.length === allCodes.length);
  const selected = allOn ? [] : value;
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return packs.filter((pack) => {
      if (!needle) return true;
      return [pack.code, pack.native_name, pack.script, ...(pack.variants || [])]
        .join(" ")
        .toLowerCase()
        .includes(needle);
    });
  }, [packs, query]);

  const pick = (code: string) => {
    if (allOn) {
      onChange([code]);
      return;
    }
    const next = selected.includes(code)
      ? selected.filter((item) => item !== code)
      : [...selected, code];
    onChange(next.length === 0 || next.length === allCodes.length ? [] : next);
  };

  const onKey = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setOpen(true);
      setActive((index) => Math.min(index + 1, Math.max(filtered.length - 1, 0)));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setActive((index) => Math.max(index - 1, 0));
      return;
    }
    if (event.key === "Enter" && open && filtered[active]) {
      event.preventDefault();
      pick(filtered[active].code);
      return;
    }
    if (event.key === "Escape") {
      setOpen(false);
    }
  };

  return (
    <div className="lang-picker">
      <div className="lang-chips">
        <button
          type="button"
          className={`chip${allOn ? " on" : ""}`}
          aria-pressed={allOn}
          onClick={() => onChange([])}
        >
          {allLabel}
        </button>
        {selected.map((code) => {
          const pack = packs.find((item) => item.code === code);
          return (
            <button type="button" key={code} className="chip on" onClick={() => pick(code)}>
              {pack ? `${pack.native_name} (${code})` : code}
            </button>
          );
        })}
      </div>
      <div className="lang-combo">
        <input
          ref={input}
          type="search"
          role="combobox"
          aria-expanded={open}
          aria-controls="klar-lang-list"
          aria-autocomplete="list"
          placeholder={searchLabel}
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
            setOpen(true);
            setActive(0);
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => window.setTimeout(() => setOpen(false), 120)}
          onKeyDown={onKey}
        />
        {open && (
          <ul id="klar-lang-list" className="lang-list" role="listbox">
            {filtered.length === 0 && <li className="muted">{emptyLabel}</li>}
            {filtered.map((pack, index) => {
              const on = allOn || selected.includes(pack.code);
              return (
                <li key={pack.code}>
                  <button
                    type="button"
                    role="option"
                    aria-selected={on}
                    className={`lang-option${index === active ? " active" : ""}${on ? " on" : ""}`}
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={() => pick(pack.code)}
                  >
                    <span>{pack.native_name}</span>
                    <span className="muted">{pack.code}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
