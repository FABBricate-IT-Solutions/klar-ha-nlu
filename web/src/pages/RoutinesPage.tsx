import { useEffect, useState } from "react";
import { api, type CustomRule, type LangOverlay } from "../api";
import { SearchSelect, useHouseCatalog, withCurrent } from "../components/SearchSelect";
import { SetupHint } from "../components/SetupHint";
import type { Messages } from "../i18n";

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
  const [rules, setRules] = useState<CustomRule[]>([]);
  const [language, setLanguage] = useState<unknown>({});
  const [phrase, setPhrase] = useState("");
  const [script, setScript] = useState("");
  const [status, setStatus] = useState("");
  const { scriptOptions } = useHouseCatalog();

  const load = (overlay: LangOverlay) => {
    setRules(overlay.custom);
    setLanguage(overlay.language);
  };

  useEffect(() => {
    api.langOverlay().then(load).catch((err) => setStatus(String(err)));
  }, []);

  const persist = async (next: CustomRule[], label: string) => {
    load(await api.saveLangOverlay({ custom: next, language, label }));
    setStatus(t.save);
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
          <button className="primary" type="button" onClick={() => void add()}>{t.addRoutine}</button>
        </div>
        {status && <p className="caption">{status}</p>}
      </div>
      <div className="card">
        {routines.length === 0 && (
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
