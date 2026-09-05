import { useState } from "react";
import { api, type LangOverlay } from "../api";
import type { Messages } from "../i18n";
import type { LanguageOverlay } from "../types";

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

export function LexiconLane({
  t,
  overlay,
  onSaved,
  onStatus,
}: {
  t: Messages;
  overlay: LangOverlay | null;
  onSaved: (next: LangOverlay) => void;
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
      <h2>{t.laneLanguage}</h2>
      <h3>{t.lexiconOverlay}</h3>
      {lexiconRows.length === 0 && <p className="muted">{t.lexiconEmpty}</p>}
      {lexiconRows.map((row) => (
        <p className="lexicon-delta" key={`${row.path}-${row.op}-${row.token}`}>
          {row.path} {row.op}
          {row.token}
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
          onClick={(ev) => {
            ev.stopPropagation();
            void apply("add");
          }}
        >
          {t.lexiconAdd}
        </button>
        <button
          className="ghost"
          onClick={(ev) => {
            ev.stopPropagation();
            void apply("remove");
          }}
        >
          {t.lexiconRemove}
        </button>
      </div>
      <h3>{t.governSeed}</h3>
      <p className="muted">{t.governEmpty}</p>
    </>
  );
}
