import { useEffect, useMemo, useState } from "react";
import { api, type LangOverlay } from "../api";
import { PolicyPath, type PolicyLane } from "../components/PolicyPath";
import { SearchSelect, useHouseCatalog, withCurrent } from "../components/SearchSelect";
import { policiesEmpty, SetupHint } from "../components/SetupHint";
import type { Messages } from "../i18n";
import { bakeVariants } from "../speechBank";
import type { EvaluateOut, Locale, MatchCatalogRow, PolicyEffect, PolicyRule, RulesView, SpeechBank } from "../types";
import { CustomPage } from "./CustomPage";
import { RoutinesPage } from "./RoutinesPage";

// App must scope any chrome hero (Speichern / Regel) to rules_view === "policies".

const fallbackIntents = ["HassTurnOn", "HassTurnOff", "HassToggle", "HassLightSet", "HassGetState", "HassClimateSetTemperature"];

const EFFECTS: PolicyEffect[] = ["confirm", "block", "allow", "prefer_entity", "prefer_area", "reply", "script", "template", "llm"];
const ACTION_EFFECTS: PolicyEffect[] = ["reply", "script", "template", "llm"];

function effectLabel(t: Messages, effect: PolicyEffect): string {
  switch (effect) {
    case "confirm":
      return t.effectConfirm;
    case "block":
      return t.effectBlock;
    case "allow":
      return t.effectAllow;
    case "prefer_entity":
      return t.effectPreferEntity;
    case "prefer_area":
      return t.effectPreferArea;
    case "reply":
      return t.effectReply;
    case "script":
      return t.effectScript;
    case "template":
      return t.effectTemplate;
    case "llm":
      return t.effectLlm;
    default: {
      const _never: never = effect;
      return _never;
    }
  }
}

function payloadHint(t: Messages, effect: PolicyEffect): string {
  switch (effect) {
    case "reply":
      return t.payloadReply;
    case "script":
      return t.payloadScript;
    case "template":
      return t.payloadTemplate;
    case "llm":
      return t.payloadLlm;
    case "confirm":
    case "block":
    case "allow":
    case "prefer_entity":
    case "prefer_area":
      return "";
    default: {
      const _never: never = effect;
      return _never;
    }
  }
}

function newRule(): PolicyRule {
  return {
    id: `rule-${Date.now().toString(36)}`,
    enabled: true,
    label: "",
    when: {},
    effect: "confirm",
  };
}

