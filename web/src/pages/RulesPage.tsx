import { useEffect, useMemo, useState } from "react";
import { api, type LangOverlay } from "../api";
import { HouseLane } from "../components/HouseLane";
import { LexiconLane } from "../components/LexiconLane";
import { MatchLane } from "../components/MatchLane";
import { PolicyPath, type PolicyLane } from "../components/PolicyPath";
import { useHouseCatalog } from "../components/SearchSelect";
import { TrainerDrawer } from "../components/TrainerDrawer";
import type { Messages } from "../i18n";
import { bakeVariants } from "../speechBank";
import type { EvaluateOut, Locale, MatchCatalogRow, MatchControl, PolicyRule, RulesView, SpeechBank } from "../types";
import { CustomPage } from "./CustomPage";
import { RoutinesPage } from "./RoutinesPage";

// App must scope any chrome hero (Speichern / Regel) to rules_view === "policies".

const fallbackIntents = ["HassTurnOn", "HassTurnOff", "HassToggle", "HassLightSet", "HassGetState", "HassClimateSetTemperature"];

const LANES: PolicyLane[] = ["match", "language", "house"];

function newRule(): PolicyRule {
  return {
    id: `rule-${Date.now().toString(36)}`,
    enabled: true,
    label: "",
    when: {},
    effect: "confirm",
  };
}

