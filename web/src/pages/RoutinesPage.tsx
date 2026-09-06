import { useState } from "react";
import { api, type CustomRule } from "../api";
import { SearchSelect, useHouseCatalog, withCurrent } from "../components/SearchSelect";
import { SetupHint } from "../components/SetupHint";
import type { Messages } from "../i18n";
import { useLangOverlay } from "../useLangOverlay";

function isRoutine(rule: CustomRule): boolean {
  const script = rule.slots.entity_id || "";
  return rule.intent === "HassTurnOn" && script.startsWith("script.");
}

function asScript(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return "";
  return trimmed.startsWith("script.") ? trimmed : `script.${trimmed}`;
}

export function RoutinesPage({ t }: { t: Messages }) {
  const { overlay, offline, status, setStatus, replace } = useLangOverlay();
  const rules = overlay?.custom ?? [];
  const language = overlay?.language ?? {};
  const [phrase, setPhrase] = useState("");
  const [script, setScript] = useState("");
  const { scriptOptions } = useHouseCatalog();

  const persist = async (next: CustomRule[], label: string) => {
    try {
      replace(await api.saveLangOverlay({ custom: next, language, label }));
      setStatus(t.save);
    } catch (err) {
      setStatus(String(err));
    }
  };

  const add = async () => {
    const entityId = asScript(script);
    const spoken = phrase.trim();
    if (spoken.length < 2 || !entityId || entityId.split(".").length !== 2) {
      setStatus(t.routineInvalid);
      return;
    }
    setPhrase("");
    setScript("");
    await persist([...rules, { phrase: spoken, intent: "HassTurnOn", slots: { entity_id: entityId } }], spoken);
  };

  const remove = async (index: number) => {
    await persist(rules.filter((_, row) => row !== index), "remove");
  };

  const routines = rules.map((rule, index) => ({ rule, index })).filter((row) => isRoutine(row.rule));

  return (
    <section className="grid two">
      <div className="card">
        <h2>{t.routines}</h2>
        <p className="muted">{t.routineHint}</p>
        <label>{t.guideRoutinesSay}</label>
        <input value={phrase} onChange={(ev) => setPhrase(ev.target.value)} placeholder={t.routinePhraseHint} />
        <label>{t.guideRoutinesScript}</label>
        <SearchSelect
          value={script}
          options={withCurrent(scriptOptions, script.startsWith("script.") ? script : script ? `script.${script}` : "")}
          onChange={setScript}
          placeholder="script.good_night"
          allowEmpty={false}
        />
        <div className="row" style={{ marginTop: 12 }}>
          <button className="primary" type="button" onClick={() => void add()} disabled={offline}>{t.addRoutine}</button>
        </div>
        {status && <p className="caption">{status}</p>}
      </div>
      <div className="card">
        {offline && <p className="muted">{t.engineOffline}</p>}
        {!offline && routines.length === 0 && (
          <div>
            <p className="muted">{t.noRoutines}</p>
            <SetupHint t={t} />
          </div>
        )}
        {routines.map(({ rule, index }) => (
          <div className="list-row" key={`${rule.phrase}-${index}`} style={{ borderBottom: "1px solid var(--line)", padding: "8px 0" }}>
            <div>
              <strong>{rule.phrase}</strong>
              <p className="mono">{rule.slots.entity_id}</p>
            </div>
            <button className="ghost danger" onClick={() => remove(index)}>{t.dismiss}</button>
          </div>
        ))}
      </div>
    </section>
  );
}
