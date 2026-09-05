import { useState } from "react";
import { api, type LangOverlay } from "../api";
import { OriginChip } from "./OriginChip";
import { fill, type Messages } from "../i18n";
import type { LanguageOverlay, PolicyRule } from "../types";

const LEXICON_PATHS = [
  "nouns.light_nouns",
  "nouns.curtain_nouns",
  "nouns.fan_nouns",
  "nouns.media_nouns",
  "nouns.door_nouns",
  "nouns.timer_nouns",
  "nouns.list_nouns",
  "nouns.calendar_nouns",
  "cues.on_words",
  "cues.off_words",
  "cues.open_words",
  "cues.close_words",
  "cues.extra_device_nouns",
];

function patchSet(language: LanguageOverlay | undefined, path: string, token: string, op: "add" | "remove"): LanguageOverlay {
  const sets = { ...(language?.sets || {}) };
  const current = sets[path] || { add: [], remove: [] };
  const add = (current.add || []).filter((item) => item !== token);
  const remove = (current.remove || []).filter((item) => item !== token);
  if (op === "add") {
    add.push(token);
  } else {
    remove.push(token);
  }
  sets[path] = { add, remove };
  return { sets };
}

function seedIsOn(house: PolicyRule[], id: string): boolean {
  const override = house.find((rule) => rule.id === id);
  return override ? override.enabled : true;
}

function toggleSeed(house: PolicyRule[], seed: PolicyRule, enabled: boolean): PolicyRule[] {
  const without = house.filter((rule) => rule.id !== seed.id);
  if (enabled) {
    return without;
  }
  return [...without, { ...seed, enabled: false }];
}

export function LexiconLane({
  t,
  overlay,
  seeds,
  house,
  selectedSeed,
  onSelectSeed,
  onSaved,
  onHouse,
  onStatus,
}: {
  t: Messages;
  overlay: LangOverlay | null;
  seeds: PolicyRule[];
  house: PolicyRule[];
  selectedSeed?: string;
  onSelectSeed?: (id: string) => void;
  onSaved: (next: LangOverlay) => void;
  onHouse: (next: PolicyRule[]) => void;
  onStatus: (status: string) => void;
}) {
  const [path, setPath] = useState("nouns.media_nouns");
  const [token, setToken] = useState("");
  const lexiconRows = Object.entries(overlay?.language?.sets || {}).flatMap(([setPath, delta]) => {
    const rows: { path: string; op: string; token: string }[] = [];
    for (const word of delta.add || []) rows.push({ path: setPath, op: "+", token: word });
    for (const word of delta.remove || []) rows.push({ path: setPath, op: "−", token: word });
    return rows;
  });
  const overlayTitle = lexiconRows.length
    ? fill(t.lexiconOverlayPlus, { count: String(lexiconRows.length) })
    : t.lexiconOverlay;

  const persist = async (language: LanguageOverlay, label: string) => {
    const saved = await api.saveLangOverlay({
      custom: overlay?.custom || [],
      language,
      label,
    });
    onSaved(saved);
    onStatus(t.save);
  };

  const apply = async (op: "add" | "remove") => {
    const nextToken = token.trim();
    const nextPath = path.trim();
    if (!nextToken || !nextPath) return;
    try {
      await persist(patchSet(overlay?.language, nextPath, nextToken, op), `lexicon-${op}`);
      setToken("");
    } catch (err) {
      onStatus(String(err));
    }
  };

  return (
    <>
      <div className="policy-lane-head">
        <h2>{t.laneLanguage}</h2>
        <OriginChip t={t} origin="seed" />
      </div>
      <h3>{overlayTitle}</h3>
      {lexiconRows.length === 0 && <p className="muted">{t.lexiconEmpty}</p>}
      {lexiconRows.map((row) => (
        <p className="lexicon-delta" key={`${row.path}-${row.op}-${row.token}`}>
          {row.op === "+" ? `${row.token} → ${row.path}` : `− ${row.token} ← ${row.path}`}
        </p>
      ))}
      <label>{t.lexiconPath}</label>
      <input list="lexicon-paths" value={path} onChange={(ev) => setPath(ev.target.value)} onClick={(ev) => ev.stopPropagation()} />
      <datalist id="lexicon-paths">
        {LEXICON_PATHS.map((item) => (
          <option key={item} value={item} />
        ))}
      </datalist>
      <label>{t.lexiconToken}</label>
      <input value={token} onChange={(ev) => setToken(ev.target.value)} onClick={(ev) => ev.stopPropagation()} />
      <div className="row" style={{ marginTop: 8 }}>
        <button
          className="secondary"
          type="button"
          onClick={(ev) => {
            ev.stopPropagation();
            void apply("add");
          }}
        >
          {t.lexiconAdd}
        </button>
        <button
          className="ghost"
          type="button"
          onClick={(ev) => {
            ev.stopPropagation();
            void apply("remove");
          }}
        >
          {t.lexiconRemove}
        </button>
      </div>
      <h3>{t.governSeed}</h3>
      {seeds.length === 0 && <p className="muted">{t.governEmpty}</p>}
      {seeds.map((seed) => {
        const enabled = seedIsOn(house, seed.id);
        return (
          <div
            className={`lane-row${selectedSeed === seed.id ? " active" : ""}`}
            key={seed.id}
            onClick={(ev) => {
              ev.stopPropagation();
              onSelectSeed?.(seed.id);
            }}
          >
            <label className="row match-toggle" onClick={(ev) => ev.stopPropagation()}>
              <input
                type="checkbox"
                checked={enabled}
                onChange={(ev) => onHouse(toggleSeed(house, seed, ev.target.checked))}
              />
              {enabled ? t.seedOn : t.seedOff}
            </label>
            <strong className="mono">{seed.id}</strong>
            <span className="chip intent">{seed.effect}</span>
            <OriginChip t={t} origin="seed" />
          </div>
        );
      })}
      {seeds.length > 0 && <p className="caption">{t.governEmpty}</p>}
    </>
  );
}
