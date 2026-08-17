import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import { bakeVariants } from "../speechBank";
import type { EvaluateOut, Locale, PolicyEffect, PolicyRule, SpeechBank } from "../types";
import { CustomPage } from "./CustomPage";

const EFFECTS: PolicyEffect[] = ["confirm", "block", "allow", "prefer_entity", "prefer_area"];

function newRule(): PolicyRule {
  return {
    id: `rule-${Date.now().toString(36)}`,
    enabled: true,
    label: "",
    when: {},
    effect: "confirm",
  };
}

export function RulesPage({ t, locale, personality, languages }: { t: Messages; locale: Locale; personality: string; languages: string[] }) {
  const [view, setView] = useState<"policies" | "sentences">("policies");
  const [rules, setRules] = useState<PolicyRule[]>([]);
  const [bank, setBank] = useState<SpeechBank>({ entries: [] });
  const [selected, setSelected] = useState(0);
  const [utterance, setUtterance] = useState("");
  const [evalOut, setEvalOut] = useState<EvaluateOut | null>(null);
  const [status, setStatus] = useState("");
  const current = rules[selected];

  useEffect(() => {
    api.policies().then((bundle) => {
      setRules(bundle.policies);
      setBank(bundle.speech_bank);
    }).catch((err) => setStatus(String(err)));
  }, []);

  const persist = async (next: PolicyRule[], nextBank = bank) => {
    const saved = await api.savePolicies({ policies: next, speech_bank: nextBank });
    setRules(saved.policies);
    setBank(saved.speech_bank);
    setStatus(t.save);
  };

  const update = (patch: Partial<PolicyRule>) => {
    if (!current) return;
    const next = rules.map((rule, index) => (index === selected ? { ...rule, ...patch } : rule));
    setRules(next);
  };

  const updateWhen = (key: keyof PolicyRule["when"], value: string) => {
    if (!current) return;
    update({ when: { ...current.when, [key]: value || undefined } });
  };

  const move = (from: number, to: number) => {
    if (to < 0 || to >= rules.length) return;
    const next = [...rules];
    const [row] = next.splice(from, 1);
    next.splice(to, 0, row);
    setRules(next);
    setSelected(to);
  };

  const evaluate = async () => {
    setEvalOut(await api.evaluatePolicies({ text: utterance, language: locale, policies: rules }));
  };

  const bake = async () => {
    if (!current) return;
    const entry = bakeVariants(current.id, current.effect, personality, languages.length ? languages : [locale]);
    const entries = [...bank.entries.filter((item) => item.rule_id !== current.id), entry];
    const nextBank = { entries };
    setBank(nextBank);
    await persist(rules, nextBank);
  };

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.rules}</h1>
          <p className="muted">{t.priority}</p>
        </div>
        <div className="row">
          <button className="secondary" onClick={() => persist(rules)}>{t.save}</button>
          <button className="primary" onClick={() => { const next = [...rules, newRule()]; setRules(next); setSelected(next.length - 1); }}>{t.addRule}</button>
        </div>
      </section>
      <nav className="subnav">
        <button className={view === "policies" ? "active" : ""} onClick={() => setView("policies")}>{t.policies}</button>
        <button className={view === "sentences" ? "active" : ""} onClick={() => setView("sentences")}>{t.sentences}</button>
      </nav>
      {view === "sentences" && <CustomPage t={t} locale={locale} embedded />}
      {view === "policies" && (
        <section className="grid two">
          <div className="card">
            {rules.length === 0 && <p className="muted">{t.noPolicies}</p>}
            {rules.map((rule, index) => (
              <div
                className={`rule-row${index === selected ? " active" : ""}`}
                key={rule.id}
                draggable
                onDragStart={(ev) => ev.dataTransfer.setData("text/plain", String(index))}
                onDragOver={(ev) => ev.preventDefault()}
                onDrop={(ev) => {
                  ev.preventDefault();
                  move(Number(ev.dataTransfer.getData("text/plain")), index);
                }}
                onClick={() => setSelected(index)}
              >
                <span className="muted">{index + 1}</span>
                <strong>{rule.label || rule.id}</strong>
                <span className="chip intent">{rule.effect}</span>
                <button className="ghost danger" onClick={() => persist(rules.filter((item) => item.id !== rule.id))}>{t.dismiss}</button>
              </div>
            ))}
          </div>
          <div className="card">
            {current ? (
              <>
                <label>{t.custom}</label>
                <input value={current.label} onChange={(ev) => update({ label: ev.target.value })} />
                <label className="row">
                  <input type="checkbox" checked={current.enabled} onChange={(ev) => update({ enabled: ev.target.checked })} style={{ width: "auto" }} />
                  {current.enabled ? "on" : "off"}
                </label>
                <label>{t.when}</label>
                {(["intent", "domain", "area", "entity_id", "floor", "name"] as const).map((key) => (
                  <input key={key} placeholder={key} value={current.when[key] || ""} onChange={(ev) => updateWhen(key, ev.target.value)} />
                ))}
                <label>{t.then}</label>
                <select value={current.effect} onChange={(ev) => update({ effect: ev.target.value as PolicyEffect })}>
                  {EFFECTS.map((effect) => <option key={effect} value={effect}>{t[effect === "confirm" ? "effectConfirm" : effect === "block" ? "effectBlock" : effect === "allow" ? "effectAllow" : effect === "prefer_entity" ? "effectPreferEntity" : "effectPreferArea"]}</option>)}
                </select>
                {(current.effect === "prefer_entity" || current.effect === "prefer_area") && (
                  <input placeholder="prefer" value={current.prefer || ""} onChange={(ev) => update({ prefer: ev.target.value })} />
                )}
                <div className="row" style={{ marginTop: 12 }}>
                  <button className="secondary" onClick={bake}>{t.bakeSpeech}</button>
                </div>
                {bank.entries.find((item) => item.rule_id === current.id)?.variants.map((variant, index) => (
                  <p className="muted" key={`${variant.language}-${index}`}>{variant.language}/{variant.personality}: {variant.text}</p>
                ))}
              </>
            ) : <p className="muted">{t.noPolicies}</p>}
          </div>
          <div className="card" style={{ gridColumn: "1 / -1" }}>
            <h2>{t.evaluator}</h2>
            <div className="row">
              <input value={utterance} onChange={(ev) => setUtterance(ev.target.value)} placeholder={t.command} />
              <button className="primary" onClick={evaluate}>{t.analyze}</button>
            </div>
            {evalOut && (
              <div className="flow" style={{ marginTop: 16 }}>
                <div className="card"><h3>{t.compiledRisk}</h3><p>{evalOut.compiled_risky ? "yes" : "no"}</p></div>
                <div className="card"><h3>{t.matchedRule}</h3><p className="mono">{evalOut.matched_rule || "—"}</p></div>
                <div className="card"><h3>{t.then}</h3><p className="mono">{evalOut.hit || "—"}</p></div>
                <div className="card"><h3>{t.finalBand}</h3><p className="mono">{evalOut.outcome.decision.type}</p></div>
                <div className="card"><h3>{t.variantPreview}</h3><p>{evalOut.speech_variant || evalOut.outcome.speech}</p></div>
              </div>
            )}
          </div>
        </section>
      )}
      {status && <p className="muted">{status}</p>}
    </div>
  );
}