function laneTitle(t: Messages, lane: PolicyLane): string {
  switch (lane) {
    case "match":
      return t.laneMatch;
    case "language":
      return t.laneLanguage;
    case "house":
      return t.laneHouse;
    default: {
      const _never: never = lane;
      return _never;
    }
  }
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
  const [seeds, setSeeds] = useState<PolicyRule[]>([]);
  const [matchControls, setMatchControls] = useState<MatchControl[]>([]);
  const [overlay, setOverlay] = useState<LangOverlay | null>(null);
  const [lane, setLane] = useState<PolicyLane>("match");
  const [selectedMatch, setSelectedMatch] = useState(0);
  const [selectedSeed, setSelectedSeed] = useState<string | undefined>();
  const { entityOptions, rooms, domains, floors } = useHouseCatalog();
  const current = rules[selected];
  const seedIds = useMemo(() => new Set(seeds.map((seed) => seed.id)), [seeds]);
  const intentOptions = useMemo(
    () => intents.map((name) => ({ value: name, label: name })),
    [intents],
  );

  useEffect(() => {
    api.policies().then((bundle) => {
      setRules(bundle.policies);
      setBank(bundle.speech_bank);
      setMatchControls(bundle.match_controls || []);
    }).catch((err) => setStatus(String(err)));
    api.policiesCatalog().then((body) => {
      setCatalog(body.matches);
      setSeeds(body.seeds || []);
    }).catch((err) => setStatus(String(err)));
    api.langOverlay().then(setOverlay).catch((err) => setStatus(String(err)));
    api.intents().then((names) => { if (names.length) setIntents(names); }).catch(() => undefined);
  }, []);

  const persist = async (next: PolicyRule[], nextBank = bank, nextControls = matchControls) => {
    const saved = await api.savePolicies({ policies: next, speech_bank: nextBank, match_controls: nextControls });
    setRules(saved.policies);
    setBank(saved.speech_bank);
    setMatchControls(saved.match_controls || []);
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
      match_controls: matchControls,
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
        if (id) setSelectedSeed(id);
        break;
      case "house":
        if (id && seedIds.has(id)) {
          setLane("language");
          setSelectedSeed(id);
          break;
        }
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

  const addRule = () => {
    const next = [...rules, newRule()];
    setRules(next);
    setSelected(next.length - 1);
    setLane("house");
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
    <div className={`page${view === "policies" ? " policy-workbench" : ""}`}>
      <section className="hero">
        <div>
          <h1>{t.rules}</h1>
          {view === "policies" && <p className="muted">{t.priority}</p>}
        </div>
        {view === "policies" && (
          <div className="row">
            <button className="secondary" type="button" onClick={() => persist(rules)}>{t.save}</button>
            <button className="primary" type="button" onClick={addRule}>{t.addRule}</button>
          </div>
        )}
      </section>
      <nav className="subnav">
        <button className={view === "routines" ? "active" : ""} type="button" onClick={() => setView("routines")}>{t.routines}</button>
        <button className={view === "sentences" ? "active" : ""} type="button" onClick={() => setView("sentences")}>{t.sentences}</button>
        <button className={view === "policies" ? "active" : ""} type="button" onClick={() => setView("policies")}>{t.policies}</button>
      </nav>
      {view === "routines" && <RoutinesPage t={t} />}
      {view === "sentences" && <CustomPage t={t} locale={locale} embedded />}
      {view === "policies" && (
        <>
          <nav className="policy-lane-tabs" aria-label={t.laneTabs}>
            {LANES.map((item) => (
              <button
                key={item}
                type="button"
                className={lane === item ? "active" : ""}
                onClick={() => setLane(item)}
              >
                {laneTitle(t, item)}
              </button>
            ))}
          </nav>
          <section className="policy-lanes">
            <div
              className={`policy-lane${lane === "match" ? " active" : ""}`}
              data-lane="match"
              onClick={() => setLane("match")}
            >
              <MatchLane
                t={t}
                catalog={catalog}
                controls={matchControls}
                selected={selectedMatch}
                onSelect={(index) => {
                  setLane("match");
                  setSelectedMatch(index);
                }}
                onChange={setMatchControls}
              />
            </div>
            <div
              className={`policy-lane${lane === "language" ? " active" : ""}`}
              data-lane="language"
              onClick={() => setLane("language")}
            >
              <LexiconLane
                t={t}
                overlay={overlay}
                seeds={seeds}
                house={rules}
                selectedSeed={selectedSeed}
                onSelectSeed={(id) => {
                  setLane("language");
                  setSelectedSeed(id);
                }}
                onSaved={setOverlay}
                onHouse={(next) => { void persist(next); }}
                onStatus={setStatus}
              />
            </div>
            <div
              className={`policy-lane${lane === "house" ? " active" : ""}`}
              data-lane="house"
              onClick={() => setLane("house")}
            >
              <HouseLane
                t={t}
                rules={rules}
                seedIds={seedIds}
                selected={selected}
                bank={bank}
                intentOptions={intentOptions}
                entityOptions={entityOptions}
                rooms={rooms}
                domains={domains}
                floors={floors}
                onSelect={(index) => {
                  setLane("house");
                  setSelected(index);
                }}
                onMove={move}
                onRemove={(id) => { void removeRule(id); }}
                onUpdate={update}
                onUpdateWhen={updateWhen}
                onUpdateWhenEntity={updateWhenEntity}
                onBake={bake}
              />
            </div>
          </section>
          <div className="card policy-evaluate">
            <h2>{t.evaluator}</h2>
            <div className="row">
              <input value={utterance} onChange={(ev) => setUtterance(ev.target.value)} placeholder={t.command} />
              <button className="primary" type="button" onClick={evaluate}>{t.analyze}</button>
            </div>
            <div className="policy-evaluate-path">
              <PolicyPath t={t} trace={evalOut?.outcome.policy_trace} onSelect={selectLane} />
            </div>
            {evalOut?.warnings?.length ? <p className="muted">{t.matchDisableWarning}</p> : null}
            {evalOut ? <p className="muted" style={{ marginTop: 12 }}>{evalOut.speech_variant || evalOut.outcome.speech}</p> : null}
          </div>
          <TrainerDrawer
            t={t}
            lane={lane}
            language={languages.length === 1 ? languages[0] : locale}
            onStatus={setStatus}
          />
        </>
      )}
      {status && <p className="muted">{status}</p>}
    </div>
  );
}