export function RulesPage({
  t,
  locale,
  personality,
  languages,
  rulesView,
  onRulesView,
}: {
  t: Messages;
  locale: Locale;
  personality: string;
  languages: string[];
  rulesView?: RulesView;
  onRulesView?: (view: RulesView) => void;
}) {
  const [localView, setLocalView] = useState<RulesView>(rulesView ?? "routines");
  const view = rulesView ?? localView;
  const setView = (next: RulesView) => {
    if (rulesView === undefined) setLocalView(next);
    onRulesView?.(next);
  };
  const [rules, setRules] = useState<PolicyRule[]>([]);
  const [bank, setBank] = useState<SpeechBank>({ entries: [] });
  const [selected, setSelected] = useState(0);
  const [utterance, setUtterance] = useState("");
  const [evalOut, setEvalOut] = useState<EvaluateOut | null>(null);
  const [status, setStatus] = useState("");
  const [intents, setIntents] = useState<string[]>(fallbackIntents);
  const [catalog, setCatalog] = useState<MatchCatalogRow[]>([]);
  const [overlay, setOverlay] = useState<LangOverlay | null>(null);
  const [lane, setLane] = useState<PolicyLane>("house");
  const [selectedMatch, setSelectedMatch] = useState(0);
  const { entityOptions, rooms, domains, floors } = useHouseCatalog();
  const current = rules[selected];
  const intentOptions = useMemo(
    () => intents.map((name) => ({ value: name, label: name })),
    [intents],
  );

  useEffect(() => {
    api.policies().then((bundle) => {
      setRules(bundle.policies);
      setBank(bundle.speech_bank);
    }).catch((err) => setStatus(String(err)));
    api.policiesCatalog().then((body) => setCatalog(body.matches)).catch((err) => setStatus(String(err)));
    api.langOverlay().then(setOverlay).catch((err) => setStatus(String(err)));
    api.intents().then((names) => { if (names.length) setIntents(names); }).catch(() => undefined);
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

  const updateWhenEntity = (entityId: string) => {
    if (!current) return;
    const next = { ...current.when, entity_id: entityId || undefined };
    if (entityId && !current.when.domain) {
      const domain = entityId.split(".")[0];
      if (domain) next.domain = domain;
    }
    update({ when: next });
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
    setEvalOut(await api.evaluatePolicies({
      text: utterance,
      language: languages.length === 1 ? languages[0] : undefined,
      policies: rules,
    }));
  };

  const selectLane = (next: PolicyLane, id?: string) => {
    setLane(next);
    switch (next) {
      case "match":
        if (id) {
          const index = catalog.findIndex((row) => row.id === id);
          if (index >= 0) setSelectedMatch(index);
        }
        break;
      case "language":
        break;
      case "house":
        if (id) {
          const index = rules.findIndex((rule) => rule.id === id);
          if (index >= 0) setSelected(index);
        }
        break;
      default: {
        const _never: never = next;
        return _never;
      }
    }
  };

  const lexiconRows = Object.entries(overlay?.language?.sets || {}).flatMap(([path, delta]) => {
    const rows: { path: string; op: string; token: string }[] = [];
    for (const token of delta.add || []) rows.push({ path, op: "+", token });
    for (const token of delta.remove || []) rows.push({ path, op: "−", token });
    return rows;
  });

  const addRule = () => {
    const next = [...rules, newRule()];
    setRules(next);
    setSelected(next.length - 1);
  };

  const removeRule = async (id: string) => {
    const next = rules.filter((item) => item.id !== id);
    setSelected((index) => Math.min(index, Math.max(0, next.length - 1)));
    await persist(next);
  };

  const bake = () => {
    if (!current) return;
    const entry = bakeVariants(current.id, current.effect, personality, languages.length ? languages : [locale]);
    setBank({ entries: [...bank.entries.filter((item) => item.rule_id !== current.id), entry] });
  };

  return (
    <div className="page">
      <section className="hero">
        <div>
          <h1>{t.rules}</h1>
          {view === "policies" && <p className="muted">{t.priority}</p>}
        </div>
        {view === "policies" && (
          <div className="row">
            <button className="secondary" onClick={() => persist(rules)}>{t.save}</button>
            <button className="primary" onClick={addRule}>{t.addRule}</button>
          </div>
        )}
      </section>
      <nav className="subnav">
        <button className={view === "routines" ? "active" : ""} onClick={() => setView("routines")}>{t.routines}</button>
        <button className={view === "sentences" ? "active" : ""} onClick={() => setView("sentences")}>{t.sentences}</button>
        <button className={view === "policies" ? "active" : ""} onClick={() => setView("policies")}>{t.policies}</button>
      </nav>
      {view === "routines" && <RoutinesPage t={t} />}
      {view === "sentences" && <CustomPage t={t} locale={locale} embedded />}
      {view === "policies" && (
        <>
          <section className="grid three policy-lanes">
            <div className={`card${lane === "match" ? " lane-open" : ""}`} onClick={() => setLane("match")}>
              <h2>{t.matchCatalog}</h2>
              <p className="caption">{t.matchReadOnly}</p>
              {catalog.map((row, index) => (
                <div
                  className={`rule-row${index === selectedMatch ? " active" : ""}`}
                  key={row.id}
                  onClick={(ev) => {
                    ev.stopPropagation();
                    setLane("match");
                    setSelectedMatch(index);
                  }}
                >
                  <span className="chip origin">{t.originEngine}</span>
                  <strong className="mono">{row.id}</strong>
                  <span className="muted">{row.precedence}</span>
                </div>
              ))}
            </div>
            <div className={`card${lane === "language" ? " lane-open" : ""}`} onClick={() => setLane("language")}>
              <h2>{t.laneLanguage}</h2>
              <h3>{t.lexiconOverlay}</h3>
              {lexiconRows.length === 0 && <p className="muted">{t.lexiconEmpty}</p>}
              {lexiconRows.map((row) => (
                <p className="lexicon-delta" key={`${row.path}-${row.op}-${row.token}`}>{row.path} {row.op}{row.token}</p>
              ))}
              <h3>{t.governSeed}</h3>
              <p className="muted">{t.governEmpty}</p>
            </div>
            <div className={`card${lane === "house" ? " lane-open" : ""}`} onClick={() => setLane("house")}>
              <h2>{t.laneHouse}</h2>
              {rules.length === 0 && (
                <div>
                  <p className="muted">{policiesEmpty(t)}</p>
                  <SetupHint t={t} />
                </div>
              )}
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
                  onClick={(ev) => {
                    ev.stopPropagation();
                    setLane("house");
                    setSelected(index);
                  }}
                >
                  <span className="muted">{index + 1}</span>
                  <strong>{rule.label || rule.id}</strong>
                  <span className="chip intent">{rule.effect}</span>
                  <span className="chip origin">{t.originOperator}</span>
                  <button className="ghost danger" onClick={() => removeRule(rule.id)}>{t.dismiss}</button>
                </div>
              ))}
              {current ? (
                <>
                  <label>{t.custom}</label>
                  <input value={current.label} onChange={(ev) => update({ label: ev.target.value })} />
                  <label className="row">
                    <input type="checkbox" checked={current.enabled} onChange={(ev) => update({ enabled: ev.target.checked })} style={{ width: "auto" }} />
                    {current.enabled ? "on" : "off"}
                  </label>
                  <label>{t.when}</label>
                  <input placeholder={t.whenPhrase} value={current.when.phrase || ""} onChange={(ev) => updateWhen("phrase", ev.target.value)} />
                  <SearchSelect
                    value={current.when.intent || ""}
                    options={withCurrent(intentOptions, current.when.intent || "")}
                    onChange={(value) => updateWhen("intent", value)}
                    placeholder="intent"
                  />
                  <SearchSelect
                    value={current.when.domain || ""}
                    options={withCurrent(domains, current.when.domain || "")}
                    onChange={(value) => updateWhen("domain", value)}
                    placeholder="domain"
                  />
                  <SearchSelect
                    value={current.when.area || ""}
                    options={withCurrent(rooms, current.when.area || "")}
                    onChange={(value) => updateWhen("area", value)}
                    placeholder="area"
                  />
                  <SearchSelect
                    value={current.when.entity_id || ""}
                    options={withCurrent(entityOptions, current.when.entity_id || "")}
                    onChange={updateWhenEntity}
                    placeholder="entity_id"
                  />
                  <SearchSelect
                    value={current.when.floor || ""}
                    options={withCurrent(floors, current.when.floor || "")}
                    onChange={(value) => updateWhen("floor", value)}
                    placeholder="floor"
                  />
                  <input placeholder="name" value={current.when.name || ""} onChange={(ev) => updateWhen("name", ev.target.value)} />
                  <label>{t.then}</label>
                  <select value={current.effect} onChange={(ev) => update({ effect: ev.target.value as PolicyEffect })}>
                    {EFFECTS.map((effect) => <option key={effect} value={effect}>{effectLabel(t, effect)}</option>)}
                  </select>
                  {current.effect === "prefer_entity" && (
                    <SearchSelect
                      value={current.prefer || ""}
                      options={withCurrent(entityOptions, current.prefer || "")}
                      onChange={(value) => update({ prefer: value || undefined })}
                      placeholder="prefer"
                      allowEmpty={false}
                    />
                  )}
                  {current.effect === "prefer_area" && (
                    <SearchSelect
                      value={current.prefer || ""}
                      options={withCurrent(rooms, current.prefer || "")}
                      onChange={(value) => update({ prefer: value || undefined })}
                      placeholder="prefer"
                      allowEmpty={false}
                    />
                  )}
                  {ACTION_EFFECTS.includes(current.effect) && (
                    <textarea placeholder={payloadHint(t, current.effect)} value={current.payload || ""} onChange={(ev) => update({ payload: ev.target.value })} />
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
          </section>
          <div className="card" style={{ marginTop: 16 }}>
            <h2>{t.evaluator}</h2>
            <div className="row">
              <input value={utterance} onChange={(ev) => setUtterance(ev.target.value)} placeholder={t.command} />
              <button className="primary" onClick={evaluate}>{t.analyze}</button>
            </div>
            {evalOut && (
              <div style={{ marginTop: 16 }}>
                <PolicyPath t={t} trace={evalOut.outcome.policy_trace} onSelect={selectLane} />
                <p className="muted" style={{ marginTop: 12 }}>{evalOut.speech_variant || evalOut.outcome.speech}</p>
              </div>
            )}
          </div>
        </>
      )}
      {status && <p className="muted">{status}</p>}
    </div>
  );
}
